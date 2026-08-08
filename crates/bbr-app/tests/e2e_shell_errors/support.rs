//! Shared support for the error-surface real-app E2E (task packet
//! `error-surface-contract`, DESIGN.md §11.8/§15).
//!
//! The suite drives the real `bbr` app over the typed testkit protocol:
//! real navigations against the deterministic fixture server (TLS,
//! redirect-loop, truncated/flaky HTTP errors, a closed loopback port), the
//! classified-incident injection seam for the offline class, and the
//! production recovery/dismiss/crash-panel paths. Every synchronization step
//! polls the typed snapshot under a deadline — no sleeps, no pixel
//! assertions.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use bbr_testkit::protocol::{Request, Snapshot};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::e2e_common::{spawn_harness, Harness};

/// The deterministic recovery-fixture title (flaky route, later requests).
pub const FLAKY_TITLE: &str = "BBR Flaky Recovery";

/// Spawn the real app on the fixture server's page one and return the
/// connected client, the deterministic fixture server (URLs for the
/// failing fixtures), and the harness identity for diagnostics. `tag` must
/// be unique per test in this binary: the harness root is
/// `/tmp/bbr-st-{tag}-{pid}`, and all tests of one binary share the pid.
pub fn spawn(
    tag: &str,
) -> (
    Harness,
    Client,
    super::e2e_common::FixtureServer,
    PathBuf,
    String,
) {
    let (fixtures, page_one, _page_two) = super::e2e_common::fixtures();
    let (harness, socket, token) = spawn_harness(tag, &page_one);
    let mut client = super::e2e_common::connect(&socket);
    client.hello(&token).expect("hello");
    client.attach(None).expect("attach");
    // Wait for the initial page so later assertions start from a settled
    // active view (the fixture server is local; the snapshot's active view
    // is authoritative).
    let mut since = 0;
    loop {
        let snapshot = client.snapshot().expect("initial snapshot");
        if snapshot.active_web_view.is_some() {
            return (harness, client, fixtures, socket, token);
        }
        assert!(
            Instant::now() < super::e2e_common::deadline(),
            "no active web view at spawn"
        );
        let replay = client.attach(None).expect("replay");
        since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(since);
        std::thread::yield_now();
    }
}

/// Draw one real GPUI frame so render-derived state (the surface snapshot,
/// the visible ids) advances after a mutation.
pub fn flush_frame(client: &mut Client) {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
}

/// Poll until the error surface shows the armed category and view (bounded
/// by the suite deadline). Returns the matching snapshot.
pub fn wait_surface(
    client: &mut Client,
    view: WebViewId,
    category: bbr_testkit::protocol::ErrorSurfaceCategory,
) -> Snapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("surface snapshot");
        if snapshot.error_surface.visible
            && snapshot.error_surface.view == Some(view)
            && snapshot.error_surface.category == Some(category)
        {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "error surface did not present {category:?} on {view:?}; last {:?}",
            snapshot.error_surface
        );
        flush_frame(client);
        std::thread::yield_now();
    }
}

/// Poll until no error surface is shown (bounded by the suite deadline).
pub fn wait_surface_cleared(client: &mut Client) -> Snapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("surface cleared snapshot");
        if !snapshot.error_surface.visible {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "error surface did not clear; last {:?}",
            snapshot.error_surface
        );
        flush_frame(client);
        std::thread::yield_now();
    }
}

/// Navigate the active view to `url` through the production path and wait
/// for the error surface of `category` (the `NavigationFailed` path may
/// also emit a legacy web event; the surface is the authority here).
pub fn navigate_to_failure(
    client: &mut Client,
    url: &str,
    category: bbr_testkit::protocol::ErrorSurfaceCategory,
) -> (Snapshot, WebViewId) {
    let before = client.snapshot().expect("pre-navigation snapshot");
    let view = before.active_web_view.expect("active view");
    client
        .send_ok(&Request::Navigate { url: url.into() })
        .expect("navigate to failing url");
    let surface = wait_surface(client, view, category);
    (surface, view)
}

/// Click the surface's primary action (the recovery button) through the
/// production mouse-down seam.
pub fn click_recover(client: &mut Client) {
    client
        .send_ok(&Request::ClickErrorAction)
        .expect("click error recover");
}

/// Click the surface's Dismiss button through the production mouse-down
/// seam.
pub fn click_dismiss(client: &mut Client) {
    client
        .send_ok(&Request::ClickErrorDismiss)
        .expect("click error dismiss");
}

/// The current tab/space/profile identity (for the never-changes
/// assertions): active space, active tab index, active view, profile,
/// session population.
pub fn identity(snapshot: &Snapshot) -> (u64, Option<usize>, Option<WebViewId>, u64, usize) {
    (
        snapshot.active_space.0,
        snapshot.active_tab_index,
        snapshot.active_web_view,
        snapshot.active_profile.0,
        snapshot.engine_sessions,
    )
}

/// Assert that two snapshots share the exact tab/space/profile identity
/// (the surface never changes identity).
pub fn assert_same_identity(a: &Snapshot, b: &Snapshot) {
    let (space_a, tab_a, view_a, profile_a, sessions_a) = identity(a);
    let (space_b, tab_b, view_b, profile_b, sessions_b) = identity(b);
    assert_eq!(space_a, space_b, "active space changed");
    assert_eq!(tab_a, tab_b, "active tab changed");
    assert_eq!(view_a, view_b, "active web view changed");
    assert_eq!(profile_a, profile_b, "active profile changed");
    assert_eq!(sessions_a, sessions_b, "live session count changed");
}
