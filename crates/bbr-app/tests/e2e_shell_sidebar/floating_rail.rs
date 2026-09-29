//! Blueprint D02 real-app E2E (DESIGN §2 §11.4): the floating rail.
//!
//! With the rail hidden, pointer samples go through the exact method the
//! native pointer observer feeds (`RailPointer`, with event timestamps, so
//! the velocity is deterministic): a slow arrival dwells 300 ms, a fast one
//! reveals at once. The rail floats in the one Portal over the card — the
//! card and the web view keep their collapsed geometry — and the Portal
//! captures the pointer over the panel but not over the notch around the
//! traffic lights or the page. The keyboard stays in the browser window and
//! reaches the floating tab list. Leaving hides it, Escape hides it and a
//! resting pointer does not reopen it, ⌘L floats it with the overlay over
//! its field, and ⌘S docks it at the remembered width. Typed snapshots under
//! deadlines; no sleeps, no pixels, the user's cursor never moves.

use super::*;
use bbr_testkit::protocol::{BrowserChange, CommandSurface, Snapshot};

fn pointer(client: &mut Client, x: f32, time_ms: f64) {
    client
        .send_ok(&bbr_testkit::Request::RailPointer {
            x,
            y: 400.0,
            time_ms,
            pressed: false,
        })
        .expect("rail pointer");
}

fn flushed(client: &mut Client) -> Snapshot {
    flush_frame(client);
    client.snapshot().expect("snapshot")
}

fn wait_phase(client: &mut Client, phase: &str) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = flushed(client);
        if snapshot.rail_reveal == phase {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "rail never reached {phase}: {}",
            snapshot.rail_reveal
        );
    }
}

/// Wait until AppKit shows (or hides) the Portal window.
fn wait_portal_shown(client: &mut Client, shown: bool) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = flushed(client);
        if snapshot.portal.native_visible == shown {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "the Portal never became shown={shown}"
        );
    }
}

fn route(client: &mut Client, x: f32, y: f32) -> bool {
    client
        .send_ok(&bbr_testkit::Request::PortalRoutePointer { x, y })
        .expect("route pointer");
    !client
        .snapshot()
        .expect("routed")
        .portal
        .ignores_mouse_events
}

#[test]
#[ignore]
fn the_hidden_rail_floats_over_the_card_and_works_like_the_docked_rail() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("floating-rail", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first view");
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let docked = flushed(&mut client);
        let remembered = docked.sidebar_width;

        // Hide the rail; the card takes the collapsed geometry.
        keystroke(&mut client, "cmd-s");
        let hidden = flushed(&mut client);
        assert!(!hidden.sidebar_visible);
        assert_eq!(hidden.rail_reveal, "idle");
        let collapsed_card = hidden.content_rect;
        let sessions = hidden.engine_sessions;

        // A slow arrival dwells, then floats.
        pointer(&mut client, 60.0, 0.0);
        pointer(&mut client, 5.0, 200.0);
        assert_eq!(client.snapshot().expect("dwell").rail_reveal, "dwelling");
        let floating = wait_phase(&mut client, "floating");
        let rail = floating.floating_rail.clone().expect("floating panel");
        assert_eq!(rail.rect[0], 8.0);
        assert_eq!(rail.rect[1], 8.0);
        assert!((rail.rect[2] - remembered).abs() < 0.5, "remembered width");
        assert!(!floating.sidebar_visible, "floating never docks");
        assert_eq!(
            floating.content_rect, collapsed_card,
            "the card never reflows"
        );
        assert_eq!(floating.engine_sessions, sessions);
        let shown = wait_portal_shown(&mut client, true);
        let regions = shown.portal.hit_regions.clone().expect("hit-region mode");
        for capture in &rail.capture {
            let capture = capture.map(f64::from);
            assert!(regions.contains(&capture), "{capture:?} in {regions:?}");
        }
        // The Portal takes the pointer over the panel, not over the page or
        // the notch the traffic lights keep.
        assert!(route(&mut client, rail.rect[0] + 120.0, 300.0));
        assert!(!route(
            &mut client,
            rail.rect[0] + rail.rect[2] + 80.0,
            300.0
        ));
        if rail.notch[0] > 0.0 {
            assert!(!route(&mut client, rail.rect[0] + 4.0, rail.rect[1] + 4.0));
        }

        // Keyboard parity: ⌥⌘S focuses the floating tab list (no docking),
        // ↑ moves the selection through the browser window's keyboard.
        keystroke(&mut client, "alt-cmd-s");
        since = wait_chrome_focus(&mut client, true, since);
        let focused = flushed(&mut client);
        assert!(
            !focused.sidebar_visible,
            "⌥⌘S on a floating rail keeps it floating"
        );
        assert_eq!(focused.active_tab_index, Some(1));
        keystroke(&mut client, "up");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);

        // Leaving the panel hides it and hands the keyboard to the page.
        pointer(&mut client, 400.0, 1000.0);
        let left = flushed(&mut client);
        assert_eq!(left.rail_reveal, "idle");
        assert!(left.floating_rail.is_none());
        wait_portal_shown(&mut client, false);
        let _ = wait_chrome_focus(&mut client, false, since);

        // A fast arrival floats at once, without the dwell.
        pointer(&mut client, 500.0, 2000.0);
        pointer(&mut client, 4.0, 2040.0);
        assert_eq!(client.snapshot().expect("fast").rail_reveal, "floating");

        // Escape hides it; a pointer resting at the edge does not reopen it.
        keystroke(&mut client, "escape");
        assert_eq!(client.snapshot().expect("escaped").rail_reveal, "idle");
        pointer(&mut client, 4.0, 3000.0);
        pointer(&mut client, 4.0, 3400.0);
        assert_eq!(client.snapshot().expect("resting").rail_reveal, "idle");

        // ⌘L on a hidden rail floats it with the overlay over its field.
        keystroke(&mut client, "cmd-l");
        let until = deadline();
        let overlay = loop {
            let snapshot = flushed(&mut client);
            let anchored = snapshot
                .address_rect
                .zip(snapshot.portal.rect)
                .zip(snapshot.floating_rail.as_ref())
                .is_some_and(|((field, card), rail)| {
                    field[0] >= rail.rect[0] && (card[0] - field[0]).abs() < 0.5
                });
            if snapshot.command_surface == CommandSurface::Overlay && anchored {
                break snapshot;
            }
            assert!(
                Instant::now() < until,
                "the overlay never anchored to the floating field"
            );
        };
        assert!(!overlay.sidebar_visible);
        keystroke(&mut client, "escape");
        let closed = flushed(&mut client);
        assert_eq!(closed.command_surface, CommandSurface::None);
        assert_eq!(
            closed.rail_reveal, "floating",
            "the first Escape closes the overlay only"
        );

        // ⌘S while floating docks at the remembered width.
        keystroke(&mut client, "cmd-s");
        let redocked = flushed(&mut client);
        assert!(redocked.sidebar_visible);
        assert!(redocked.floating_rail.is_none());
        assert!((redocked.sidebar_width - remembered).abs() < 0.5);
        // Docked: the edge never floats a second rail.
        pointer(&mut client, 500.0, 5000.0);
        pointer(&mut client, 4.0, 5040.0);
        assert_eq!(client.snapshot().expect("docked").rail_reveal, "idle");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
