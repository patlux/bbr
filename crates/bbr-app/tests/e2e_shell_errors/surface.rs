//! Surface scenarios (task packet `error-surface-contract`, DESIGN.md
//! §11.8/§15): real navigations against the deterministic fixture server
//! prove the native failure → closed-taxonomy classification → presented
//! surface end to end, while the plain-500 case proves the surface never
//! invents an incident for a status WebKit rendered. The offline class is
//! the only injection: a genuinely-offline host cannot be toggled
//! deterministically, so the *classified* incident enters through the exact
//! production `on_incident` seam (the native-code → taxonomy mapping is
//! unit-proven in `bbr-web-macos::error_mapping`). Every scenario asserts
//! that showing the surface never changes tab, session, WebView, profile,
//! or space identity.

use bbr_testkit::protocol::{ErrorSurfaceCategory, Request};
use bbr_web::web_errors::{IncidentSubReason, OfflineReason};

use crate::e2e_common::*;
use crate::support::*;

/// The deterministic server-error page title (a 500 with a delivered body
/// is rendered by the engine, never turned into an incident).
const SERVER_ERROR_TITLE: &str = "Server Error";

/// The §11.8-style surface ids the content area renders while the
/// classified incident is shown (the active view's native slot is replaced,
/// so the slot id is absent).
const SURFACE_IDS: &[&str] = &[
    "bbr-error-panel",
    "bbr-error-title",
    "bbr-error-copy",
    "bbr-error-action",
    "bbr-error-dismiss",
];

/// HTTPS against the plaintext fixture server fails the TLS handshake; the
/// native code maps to the TLS class and the surface presents the calm
/// handshake copy with the contract-legal Retry action and the
/// privacy-safe origin. Showing (and then dismissing) the surface never
/// changes identity.
#[test]
#[ignore]
fn tls_https_against_plaintext_presents_tls_surface() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, fixtures, socket, token) = spawn("errors-tls");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let https_url = fixtures.url("/page1.html").replace("http://", "https://");
        let before = client.snapshot().expect("pre-navigation snapshot");
        let view = before.active_web_view.expect("active view");

        let (surface, shown_view) =
            navigate_to_failure(&mut client, &https_url, ErrorSurfaceCategory::Tls);
        assert_eq!(shown_view, view);
        // The classified subreason is the handshake class (or the
        // certificate-trust class — both map to TLS; the honest copy is
        // derived from the subreason, never a raw platform code).
        let reason = surface.error_surface.reason.as_deref().expect("reason");
        assert!(reason.starts_with("tls: "), "TLS classification: {reason}");
        assert_eq!(
            surface.error_surface.title.as_deref(),
            Some("This page can't be loaded securely")
        );
        assert_eq!(surface.error_surface.action.as_deref(), Some("Try again"));
        assert_same_identity(&before, &surface);
        // The privacy-safe origin is presented (scheme://host[:port]).
        let origin = surface
            .error_surface
            .origin
            .as_deref()
            .expect("failing origin");
        assert!(origin.starts_with("https://"), "reduced origin: {origin}");
        assert!(
            !origin.contains("page1.html"),
            "origin is host-only: {origin}"
        );

        // The rendered surface replaces the native slot: the §11.8 ids are
        // visible and the slot id is not advertised.
        flush_frame(&mut client);
        let rendered = client.snapshot().expect("tls surface snapshot");
        assert_ids(&rendered, SURFACE_IDS);
        assert!(
            !rendered
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-native-web-slot-")),
            "the surface replaces the native slot"
        );

        // Dismiss is a lifecycle event: surface clears, identity unchanged,
        // and the native slot returns to the visible ids.
        click_dismiss(&mut client);
        let cleared = wait_surface_cleared(&mut client);
        assert_same_identity(&surface, &cleared);
        flush_frame(&mut client);
        let restored = client.snapshot().expect("restored snapshot");
        assert!(
            restored
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-native-web-slot-")),
            "the native slot returns after dismissal"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// A closed loopback port is an unreachable host: the engine fails the
/// navigation and the surface presents the navigation class with the
/// connection-refused subreason.
#[test]
#[ignore]
fn closed_port_presents_navigation_surface() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, _fixtures, socket, token) = spawn("errors-closed-port");

    let result = catch_unwind(AssertUnwindSafe(|| {
        // Deterministic unreachable host: bind, read the port, drop the
        // listener (the same shape the probe uses).
        let closed_port_url = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            format!("http://127.0.0.1:{port}/")
        };
        let before = client.snapshot().expect("pre-navigation snapshot");
        let view = before.active_web_view.expect("active view");

        let (surface, shown_view) = navigate_to_failure(
            &mut client,
            &closed_port_url,
            ErrorSurfaceCategory::Navigation,
        );
        assert_eq!(shown_view, view);
        let reason = surface.error_surface.reason.as_deref().expect("reason");
        assert!(
            reason.starts_with("navigation: ") && reason.contains("connection refused"),
            "closed-port classification: {reason}"
        );
        assert_eq!(
            surface.error_surface.title.as_deref(),
            Some("This page can't be loaded")
        );
        assert_same_identity(&before, &surface);
        flush_frame(&mut client);
        let rendered = client.snapshot().expect("closed-port surface snapshot");
        assert_ids(&rendered, SURFACE_IDS);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The redirect-loop fixture (an endless 302 chain) is an HTTP error
/// condition WebKit actually fails (`NSURLErrorHTTPTooManyRedirects`); the
/// adapter classifies it as `Navigation/RedirectLoop` and the surface
/// presents the honest redirect-loop copy.
#[test]
#[ignore]
fn redirect_loop_presents_redirect_loop_surface() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, fixtures, socket, token) = spawn("errors-redirect-loop");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let redirect_url = fixtures.url("/__fx/error/redirect-loop");
        let before = client.snapshot().expect("pre-navigation snapshot");
        let view = before.active_web_view.expect("active view");

        let (surface, shown_view) =
            navigate_to_failure(&mut client, &redirect_url, ErrorSurfaceCategory::Navigation);
        assert_eq!(shown_view, view);
        let reason = surface.error_surface.reason.as_deref().expect("reason");
        assert!(
            reason.starts_with("navigation: ") && reason.contains("redirect loop"),
            "redirect-loop classification: {reason}"
        );
        assert_same_identity(&before, &surface);
        flush_frame(&mut client);
        let rendered = client.snapshot().expect("redirect-loop surface snapshot");
        assert_ids(&rendered, SURFACE_IDS);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The truncated-500 fixture declares a body it never delivers; the engine
/// fails the navigation with an HTTP-domain code the adapter classifies as
/// the navigation class (transient transport failure).
#[test]
#[ignore]
fn truncated_500_presents_navigation_surface() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, fixtures, socket, token) = spawn("errors-truncated");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let truncated_url = fixtures.url("/__fx/error/truncated-error");
        let before = client.snapshot().expect("pre-navigation snapshot");

        let (surface, _view) = navigate_to_failure(
            &mut client,
            &truncated_url,
            ErrorSurfaceCategory::Navigation,
        );
        let reason = surface.error_surface.reason.as_deref().expect("reason");
        assert!(
            reason.starts_with("navigation: "),
            "truncated-500 classification: {reason}"
        );
        assert_same_identity(&before, &surface);
        flush_frame(&mut client);
        let rendered = client.snapshot().expect("truncated surface snapshot");
        assert_ids(&rendered, SURFACE_IDS);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// A plain 500 with a delivered body is a *successful* HTTP response as far
