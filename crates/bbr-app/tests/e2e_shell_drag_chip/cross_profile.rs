//! Cross-profile drag-to-chip evidence (DESIGN.md §11.3/§12): dragging a
//! tab onto a chip of another profile destroys the source session and
//! recreates it exactly once in the destination profile's store — the wire
//! event asserts `recreated: true`, the net session count stays, and the
//! destination activation proves the fresh view plus the isolated store.

use super::*;
use super::{assert_footer_invariant, chip_center, chip_index, drag_onto};
use bbr_testkit::protocol::{dsl, BrowserChange};

/// Drag the alpha-cookie tab onto the private profile's chip: exactly one
/// recreation in the destination store (the recreated view reads beta, never
/// the source's alpha), the net session count is unchanged, and the source
/// selection repairs to the remaining tab.
#[test]
#[ignore]
fn cross_profile_drag_recreates_exactly_once_in_destination_store() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let set_alpha = _fixtures.url("/__fx/cookie/set-alpha");
    let set_beta = _fixtures.url("/__fx/cookie/set-beta");
    let read_cookie = _fixtures.url("/__fx/cookie/read");
    let (mut harness, socket, token) = spawn_harness("drag-chip-cross", &page_one);

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

        // The alpha tab: page_two with the alpha cookie, then read back.
        let alpha_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::navigate(&set_alpha))
            .expect("set alpha");
        since = wait_title(&mut client, alpha_view, "Cookie Set alpha", since);
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read alpha");
        since = wait_title(&mut client, alpha_view, "Cookie alpha", since);

        // A private profile + space with its own store: seed beta there.
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let private_space = match created {
            BrowserChange::ProfileCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
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
        let beta_view = open_tab(&mut client, &mut since, &set_beta, Some("Cookie Set beta"));
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read beta");
        since = wait_title(&mut client, beta_view, "Cookie beta", since);
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
        // The dragged row is the alpha tab (index 1); the private chip is
        // the exact persistent destination.
        // One flushed frame so the strip has laid out both chips.
        let seeded = strip_presence(&mut client);
        assert_eq!(seeded.space_chips.len(), 2, "both profile chips render");
        assert_footer_invariant(&seeded);
        let sessions_before = seeded.engine_sessions;
        let target_center = chip_center(
            &seeded.space_strip_scroll,
            chip_index(&seeded, private_space),
        );
        let during = drag_onto(&mut client, 1, target_center.0, target_center.1);
        assert_eq!(
            during.drag_chip,
            Some(private_space),
            "the pointer targets the private profile's exact chip"
        );
        assert!(
            during.space_chips[chip_index(&during, private_space)].drag_target,
            "the private chip reports the drag target"
        );

        // Release: the cross-profile drop recreates the session exactly once
        // in the private store.
        client.send_ok(&dsl::end_tab_drag()).expect("end drag");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: private_space,
                recreated: true,
            },
            since,
        );
        let cross = client.snapshot().expect("cross-profile move snapshot");
        assert_eq!(cross.tabs.len(), 1, "the source keeps page one");
        assert_eq!(
            cross.engine_sessions, sessions_before,
            "destroy + recreate exactly once keeps the session count"
        );
        assert_eq!(
            cross.active_web_view,
            Some(first_view),
            "the source selection repairs to the remaining tab"
        );
        assert!(
            cross.chrome_focused,
            "the drop keeps the chrome focus domain"
        );
        assert_footer_invariant(&cross);

        // Destination activation: the moved tab has a fresh incarnation and
        // lives in the private store (beta), never the source (alpha).
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
        assert_ne!(fresh, alpha_view, "cross-profile move recreates the view");
        assert_ne!(
            fresh, beta_view,
            "the destination is a single fresh session"
        );
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
            "the recreated session lives in the destination store (beta), not the source (alpha)"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
