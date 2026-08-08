//! Window-lifecycle hardening scenarios for the one native portal host
//! (task packet `native-portal-hardening`, queue item 14).
//!
//! These tests drive the real AppKit window-lifecycle events that reliably
//! break native overlays — minimize/restore, Window Server space-assignment
//! transitions, and the parent's close with an open presentation — and
//! prove the host stays correct or is cleanly torn down (never orphaned,
//! never occluding the web view, never intercepting input) through the same
//! owner-classified Window Server evidence the merged portal suite uses.
//! Fullscreen enter/leave is **environment-blocked on this host**: on the
//! macOS 27 Window Server, a programmatic `toggleFullScreen:` starts the
//! transition (`NSWindowWillEnterFullScreenNotification` + mask flip) but
//! never completes — `NSWindowDidEnterFullScreenNotification` never posts,
//! even for a plain standalone window (verified with an independent Swift
//! probe), so there is no deterministic completion signal and the mask-based
//! waits would converge on cancelled transitions. The host-side fullscreen
//! hardening (DidEnter/DidExit observers, re-pin, re-order) stays; the
//! scenario is documented as blocked in the task packet. Split from the
//! portal suite's main file so both stay under the ~500-line budget; every
//! step synchronizes on typed snapshot state under the standard deadline
//! (no sleeps/pixels).

use super::e2e_common::*;
use super::support::*;

