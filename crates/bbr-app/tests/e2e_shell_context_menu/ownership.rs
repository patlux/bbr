//! Ownership + isolation scenarios (task packet `bbr-context-menus-over-web`).
//!
//! Proves the native-boundary contract and the no-orphan rule:
//!
//! - the real-app bridge is installed and observing right-clicks (typed
//!   diagnostics: monitor installed, counters live) while the BBR open path
//!   never touches WebKit's ownership — page-vs-element classification is
//!   deterministic and unit-proven in the adapter (page menus are
//!   BBR-owned; link/image/selection menus stay WebKit-owned and are never
//!   cancelled);
//! - Copy Link writes the page URL to the real system pasteboard (read back
//!   through `pbpaste`) and Open in New Tab opens exactly one new
//!   tab/session;
//! - resizing the parent while the menu is open re-pins the ONE portal host
//!   (same portal window, frame synced, menu state intact) and every
//!   dismissal leaves no orphan (host back to its typed closed default).

use std::process::Command;

use crate::e2e_common::*;
use crate::support::*;

/// The real app attaches the native context-menu bridge: the right-click
/// monitor is installed and the ownership counters exist. The BBR open path
/// (the injected `ContextMenuRequested`) is pure chrome — it never bumps
/// the ownership counters, because those move only when a REAL right-click
/// over a live web view is classified (element menus stay WebKit-owned and
/// are never cancelled; the deterministic classification is unit-proven in
/// the adapter).
#[test]
#[ignore]
fn native_bridge_is_installed_and_bbr_open_path_is_pure_chrome() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-native", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);

        let native = &settled.context_menu.native;
        assert!(
            native.monitor_installed,
            "the real app installs the right-click monitor"
        );
        assert_eq!(
            native.right_clicks_over_web, 0,
            "no synthetic right-clicks in the harness"
        );
        assert_eq!(native.page_owned, 0);
        assert_eq!(native.element_owned, 0);

        // The BBR open path is pure chrome: opening and closing the menu
        // never moves the ownership counters (they classify real WebKit
        // menus, which the harness does not synthesize).
        let point = [320.0, 310.0];
        let open = open_menu(&mut client, view, point);
        let native_open = &open.context_menu.native;
        assert_eq!(native_open.right_clicks_over_web, 0);
        assert_eq!(native_open.page_owned, 0);
        assert_eq!(native_open.element_owned, 0);
        client
            .send_ok(&bbr_testkit::Request::ContextMenuClose)
            .expect("close");
        let closed = wait_menu_closed(&mut client, deadline());
        assert_eq!(closed.context_menu.native.right_clicks_over_web, 0);
        assert_identity_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Copy Link writes the page URL to the real system pasteboard through the
