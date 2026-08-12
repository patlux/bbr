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
//! - Copy Page Link writes the page URL to the real system pasteboard (read
//!   back through `pbpaste`), the enriched menu renders exactly its owned
//!   rows (never unowned Select All / Share / Save / Source / Inspect or the
//!   retired Open in New Tab), and the B2-gated Bookmark Page / Remove
//!   Bookmark row toggles through Cmd-D's one app-owned command authority
//!   from a unique unsaved fixture;
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

/// Copy Page Link writes the page URL to the real system pasteboard
/// through the engine's `WebAction::CopyLink` (read back via `pbpaste`) —
/// the one action that reaches outside the menu chrome. The enriched menu
/// owns exactly the rows it renders: no Open in New Tab, no Select All, no
/// Share, no Save/Source/Inspect (no capability → the row never appears).
#[test]
#[ignore]
fn copy_link_writes_the_real_pasteboard_and_unowned_rows_never_render() {
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
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);

        // Copy Page Link: the menu's page URL is written to the general
        // pasteboard (the real engine owns the platform pasteboard).
        let point = [340.0, 320.0];
        let open = open_menu(&mut client, view, point);
        let menu_url = open.context_menu.url.clone().expect("menu page URL");
        assert_eq!(menu_url, page_one, "the menu URL is the page URL");
        assert_eq!(
            open.context_menu.title.as_deref(),
            Some(PAGE_ONE_TITLE),
            "the menu captures the focused page title for B2"
        );
        client
            .send_ok(&bbr_testkit::Request::ContextMenuActivate { index: 4 })
            .expect("activate Copy Page Link");
        wait_menu_closed(&mut client, deadline());
        let pasted = read_pasteboard();
        assert_eq!(
            pasted.as_deref(),
            Some(page_one.as_str()),
            "Copy Page Link wrote the page URL to the system pasteboard (got {pasted:?})"
        );
        assert_identity_preserved(&client.snapshot().expect("copy snapshot"), &settled);

        // Unowned capabilities never render rows: the enriched menu carries
        // exactly Back / Forward / Reload / Bookmark / Copy Page Link —
        // never Select All (no WebAction), Share (no native adapter), Save
        // Page As / View Page Source / Inspect (no integrated capability),
        // or the retired Open in New Tab row.
        let reopen = open_menu(&mut client, view, point);
        let keys: Vec<&str> = reopen
            .context_menu
            .items
            .iter()
            .map(|item| item.key.as_str())
            .collect();
        assert_eq!(keys, crate::support::MENU_ROW_KEYS.to_vec());
        for forbidden in [
            "select-all",
            "share",
            "save",
            "view-source",
            "inspect",
            "open-new-tab",
        ] {
            assert!(
                !keys.contains(&forbidden),
                "the unowned {forbidden:?} row never renders"
            );
        }
        client
            .send_ok(&bbr_testkit::Request::ContextMenuClose)
            .expect("close");
        let closed = wait_menu_closed(&mut client, deadline());
        assert_identity_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The B2 bookmark row is capability-gated and shares Cmd-D's one app-owned