/// as WebKit is concerned: the engine renders the server's error page and
/// the surface must never invent an incident for a status it accepted.
#[test]
#[ignore]
fn plain_500_renders_the_page_without_an_incident() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, fixtures, socket, token) = spawn("errors-server-error");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let server_error_url = fixtures.url("/__fx/error/server-error");
        let before = client.snapshot().expect("pre-navigation snapshot");
        let view = before.active_web_view.expect("active view");

        client
            .send_ok(&Request::Navigate {
                url: server_error_url.clone(),
            })
            .expect("navigate to server-error page");
        // The page renders: poll for the deterministic server-error title
        // under the deadline, then assert no incident was ever invented.
        let until = Instant::now() + Duration::from_secs(30);
        loop {
            let snapshot = client.snapshot().expect("server-error page snapshot");
            assert!(
                !snapshot.error_surface.visible,
                "a rendered 500 must never present an error surface: {:?}",
                snapshot.error_surface
            );
            if snapshot.title.as_deref() == Some(SERVER_ERROR_TITLE) {
                assert_eq!(
                    snapshot.active_web_view,
                    Some(view),
                    "the delivered page keeps the active view"
                );
                break;
            }
            assert!(
                Instant::now() < until,
                "the server-error page never rendered; title {:?}",
                snapshot.title
            );
            flush_frame(&mut client);
            std::thread::yield_now();
        }
        assert_same_identity(&before, &client.snapshot().expect("final snapshot"));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The offline class enters through the exact production `on_incident` seam
/// (a genuinely-offline host cannot be toggled deterministically): the
/// classified incident presents the offline surface with unknown origin and
/// the "You're offline" tone, and Dismiss clears it without touching
/// identity.
#[test]
#[ignore]
fn offline_class_enters_through_the_production_incident_seam() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, _fixtures, socket, token) = spawn("errors-offline");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let before = client.snapshot().expect("pre-injection snapshot");
        let view = before.active_web_view.expect("active view");

        client
            .send_ok(&dsl::inject_error_incident(
                before.active_profile,
                view,
                IncidentSubReason::Offline(OfflineReason::NoNetworkConnection),
                None,
            ))
            .expect("inject classified offline incident");
        let surface = wait_surface(&mut client, view, ErrorSurfaceCategory::Offline);
        let reason = surface.error_surface.reason.as_deref().expect("reason");
        assert!(
            reason.starts_with("offline: "),
            "offline classification: {reason}"
        );
        assert_eq!(
            surface.error_surface.title.as_deref(),
            Some("You're offline")
        );
        assert_eq!(
            surface.error_surface.origin, None,
            "an unknown origin omits the URL line"
        );
        assert_same_identity(&before, &surface);
        flush_frame(&mut client);
        let rendered = client.snapshot().expect("offline surface snapshot");
        assert_ids(&rendered, SURFACE_IDS);
        assert!(
            !rendered.visible_ids.iter().any(|id| id == "bbr-error-url"),
            "no URL line without an origin"
        );

        click_dismiss(&mut client);
        let cleared = wait_surface_cleared(&mut client);
        assert_same_identity(&surface, &cleared);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
