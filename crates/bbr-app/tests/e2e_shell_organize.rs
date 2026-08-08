//! Real-app tab-organization E2E (Phase-4 tab drag/reorder slice).
//!
//! Covers the keyboard commands and the pointer drag over real WKWebViews:
//! pin/unpin (`Cmd-Shift-P` + the row pin control), region-bounded reorder
//! (`Cmd-Shift-↑/↓` + drag with a typed insertion tick), move between spaces
//! (`Cmd-Shift-M` staging), cancellation (Escape on the staged move and on
//! an in-flight drag), the focus-domain contract, session identity
//! (same-profile moves keep the WebViewId; cross-profile moves recreate
//! exactly once in the destination store), and profile isolation. Every step
//! synchronizes on semantic events or typed snapshot state under a deadline
//! — no sleeps. The pointer drag goes through the production
//! mouse-down/move/up listener methods with synthetic events (exactly like
//! the sidebar-divider drag seam), never a direct reorder or move command.
//!
//! Drag geometry: §8 density rows are 36 pt in the default density the shell
//! renders; the app resolves targets relative to the press position, so this
//! constant (mirroring `theme.density`) is the only layout value the
//! scenarios need. The revealed-list drag-to-space band was removed with the
//! legacy `[Spaces]` toggle; drag-to-chip is a later explicit F2a layer.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::CommandBarMode;
use e2e_common::*;

/// §8 normal-density tab row height (theme.density::Normal.row()).
const ROW_H: f32 = 36.0;

/// Assert the full tab row order by URL plus the pinned boundary and the
/// selected row, so a wrong order or a wrong boundary fails loudly.
fn assert_tab_order(snapshot: &bbr_testkit::protocol::Snapshot, urls: &[&str], pinned: usize) {
    let got: Vec<&str> = snapshot.tabs.iter().map(|t| t.url.as_str()).collect();
    assert_eq!(got, urls, "tab order mismatch");
    assert_eq!(snapshot.pinned_count, pinned, "pinned boundary mismatch");
}

