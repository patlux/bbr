//! Drag-to-chip invariants (DESIGN.md §11.3/§11.4/§17.11): the footer
//! `[Downloads][Space Strip][+]` composition, the active space's curated
//! theme tint (typed on the active chip and the appearance evidence), the
//! tracked strip scroll geometry, and the downloads panel state are all
//! invariant across a completed drag-to-chip move.

use super::*;
use super::{assert_footer_invariant, chip_center, chip_index, drag_onto};
use bbr_testkit::protocol::{dsl, BrowserChange};

/// A completed same-profile drag-to-chip move leaves the footer
/// composition, the active tint, the strip scroll geometry, and the
/// downloads panel byte-for-byte unchanged.
#[test]
#[ignore]
fn drag_to_chip_keeps_footer_tint_scroll_and_downloads_invariants() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("drag-chip-invariants", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        let default_space = initial.active_space;
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client.send_ok(&dsl::click_tab(0)).expect("select row 0");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        client
            .send_ok(&dsl::create_space(initial.active_profile, "Reading"))
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

        // The pre-drag invariant baseline (one flushed frame so the
        // strip geometry and chips are render-settled).
        let before = strip_presence(&mut client);
        assert_footer_invariant(&before);
        let appearance_before = before
            .active_appearance
            .as_ref()
            .expect("active appearance renders");
        let active_chip_before = before
            .space_chips
            .iter()
            .find(|chip| chip.space == before.active_space)
            .expect("active chip");
        assert_eq!(
            active_chip_before.theme, appearance_before.theme,
            "the active chip tints from the one appearance authority"
        );
        let scroll_before = &before.space_strip_scroll;
        let downloads_before = before.downloads;
        let chip_count_before = scroll_before.chip_bounds.len();
        assert_eq!(chip_count_before, 2);
        let center = chip_center(scroll_before, chip_index(&before, reading));

        // The move.
        let during = drag_onto(&mut client, 1, center.0, center.1);
        assert_eq!(during.drag_chip, Some(reading));
        assert_eq!(during.engine_sessions, 2, "no session churn mid-drag");
        client.send_ok(&dsl::end_tab_drag()).expect("end drag");
        wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: reading,
                recreated: false,
            },
            since,
        );

        // Every invariant holds after the commit.
        let after = client.snapshot().expect("post-drag snapshot");
        assert_footer_invariant(&after);
        assert_eq!(
            after.active_space, default_space,
            "the move never switches the active space"
        );
        let appearance_after = after
            .active_appearance
            .as_ref()
            .expect("active appearance unchanged");
        assert_eq!(appearance_after.space, appearance_before.space);
        assert_eq!(
            appearance_after.theme, appearance_before.theme,
            "tint unchanged"
        );
        assert_eq!(appearance_after.identity, appearance_before.identity);
        let active_chip_after = after
            .space_chips
            .iter()
            .find(|chip| chip.space == after.active_space)
            .expect("active chip after");
        assert_eq!(
            active_chip_after.theme, appearance_after.theme,
            "the active tint still resolves from the one authority"
        );
        let scroll_after = &after.space_strip_scroll;
        assert_eq!(
            scroll_after.viewport, scroll_before.viewport,
            "strip viewport fixed"
        );
        assert_eq!(scroll_after.offset, scroll_before.offset, "no scroll drift");
        assert_eq!(scroll_after.max_offset, scroll_before.max_offset);
        assert_eq!(
            scroll_after.chip_bounds.len(),
            chip_count_before,
            "the strip still lays out every chip"
        );
        assert_eq!(
            after.downloads, downloads_before,
            "downloads state untouched"
        );
        assert!(!after.downloads.open);
        assert_eq!(
            after.engine_sessions, 2,
            "same-profile move recreates nothing"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
