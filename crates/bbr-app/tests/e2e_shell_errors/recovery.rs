//! Recovery scenarios (task packet `error-surface-contract`, DESIGN.md
//! §11.8/§15): the surface's primary action retries the failing URL through
//! the production controller path. On the flaky fixture the retried
//! navigation succeeds, the incident recovers, and the surface clears; on a
//! still-failing route the retry fails again and a fresh incident
//! supersedes the old one (the lifecycle returns to a newly reported
//! incident). Dismiss is a lifecycle event that clears the surface and
//! restores the native slot. The renderer-crash class never draws the error
//! surface (the §11.8 crash panel owns it), and the panel's Reload rebuilds
//! the session. Every scenario asserts identity never changes.

use bbr_testkit::protocol::{ErrorSurfaceCategory, Request, Snapshot};
use bbr_web::web_errors::{CrashReason, IncidentSubReason};
use bbr_web::{WebAction, WebEvent};

use crate::e2e_common::*;
use crate::support::*;

/// The crash-panel ids the content area renders for the process-crash class
/// (the §11.8 panel, never the error surface).
const CRASH_PANEL_IDS: &[&str] = &["bbr-crash-panel", "bbr-crash-reload", "bbr-crash-close"];

/// Poll until the error surface shows a **freshly reported** incident on
/// `view` (the retried navigation failed again and a new incident
/// superseded the recovery-requested one — the lifecycle re-reports).
fn wait_superseded_reported(
    client: &mut Client,
    view: WebViewId,
    category: ErrorSurfaceCategory,
) -> Snapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("supersession snapshot");
        if snapshot.error_surface.visible
            && snapshot.error_surface.view == Some(view)
            && snapshot.error_surface.category == Some(category)
            && snapshot.error_surface.lifecycle.as_deref() == Some("reported")
        {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "no fresh reported incident superseded the retry; last {:?}",
            snapshot.error_surface
        );
        flush_frame(client);
        std::thread::yield_now();
    }
}

/// The primary action retries the failing URL through the production
/// controller path. On the flaky fixture the retried navigation succeeds:
/// the incident recovers, the surface clears, and the deterministic
/// recovery page renders — identity never changes.
#[test]
#[ignore]
fn retry_on_the_flaky_route_recovers_and_clears() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, fixtures, socket, token) = spawn("errors-retry-flaky");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let flaky_url = fixtures.url("/__fx/error/flaky");
        let before = client.snapshot().expect("pre-navigation snapshot");
        let view = before.active_web_view.expect("active view");

        // First request to the flaky route fails (truncated 500) → the
        // navigation surface presents.
        let (surface, _view) =
            navigate_to_failure(&mut client, &flaky_url, ErrorSurfaceCategory::Navigation);
        assert_same_identity(&before, &surface);

        // Retry through the production recovery seam: the retried
        // navigation starts, the incident recovers, and the surface clears.
        click_recover(&mut client);
        let cleared = wait_surface_cleared(&mut client);
        assert_same_identity(&surface, &cleared);

        // The retried request (request 2) succeeded: the deterministic
        // recovery page rendered.
        let until = Instant::now() + Duration::from_secs(30);
        loop {
            let snapshot = client.snapshot().expect("recovery page snapshot");
            assert!(
                !snapshot.error_surface.visible,
                "the recovered surface stays cleared: {:?}",
                snapshot.error_surface
            );
            if snapshot.title.as_deref() == Some(FLAKY_TITLE) {
                assert_eq!(snapshot.active_web_view, Some(view));
                break;
            }
            assert!(
                Instant::now() < until,
                "the flaky recovery page never rendered; title {:?}",
                snapshot.title
            );
            flush_frame(&mut client);
            std::thread::yield_now();
        }
        assert_same_identity(&before, &client.snapshot().expect("final snapshot"));
        flush_frame(&mut client);
        let rendered = client.snapshot().expect("post-recovery snapshot");
        assert!(
            !rendered
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-error-")),
            "no error-surface ids after recovery"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Retrying a still-failing route (the redirect loop) fails again: the
/// retried navigation starts and then the engine reports a fresh failure,
/// so a new incident supersedes the recovery-requested one and the surface
/// re-presents the same classification from a freshly reported lifecycle.
/// Identity never changes.
#[test]
#[ignore]
fn retry_on_a_still_failing_route_supersedes_the_incident() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, fixtures, socket, token) = spawn("errors-retry-loop");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let redirect_url = fixtures.url("/__fx/error/redirect-loop");
        let before = client.snapshot().expect("pre-navigation snapshot");
        let view = before.active_web_view.expect("active view");

        let (surface, _view) =
            navigate_to_failure(&mut client, &redirect_url, ErrorSurfaceCategory::Navigation);
        assert_eq!(
            surface.error_surface.lifecycle.as_deref(),
            Some("reported"),
            "the first incident reports"
        );
        assert_same_identity(&before, &surface);

        // Retry: the retried navigation starts (surface may clear
        // briefly), then the loop fails again and a fresh incident
        // supersedes the recovery-requested one — the lifecycle re-reports.
        click_recover(&mut client);
        let superseded =
            wait_superseded_reported(&mut client, view, ErrorSurfaceCategory::Navigation);
        let reason = superseded.error_surface.reason.as_deref().expect("reason");
        assert!(
            reason.starts_with("navigation: ") && reason.contains("redirect loop"),
            "the superseding incident keeps the classification: {reason}"
        );
        assert_same_identity(&surface, &superseded);
        flush_frame(&mut client);
        let rendered = client.snapshot().expect("superseded surface snapshot");
        assert_ids(&rendered, &["bbr-error-panel", "bbr-error-action"]);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Dismiss is a lifecycle event: the surface clears, the native slot
