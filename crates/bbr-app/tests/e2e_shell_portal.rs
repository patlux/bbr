//! Real-app native portal host E2E (task packet `native-portal-host`).
//!
//! Launches the real `bbr` app with a real WKWebView and proves the one
//! implementation-owned native portal host end to end through the typed
//! testkit protocol (DESIGN.md §13/§15, task packet evidence section):
//!
//! - the host is created hidden and non-activating at window construction
//!   (no standalone flash: hidden state is typed, `native_visible` false);
//! - opening the production probe yields `NativeAboveWeb` state, child
//!   attachment above the browser window, exact parent-content frame sync,
//!   and AppKit frontmost hit-test ownership at the probe card's point over
//!   the real WKWebView — while the active WebViewId/session/profile/space/
//!   focus stay untouched (no session creation, copy, move, or replacement);
//! - resizing the parent while the probe is open re-pins the native frame
//!   without recreating the portal window or any web session;
//! - the production outside-dismiss seam (one outside pointer action, §13)
//!   orders the host out, preserves the original state, and a fresh
//!   presentation re-opens with the same stable ids.
//!
//! No sleeps/pixels: every step synchronizes on typed snapshot state under a
//! deadline. The shared harness lives in `e2e_common`.

#![cfg(target_os = "macos")]

mod e2e_common;

#[path = "e2e_shell_portal/support.rs"]
mod support;

#[path = "e2e_shell_portal/activation.rs"]
mod activation;

#[path = "e2e_shell_portal/hardening.rs"]
mod hardening;

use bbr_testkit::protocol::dsl;
use e2e_common::*;
use support::*;

/// Regression for Patrick's intermittent manual report: while the portal is
/// idle, its retained native child window must remain ordered out and must not
/// intercept the content point. Three independent cold app runs open a second
/// real WKWebView, navigate it, and repeatedly switch between both sessions;
/// every presentation is corroborated by WebKit title events plus an AppKit
/// Window Server hit-test that either names the browser parent or provably
/// belongs to another process (an environmental overlay), never the portal.
/// The idle acceptance classifies the raw absolute frontmost answer with the
/// measured `frontmost_at_probe_point_is_this_app` fact, so a foreign
/// always-on-top window over the probe point is proven foreign instead of
/// failing the run (see [`wait_for_idle_parent_ownership`]).
#[test]
#[ignore]
fn idle_portal_never_occludes_or_intercepts_repeated_web_presentations() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    for cold_run in 0..3 {
        let tag = format!("portal-idle-{cold_run}");
        let (mut harness, socket, token) = spawn_harness_shown(&tag, &page_one);

        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut client = connect(&socket);
            client.hello(&token).expect("hello");
            let replay = client.attach(None).expect("attach");
            let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

            let initial = wait_for_window_settled(&mut client, deadline());
            let first_view = initial.active_web_view.expect("first native view");
            if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
                since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
            }
            let idle = wait_for_idle_parent_ownership(&mut client, deadline());
            assert_eq!(idle.active_web_view, Some(first_view));
            assert_eq!(idle.engine_sessions, 1);
            assert!(idle.web_view_hosted);

            let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
            assert_ne!(second_view, first_view);
            let opened = wait_for_idle_parent_ownership(&mut client, deadline());
            assert_eq!(opened.active_web_view, Some(second_view));
            assert_eq!(opened.engine_sessions, 2);
            assert_eq!(opened.title.as_deref(), Some(PAGE_TWO_TITLE));

            // Navigate the active second view through the production engine
            // path; the title event proves WebKit received and presented it
            // while the idle portal stayed out of the event path.
            client
                .send_ok(&dsl::navigate(&page_one))
                .expect("navigate second view");
            since = wait_title(&mut client, second_view, PAGE_ONE_TITLE, since);
            let navigated = wait_for_idle_parent_ownership(&mut client, deadline());
            assert_eq!(navigated.active_web_view, Some(second_view));
            assert_eq!(navigated.title.as_deref(), Some(PAGE_ONE_TITLE));

            for cycle in 0..4 {
                client
                    .send_ok(&dsl::select_tab(0))
                    .expect("select first tab");
                since = wait_browser_change(
                    &mut client,
                    BrowserChange::TabSelected { index: 0 },
                    since,
                );
                let first = wait_for_idle_parent_ownership(&mut client, deadline());
                assert_eq!(first.active_web_view, Some(first_view), "cycle {cycle}");
                assert_eq!(first.engine_sessions, 2);

                client
                    .send_ok(&dsl::select_tab(1))
                    .expect("select second tab");
                since = wait_browser_change(
                    &mut client,
                    BrowserChange::TabSelected { index: 1 },
                    since,
                );
                let second = wait_for_idle_parent_ownership(&mut client, deadline());
                assert_eq!(second.active_web_view, Some(second_view), "cycle {cycle}");
                assert_eq!(second.engine_sessions, 2);
            }

            client.quit().expect("quit");
            let status =
                wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
            assert!(status.success(), "app exit: {status}");
        }));

        finish(result, &mut harness, &socket, &token);
    }
}

