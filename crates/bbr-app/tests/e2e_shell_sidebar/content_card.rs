//! DESIGN §2 seated content card over the real app (task packet
//! `seated-content-card`): the native slot is the card's inner box — flush
//! with the rail, `sp:content-inset` on the other edges — and the real
//! WKWebView backing layer clips to `radius:lg`. Resize keeps the frame,
//! collapse gains the leading inset and keeps the native titlebar band as
//! empty draggable chassis, and the full-screen presentation is full-bleed
//! with the clip removed. Session identity never changes. Real AppKit
//! full screen cannot complete on this host's Window Server (see the portal
//! hardening suite), so full screen is driven through the typed
//! presentation seam. Every step synchronizes on typed snapshot state.

use super::*;
use bbr_testkit::protocol::Snapshot;

/// Flush frames until `done` holds for the card evidence and the native
/// slot rect (the clip lands in the same prepaint as the frame).
fn wait_card(client: &mut Client, what: &str, done: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let deadline = deadline();
    loop {
        flush_frame(client);
        let snapshot = client.snapshot().expect("card snapshot");
        if done(&snapshot) {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "{what}: card {:?}, rect {:?}",
            snapshot.content_card,
            snapshot.content_rect
        );
        std::thread::yield_now();
    }
}

fn close(a: [f32; 4], b: [f32; 4]) -> bool {
    a.iter().zip(b).all(|(a, b)| (a - b).abs() <= 0.5)
}

#[test]
#[ignore]
fn content_card_seats_the_native_slot_through_resize_collapse_and_full_screen() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("content-card", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let sessions = settled.engine_sessions;
        let window_w = settled.window_size[0];

        // Expanded: flush with the rail, inset elsewhere, clipped natively.
        let expanded = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            window_w - SIDEBAR_DEFAULT - CONTENT_INSET,
            deadline(),
        );
        let height = expanded.content_rect[3];
        assert_eq!(expanded.content_rect[1], CONTENT_INSET);
        let card = expanded.content_card;
        assert_eq!(
            (card.inset_top, card.inset_trailing, card.inset_bottom),
            (CONTENT_INSET, CONTENT_INSET, CONTENT_INSET)
        );
        assert_eq!(card.inset_leading, 0.0, "the leading edge is never inset");
        assert_eq!(
            (card.radius, card.shadow, card.full_screen),
            (10.0, true, false)
        );
        assert_ids(&expanded, &["bbr-content-card", "bbr-chassis-band"]);
        let expanded = wait_card(&mut client, "native clip", |s| {
            s.content_card.native_radius == Some(10.0)
        });
        assert_view_and_sessions_preserved(&expanded, view, sessions);

        // Resize: the frame follows the rail; the clip stays.
        client
            .send_ok(&dsl::drag_sidebar_divider(40.0))
            .expect("drag divider");
        let wide = SIDEBAR_DEFAULT + 40.0;
        let resized = wait_card(&mut client, "resized card", |s| {
            close(s.content_rect, content_rect_for(wide, window_w, height))
                && s.content_card.native_radius == Some(10.0)
        });
        assert_view_and_sessions_preserved(&resized, view, sessions);

        // Collapse: the leading inset appears and the native titlebar band
        // stays empty chassis above the card.
        keystroke(&mut client, "cmd-b");
        let band = client
            .snapshot()
            .expect("collapsed snapshot")
            .titlebar
            .safe_area_height;
        let collapsed = wait_card(&mut client, "collapsed card", |s| {
            !s.sidebar_visible
                && close(s.content_rect, collapsed_rect_for(window_w, height, band))
                && s.content_card.native_radius == Some(10.0)
        });
        assert_eq!(collapsed.content_card.inset_top, band.max(CONTENT_INSET));
        assert_eq!(collapsed.content_card.inset_leading, CONTENT_INSET);
        assert_ids(&collapsed, &["bbr-content-card", "bbr-chassis-band"]);
        assert_view_and_sessions_preserved(&collapsed, view, sessions);

        keystroke(&mut client, "cmd-b");
        let restored = wait_card(&mut client, "expanded again", |s| {
            s.sidebar_visible && close(s.content_rect, content_rect_for(wide, window_w, height))
        });
        assert_view_and_sessions_preserved(&restored, view, sessions);

        // Full screen: full-bleed, no radius, no shadow, no band, and the
        // native clip is removed in the same frame.
        client
            .send_ok(&dsl::set_full_screen_override(Some(true)))
            .expect("present full screen");
        let full = wait_card(&mut client, "full-bleed card", |s| {
            close(
                s.content_rect,
                [wide, 0.0, window_w - wide, height + 2.0 * CONTENT_INSET],
            ) && s.content_card.native_radius == Some(0.0)
        });
        let card = full.content_card;
        assert!(card.full_screen && !card.shadow);
        assert_eq!(
            (
                card.inset_top,
                card.inset_trailing,
                card.inset_bottom,
                card.radius
            ),
            (0.0, 0.0, 0.0, 0.0)
        );
        assert!(
            !full.visible_ids.iter().any(|id| id == "bbr-chassis-band"),
            "full screen has no chassis band"
        );
        assert_view_and_sessions_preserved(&full, view, sessions);

        client
            .send_ok(&dsl::set_full_screen_override(None))
            .expect("leave full screen");
        let seated = wait_card(&mut client, "seated again", |s| {
            close(s.content_rect, content_rect_for(wide, window_w, height))
                && s.content_card.native_radius == Some(10.0)
        });
        assert_view_and_sessions_preserved(&seated, view, sessions);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
