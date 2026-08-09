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

/// Poll until both the logical presentation and deferred AppKit ordering are
/// visible in the same typed snapshot. The native bridge deliberately applies
/// `orderFront:` on a bare run-loop turn to avoid GPUI App re-entrancy, so a
/// response/frame flush can precede the native visibility transition.
pub(super) fn wait_probe_open(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("probe open snapshot");
        let p = &snapshot.portal;
        if p.presentation == Some(PROBE_PRESENTATION_ID)
            && p.visible
            && p.native_visible
            && p.frame_synced
        {
            assert_probe_open(&snapshot);
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "portal probe did not become natively visible; presentation {:?}, logical {}, native {}, frame_synced {}",
            p.presentation,
            p.visible,
            p.native_visible,
            p.frame_synced
        );
        std::thread::yield_now();
    }
}

/// Poll until dismissal and deferred `orderOut:` have both converged. The
/// existing closed-host assertion remains the acceptance surface.
pub(super) fn wait_host_closed(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("portal closed snapshot");
        let p = &snapshot.portal;
        if p.presentation.is_none() && !p.visible && !p.native_visible {
            assert_host_closed(&snapshot);
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "portal host did not become natively hidden; presentation {:?}, logical {}, native {}",
            p.presentation,
            p.visible,
            p.native_visible
        );
        std::thread::yield_now();
    }
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

/// Poll until the portal snapshot satisfies `predicate` (the native-portal-
/// hardening lifecycle waits: fullscreen/minimize/space transitions settle
/// asynchronously through real AppKit + the Window Server, so the tests
/// synchronize on typed snapshot state under the standard deadline).
/// Returns the satisfying snapshot.
pub(super) fn wait_portal_state(
    client: &mut Client,
    deadline: Instant,
    what: &str,
    predicate: impl Fn(&bbr_testkit::protocol::PortalSnapshot) -> bool,
) -> bbr_testkit::protocol::Snapshot {
    loop {
        let snapshot = client.snapshot().expect("portal lifecycle snapshot");
        if predicate(&snapshot.portal) {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "portal lifecycle state {what:?} did not converge; last: \
             parent_fullscreen {}, parent_minimized {}, frame_synced {}, \
             native_visible {}, owns_probe {}, portal #{}, parent #{}",
            snapshot.portal.parent_fullscreen,
            snapshot.portal.parent_minimized,
            snapshot.portal.frame_synced,
            snapshot.portal.native_visible,
            snapshot.portal.portal_owns_probe_point,
            snapshot.portal.portal_window_number,
            snapshot.portal.parent_window_number
        );
        std::thread::yield_now();
    }
}

/// Assert the active-space membership invariant: the portal is an attached
/// child, so it must always agree with its parent on Window Server
/// active-space membership (real `isOnActiveSpace` queries).
pub(super) fn assert_active_space_agreement(snapshot: &Snapshot) {
    let p = &snapshot.portal;
    assert!(
        p.parent_on_active_space,
        "the browser window must be on the active space"
    );
    assert_eq!(
        p.parent_on_active_space, p.portal_on_active_space,
        "the portal must follow its parent's space membership"
    );
}

/// Poll the real Window Server (from the test process) until the app process
/// `pid` has no on-screen window left. Bounded by `deadline`; on timeout the
/// failure reports the current on-screen count. The native-portal-hardening
/// close evidence: closing the browser window while a portal presentation is
/// open must never leave the portal window on screen (never orphaned, never
/// occluding, never intercepting input).
pub(super) fn wait_no_onscreen_windows(pid: i32, deadline: Instant) -> usize {
    loop {
        let count = bbr_web_macos::window_server::onscreen_window_count(pid);
        if count == 0 {
            return count;
        }
        assert!(
            Instant::now() < deadline,
            "BBR windows still on screen after the browser window closed: {count} \
             (the portal must never be left orphaned/occluding/intercepting)"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
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

/// Poll the real AppKit Window Server until the probe point is owned either
/// by the shown browser parent or by a window that is provably NOT this
/// app's (a foreign overlay), while the portal is idle. This is the
/// native-boundary complement to [`assert_host_closed`]: a logically closed
/// snapshot alone could miss a stale on-screen child window, whereas the
/// raw window-number query proves the hidden portal neither occludes nor
/// receives pointer events at the probe point.
///
/// The acceptance does NOT assume that no foreign window covers the probe
/// point: `windowNumberAtPoint:belowWindowWithWindowNumber:` answers across
/// ALL applications, so another app's always-on-top window or a system
/// overlay can legitimately be frontmost there while the BBR parent is
/// perfectly ordinary underneath. The measured
/// `frontmost_at_probe_point_is_this_app` fact (an `NSApplication.windows`
/// membership test on the macOS host) classifies the raw answer:
///
/// - BBR-owned frontmost at the point ⇒ it MUST be the parent window; a
///   stale portal or any other BBR window there is the regression this test
///   exists for (the portal is additionally excluded by a dedicated hard
///   assertion on every poll);
/// - foreign frontmost at the point ⇒ accepted as environmental, but only
///   after every hard invariant (host closed, not natively visible, not
///   owning hit testing, never the portal window) held on the same snapshot;
/// - no window at the point ⇒ hard fail (consistent with the open-state
///   helper): an on-screen shown parent must resolve some window.
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
        let parent_owns = p.frontmost_at_probe_point == Some(p.parent_window_number);
        let foreign_overlay =
            p.frontmost_at_probe_point.is_some() && !p.frontmost_at_probe_point_is_this_app;
        if parent_owns || foreign_overlay {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "shown browser parent did not regain Window Server ownership while portal was idle: \
             frontmost {:?} (own-process {}, parent {}, portal {}), native_visible {}, \
             owns_probe {}",
            p.frontmost_at_probe_point,
            p.frontmost_at_probe_point_is_this_app,
            p.parent_window_number,
            p.portal_window_number,
            p.native_visible,
            p.portal_owns_probe_point
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