/// Keyboard tab organization: real shortcuts (`Cmd-Shift-P`, `Cmd-Shift-↑/↓`,
/// `Cmd-Shift-M`) through the GPUI keybinding path, staging a destination
/// space by name, Escape cancellation, same-profile moves that keep the live
/// session, and a cross-profile move that recreates the session exactly once
/// in the destination profile's isolated store.
#[test]
#[ignore]
fn keyboard_pins_unpins_reorders_and_moves_tabs() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");
    let set_alpha = fixtures.url("/__fx/cookie/set-alpha");
    let set_beta = fixtures.url("/__fx/cookie/set-beta");
    let read_cookie = fixtures.url("/__fx/cookie/read");

    let (mut harness, socket, token) = spawn_harness("organize-keyboard", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // §11.3/§11.4: the permanent space strip is always part of the rail;
        // the strip presence seam verifies it before the space-related
        // keyboard steps.
        strip_presence(&mut client);

        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };

        // Two visible tabs; focus the tab list through the production pointer
        // path so the keyboard commands run from the chrome domain.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client.send_ok(&dsl::click_tab(1)).expect("focus tab list");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let two = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two.tabs.len(), 2);
        assert_eq!(two.engine_sessions, 2);
        assert!(two.chrome_focused, "pointer click keeps chrome focus");

        // Cmd-Shift-P pins the active tab: the pinned/regular boundary moves,
        // the selected tab stays selected, and the live session survives.
        keystroke(&mut client, "cmd-shift-p");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        let pinned = client.snapshot().expect("pinned snapshot");
        assert_tab_order(&pinned, &[&page_two, &page_one], 1);
        assert_eq!(pinned.active_tab_index, Some(0), "pinned tab stays active");
        assert_eq!(pinned.active_web_view, Some(second_view));
        assert_eq!(pinned.engine_sessions, 2, "pin never recreates a session");
        assert!(pinned.chrome_focused, "keyboard command keeps chrome focus");

        // Cmd-Shift-P again unpins (to the end of the regular region).
        keystroke(&mut client, "cmd-shift-p");
        since = wait_browser_change(&mut client, BrowserChange::TabUnpinned { index: 1 }, since);
        let unpinned = client.snapshot().expect("unpinned snapshot");
        assert_tab_order(&unpinned, &[&page_one, &page_two], 0);
        assert_eq!(unpinned.active_tab_index, Some(1));
        assert_eq!(unpinned.active_web_view, Some(second_view));
        assert_eq!(unpinned.engine_sessions, 2);

        // Pin again, then try to step the pinned tab across the boundary:
        // the step is rejected whole (no TabReordered, no partial mutation).
        keystroke(&mut client, "cmd-shift-p");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        assert_tab_order(
            &client.snapshot().expect("repinned snapshot"),
            &[&page_two, &page_one],
            1,
        );
        keystroke(&mut client, "cmd-shift-down");
        let rejected = client.snapshot().expect("rejected snapshot");
        assert_tab_order(&rejected, &[&page_two, &page_one], 1);
        assert_eq!(
            rejected.active_tab_index,
            Some(0),
            "illegal step leaves selection untouched"
        );
        let illegal_events = client
            .attach(Some(since))
            .expect("illegal-step replay")
            .into_iter()
            .filter(|(_, event)| {
                matches!(
                    event,
                    Event::BrowserChanged {
                        kind: BrowserChange::TabReordered { .. }
                    }
                )
            })
            .count();
        assert_eq!(
            illegal_events, 0,
            "boundary-crossing step emits no reorder event"
        );

        // The regular-region mirror: select the regular tab and try to step
        // it up into the pinned region — also rejected whole.
        client.send_ok(&dsl::click_tab(1)).expect("select regular");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        keystroke(&mut client, "cmd-shift-up");
        let still = client.snapshot().expect("still pinned snapshot");
        assert_tab_order(&still, &[&page_two, &page_one], 1);
        assert_eq!(still.active_tab_index, Some(1));

        // Legal keyboard reorder needs a third tab so the active row has a
        // same-region neighbor in both directions. Cmd-Shift-P toggles the
        // *active* tab, so unpin the pinned row by selecting it first.
        client.send_ok(&dsl::click_tab(0)).expect("select pinned");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        keystroke(&mut client, "cmd-shift-p");
        since = wait_browser_change(&mut client, BrowserChange::TabUnpinned { index: 1 }, since);
        let unpinned = client.snapshot().expect("unpinned regular snapshot");
        assert_tab_order(&unpinned, &[&page_one, &page_two], 0);
        let third_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client.send_ok(&dsl::click_tab(1)).expect("select middle");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let three = client.snapshot().expect("three-tab snapshot");
        assert_tab_order(&three, &[&page_one, &page_two, &page_two], 0);
        let middle_view = three.tabs[1].web_view.expect("middle view");

        keystroke(&mut client, "cmd-shift-up");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabReordered { from: 1, to: 0 },
            since,
        );
        let up = client.snapshot().expect("reordered up snapshot");
        assert_tab_order(&up, &[&page_two, &page_one, &page_two], 0);
        assert_eq!(up.active_tab_index, Some(0), "selection follows the move");
        assert_eq!(
            up.tabs[0].web_view,
            Some(middle_view),
            "reorder never recreates the session"
        );
        assert_eq!(up.engine_sessions, 3);

        keystroke(&mut client, "cmd-shift-down");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabReordered { from: 0, to: 1 },
            since,
        );
        let down = client.snapshot().expect("reordered down snapshot");
        assert_tab_order(&down, &[&page_one, &page_two, &page_two], 0);
        assert_eq!(down.active_tab_index, Some(1));
        assert_eq!(down.tabs[1].web_view, Some(middle_view));
        assert!(down.chrome_focused, "reorder keeps the chrome domain");

        // Cmd-Shift-M stages a destination space; nothing moves until Enter.
        let default_profile = down.active_profile;
        let default_space = down.active_space;
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading space");
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
        keystroke(&mut client, "cmd-shift-m");
        since = wait_chrome_focus(&mut client, true, since);
        let staged = client.snapshot().expect("staged move snapshot");
        assert_eq!(staged.command_bar_mode, CommandBarMode::MoveTab);
        assert_eq!(staged.active_tab_index, Some(1), "staging moves nothing");
        assert_tab_order(&staged, &[&page_one, &page_two, &page_two], 0);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "Reading".into(),
            })
            .expect("set move destination");
        let hinted = client.snapshot().expect("hinted snapshot");
        assert_eq!(
            hinted.command_bar_hint.as_deref(),
            Some("→ Reading"),
            "same-profile hint names the space"
        );

        // Escape cancels the staged move: mode and focus are restored and
        // nothing moved.
        keystroke(&mut client, "escape");
        let cancelled = client.snapshot().expect("cancelled move snapshot");
        assert_eq!(cancelled.command_bar_mode, CommandBarMode::Inactive);
        assert_eq!(cancelled.active_tab_index, Some(1));
        assert_tab_order(&cancelled, &[&page_one, &page_two, &page_two], 0);

        // Enter commits: the active tab moves into Reading (same profile),
        // its live session follows (same WebViewId), the source selection is
        // repaired to the neighbor, and chrome focus stays in the tab list.
        keystroke(&mut client, "cmd-shift-m");
        since = wait_chrome_focus(&mut client, true, since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "Reading".into(),
            })
            .expect("set move destination again");
        keystroke(&mut client, "enter");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: reading,
                recreated: false,
            },
            since,
        );
        let moved = client.snapshot().expect("moved snapshot");
        assert_tab_order(&moved, &[&page_one, &page_two], 0);
        assert_eq!(
            moved.active_tab_index,
            Some(1),
            "source selection repaired to the next sibling"
        );
        assert_eq!(moved.active_web_view, Some(third_view));
        assert_eq!(moved.engine_sessions, 3, "same-profile move keeps sessions");
        assert!(moved.chrome_focused, "committed move keeps chrome focus");
        let reading_row = moved.spaces.iter().find(|s| s.space == reading).unwrap();
        assert_eq!(reading_row.tab_count, 1);
        assert_eq!(
            reading_row.active_view,
            Some(second_view),
            "moved tab keeps its live WebViewId"
        );

        // Cross-profile move: create a Private profile with its own store,
        // seed distinct cookies, then move the alpha-cookie tab across the
        // boundary. Its session is destroyed and recreated exactly once in
        // the private store, which reads beta — never alpha.
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created_profile) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let private_space = match created_profile {
            BrowserChange::ProfileCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::navigate(&set_alpha))
            .expect("set alpha");
        since = wait_title(&mut client, third_view, "Cookie Set alpha", since);
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read alpha");
        since = wait_title(&mut client, third_view, "Cookie alpha", since);
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("open private");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_view = open_tab(&mut client, &mut since, &set_beta, Some("Cookie Set beta"));
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read beta");
        since = wait_title(&mut client, private_view, "Cookie beta", since);
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("back to default");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        client
            .send_ok(&dsl::click_tab(1))
            .expect("select the alpha tab");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);

        let sessions_before_move = client.snapshot().expect("before move").engine_sessions;
        keystroke(&mut client, "cmd-shift-m");
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "Private".into(),
            })
            .expect("set private destination");
        keystroke(&mut client, "enter");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: private_space,
                recreated: true,
            },
            since,
        );
        let cross = client.snapshot().expect("cross-profile move snapshot");
        assert_tab_order(&cross, &[&page_one], 0);
        assert_eq!(
            cross.engine_sessions, sessions_before_move,
            "destroy + recreate once keeps the session count"
        );
        assert_eq!(
            cross.active_web_view,
            Some(first_view),
            "source repairs to the remaining tab"
        );
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("view the moved tab");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let moved_private = client.snapshot().expect("moved tab in private");
        let fresh = moved_private.active_web_view.expect("recreated view");
        assert_ne!(fresh, third_view, "cross-profile move recreates the view");
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read moved tab cookie");
        wait_title(&mut client, fresh, "Cookie beta", since);
        assert_eq!(
            client
                .snapshot()
                .expect("isolated snapshot")
                .title
                .as_deref(),
            Some("Cookie beta"),
            "recreated session lives in the destination store (beta), not the source (alpha)"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Pointer tab organization: the row pin control through the production
/// mouse-down listener path, drag reorder within a region through the
/// production press/move/release listeners with the §10.9 insertion tick and
/// its legality, an illegal boundary drop that is rejected whole, and Escape
/// cancellation. Every committed move keeps the live session.
#[test]
#[ignore]
fn pointer_pin_control_and_drag_reorder_respect_regions() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");

    let (mut harness, socket, token) = spawn_harness("organize-pointer", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // §11.3/§11.4: the permanent space strip is always part of the rail
        // (the strip presence seam verifies it; there is no revealed region
        // for the pin/reorder pointers).
        strip_presence(&mut client);

        // Seed the pinned region through the visible pin controls: two tabs,
        // both pinned via `bbr-tab-pin-<n>` on the production listener path.
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::click_tab_pin(1))
            .expect("pin second row");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        client
            .send_ok(&dsl::click_tab_pin(1))
            .expect("pin first row (now index 1)");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 1 }, since);
        let pinned_two = client.snapshot().expect("two pinned snapshot");
        assert_tab_order(&pinned_two, &[&page_two, &page_one], 2);
        assert_eq!(pinned_two.engine_sessions, 2);
        assert_ids(&pinned_two, &["bbr-tab-pin-0", "bbr-tab-pin-1"]);

        // A regular region below: two more tabs.
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let fourth_view = open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));
        let four = client.snapshot().expect("four-tab snapshot");
        assert_tab_order(&four, &[&page_two, &page_one, &page_two, &page_one], 2);
        let regular_view = four.tabs[2].web_view.expect("regular row view");

        // Drag the first pinned row two rows down (dy = 2 × 36): the target
        // resolves to slot 2, the end of the pinned region — legal. The
        // snapshot reports the typed source/target/validity and the §15 tick
        // id renders at the slot.
        client
            .send_ok(&dsl::begin_tab_drag(0, 400.0, ROW_H / 2.0))
            .expect("begin drag");
        client
            .send_ok(&dsl::move_tab_drag(2.0 * ROW_H))
            .expect("move drag down");
        let during = client.snapshot().expect("during drag snapshot");
        assert_eq!(during.drag_source, Some(0));
        assert_eq!(during.drag_slot, Some(2));
        assert!(during.drag_valid, "slot 2 is the pinned region's own end");
        assert_ids(&during, &["bbr-drag-tick-2"]);
        client.send_ok(&dsl::end_tab_drag()).expect("end drag");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabReordered { from: 0, to: 1 },
            since,
        );
        let reordered = client.snapshot().expect("dragged reorder snapshot");
        assert_tab_order(&reordered, &[&page_one, &page_two, &page_two, &page_one], 2);
        assert_eq!(
            reordered.tabs[0].web_view,
            Some(first_view),
            "drag reorder preserves the live session"
        );
        assert_eq!(reordered.engine_sessions, 4);

        // Illegal boundary drop: drag the second pinned row (index 1) two
        // rows down into the regular region. The target clamps to the
        // boundary edge (slot 2) and reports invalid; release commits
        // nothing.
        client
            .send_ok(&dsl::begin_tab_drag(1, 400.0, ROW_H / 2.0))
            .expect("begin boundary drag");
        client
            .send_ok(&dsl::move_tab_drag(2.0 * ROW_H))
            .expect("move into the regular region");
        let illegal = client.snapshot().expect("illegal drag snapshot");
        assert_eq!(illegal.drag_source, Some(1));
        assert_eq!(illegal.drag_slot, Some(2), "tick clamps to the region edge");
        assert!(!illegal.drag_valid, "pinned → regular crossing is rejected");
        assert_ids(&illegal, &["bbr-drag-tick-2"]);
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("release illegal");
        let unchanged = client.snapshot().expect("unchanged snapshot");
        assert_tab_order(&unchanged, &[&page_one, &page_two, &page_two, &page_one], 2);
        let boundary_events = client
            .attach(Some(since))
            .expect("boundary replay")
            .into_iter()
            .filter(|(_, event)| {
                matches!(
                    event,
                    Event::BrowserChanged {
                        kind: BrowserChange::TabReordered { .. }
                    }
                )
            })
            .count();
        assert_eq!(boundary_events, 0, "illegal drop emits no reorder");

        // Legal regular-region reorder: drag the first regular row (index 2)
        // two rows down — slot 4, the regular region's own end.
        client
            .send_ok(&dsl::begin_tab_drag(2, 400.0, ROW_H / 2.0))
            .expect("begin regular drag");
        client
            .send_ok(&dsl::move_tab_drag(2.0 * ROW_H))
            .expect("move regular down");
        assert!(
            client.snapshot().expect("regular drag snapshot").drag_valid,
            "within the regular region the drop is legal"
        );
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("end regular drag");
        wait_browser_change(
            &mut client,
            BrowserChange::TabReordered { from: 2, to: 3 },
            since,
        );
        let regular_swapped = client.snapshot().expect("regular swap snapshot");
        assert_tab_order(
            &regular_swapped,
            &[&page_one, &page_two, &page_one, &page_two],
            2,
        );
        assert_eq!(regular_swapped.engine_sessions, 4);
        assert_eq!(
            regular_swapped.tabs[3].web_view,
            Some(regular_view),
            "the dragged tab keeps its live session at its destination"
        );
        assert_eq!(
            regular_swapped.tabs[2].web_view,
            Some(fourth_view),
            "the displaced tab keeps its live session at its new slot"
        );

        // Escape cancels an in-flight drag: the target clears, nothing is
        // committed, and chrome focus stays in the tab list.
        client
            .send_ok(&dsl::begin_tab_drag(2, 400.0, ROW_H / 2.0))
            .expect("begin cancel drag");
        client
            .send_ok(&dsl::move_tab_drag(2.0 * ROW_H))
            .expect("move cancel drag");
        assert!(client
            .snapshot()
            .expect("pre-cancel snapshot")
            .drag_source
            .is_some());
        client
            .send_ok(&dsl::cancel_tab_drag())
            .expect("cancel drag");
        let cancelled = client.snapshot().expect("cancelled drag snapshot");
        assert_eq!(cancelled.drag_source, None);
        assert_eq!(cancelled.drag_slot, None);
        assert!(!cancelled.drag_valid);
        assert_tab_order(&cancelled, &[&page_one, &page_two, &page_one, &page_two], 2);
        assert!(
            cancelled.chrome_focused,
            "cancelling the drag keeps the chrome domain"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
