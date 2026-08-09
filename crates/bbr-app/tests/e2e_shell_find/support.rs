//! Shared assertions and shown-harness support for the find-in-page E2E
//! (task packet `find-in-page-portal-bar`).
//!
//! Reuses the merged native-portal evidence contract (owner-classified
//! frontmost fact, child attachment, frame sync) for the find bar: the bar
//! renders inside the one `NativeAboveWeb` portal window, whose frame is
//! pinned to the parent content rect, so the same AppKit Window Server
//! evidence proves the bar wins hit testing above the active WKWebView.
//!
//! All geometry mirrors the settled S1 tokens the app resolves against
//! (`FIND_BAR_WIDTH` = the §11.4 compact clamp 360 pt, `FIND_BAR_HEIGHT` =
//! `density:row` 36 pt at the default Normal density, `FIND_GAP` = `sp:2`
//! 8 pt, top-right anchored). No invented sizes.

use std::path::PathBuf;
use std::time::Instant;

use crate::e2e_common::*;
use bbr_testkit::protocol::{dsl, Snapshot};

/// The find bar's presentation id (`PresentationId(0xC3)` in
/// `crate::app::find`), exposed through the snapshot's typed
/// `portal.presentation` field.
pub(super) const FIND_PRESENTATION_ID: u64 = 0xC3;
/// The §11.4 command-surface compact clamp width (settled S1 token).
pub(super) const FIND_BAR_WIDTH: f32 = 360.0;
/// `density:row` at the default Normal density (settled S1 token).
pub(super) const FIND_BAR_HEIGHT: f32 = 36.0;
/// `sp:2` (settled S1 token): the top/right gap and the snap margin.
pub(super) const FIND_GAP: f32 = 8.0;
/// The stable §15 ids the find bar renders (DESIGN.md §15; registered in
/// `crate::portal::probe::PortalProbeState::ids`).
pub(super) const FIND_IDS: [&str; 7] = [
    "bbr-find-bar",
    "bbr-find-field",
    "bbr-find-text",
    "bbr-find-count",
    "bbr-find-previous",
    "bbr-find-next",
    "bbr-find-close",
];

pub(super) const FIND_REPEATED_TITLE: &str = "BBR Find Repeated";
pub(super) const FIND_ZERO_TITLE: &str = "BBR Find Zero";
pub(super) const FIND_MUTATE_TITLE: &str = "BBR Find Mutate";
pub(super) const FIND_MUTATE_CHANGED_TITLE: &str = "BBR Find Mutate (changed)";
pub(super) const FIND_ISOLATION_TITLE: &str = "BBR Find Isolation (probe active)";

/// Start the fixture server on the deterministic find pages.
pub(super) fn find_fixtures() -> (bbr_fixtures::FixtureServer, String, String, String, String) {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = bbr_fixtures::FixtureServer::start(&fixtures_dir).expect("fixture server");
    let repeated = fixtures.url("/find-repeated.html");
    let zero = fixtures.url("/find-zero.html");
    let mutate = fixtures.url("/find-mutate.html");
    let isolation = fixtures.url("/find-isolation.html");
    (fixtures, repeated, zero, mutate, isolation)
}

