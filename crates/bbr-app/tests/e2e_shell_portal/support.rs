//! Shared assertions and shown-harness support for native portal E2E.

use std::path::PathBuf;
use std::time::Instant;

use crate::e2e_common::*;
use bbr_testkit::protocol::{dsl, Snapshot};

/// The probe's presentation id (`PortalPresentationId(0xB9)` in
/// `crate::portal::probe`), exposed through the snapshot's typed
/// `portal.presentation` field.
pub(super) const PROBE_PRESENTATION_ID: u64 = 0xB9;
/// The probe card size in points (the general centered placement).
pub(super) const PROBE_WIDTH: f32 = 480.0;
pub(super) const PROBE_HEIGHT: f32 = 360.0;

/// Spawn the harness with a SHOWN, activated window (no `--bbr-hidden`).
///
/// The hidden harness never orders the parent window in, so AppKit
/// hit-testing at the probe point resolves to whatever the desktop has there
/// and the required frontmost-ownership evidence (task packet evidence
/// section) is structurally unobservable. The portal frontmost test needs a
/// real on-screen parent; this mirrors the performance suite, which already
/// spawns shown windows (`perf_common/scenarios.rs`). Geometry is still
/// deterministic after [`wait_for_window_settled`] (the settle contract
/// outlives the yabai tile adjustment, see e2e_common).
pub(super) fn spawn_harness_shown(root_tag: &str, page_one: &str) -> (Harness, PathBuf, String) {
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

pub(super) fn flush_frame(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

/// The centered probe rect for a window of `w`×`h` (points), computed from
/// the same present-time viewport the app resolves against.
pub(super) fn centered_rect(w: f32, h: f32) -> [f32; 4] {
    let x = (w - PROBE_WIDTH) / 2.0;
    let y = (h - PROBE_HEIGHT) / 2.0;
    [x, y, PROBE_WIDTH, PROBE_HEIGHT]
}

/// Assert the typed closed-host default (DESIGN §13/§15): nothing active,
/// nothing visible, no rect/ids, but the host window exists and stays
/// attached as an implementation-owned child of the browser window.
pub(super) fn assert_host_closed(snapshot: &Snapshot) {
    let p = &snapshot.portal;
    assert_eq!(p.presentation, None, "no active presentation");
    assert_eq!(p.host, None, "no active host");
    assert!(!p.visible, "logical visibility false while closed");
    assert_eq!(p.rect, None, "no resolved rect while closed");
    assert!(
        p.ids.is_empty(),
        "no bbr-portal-* ids while closed: {:?}",
        p.ids
    );
    assert!(
        p.child_attached,
        "host stays attached to the browser window"
    );
    assert!(
        !p.native_visible,
        "AppKit window is never shown while closed"
    );
    assert!(
        p.frame_synced,
        "frame stays pinned to the parent content rect"
    );
    assert!(
        p.nonactivating_panel,
        "the borderless non-activating-panel policy holds while closed"
    );
    assert!(!p.portal_is_key, "non-key policy holds");
    assert!(!p.portal_is_main, "non-main policy holds");
    assert!(p.parent_window_number > 0, "browser window exists");
}

/// Assert the typed open-probe evidence (task packet evidence section):
/// `NativeAboveWeb` presentation, child attachment, native visibility,
/// exact parent-content frame sync, and the non-key/non-main policy. The
/// frontmost hit-test ownership is asserted separately ([`assert_frontmost_ownership`])
/// because it requires an on-screen parent window (hidden harnesses cannot
/// observe it).
pub(super) fn assert_probe_open(snapshot: &Snapshot) {
    let p = &snapshot.portal;
    assert_eq!(p.presentation, Some(PROBE_PRESENTATION_ID));
    assert_eq!(p.host.as_deref(), Some("native-above-web"));
    assert!(p.visible);
    assert_eq!(
        p.ids,
        vec![
            "bbr-portal-probe".to_string(),
            "bbr-portal-backdrop".to_string(),
            "bbr-portal-card".to_string(),
        ]
    );
    assert!(p.child_attached);
    assert!(p.native_visible, "AppKit window is ordered in");
    assert!(
        p.frame_synced,
        "portal frame pinned to the parent content rect"
    );
    assert!(
        p.nonactivating_panel,
        "the non-activating panel policy is applied natively"
    );
    assert!(
        p.portal_window_number > 0 && p.parent_window_number > 0,
        "both windows have AppKit numbers"
    );
    assert_ne!(
        p.portal_window_number, p.parent_window_number,
        "portal is its own window, not the browser window"
    );
    assert!(!p.portal_is_key, "showing never makes the portal key");
    assert!(!p.portal_is_main, "showing never makes the portal main");
}

/// Assert native hit-test ownership at the probe card's point over the parent
/// content and therefore over its WKWebViews.
///
/// `portal_owns_probe_point` proves the deterministic mechanism: the portal is
/// an attached, visible child window whose frame matches the parent content.
/// At least one real AppKit Window Server query must independently corroborate
/// that ordering:
///
/// - the absolute query reports the portal or its BBR parent; or
/// - the portal-relative query reports the BBR parent immediately below it.
///
/// Requiring both raw queries is not portable across ordinary macOS desktop
/// state: a foreign window can cover the screen point while the relative query
/// remains correct, and all-spaces/system overlays can appear in the relative
/// query while the absolute query still reports BBR. Either corroboration plus
/// the attached-child invariant proves the native portal is above the parent's
/// WKWebViews without replacing the native boundary with semantic-only evidence.
pub(super) fn assert_frontmost_ownership(snapshot: &Snapshot) {
    let p = &snapshot.portal;
    assert!(
        p.portal_owns_probe_point,
        "the portal owns hit testing at the probe point over the web content"
    );
    let absolute_bbr = p.frontmost_at_probe_point == Some(p.portal_window_number)
        || p.frontmost_at_probe_point == Some(p.parent_window_number);
    let relative_parent = p.below_portal_at_probe_point == Some(p.parent_window_number);
    assert!(
        absolute_bbr || relative_parent,
        "AppKit did not corroborate portal-over-parent ordering at the probe \
         point: frontmost {:?}, below portal {:?}, parent {}, portal {}",
        p.frontmost_at_probe_point,
        p.below_portal_at_probe_point,
        p.parent_window_number,
        p.portal_window_number
    );
}

/// Poll until the portal's deferred frame application and at least one real
/// AppKit Window Server query corroborate the attached-child ordering. Bounded
/// by the standard deadline; on timeout the failure reports both raw queries.
pub(super) fn wait_for_frontmost_evidence(client: &mut Client, deadline: Instant) {
    loop {
        // Keep the shown BBR window active where the desktop allows it. The
        // assertion does not assume activation excludes foreign always-on-top
        // windows or system overlays.
        if let Err(error) = client.send_ok(&dsl::activate_app()) {
            assert!(
                Instant::now() < deadline,
                "activate app did not recover: {error}"
            );
            std::thread::yield_now();
            continue;
        }
        let snapshot = client.snapshot().expect("frontmost-evidence snapshot");
        let p = &snapshot.portal;
        let absolute_bbr = p.frontmost_at_probe_point == Some(p.portal_window_number)
            || p.frontmost_at_probe_point == Some(p.parent_window_number);
        let relative_parent = p.below_portal_at_probe_point == Some(p.parent_window_number);
        if p.portal_owns_probe_point && (absolute_bbr || relative_parent) {
            assert_frontmost_ownership(&snapshot);
            return;
        }
        assert!(
            Instant::now() < deadline,
            "AppKit hit-test evidence at the probe point did not converge: \
             frontmost {:?} (parent {} / portal {}), below {:?}, owns {}",
            p.frontmost_at_probe_point,
            p.parent_window_number,
            p.portal_window_number,
            p.below_portal_at_probe_point,
            p.portal_owns_probe_point
        );
        std::thread::yield_now();
    }
}

/// Poll the real AppKit Window Server until the shown browser parent owns the
/// center of its web-content area while the portal is idle. This is the
/// native-boundary complement to [`assert_host_closed`]: a logically closed
/// snapshot alone could miss a stale on-screen child window, whereas the raw
/// window-number query proves the hidden portal neither occludes nor receives
/// pointer events at the probe point.
pub(super) fn wait_for_idle_parent_ownership(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        if let Err(error) = client.send_ok(&dsl::activate_app()) {
            assert!(
                Instant::now() < deadline,
                "activate app did not recover while checking idle portal: {error}"
            );
            std::thread::yield_now();
            continue;
        }
        let snapshot = client.snapshot().expect("idle portal ownership snapshot");
        assert_host_closed(&snapshot);
        let p = &snapshot.portal;
        assert!(
            !p.portal_owns_probe_point,
            "a closed portal must never own hit testing over web content"
        );
        assert_ne!(
            p.frontmost_at_probe_point,
            Some(p.portal_window_number),
            "the hidden portal window must never be frontmost at the content probe"
        );
        if p.frontmost_at_probe_point == Some(p.parent_window_number) {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "shown browser parent did not regain Window Server ownership while portal was idle: \
             frontmost {:?}, parent {}, portal {}, native_visible {}",
            p.frontmost_at_probe_point,
            p.parent_window_number,
            p.portal_window_number,
            p.native_visible
        );
        std::thread::yield_now();
    }
}

/// Assert that browser ownership and the active session identity are
/// structurally untouched by a portal mutation.
pub(super) fn assert_ownership_preserved(snapshot: &Snapshot, baseline: &Snapshot) {
    assert_eq!(
        snapshot.active_web_view, baseline.active_web_view,
        "portal mutations must never change the selected WebViewId"
    );
    assert_eq!(
        snapshot.engine_sessions, baseline.engine_sessions,
        "portal mutations must never create, destroy, or recreate sessions"
    );
    assert_eq!(
        snapshot.active_profile, baseline.active_profile,
        "portal mutations must never change the active profile"
    );
    assert_eq!(
        snapshot.active_space, baseline.active_space,
        "portal mutations must never change the active space"
    );
    assert_eq!(
        snapshot.chrome_focused, baseline.chrome_focused,
        "the non-activating portal must never steal chrome focus"
    );
}