/// The host is created hidden at window construction and the production
/// probe opens above a real WKWebView: `NativeAboveWeb` state, child
/// attachment, exact frame sync, AppKit frontmost ownership at the probe
/// point, and untouched browser ownership/session/focus. This test uses a
/// SHOWN window because the frontmost hit-test evidence requires an
/// on-screen parent (see [`spawn_harness_shown`]).
#[test]
#[ignore]
fn host_starts_hidden_and_probe_opens_above_real_webview() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("portal-open", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // The AppKit frame clamp / yabai tile adjustment settles first (see
        // e2e_common), so the expected centered rect is derived from the
        // real settled size.
        let settled = wait_for_window_settled(&mut client, deadline());
        let w = settled.window_size[0];
        let h = settled.window_size[1];
        assert!(
            w > PROBE_WIDTH + 16.0 && h > PROBE_HEIGHT + 16.0,
            "the spawned window must be larger than the probe card"
        );
        let first_view = settled.active_web_view.expect("first native view");
        assert!(first_view.0 > 0, "a real WKWebView backs the active tab");
        assert_eq!(settled.engine_kind, "native-wkwebview");
        assert!(settled.web_view_hosted);

        // Initial host: created hidden at window construction. Typed
        // evidence that the host never flashed as a standalone window: the
        // portal window exists and is attached, but nothing is visible and
        // no portal surface is rendered.
        assert_host_closed(&settled);
        assert!(
            !settled
                .visible_ids
                .iter()
                .any(|id| id == "bbr-portal-probe"),
            "no portal surface ids in the main window render while closed"
        );

        // Open the production probe through the general Portal API seam.
        client
            .send_ok(&dsl::portal_open_probe())
            .expect("open probe");
        flush_frame(&mut client);
        let open = wait_probe_open(&mut client, deadline());
        // The probe card must resolve centered from the present-time window:
        // the rect is frozen at present and the snapshot observes the same
        // geometry, so the expectation is derived from this snapshot's own
        // window size (a shown window can be re-tiled between the earlier
        // settle and the present, which would make a settle-derived
        // expectation stale).
        assert_eq!(
            open.portal.rect,
            Some(centered_rect(open.window_size[0], open.window_size[1])),
            "the probe card resolves centered from the present-time window"
        );
        wait_for_frontmost_evidence(&mut client, deadline());
        assert_ownership_preserved(&open, &settled);

        // A second present refreshes in place: same presentation id, same
        // window, still exactly one presentation.
        client
            .send_ok(&dsl::portal_open_probe())
            .expect("re-present probe");
        flush_frame(&mut client);
        let refreshed = wait_probe_open(&mut client, deadline());
        wait_for_frontmost_evidence(&mut client, deadline());
        assert_eq!(
            refreshed.portal.presentation,
            Some(PROBE_PRESENTATION_ID),
            "re-presenting the probe keeps its stable id"
        );
        assert_ownership_preserved(&refreshed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Resizing the parent while the probe is open re-pins the native portal
/// frame to the new content rect without recreating the portal window or
/// any web session: window size changes, `frame_synced` stays true, the
/// portal keeps hit-test ownership, and the active WebViewId/session stay
/// stable.
#[test]
#[ignore]
fn portal_follows_parent_resize_without_session_recreation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("portal-resize", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first native view");
        let baseline_w = settled.window_size[0];
        let baseline_h = settled.window_size[1];

        client
            .send_ok(&dsl::portal_open_probe())
            .expect("open probe");
        flush_frame(&mut client);
        let open = wait_probe_open(&mut client, deadline());
        let portal_window_number = open.portal.portal_window_number;
        assert!(portal_window_number > 0);
        assert_ownership_preserved(&open, &settled);

        // Resize the browser window while the probe is open (a material
        // delta, still larger than the card so the centered rect stays
        // interior). The portal re-pins to the new content rect through the
        // AppKit observer; the probe card itself keeps its present-time
        // position (documented bounded probe behavior) — the frame-sync
        // evidence is the native frame, not the card offset. The harness is
        // hidden here: a shown window can be re-tiled by the window manager
        // mid-resize, so the deterministic hidden bounds are the stable
        // resize surface. Frontmost hit-test ownership is not asserted in
        // hidden mode (it needs an on-screen parent; see
        // [`assert_frontmost_ownership`]).
        let new_w = baseline_w - 220.0;
        let new_h = baseline_h - 140.0;
        assert!(
            new_w > PROBE_WIDTH + 16.0 && new_h > PROBE_HEIGHT + 16.0,
            "resized window still larger than the probe card"
        );
        client
            .send_ok(&bbr_testkit::Request::SetWindowSize {
                width: new_w,
                height: new_h,
            })
            .expect("resize parent");
        flush_frame(&mut client);

        // Wait for the window to reach the new size, then assert the portal
        // followed: frame still pinned to the (new) content rect, still
        // visible, same portal window (no recreation), and no
        // session/view/profile/space/focus change. The snapshot's
        // `window_size` is the GPUI window bounds, which include the macOS
        // titlebar, so the width is the exact resize signal (no vertical
        // titlebar delta, same convention as the sidebar suites); the
        // height-change contract is asserted separately below.
        let deadline = deadline();
        let resized = loop {
            let snapshot = client.snapshot().expect("resize snapshot");
            if (snapshot.window_size[0] - new_w).abs() <= 0.5
                && snapshot.portal.frame_synced
                && portal_visibility_settled(&snapshot.portal)
            {
                break snapshot;
            }
            assert!(
                Instant::now() < deadline,
                "parent resize did not land; window {:?}, portal frame_synced {} (target width [{new_w}] from baseline [{baseline_w}, {baseline_h}])",
                snapshot.window_size,
                snapshot.portal.frame_synced
            );
            std::thread::yield_now();
        };
        assert!(
            (resized.window_size[1] - baseline_h).abs() > 1.0,
            "the resize must change the window height (window {:?})",
            resized.window_size
        );
        assert_probe_open(&resized);
        assert_eq!(
            resized.portal.portal_window_number, portal_window_number,
            "resize must reuse the same portal window, never recreate it"
        );
        assert_eq!(
            resized.active_web_view,
            Some(first_view),
            "resize must never recreate the web session"
        );
        assert_eq!(resized.engine_sessions, settled.engine_sessions);
        assert_ownership_preserved(&resized, &settled);
        // The probe card keeps its present-time position (bounded probe
        // behavior, see crate::portal::probe module docs): the rect from
        // the snapshot is the original present-time resolution, while the
        // native frame evidence above proves the host followed.
        assert_eq!(resized.portal.rect, open.portal.rect);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The production outside-dismiss seam (DESIGN §13 one-outside-pointer-
/// action) orders the native host out and preserves the original state; a
/// fresh presentation re-opens with the same stable ids, and the explicit
/// close seam dismisses again.
#[test]
#[ignore]
fn outside_dismiss_orders_host_out_and_preserves_state() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("portal-dismiss", &page_one);

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
        let open = wait_probe_open(&mut client, deadline());
        let portal_window_number = open.portal.portal_window_number;

        // One outside pointer action (the exact production method the
        // backdrop press drives): the host orders out and the presentation
        // clears, while the browser state stays untouched.
        client
            .send_ok(&dsl::portal_outside_dismiss())
            .expect("outside dismiss");
        flush_frame(&mut client);
        let dismissed = wait_host_closed(&mut client, deadline());
        assert_eq!(
            dismissed.portal.portal_window_number, portal_window_number,
            "the same portal window is reused after dismissal"
        );
        assert_ownership_preserved(&dismissed, &settled);
        assert_eq!(dismissed.active_web_view, Some(first_view));

        // A fresh presentation re-opens with the same stable id and window.
        client
            .send_ok(&dsl::portal_open_probe())
            .expect("re-open probe");
        flush_frame(&mut client);
        let reopened = wait_probe_open(&mut client, deadline());
        assert_eq!(
            reopened.portal.portal_window_number, portal_window_number,
            "re-open must reuse the one portal window"
        );
        assert_eq!(reopened.portal.presentation, Some(PROBE_PRESENTATION_ID));
        assert_ownership_preserved(&reopened, &settled);

        // The explicit close seam dismisses identically.
        client
            .send_ok(&dsl::portal_close_probe())
            .expect("close probe");
        flush_frame(&mut client);
        let closed = wait_host_closed(&mut client, deadline());
        assert_ownership_preserved(&closed, &settled);

        // Deterministic teardown of the one native portal host (task packet
        // teardown evidence): the child relation is removed, the window is
        // ordered out and closed, and the bridge is dropped — the snapshot
        // afterwards reports the empty/closed host, while the browser
        // window and its session stay untouched.
        client
            .send_ok(&dsl::portal_teardown())
            .expect("teardown host");
        let torn_down = client.snapshot().expect("teardown snapshot");
        assert_eq!(
            torn_down.portal.presentation, None,
            "no presentation survives teardown"
        );
        assert_eq!(torn_down.portal.host, None);
        assert!(!torn_down.portal.visible);
        assert!(torn_down.portal.ids.is_empty());
        assert_eq!(
            torn_down.portal.portal_window_number, 0,
            "the portal window is closed and gone after teardown"
        );
        assert!(
            !torn_down.portal.child_attached,
            "the child relation is removed by teardown"
        );
        assert!(
            !torn_down.portal.native_visible,
            "the portal window is ordered out by teardown"
        );
        assert!(!torn_down.portal.frame_synced, "no frame survives teardown");
        assert_ownership_preserved(&torn_down, &settled);
        assert_eq!(torn_down.active_web_view, Some(first_view));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
