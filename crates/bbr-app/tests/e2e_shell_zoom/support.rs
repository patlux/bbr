//! Shared harness support for the per-site zoom E2E (queue item 11, part A).
//!
//! Two fixture servers on different loopback ports give two distinct
//! normalized origins (`http://127.0.0.1:<port>` — the port is part of the
//! `SiteKey` identity), so per-site isolation is proven against real
//! distinct sites, never two paths of one origin. The standard
//! `page1.html`/`page2.html` fixtures are reused (zoom applies to any
//! page); same-origin navigation uses the second page of the same server.
//!
//! Every assertion is typed against the app's `zoom` snapshot (effective
//! level, normalized origin, applied level, pending write) plus the
//! unchanged identity fields (active view, session count, profile, space).

use std::path::PathBuf;
use std::time::Instant;

use crate::e2e_common::*;
use bbr_testkit::protocol::Snapshot;
use bbr_testkit::Client;
use bbr_web::WebViewId;

/// The default zoom level of the merged contract.
pub(super) const ZOOM_DEFAULT: u16 = 100;
/// The contract step size (25 percentage points).
pub(super) const ZOOM_STEP: u16 = 25;
/// The contract bounds.
pub(super) const ZOOM_MIN: u16 = 25;
pub(super) const ZOOM_MAX: u16 = 400;

/// Two fixture servers (two origins) plus their deterministic page URLs.
pub(super) struct ZoomFixtures {
    pub server_a: FixtureServer,
    pub server_b: FixtureServer,
    /// Origin A page 1 (the initial page).
    pub a1: String,
    /// Origin A page 2 (same-origin navigation target).
    pub a2: String,
    /// Origin B page (distinct origin).
    pub b1: String,
}

/// Start the two fixture servers and the deterministic page URLs.
pub(super) fn zoom_fixtures() -> ZoomFixtures {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let server_a = FixtureServer::start(&fixtures_dir).expect("fixture server A");
    let server_b = FixtureServer::start(&fixtures_dir).expect("fixture server B");
    let a1 = server_a.url("/page1.html");
    let a2 = server_a.url("/page2.html");
    let b1 = server_b.url("/page1.html");
    ZoomFixtures {
        server_a,
        server_b,
        a1,
        a2,
        b1,
    }
}

/// The normalized origin (`SiteKey` display: `scheme://host:port`) of a
/// fixture server's base URL — the `site` field the zoom snapshot reports
/// (the profile-scoped key display is `profile=1 site=<origin>` in the
/// default profile).
pub(super) fn origin_site(base_url: &str) -> String {
    format!("profile=1 site={base_url}")
}

// The shared hidden-harness spawn (`e2e_common::spawn_harness` — the
// scenario modules import it from the shared harness glob directly).

/// Dispatch one keystroke through GPUI's real input path (the same seam the
/// command suite uses): `cmd-+` resolves the `ZoomIn` binding, `cmd--` the
/// `ZoomOut` binding, `cmd-0` the `ZoomReset` binding.
pub(super) fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

/// Wait until the active view finished loading `title` (same synchronization
/// the merged suites use — never race a late `LoadingChanged`).
pub(super) fn wait_fixture_loaded(
    client: &mut Client,
    view: WebViewId,
    title: &str,
    since: u64,
) -> u64 {
    let snapshot = client.snapshot().expect("fixture title snapshot");
    if snapshot.title.as_deref() == Some(title) {
        return since;
    }
    wait_title(client, view, title, since)
}

/// The shared harness deadline (re-exported so scenario modules read
/// `deadline()` without importing the harness module twice).
pub(super) use crate::e2e_common::deadline;

/// Poll until the snapshot reports the exact URL (the UrlChanged-driven
/// `nav.url`), bounded by the deadline.
pub(super) fn wait_url(client: &mut Client, url: &str, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("url snapshot");
        if snapshot.url.as_deref() == Some(url) {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "navigation to {url} did not land; last url {:?}",
            snapshot.url
        );
        std::thread::yield_now();
    }
}

/// Poll until the zoom snapshot reports exactly `level` (applied to the
/// active view), bounded by the deadline.
pub(super) fn wait_zoom(client: &mut Client, level: u16, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("zoom snapshot");
        let zoom = &snapshot.zoom;
        if zoom.level == level && zoom.applied == Some(level) && !zoom.pending {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "zoom never reached {level}; last {:?}",
            zoom
        );
        std::thread::yield_now();
    }
}