/// engine's `WebAction::CopyLink` (read back via `pbpaste`), and Open in
/// New Tab opens exactly one new tab/session with the menu's page URL —
/// the two actions that reach outside the menu chrome.
#[test]
#[ignore]
fn copy_link_writes_the_real_pasteboard_and_open_new_tab_opens_exactly_one() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-actions", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);

        // Copy Link: the menu's page URL is written to the general
        // pasteboard (the real engine owns the platform pasteboard).
        let point = [340.0, 320.0];
        let open = open_menu(&mut client, view, point);
        let menu_url = open.context_menu.url.clone().expect("menu page URL");
        assert_eq!(menu_url, page_one, "the menu URL is the page URL");
        client
            .send_ok(&bbr_testkit::Request::ContextMenuActivate { index: 3 })
            .expect("activate Copy Link");
        wait_menu_closed(&mut client, deadline());
        let pasted = read_pasteboard();
        assert_eq!(
            pasted.as_deref(),
            Some(page_one.as_str()),
            "Copy Link wrote the page URL to the system pasteboard (got {pasted:?})"
        );
        assert_identity_preserved(&client.snapshot().expect("copy snapshot"), &settled);

        // Open in New Tab: exactly one new tab/session with the menu URL,
        // and the menu is dismissed.
        let reopen = open_menu(&mut client, view, point);
        assert!(row(&reopen, "open-new-tab").enabled);
        let before = client.snapshot().expect("before open-new-tab");
        client
            .send_ok(&bbr_testkit::Request::ContextMenuActivate { index: 4 })
            .expect("activate Open in New Tab");
        let closed = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            closed.tabs.len(),
            before.tabs.len() + 1,
            "Open in New Tab opens exactly one new tab"
        );
        assert_eq!(
            closed.engine_sessions,
            before.engine_sessions + 1,
            "exactly one new session"
        );
        assert_ne!(closed.active_web_view, Some(view));
        let new_view = closed.active_web_view.expect("new tab view");
        if closed.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, new_view, PAGE_ONE_TITLE, since);
        }
        let _ = since;
        assert_eq!(closed.active_profile, settled.active_profile);
        assert_eq!(closed.active_space, settled.active_space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Resizing the parent while the menu is open re-pins the ONE portal host:
/// the same portal window number is reused (never recreated), the frame
/// stays synced to the new parent content rect, and the menu stays open
/// focused over the same view with the session untouched. Closing after the
/// resize leaves no orphan: the host returns to its typed closed default.
#[test]
#[ignore]
fn resize_repins_the_one_host_and_close_leaves_no_orphan() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-resize", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);
        let baseline_w = settled.window_size[0];
        let baseline_h = settled.window_size[1];

        let point = [260.0, 280.0];
        let open = open_menu(&mut client, view, point);
        let portal_window_number = open.portal.portal_window_number;
        let menu_rect = open.portal.rect.expect("menu rect");
        assert!(portal_window_number > 0);

        // Shrink the window (still comfortably larger than the menu card):
        // the portal frame follows, the portal window is reused, the menu
        // keeps its state and identity.
        let new_w = baseline_w - 220.0;
        let new_h = baseline_h - 140.0;
        assert!(
            new_w > menu_rect[0] + MENU_WIDTH + MENU_MARGIN,
            "resized window still fits the open menu"
        );
        client
            .send_ok(&bbr_testkit::Request::SetWindowSize {
                width: new_w,
                height: new_h,
            })
            .expect("resize parent");
        flush_frame(&mut client);
        let resized = loop {
            let snapshot = client.snapshot().expect("resize snapshot");
            if (snapshot.window_size[0] - new_w).abs() <= 0.5 && snapshot.portal.frame_synced {
                break snapshot;
            }
            assert!(
                Instant::now() < deadline(),
                "parent resize did not land; window {:?}, frame_synced {}",
                snapshot.window_size,
                snapshot.portal.frame_synced
            );
            std::thread::yield_now();
        };
        assert!(
            (resized.window_size[1] - baseline_h).abs() > 1.0,
            "the resize must change the window height"
        );
        assert_eq!(
            resized.portal.portal_window_number, portal_window_number,
            "resize reuses the same portal window, never recreates it"
        );
        assert!(resized.portal.child_attached, "host stays attached");
        assert!(resized.context_menu.open, "menu stays open");
        assert!(resized.context_menu.focused, "menu keeps keyboard focus");
        assert_eq!(resized.context_menu.bound_view, Some(view));
        assert_identity_preserved(&resized, &settled);

        // Close after the resize: the host returns to its typed closed
        // default — no orphan window, no stale presentation, no ids.
        client
            .send_ok(&bbr_testkit::Request::ContextMenuClose)
            .expect("close after resize");
        let closed = wait_menu_closed(&mut client, deadline());
        assert_eq!(closed.portal.portal_window_number, portal_window_number);
        assert_identity_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Read the system general pasteboard through `pbpaste` (the macOS CLI).
fn read_pasteboard() -> Option<String> {
    let output = Command::new("pbpaste").output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
    )
}