/// returns to the visible ids, and identity never changes.
#[test]
#[ignore]
fn dismiss_clears_the_surface_and_restores_the_native_slot() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, _fixtures, socket, token) = spawn("errors-dismiss");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let closed_port_url = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            format!("http://127.0.0.1:{port}/")
        };
        let _before = client.snapshot().expect("pre-navigation snapshot");

        let (surface, _view) = navigate_to_failure(
            &mut client,
            &closed_port_url,
            ErrorSurfaceCategory::Navigation,
        );
        assert_eq!(surface.error_surface.lifecycle.as_deref(), Some("reported"));

        // Dismiss through the production mouse-down seam: a lifecycle event,
        // never a tab mutation.
        click_dismiss(&mut client);
        let cleared = wait_surface_cleared(&mut client);
        assert_same_identity(&surface, &cleared);
        assert_eq!(cleared.error_surface.view, None);
        flush_frame(&mut client);
        let restored = client.snapshot().expect("restored snapshot");
        assert!(
            restored
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-native-web-slot-")),
            "the native slot returns after dismissal"
        );
        assert!(
            !restored
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-error-")),
            "no error-surface ids after dismissal"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The tab/space/profile identity (the crash lifecycle rebuilds the
/// session — a fresh view id is the point of Reload — so view/session
/// counts are asserted separately for the crash class).
fn tab_identity(snapshot: &Snapshot) -> (u64, Option<usize>, u64) {
    (
        snapshot.active_space.0,
        snapshot.active_tab_index,
        snapshot.active_profile.0,
    )
}

/// Assert that two snapshots share tab/space/profile identity.
fn assert_tab_identity(a: &Snapshot, b: &Snapshot) {
    assert_eq!(
        tab_identity(a),
        tab_identity(b),
        "tab/space/profile changed"
    );
}

/// The process-crash class is classified through the same seam but never
/// draws the error surface — the §11.8 crash panel owns it. The panel's
/// Reload (the production `reload_active_from_panel` path, driven here by
/// the same `WebAction::Reload` the panel's button dispatches) rebuilds the
/// session: the crashed session is destroyed exactly once and Reload
/// recreates it. Tab/space/profile identity never changes.
#[test]
#[ignore]
fn crash_class_stays_on_the_panel_and_reload_recovers() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, _fixtures, socket, token) = spawn("errors-crash");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let before = client.snapshot().expect("pre-injection snapshot");
        let view = before.active_web_view.expect("active view");

        // The classified crash incident enters through the production seam;
        // the error surface must NOT draw (the crash class is the §11.8
        // panel's surface, never the error surface).
        client
            .send_ok(&dsl::inject_error_incident(
                before.active_profile,
                view,
                IncidentSubReason::Crash(CrashReason::WebContentCrashed),
                None,
            ))
            .expect("inject classified crash incident");
        flush_frame(&mut client);
        let after_crash_class = client.snapshot().expect("crash-class snapshot");
        assert!(
            !after_crash_class.error_surface.visible,
            "the crash class never draws the error surface: {:?}",
            after_crash_class.error_surface
        );
        assert!(
            !after_crash_class
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-error-")),
            "no error-surface ids for the crash class"
        );
        assert_tab_identity(&before, &after_crash_class);
        assert_eq!(
            after_crash_class.engine_sessions, before.engine_sessions,
            "the classified crash incident itself touches no session"
        );

        // The renderer dies (production event path): the §11.8 crash panel
        // presents with typed buttons and the dead session is destroyed.
        client
            .send_ok(&dsl::inject_web_event(WebEvent::PageCrashed(view)))
            .expect("inject renderer crash");
        let until = Instant::now() + Duration::from_secs(30);
        let crashed = loop {
            flush_frame(&mut client);
            let snapshot = client.snapshot().expect("crashed snapshot");
            if snapshot
                .visible_ids
                .iter()
                .any(|id| id == "bbr-crash-panel")
            {
                break snapshot;
            }
            assert!(
                Instant::now() < until,
                "the crash panel never presented; ids {:?}",
                snapshot.visible_ids
            );
            std::thread::yield_now();
        };
        assert_ids(&crashed, CRASH_PANEL_IDS);
        assert_eq!(
            crashed.engine_sessions, 0,
            "the crashed session is destroyed exactly once"
        );
        assert_tab_identity(&before, &crashed);

        // The panel's Reload (production `reload_active_from_panel` via the
        // controller's Reload route) rebuilds the session: the panel goes
        // away, the tab is not crashed, a fresh session exists, and no
        // error surface is left behind.
        client
            .send_ok(&Request::WebAction(WebAction::Reload))
            .expect("reload crashed tab");
        let until = Instant::now() + Duration::from_secs(30);
        let recovered = loop {
            flush_frame(&mut client);
            let snapshot = client.snapshot().expect("recovered snapshot");
            let panel_gone = !snapshot
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-crash-"));
            let not_crashed = !snapshot.tabs[0].crashed;
            if panel_gone && not_crashed && !snapshot.error_surface.visible {
                break snapshot;
            }
            assert!(
                Instant::now() < until,
                "the crashed tab never recovered; tab crashed: {}; ids {:?}; surface {:?}",
                snapshot.tabs[0].crashed,
                snapshot.visible_ids,
                snapshot.error_surface
            );
            std::thread::yield_now();
        };
        assert!(!recovered.tabs[0].crashed);
        assert!(!recovered.error_surface.visible);
        assert_eq!(
            recovered.engine_sessions, 1,
            "Reload recreates exactly one session"
        );
        assert!(
            recovered
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-native-web-slot-")),
            "the native slot returns after reload"
        );
        assert_tab_identity(&before, &recovered);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