/// Minimize/restore (native-portal-hardening): while the browser window is
/// miniaturized through the real AppKit path, the portal must be ordered out
/// (never occluding the desktop, never intercepting input) while the
/// presentation stays armed; restoring re-shows the same portal window,
/// re-pins the frame, and the host wins hit testing again — one host, no
/// session/view/profile/space/focus change.
#[test]
#[ignore]
fn host_hides_on_minimize_and_restores() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("portal-minimize", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        assert!(!settled.portal.parent_minimized);
        let first_view = settled.active_web_view.expect("first native view");

        client
            .send_ok(&dsl::portal_open_probe())
            .expect("open probe");
        flush_frame(&mut client);
        let open = client.snapshot().expect("open snapshot");
        assert_probe_open(&open);
        wait_for_frontmost_evidence(&mut client, deadline());
        let portal_window_number = open.portal.portal_window_number;

        // Minimize through the real AppKit path: the portal must be ordered
        // out — never visible, never owning hit testing, never key/main —
        // while the presentation stays armed (paused, not dismissed) and the
        // host stays attached.
        client
            .send_ok(&dsl::portal_set_minimized(true))
            .expect("minimize");
        let minimized = wait_portal_state(&mut client, deadline(), "minimize", |p| {
            p.parent_minimized && !p.native_visible
        });
        assert!(minimized.portal.parent_minimized);
        assert_eq!(
            minimized.portal.presentation,
            Some(PROBE_PRESENTATION_ID),
            "minimize pauses the presentation, never dismisses it"
        );
        assert!(
            !minimized.portal.native_visible,
            "the portal is ordered out while the parent is minimized"
        );
        assert!(
            !minimized.portal.portal_owns_probe_point,
            "a minimized portal must never own hit testing over the desktop"
        );
        assert!(
            minimized.portal.child_attached,
            "minimize must not detach or tear down the host"
        );
        assert!(!minimized.portal.portal_is_key && !minimized.portal.portal_is_main);
        assert_eq!(
            minimized.portal.portal_window_number, portal_window_number,
            "minimize must reuse the same portal window"
        );
        assert_ownership_preserved(&minimized, &settled);
        assert_eq!(minimized.active_web_view, Some(first_view));

        // Restore: the portal re-shows, re-pins, and wins hit testing again.
        client
            .send_ok(&dsl::portal_set_minimized(false))
            .expect("restore");
        let restored = wait_portal_state(&mut client, deadline(), "restore", |p| {
            !p.parent_minimized && p.native_visible && p.frame_synced
        });
        assert_probe_open(&restored);
        assert_eq!(
            restored.portal.portal_window_number, portal_window_number,
            "restore must reuse the same portal window"
        );
        wait_for_frontmost_evidence(&mut client, deadline());
        assert_ownership_preserved(&restored, &settled);
        assert_eq!(restored.active_web_view, Some(first_view));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Window Server space-assignment transition (native-portal-hardening):
/// setting/clearing the `CanJoinAllSpaces` collection behavior on the parent
/// makes the Window Server place the window (and its attached child) on
/// every space and back. The one host must stay pinned, attached, and
/// winning hit testing across the transition, with the portal always
/// agreeing with its parent on active-space membership. A real desktop
/// space switch is not drivable without private CGS APIs or user input on
/// this host (reported separately); the fullscreen test covers the genuine
/// space move to/from the fullscreen space, and this test covers the
/// space-assignment machinery itself.
#[test]
#[ignore]
fn host_follows_window_server_space_assignment() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("portal-spaces", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first native view");
        assert_active_space_agreement(&settled);

        client
            .send_ok(&dsl::portal_open_probe())
            .expect("open probe");
        flush_frame(&mut client);
        let open = client.snapshot().expect("open snapshot");
        assert_probe_open(&open);
        wait_for_frontmost_evidence(&mut client, deadline());
        let portal_window_number = open.portal.portal_window_number;
        assert_active_space_agreement(&open);

        // Join every space: the Window Server re-assigns the parent's space
        // membership (the attached child follows). The host must stay
        // correct — pinned, attached, owning hit testing.
        client
            .send_ok(&dsl::portal_set_all_spaces(true))
            .expect("join all spaces");
        let joined = wait_portal_state(&mut client, deadline(), "join all spaces", |p| {
            p.frame_synced && p.portal_owns_probe_point
        });
        assert_probe_open(&joined);
        assert_eq!(
            joined.portal.portal_window_number, portal_window_number,
            "space re-assignment must reuse the same portal window"
        );
        assert_active_space_agreement(&joined);
        wait_for_frontmost_evidence(&mut client, deadline());
        assert_ownership_preserved(&joined, &settled);
        assert_eq!(joined.active_web_view, Some(first_view));

        // Leave all spaces again: back to ordinary single-space membership.
        client
            .send_ok(&dsl::portal_set_all_spaces(false))
            .expect("leave all spaces");
        let back = wait_portal_state(&mut client, deadline(), "leave all spaces", |p| {
            p.frame_synced && p.portal_owns_probe_point
        });
        assert_probe_open(&back);
        assert_eq!(
            back.portal.portal_window_number, portal_window_number,
            "leaving all spaces must reuse the same portal window"
        );
        assert_active_space_agreement(&back);
        wait_for_frontmost_evidence(&mut client, deadline());
        assert_ownership_preserved(&back, &settled);
        assert_eq!(back.active_web_view, Some(first_view));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Window close with an open presentation (native-portal-hardening): the
/// parent closes through the real AppKit close path while the probe is
/// presented. The Window Server (queried from the test process) must show
/// zero BBR windows once the close lands — the portal is never left
/// orphaned, occluding the desktop, or intercepting input — the app stays
/// healthy (macOS quit-mode is Explicit: a zero-window app keeps running),
/// and the gateway still answers so the test quits cleanly.
#[test]
#[ignore]
fn host_tears_down_when_parent_closes_with_open_presentation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("portal-close", &page_one);
    let pid = harness.child.id() as i32;

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first native view");

        client
            .send_ok(&dsl::portal_open_probe())
            .expect("open probe");
        flush_frame(&mut client);
        let open = client.snapshot().expect("open snapshot");
        assert_probe_open(&open);
        wait_for_frontmost_evidence(&mut client, deadline());
        assert_ownership_preserved(&open, &settled);
        assert_eq!(open.active_web_view, Some(first_view));

        // The Window Server sees both the browser window and the portal on
        // screen before the close (real cross-process query).
        let on_screen = bbr_web_macos::window_server::onscreen_window_count(pid);
        assert!(
            on_screen >= 2,
            "parent + portal must both be on screen before the close (found {on_screen})"
        );

        // Close the browser window through the real AppKit close path while
        // the portal presentation is open.
        client
            .send_ok(&dsl::portal_close_window())
            .expect("close browser window");

        // The host must tear down deterministically: no BBR window may
        // remain on screen — the portal is never left orphaned, never
        // occluding the desktop, never intercepting input.
        let remaining = wait_no_onscreen_windows(pid, deadline());
        assert_eq!(remaining, 0, "no BBR window may survive the close");

        // The app stays alive and healthy (macOS quit-mode Explicit keeps a
        // zero-window app running): the gateway still answers and the app
        // quits cleanly when asked.
        client
            .send_ok(&bbr_testkit::Request::Ping)
            .expect("gateway stays alive after the window closes");
        client.quit().expect("quit after close");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit after close: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