/// Typed zoom + identity assertion: the active view's site is at `level`,
/// applied natively, with the exact normalized origin and no pending write;
/// the identity fields are exactly the captured baseline.
#[allow(clippy::too_many_arguments)]
pub(super) fn assert_zoom_at(
    snapshot: &Snapshot,
    level: u16,
    site: &str,
    baseline: &IdentityBaseline,
) {
    let zoom = &snapshot.zoom;
    assert_eq!(zoom.level, level, "effective zoom level");
    assert_eq!(
        zoom.applied,
        Some(level),
        "level applied to the native view"
    );
    assert_eq!(zoom.site.as_deref(), Some(site), "normalized origin");
    assert!(!zoom.pending, "no unacknowledged persistence write");
    assert_identity(snapshot, baseline);
}

/// The tab/session/profile/space identity captured at suite setup; every
/// zoom mutation must leave it untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct IdentityBaseline {
    pub view: WebViewId,
    pub sessions: usize,
    pub profile: bbr_testkit::protocol::TestProfileId,
    pub space: bbr_testkit::protocol::TestSpaceId,
}

pub(super) fn identity_baseline(snapshot: &Snapshot) -> IdentityBaseline {
    IdentityBaseline {
        view: snapshot.active_web_view.expect("active view"),
        sessions: snapshot.engine_sessions,
        profile: snapshot.active_profile,
        space: snapshot.active_space,
    }
}

pub(super) fn assert_identity(snapshot: &Snapshot, baseline: &IdentityBaseline) {
    assert_eq!(
        snapshot.active_web_view,
        Some(baseline.view),
        "the active WebViewId is unchanged by zoom"
    );
    assert_eq!(
        snapshot.engine_sessions, baseline.sessions,
        "zoom never creates or destroys sessions"
    );
    assert_eq!(
        snapshot.active_profile, baseline.profile,
        "the active profile is unchanged by zoom"
    );
    assert_eq!(
        snapshot.active_space, baseline.space,
        "the active space is unchanged by zoom"
    );
    assert!(snapshot.web_view_hosted, "a real native view backs the tab");
}

/// The initial settled zoom state (default level, no site yet if the URL
/// hasn't produced a UrlChanged — call after `wait_fixture_loaded`).
pub(super) fn settled_zoom(snapshot: &Snapshot) {
    let zoom = &snapshot.zoom;
    assert_eq!(zoom.level, ZOOM_DEFAULT, "fresh site starts at the default");
    assert!(zoom.site.is_some(), "the initial fixture origin resolved");
    assert_eq!(zoom.applied, Some(ZOOM_DEFAULT), "default applied natively");
    assert!(!zoom.pending);
}

/// Drive one `Cmd-Plus` step through the real key path and assert the exact
/// resulting level, returning the snapshot.
pub(super) fn zoom_in_key(
    client: &mut Client,
    expect: u16,
    site: &str,
    baseline: &IdentityBaseline,
) -> Snapshot {
    key(client, "cmd-+");
    let snapshot = wait_zoom(client, expect, deadline());
    assert_zoom_at(&snapshot, expect, site, baseline);
    snapshot
}

/// Drive one `Cmd-Minus` step through the real key path and assert the
/// exact resulting level.
pub(super) fn zoom_out_key(
    client: &mut Client,
    expect: u16,
    site: &str,
    baseline: &IdentityBaseline,
) -> Snapshot {
    key(client, "cmd--");
    let snapshot = wait_zoom(client, expect, deadline());
    assert_zoom_at(&snapshot, expect, site, baseline);
    snapshot
}

/// Start the harness, wait for the window to settle and the initial fixture
/// to load, and capture the identity baseline.
pub(super) fn start(client: &mut Client, title: &str, since: u64) -> (IdentityBaseline, WebViewId) {
    let settled = wait_for_window_settled(client, deadline());
    let view = settled.active_web_view.expect("first native view");
    assert!(view.0 > 0, "a real WKWebView backs the active tab");
    assert_eq!(settled.engine_kind, "native-wkwebview");
    let _ = wait_fixture_loaded(client, view, title, since);
    let loaded = client.snapshot().expect("loaded snapshot");
    let baseline = identity_baseline(&loaded);
    settled_zoom(&loaded);
    (baseline, view)
}