/// command authority: on an unsaved page the row reads `Bookmark Page` and
/// saving through it flips the row to `Remove Bookmark` (the record exists,
/// `saved_pages_count == 1`); a subsequent Cmd-D (the keyboard seam) removes
/// it through the same authority, and the row reads `Bookmark Page` again.
/// The page/session identity never changes and WebKit-native element-menu
/// ownership is untouched (the native diagnostics counters stay zero — the
/// harness drives the BBR open path, never a real WebKit menu).
#[test]
#[ignore]
fn bookmark_row_gates_and_toggles_with_cmd_d_parity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, base_page_one, _page_two) = fixtures();
    // The bookmark scenario must start from a unique, unsaved page: a
    // leftover harness profile from a previous crashed run (pid reuse)
    // can already hold a Saved Page record for the shared fixture URL,
    // which would flip the row to `Remove Bookmark` before the test
    // begins. A per-run query string keeps the fixture content/title
    // identical while making the canonical URL unique to this run (the
    // canonical seam preserves query strings byte-for-byte), so the store
    // genuinely starts empty and the row reads `Bookmark Page`.
    let run_nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    let page_one = format!("{}?run={}-{run_nonce}", base_page_one, std::process::id());
    let (mut harness, socket, token) = spawn_harness("menu-bookmark", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);
        assert_eq!(
            client
                .snapshot()
                .expect("store snapshot")
                .store
                .saved_pages_count,
            0,
            "the fixture page starts unsaved"
        );

        // An unsaved page offers `Bookmark Page`, enabled, bound to the
        // focused page/session.
        let point = [360.0, 340.0];
        let open = open_menu(&mut client, view, point);
        let bookmark = row(&open, "bookmark");
        assert_eq!(bookmark.title, "Bookmark Page");
        assert!(bookmark.enabled);
        assert_eq!(bookmark.id, "bbr-page-menu-bookmark");
        assert_eq!(open.context_menu.url.as_deref(), Some(page_one.as_str()));

        // Activating the row dispatches B2's one app-owned command (the
        // same `SavedPageCommand::toggle` authority Cmd-D uses): the record
        // appears and the menu dismisses with the session untouched.
        client
            .send_ok(&bbr_testkit::Request::ContextMenuActivate { index: 3 })
            .expect("activate Bookmark Page");
        let saved = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            saved.store.saved_pages_count, 1,
            "the menu row saved exactly one record through B2"
        );
        assert_identity_preserved(&saved, &settled);

        // Reopen: the row now reads `Remove Bookmark` (the B2 state gate).
        let reopen = open_menu(&mut client, view, point);
        let bookmark = row(&reopen, "bookmark");
        assert_eq!(bookmark.title, "Remove Bookmark");
        assert!(bookmark.enabled);

        // Cmd-D parity: the keyboard seam removes through the same
        // authority, and the row flips back to `Bookmark Page`. The menu is
        // still presented while Cmd-D fires (the row model is built at
        // open, so the open menu keeps its pre-toggle state), so it is
        // dismissed through the production close seam before the next real
        // open rebuilds the row from the B2 state.
        key(&mut client, "cmd-d");
        let removed = wait_saved_pages_count(&mut client, 0);
        assert_identity_preserved(&removed, &settled);
        client
            .send_ok(&bbr_testkit::Request::ContextMenuClose)
            .expect("close after cmd-d");
        wait_menu_closed(&mut client, deadline());
        let final_open = open_menu(&mut client, view, point);
        assert_eq!(row(&final_open, "bookmark").title, "Bookmark Page");
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

/// The menu binds to the focused pane/active view only: an injected
/// `ContextMenuRequested` for a stale/unknown view id is a no-op — the
/// presentation never opens and the session identity is untouched. This is
/// the app-side half of the focused-pane source-binding contract (the
/// portal/placement evidence is the open path over the real active view).
#[test]
#[ignore]
fn request_for_a_non_active_view_is_a_no_op() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menu-non-active", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, PAGE_ONE_TITLE, since);

        // A stale view id (never created in this session) must never open
        // the BBR menu: the menu is chrome over the focused pane only.
        let stale = bbr_web::WebViewId(view.0 + 1000);
        let point = [300.0, 300.0];
        client
            .send_ok(&bbr_testkit::Request::InjectWebEvent {
                event: WebEvent::ContextMenuRequested(
                    stale,
                    bbr_web::WebRect::new(point[0], point[1], 0.0, 0.0),
                ),
            })
            .expect("inject stale context menu request");
        flush_frame(&mut client);
        let snapshot = client.snapshot().expect("stale request snapshot");
        assert!(
            !snapshot.context_menu.open,
            "a request for a non-active view never opens the menu"
        );
        assert_eq!(snapshot.portal.presentation, None, "no portal presentation");
        assert_identity_preserved(&snapshot, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Poll until `store.saved_pages_count` equals `expected` (the Cmd-D parity
/// step synchronizes on the durable store state, not a fixed sleep).
fn wait_saved_pages_count(client: &mut Client, expected: usize) -> bbr_testkit::protocol::Snapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("saved-pages count snapshot");
        if snapshot.store.saved_pages_count == expected {
            return snapshot;
        }
        assert!(Instant::now() < until, "saved_pages_count != {expected}");
        std::thread::yield_now();
    }
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
