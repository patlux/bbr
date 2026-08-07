//! NativeAboveWeb boundary evidence for the U1 command surfaces.
//!
//! Hidden command scenarios assert the deterministic child-window contract on
//! every central palette/overlay snapshot. One focused shown-window scenario
//! adds a real AppKit Window Server query at the centered palette point over a
//! real WKWebView. No pixel comparisons or sleeps.

use super::*;
use bbr_testkit::protocol::{CommandSurface, Snapshot};
use std::path::PathBuf;

pub const PALETTE_PRESENTATION_ID: u64 = 0xC1;
pub const OVERLAY_PRESENTATION_ID: u64 = 0xC2;

pub fn assert_command_portal_open(snapshot: &Snapshot, surface: CommandSurface) {
    let (presentation, id) = match surface {
        CommandSurface::Palette => (PALETTE_PRESENTATION_ID, "bbr-palette"),
        CommandSurface::Overlay => (OVERLAY_PRESENTATION_ID, "bbr-command-overlay"),
        CommandSurface::None => panic!("closed command surface has no open portal evidence"),
    };
    assert_eq!(snapshot.command_surface, surface);
    let portal = &snapshot.portal;
    assert_eq!(portal.presentation, Some(presentation));
    assert_eq!(portal.host.as_deref(), Some("native-above-web"));
    assert!(portal.visible, "logical portal visibility");
    assert_eq!(portal.ids, vec![id.to_string()]);
    assert!(
        portal.rect.is_some(),
        "the command card has resolved geometry"
    );
    assert!(
        portal.child_attached,
        "portal remains a child of the browser"
    );
    assert!(portal.native_visible, "AppKit orders the portal window in");
    assert!(portal.frame_synced, "portal frame follows parent content");
    assert!(portal.nonactivating_panel, "portal cannot activate itself");
    assert!(portal.parent_window_number > 0);
    assert!(portal.portal_window_number > 0);
    assert_ne!(portal.parent_window_number, portal.portal_window_number);
    assert!(!portal.portal_is_key, "command portal never becomes key");
    assert!(!portal.portal_is_main, "command portal never becomes main");
}

pub fn assert_command_portal_closed(snapshot: &Snapshot, window_number: i64) {
    let portal = &snapshot.portal;
    assert_eq!(portal.presentation, None);
    assert_eq!(portal.host, None);
    assert!(!portal.visible);
    assert_eq!(portal.rect, None);
    assert!(portal.ids.is_empty());
    assert!(
        portal.child_attached,
        "the one host remains attached for reuse"
    );
    assert!(!portal.native_visible, "closed portal is ordered out");
    assert!(portal.frame_synced);
    assert_eq!(portal.portal_window_number, window_number);
    assert!(!portal.portal_is_key);
    assert!(!portal.portal_is_main);
}

fn spawn_harness_shown(root_tag: &str, page_one: &str) -> (Harness, PathBuf, String) {
    let root = std::env::temp_dir().join(format!("bbr-st-{root_tag}-{}", std::process::id()));
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = profile.join("bbr-test.sock");
    let token = format!("shell-{root_tag}-{}", std::process::id());
    let bin = env!("CARGO_BIN_EXE_bbr");
    let harness = Harness::spawn(
        bin,
        &[
            "--bbr-initial-url",
            page_one,
            "--bbr-profile-dir",
            profile.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
        ],
        &root,
    );
    (harness, socket, token)
}

fn wait_for_frontmost_evidence(client: &mut Client) -> Snapshot {
    let until = deadline();
    loop {
        client
            .send_ok(&dsl::activate_app())
            .expect("activate shown BBR app");
        let snapshot = client.snapshot().expect("command portal evidence snapshot");
        let portal = &snapshot.portal;
        let absolute_bbr = portal.frontmost_at_probe_point == Some(portal.portal_window_number)
            || portal.frontmost_at_probe_point == Some(portal.parent_window_number);
        let relative_parent =
            portal.below_portal_at_probe_point == Some(portal.parent_window_number);
        if portal.portal_owns_probe_point && (absolute_bbr || relative_parent) {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "AppKit did not corroborate command portal ordering: frontmost {:?}, below {:?}, parent {}, portal {}, owns {}",
            portal.frontmost_at_probe_point,
            portal.below_portal_at_probe_point,
            portal.parent_window_number,
            portal.portal_window_number,
            portal.portal_owns_probe_point
        );
        std::thread::yield_now();
    }
}

#[test]
#[ignore]
fn palette_is_window_server_frontmost_over_the_real_webview() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("cmd-portal", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let baseline = wait_for_window_settled(&mut client, deadline());
        let first_view = baseline.active_web_view.expect("real native view");
        if baseline.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        assert_eq!(baseline.engine_kind, "native-wkwebview");
        assert!(baseline.web_view_hosted);

        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush palette frame");
        let palette = wait_for_frontmost_evidence(&mut client);
        assert_command_portal_open(&palette, CommandSurface::Palette);
        assert!(
            palette.portal.portal_owns_probe_point,
            "the child portal owns the centered point above WKWebView"
        );
        assert_eq!(palette.active_web_view, Some(first_view));
        assert_eq!(palette.engine_sessions, baseline.engine_sessions);
        let window_number = palette.portal.portal_window_number;

        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        let closed = client.snapshot().expect("closed portal snapshot");
        assert_eq!(closed.command_surface, CommandSurface::None);
        assert_command_portal_closed(&closed, window_number);
        assert_eq!(closed.active_web_view, Some(first_view));
        assert_eq!(closed.engine_sessions, baseline.engine_sessions);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