/// Spawn the harness with a SHOWN, activated window (no `--bbr-hidden`).
///
/// The find bar's frontmost-ownership evidence (the bar's native portal
/// window wins AppKit hit testing above the active WKWebView) needs a real
/// on-screen parent, exactly like the merged portal suite. Geometry stays
/// deterministic after [`wait_for_window_settled`].
pub(super) fn spawn_harness_shown(root_tag: &str, page: &str) -> (Harness, PathBuf, String) {
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
            page,
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

/// Dispatch one keystroke through GPUI's real input path (the same seam the
/// command suite uses): `cmd-f` resolves the `FindInPage` key binding,
/// `enter`/`shift-enter` reach `on_find_key_down`, `escape` resolves the
/// `FocusWeb` binding that closes an open find presentation.
pub(super) fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

/// The find bar's resolved rect for a window of `w`×`h` (points): top-right
/// anchored with the `sp:2` gap, derived from the same placement math the
/// app resolves (`crate::portal::geometry::anchored_rect` with
/// `SnapToWindowWithMargin(sp2)` and no client inset).
pub(super) fn find_bar_rect(w: f32, h: f32) -> [f32; 4] {
    let _ = h;
    [
        w - FIND_GAP - FIND_BAR_WIDTH,
        FIND_GAP,
        FIND_BAR_WIDTH,
        FIND_BAR_HEIGHT,
    ]
}

/// Assert the typed open find bar (task packet evidence section): the
/// `0xC3` `NativeAboveWeb` presentation, the stable `bbr-find-*` ids, field
/// focus, child attachment, native visibility, exact parent-content frame
/// sync, the resolved top-right rect, and the non-key/non-main policy. The
/// frontmost hit-test ownership is asserted separately
/// ([`assert_frontmost_ownership`]) because it needs an on-screen parent.
pub(super) fn assert_find_open(snapshot: &Snapshot) {
    let p = &snapshot.portal;
    let f = &snapshot.find;
    assert!(f.open, "find presentation open");
    assert!(f.focused, "find field owns GPUI focus while open");
    assert_eq!(p.presentation, Some(FIND_PRESENTATION_ID));
    assert_eq!(p.host.as_deref(), Some("native-above-web"));
    assert!(p.visible);
    assert_eq!(
        p.ids,
        FIND_IDS.iter().map(|s| s.to_string()).collect::<Vec<_>>()
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
        "the find bar is its own window, not the browser window"
    );
    assert!(!p.portal_is_key, "showing never makes the portal key");
    assert!(!p.portal_is_main, "showing never makes the portal main");
    assert_eq!(
        p.rect,
        Some(find_bar_rect(
            snapshot.window_size[0],
            snapshot.window_size[1]
        )),
        "the find bar resolves top-right anchored from the present-time window"
    );
}

/// Assert the find presentation is fully dismissed: `find.open` false and
/// the one portal host back to its typed closed default (nothing visible,
/// no ids, still attached, never key/main).
pub(super) fn assert_find_closed(snapshot: &Snapshot) {
    assert!(!snapshot.find.open, "find presentation closed");
    assert!(!snapshot.find.focused, "find field focus released");
    let p = &snapshot.portal;
    assert_eq!(p.presentation, None, "no active presentation");
    assert_eq!(p.host, None, "no active host");
    assert!(!p.visible, "logical visibility false while closed");
    assert_eq!(p.rect, None, "no resolved rect while closed");
    assert!(
        p.ids.is_empty(),
        "no bbr-find-* ids while closed: {:?}",
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
    assert!(!p.portal_is_key, "non-key policy holds");
    assert!(!p.portal_is_main, "non-main policy holds");
}

/// Assert that browser ownership and the active session identity are
/// structurally untouched by find mutations (the merged portal suite's
/// invariant, minus the chrome-focus equality: the find bar's focus
/// choreography deliberately restores the captured origin focus domain, so
/// a chrome-origin session legitimately ends chrome-focused; the bar's own
/// focus state is asserted separately via the typed `find.focused`).
pub(super) fn assert_ownership_preserved(snapshot: &Snapshot, baseline: &Snapshot) {
    assert_eq!(
        snapshot.active_web_view, baseline.active_web_view,
        "find mutations must never change the selected WebViewId"
    );
    assert_eq!(
        snapshot.engine_sessions, baseline.engine_sessions,
        "find mutations must never create, destroy, or recreate sessions"
    );
    assert_eq!(
        snapshot.active_profile, baseline.active_profile,
        "find mutations must never change the active profile"
    );
    assert_eq!(
        snapshot.active_space, baseline.active_space,
        "find mutations must never change the active space"
    );
}

/// Poll until the find report reaches `match_count`/`current_match` with no
/// request pending (a WebKit completion landed). Bounded by the standard
/// deadline; on timeout the failure reports the last snapshot state.
pub(super) fn wait_find_report(
    client: &mut Client,
    count: usize,
    ordinal: Option<usize>,
    deadline: Instant,
) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("find report snapshot");
        let f = &snapshot.find;
        if f.match_count == Some(count) && f.current_match == ordinal && !f.pending {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "find report did not land: expected count {count} ordinal {ordinal:?}; last {:?}",
            f
        );
        std::thread::yield_now();
    }
}

/// Poll until a find presentation is open (the `Cmd-F` action path defers
/// into the next update, so the snapshot can observe the pre-open frame).
pub(super) fn wait_find_open(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("find open snapshot");
        if snapshot.find.open
            && snapshot.portal.presentation == Some(FIND_PRESENTATION_ID)
            && snapshot.portal.visible
            && snapshot.portal.native_visible
            && snapshot.portal.frame_synced
        {
            assert_find_open(&snapshot);
            return snapshot;
        }
        assert!(Instant::now() < deadline, "find did not open");
        std::thread::yield_now();
    }
}

/// Poll until the find presentation is fully dismissed ([`assert_find_closed`]
/// holds).
pub(super) fn wait_for_find_closed(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("find closed snapshot");
        if !snapshot.find.open
            && snapshot.portal.presentation.is_none()
            && !snapshot.portal.visible
            && !snapshot.portal.native_visible
        {
            assert_find_closed(&snapshot);
            return snapshot;
        }
        assert!(Instant::now() < deadline, "find did not close");
        std::thread::yield_now();
    }
}

/// Poll until a document change made the reported find results stale (the
/// bar stays open; [`assert_find_closed`] is the dismissal contract).
pub(super) fn wait_for_find_stale(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("find stale snapshot");
        if snapshot.find.stale {
            assert!(snapshot.find.open, "stale find presentation must stay open");
            return snapshot;
        }
        assert!(Instant::now() < deadline, "find did not turn stale");
        std::thread::yield_now();
    }
}

/// Wait until the fixture page finished loading (its title is known), the
/// same load-completion synchronization the merged portal suite uses: the
/// find query must never race the page's late `LoadingChanged` events, or
/// the in-flight request is invalidated and its WebKit completion dropped.
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

/// Assert native hit-test ownership at the portal frame center over the
/// parent content and therefore over its WKWebViews (the merged portal
/// suite's owner-classified evidence; the find bar's portal window is pinned
/// to the same parent-content frame, so the same Window Server queries prove
/// the bar wins hit testing above the active web view).
pub(super) fn assert_frontmost_ownership(snapshot: &Snapshot) {
    let p = &snapshot.portal;
    assert!(
        p.portal_owns_probe_point,
        "the find portal owns hit testing at the probe point over the web content"
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
/// AppKit Window Server query corroborate the attached-child ordering
/// (merged portal suite helper, reused verbatim).
pub(super) fn wait_for_frontmost_evidence(client: &mut Client, deadline: Instant) {
    loop {
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
