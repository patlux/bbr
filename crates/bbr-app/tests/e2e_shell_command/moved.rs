//! Moved-tab activation regression (DESIGN.md §11.5).
//!
//! The ranked `SelectTab` activation carries the owning space from rank time.
//! When the ranked tab moves spaces before Enter, activation must route
//! through the tab's CURRENT `all_tabs` record — current owning space and
//! current owning-space index — never through the stale ranked space
//! (selecting the stale numeric index in the old space could activate an
//! unrelated tab). This scenario ranks a tab in the active space, moves it
//! into another space before Enter, and proves the exact moved tab (persistent
//! id, warm view, current space/index) activates while the unrelated tab that
//! used to sit at the stale index stays untouched.

use super::*;

#[test]
#[ignore]
fn moved_tab_activates_through_its_current_space_not_the_stale_rank() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-movedtab", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_profile = first.active_profile;
        let default_space = first.active_space;

        // The default space owns two tabs: [T0 (page_one), T1 (page_two)].
        // After T0 moves away, T1 sits at index 0 — the numeric index the
        // stale-rank bug could activate.
        let t1_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let state = client.snapshot().expect("two-tab snapshot");
        let t0 = state
            .all_tabs
            .iter()
            .find(|tab| tab.space == default_space && tab.url == page_one)
            .cloned()
            .expect("t0 identity");
        let t1 = state
            .all_tabs
            .iter()
            .find(|tab| tab.space == default_space && tab.url == page_two)
            .cloned()
            .expect("t1 identity");
        assert_eq!(t1.web_view, Some(t1_view));
        assert_eq!(t0.index, 0, "the moved tab ranks at index 0");

        // A second warm space with one warm tab: after the move, T0 lives
        // there at index 1 (behind RC).
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::activate_space(reading))
            .expect("activate reading");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let rc = client
            .snapshot()
            .expect("reading tab snapshot")
            .all_tabs
            .iter()
            .find(|tab| tab.space == reading)
            .cloned()
            .expect("reading tab identity");

        // Back to the default space, then rank T0 through the command
        // surface. "page1" matches only T0's URL.
        client
            .send_ok(&dsl::activate_space(default_space))
            .expect("activate default");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "page1".into(),
            })
            .expect("query page1");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush page1");
        let rows = client.snapshot().expect("page1 rows");
        let row = rows
            .command_results
            .iter()
            .find(|row| row.key == format!("tab:{}", t0.id))
            .expect("t0 row");
        assert!(
            row.selected,
            "the ranked tab is the selected row the Enter below commits"
        );
        let sessions = rows.engine_sessions;

        // Before Enter, T0 moves to the Reading space (production move seam;
        // the ranked row still carries the stale default-space ownership).
        client
            .send_ok(&bbr_testkit::Request::MoveTab {
                index: t0.index,
                to_space: reading,
            })
            .expect("move t0 to reading");
        let (seq, _moved) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::TabMoved { to_space, .. } if *to_space == reading),
            since,
        );
        since = seq;
        let moved_state = client.snapshot().expect("moved state");
        let moved_t0 = moved_state
            .all_tabs
            .iter()
            .find(|tab| tab.id == t0.id)
            .expect("t0 still exists after the move");
        assert_eq!(moved_t0.space, reading, "t0 now owns the reading space");
        assert_eq!(
            moved_t0.index,
            rc.index + 1,
            "t0 sits behind the reading tab in the destination space"
        );
        let t0_view = moved_t0
            .web_view
            .expect("same-profile move keeps the warm session");

        // Enter on the ranked row: the activation must route through the
        // CURRENT record — activate Reading, re-read T0 by persistent id,
        // verify it still lives there, and select its current index. The
        // stale rank's default space must never select numeric index 1 in
        // the default space (that is T1 — an unrelated tab).
        key(&mut client, "enter");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let activated = client.snapshot().expect("moved-tab activation snapshot");
        assert_eq!(
            activated.active_space, reading,
            "the current owning space is activated, never the stale rank space"
        );
        assert_eq!(
            activated.active_web_view,
            Some(t0_view),
            "the exact moved tab's warm view is presented"
        );
        let active_tab = activated
            .all_tabs
            .iter()
            .find(|tab| tab.web_view == activated.active_web_view)
            .expect("active tab identity");
        assert_eq!(
            active_tab.id, t0.id,
            "the activated tab is the moved tab by persistent id"
        );
        assert_eq!(
            active_tab.space, reading,
            "the activated tab lives in the current owning space"
        );
        assert_eq!(
            activated.active_tab_index,
            Some(moved_t0.index),
            "the exact current owning-space index is selected"
        );
        assert_eq!(
            activated.engine_sessions, sessions,
            "warm move + warm activation never recreate sessions"
        );
        assert!(
            !activated.chrome_focused,
            "Enter selects the tab into web focus"
        );
        assert_eq!(activated.command_bar_mode, CommandBarMode::Inactive);
        // The unrelated tab that used to sit at the stale index in the
        // default space is untouched: T1 still lives there with its own view.
        let t1_after = activated
            .all_tabs
            .iter()
            .find(|tab| tab.id == t1.id)
            .expect("t1 untouched");
        assert_eq!(t1_after.space, default_space);
        assert_ne!(
            activated.active_web_view, t1.web_view,
            "no unrelated tab activated through the stale index"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
