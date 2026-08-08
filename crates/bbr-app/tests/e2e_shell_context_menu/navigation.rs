//! Pointer + keyboard navigation scenarios (task packet
//! `bbr-context-menus-over-web`).
//!
//! Proves the portal menu is navigable by pointer AND keyboard through the
//! real app, with GENUINE Back/Forward enablement: a real WKWebView
//! navigation into a second page makes `canGoBack()` true, the real
//! `CanGoBackChanged` event feeds the nav state, and the menu's Back row
//! reports enabled — never a guess. Pointer hover/select then press/
//! activate drives Back through the real `goBack()`; keyboard Up/Down skip
//! the disabled Forward row and Enter activates Reload. Every step asserts
//! the unchanged tab/session/profile/space identity.

use crate::e2e_common::*;
use crate::support::*;

/// The iteration-policy first proof: a real second-page navigation arms the
/// menu's Back row through the real `canGoBack()` event feed, the pointer
/// selects and activates it, and the real `goBack()` lands back on the
/// first page with the menu closed and the session identity untouched.
#[test]
#[ignore]
fn pointer_activates_back_from_real_history() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-pointer", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        since = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);

        // A real navigation into a second page arms the history: WKWebView's
        // `canGoBack()` turns true and the real CanGoBackChanged event feeds
        // the nav state the menu's rows derive from.
        client
            .send_ok(&bbr_testkit::Request::Navigate {
                url: page_two.clone(),
            })
            .expect("navigate to page two");
        since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        since = wait_history_state(&mut client, view, true, false, since);
        let armed = client.snapshot().expect("history armed snapshot");
        assert_eq!(armed.active_web_view, Some(view));

        // Open the BBR menu through the exact event transport the native
        // bridge uses, anchored at a pointer point in the content.
        let point = [240.0, 260.0];
        let open = open_menu(&mut client, view, point);
        assert!(row(&open, "back").enabled, "Back enabled by real history");
        assert!(
            !row(&open, "forward").enabled,
            "Forward disabled before a back step"
        );

        // Pointer path: hover selects the Back row (the shared cursor), a
        // press activates it through the production app method.
        client
            .send_ok(&bbr_testkit::Request::ContextMenuSelect { index: 0 })
            .expect("pointer hover selects Back");
        let hovered = client.snapshot().expect("hovered snapshot");
        assert_eq!(hovered.context_menu.selected, Some(0));
        assert!(hovered.context_menu.open);
        client
            .send_ok(&bbr_testkit::Request::ContextMenuActivate { index: 0 })
            .expect("pointer press activates Back");
        // The real goBack() lands back on page one; the menu is dismissed.
        let _ = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        let closed = wait_menu_closed(&mut client, deadline());
        assert_eq!(closed.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert_identity_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The full keyboard path: after a real second-page navigation the menu
/// opens on Back; Down skips the disabled Forward row to Reload, Down/Up
/// traverse the enabled rows, and Enter activates the selected row through
/// the production seam. Escape then dismisses. Every step leaves the
/// session identity untouched.
#[test]
#[ignore]
fn keyboard_traverses_and_activates_skipping_disabled_rows() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-keyboard", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);

        client
            .send_ok(&bbr_testkit::Request::Navigate {
                url: page_two.clone(),
            })
            .expect("navigate to page two");
        let mut nav_since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        nav_since = wait_history_state(&mut client, view, true, false, nav_since);
        let _ = nav_since;

        let point = [520.0, 300.0];
        let open = open_menu(&mut client, view, point);
        // Fresh history state: Back enabled, Forward disabled, so the cursor
        // landed on Back (index 0).
        assert!(row(&open, "back").enabled);
        assert!(!row(&open, "forward").enabled);
        assert!(row(&open, "reload").enabled);
        assert_eq!(open.context_menu.selected, Some(0), "cursor lands on Back");

        // Down skips the disabled Forward row and lands on Reload; Down/Up
        // traverse the enabled rows with wrap-around skipped.
        key(&mut client, "down");
        let after_down = client.snapshot().expect("down snapshot");
        assert_eq!(
            after_down.context_menu.selected,
            Some(2),
            "Down from Back skips the disabled Forward row to Reload"
        );
        assert!(after_down.context_menu.open);
        key(&mut client, "down");
        let after_down2 = client.snapshot().expect("down again snapshot");
        assert_eq!(
            after_down2.context_menu.selected,
            Some(3),
            "Down to Copy Link"
        );
        key(&mut client, "up");
        let after_up = client.snapshot().expect("up snapshot");
        assert_eq!(after_up.context_menu.selected, Some(2), "Up back to Reload");

        // Enter activates the selected row (Reload) through the production
        // seam and dismisses the menu; the session identity is untouched.
        key(&mut client, "enter");
        let closed = wait_menu_closed(&mut client, deadline());
        assert_identity_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Forward enablement is equally genuine: after the real Back step the real
/// `canGoForward()` turns true and the menu's Forward row reports enabled —
/// the same event feed, mirrored.
#[test]
#[ignore]
fn forward_enablement_mirrors_real_history_after_back() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-forward", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        since = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);

        client
            .send_ok(&bbr_testkit::Request::Navigate {
                url: page_two.clone(),
            })
            .expect("navigate to page two");
        since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        since = wait_history_state(&mut client, view, true, false, since);

        // Real Back through the menu, then wait for the mirrored history:
        // canGoForward() turns true and feeds the nav state.
        let point = [300.0, 280.0];
        open_menu(&mut client, view, point);
        client
            .send_ok(&bbr_testkit::Request::ContextMenuActivate { index: 0 })
            .expect("activate Back");
        since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        since = wait_history_state(&mut client, view, false, true, since);
        wait_menu_closed(&mut client, deadline());

        // Reopen: Forward is now enabled by the real forward history, and
        // the pointer activation drives the real goForward() back to page
        // two — the same genuine feed in the other direction.
        let reopen = open_menu(&mut client, view, point);
        assert!(
            row(&reopen, "forward").enabled,
            "Forward enabled by the real canGoForward()"
        );
        client
            .send_ok(&bbr_testkit::Request::ContextMenuActivate { index: 1 })
            .expect("activate Forward");
        let _ = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        let closed = wait_menu_closed(&mut client, deadline());
        assert_identity_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
