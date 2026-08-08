//! Dismissal scenarios (task packet `bbr-context-menus-over-web`).
//!
//! Proves every dismissal path — the Escape key, the close seam (the exact
//! method the portal backdrop's one-pointer-action press calls), and a
//! press on a disabled row (which, like a native menu, dismisses without
//! activating) — never creates, destroys, moves, or reselects a web
//! session and restores the origin focus domain exactly (web-origin and
//! chrome-origin sessions both).

use crate::e2e_common::*;
use crate::support::*;

/// Escape dismisses an open menu and restores the web origin: the menu
/// closes, the page keeps focus (no chrome focus stolen), and the
/// tab/session/profile/space identity is byte-for-byte untouched.
#[test]
#[ignore]
fn escape_dismisses_and_restores_web_origin() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-escape", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);
        assert!(
            !settled.chrome_focused,
            "the settled session owns web focus (the origin domain)"
        );

        let point = [260.0, 270.0];
        let open = open_menu(&mut client, view, point);
        assert!(
            !open.chrome_focused,
            "the menu owns a non-chrome focus domain while open"
        );

        key(&mut client, "escape");
        let closed = wait_menu_closed(&mut client, deadline());
        assert!(!closed.chrome_focused, "web origin restored after Escape");
        assert_identity_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The close seam (the exact method the portal backdrop's pointer press and
/// every dismissal path calls) dismisses without mutating the session, and
/// re-opening after a chrome-origin capture restores chrome focus exactly:
/// a chrome-focused origin closes back chrome-focused, a web origin closes
/// back web-focused.
#[test]
#[ignore]
fn close_seam_restores_the_captured_origin_domain() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-close", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);

        // Web origin: open → close seam → web focus restored, identity
        // untouched, and the second open does not drift the origin capture.
        let point = [280.0, 290.0];
        open_menu(&mut client, view, point);
        client
            .send_ok(&bbr_testkit::Request::ContextMenuClose)
            .expect("close seam");
        let closed_web = wait_menu_closed(&mut client, deadline());
        assert!(
            !closed_web.chrome_focused,
            "web origin restored by the close seam"
        );
        assert_identity_preserved(&closed_web, &settled);

        // Chrome origin: the production pointer path focuses the tab list
        // (chrome domain), then the menu captures that origin at open and
        // restores it exactly on close.
        client
            .send_ok(&bbr_testkit::Request::ClickTab { index: 0 })
            .expect("focus the tab list (chrome domain)");
        let _ = wait_chrome_focus(&mut client, true, since);
        let chrome_origin = client.snapshot().expect("chrome origin snapshot");
        assert!(chrome_origin.chrome_focused, "tab list owns chrome focus");
        assert_eq!(chrome_origin.active_web_view, Some(view));

        let open_chrome = open_menu(&mut client, view, point);
        assert!(
            open_chrome.context_menu.origin_chrome_focused,
            "the chrome origin was captured at open"
        );
        assert!(
            !open_chrome.chrome_focused,
            "the menu owns a non-chrome focus domain while open"
        );
        client
            .send_ok(&bbr_testkit::Request::ContextMenuClose)
            .expect("close seam (chrome origin)");
        let closed_chrome = wait_menu_closed(&mut client, deadline());
        assert!(
            closed_chrome.chrome_focused,
            "the captured chrome origin was restored exactly"
        );
        assert_eq!(
            closed_chrome.active_web_view,
            Some(view),
            "the tab list session stays selected"
        );
        assert_identity_preserved(&closed_chrome, &chrome_origin);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// A press on a disabled row never activates and, exactly like a native
/// menu, dismisses the menu — the page title and the session identity stay
/// untouched (no Back/Forward/Reload/Copy/Open happened).
#[test]
#[ignore]
fn disabled_row_press_dismisses_without_activating() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-disabled", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);

        // Fresh history: Back/Forward disabled, so a press on Back (index 0)
        // is a press on a disabled row.
        let point = [300.0, 300.0];
        let open = open_menu(&mut client, view, point);
        assert!(!row(&open, "back").enabled, "Back disabled with no history");
        assert!(!row(&open, "forward").enabled, "Forward disabled");

        client
            .send_ok(&bbr_testkit::Request::ContextMenuActivate { index: 0 })
            .expect("press the disabled Back row");
        let closed = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            closed.title.as_deref(),
            Some(PAGE_ONE_TITLE),
            "no navigation happened from the disabled row"
        );
        assert_identity_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
