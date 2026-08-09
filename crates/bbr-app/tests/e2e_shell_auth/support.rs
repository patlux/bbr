//! Shared assertions and shown-harness support for the credential-sheet E2E
//! (queue item 10).
//!
//! The sheet renders inside the one `NativeAboveWeb` portal window pinned to
//! the parent content rect, so the merged native-portal evidence contract
//! (owner-classified frontmost fact, child attachment, frame sync) proves
//! the sheet wins hit testing above the active WKWebView exactly like the
//! find bar. Geometry mirrors the settled app tokens: `AUTH_SHEET_WIDTH` =
//! the §11.4 compact clamp 360 pt, `AUTH_SHEET_HEIGHT` = the conventional
//! credential-sheet 236 pt, centered with the `sp:2`-family 8 pt margin.

use std::path::PathBuf;
use std::time::Instant;

use crate::e2e_common::*;
use bbr_testkit::protocol::{dsl, Snapshot};
use bbr_web::WebViewId;

/// The credential sheet's presentation id (`PresentationId(0xC4)` in
/// `crate::app::auth`), exposed through the snapshot's typed
/// `portal.presentation` field.
pub(super) const AUTH_PRESENTATION_ID: u64 = 0xC4;
/// The §11.4 command-surface compact clamp width (settled S1 token).
pub(super) const AUTH_SHEET_WIDTH: f32 = 360.0;
/// The conventional macOS credential sheet height (header + two fields +
/// two buttons; settled app token).
pub(super) const AUTH_SHEET_HEIGHT: f32 = 236.0;
/// `sp:2` (settled S1 token): the centered-placement snap margin.
pub(super) const AUTH_MARGIN: f32 = 8.0;
/// The stable §15 ids the credential sheet renders (DESIGN.md §15;
/// registered in `crate::portal::probe::PortalProbeState::ids`).
pub(super) const AUTH_IDS: [&str; 6] = [
    "bbr-auth-sheet",
    "bbr-auth-header",
    "bbr-auth-username",
    "bbr-auth-password",
    "bbr-auth-submit",
    "bbr-auth-cancel",
];

/// The deterministic challenge fixture the suites present: an HTTPS origin
/// Basic challenge for `intranet.example` with realm `Finance VPN` bound to
/// the owning web view passed by the caller.
pub(super) struct ChallengeFixture {
    pub profile: bbr_web::ProfileId,
    pub web_view: WebViewId,
}

impl ChallengeFixture {
    pub fn for_view(web_view: WebViewId) -> Self {
        Self {
            profile: bbr_web::ProfileId(1),
            web_view,
        }
    }

    /// Record the awaiting challenge WITHOUT presenting the sheet (the
    /// benchmark fixture and the staged-presentation scenarios).
    pub fn record(&self, client: &mut Client) {
        client
            .send_ok(&dsl::auth_record_challenge(self.profile, self.web_view))
            .expect("record auth challenge");
    }

    /// Present the challenge through the production challenge-arrival seam
    /// (`BbrApp::auth_on_challenge`): the sheet opens over the active window.
    pub fn present(&self, client: &mut Client) {
        client
            .send_ok(&dsl::auth_present_challenge(self.profile, self.web_view))
            .expect("present auth challenge");
    }
}

/// Spawn the harness with a SHOWN, activated window (no `--bbr-hidden`).
///
/// The sheet's frontmost-ownership evidence (the sheet's native portal
/// window wins AppKit hit testing above the active WKWebView) needs a real
/// on-screen parent, exactly like the merged portal suite.
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

/// Dispatch one keystroke through GPUI's real input path (the same seam the
/// command/find suites use): the main-window root listener routes printable
/// chars to the focused sheet field, `tab` moves between fields, `enter`
/// submits, `escape` cancels.
pub(super) fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&dsl::keystroke(keystroke))
        .expect("dispatch keystroke");
}

/// Type one printable character into the focused sheet field (production key
/// path). Typing is deliberately char-by-char so the sheet's redacted edit
/// state is exercised one event at a time.
pub(super) fn type_char(client: &mut Client, ch: char) {
    key(client, &ch.to_string());
}

/// The credential sheet's resolved rect for a window of `w`×`h` (points):
/// centered with the `sp:2`-family margin, clamped to the viewport (the
/// same placement math the app resolves via
/// `crate::portal::geometry::centered_rect`).
pub(super) fn auth_sheet_rect(w: f32, h: f32) -> [f32; 4] {
    let x = ((w - AUTH_SHEET_WIDTH) * 0.5).max(AUTH_MARGIN);
    let y = ((h - AUTH_SHEET_HEIGHT) * 0.5).max(AUTH_MARGIN);
    [x, y, AUTH_SHEET_WIDTH, AUTH_SHEET_HEIGHT]
}

/// Assert the typed open credential sheet (queue item 10 evidence section):
/// the `0xC4` `NativeAboveWeb` presentation, the stable `bbr-auth-*` ids,
/// the redacted authority metadata of the presented challenge, the
/// `response_requested` contract lifecycle, a fresh masked edit state, child
/// attachment, native visibility, exact parent-content frame sync, the
/// centered rect, and the non-key/non-main policy. `web_view` must equal the
/// owning view of the presented challenge (a sheet can never present another
/// tab's challenge).
pub(super) fn assert_sheet_open(snapshot: &Snapshot, web_view: WebViewId) {
    let a = &snapshot.auth;
    let p = &snapshot.portal;
    assert!(a.open, "credential sheet open");
    assert_eq!(
        a.challenge,
        Some(1),
        "the first fixture challenge is presented"
    );
    assert_eq!(
        a.web_view,
        Some(web_view),
        "the sheet presents the owning view's challenge"
    );
    assert_eq!(a.host, "intranet.example");
    assert_eq!(a.realm.as_deref(), Some("Finance VPN"));
    assert_eq!(a.scheme, "Basic");
    assert_eq!(a.authority, "origin");
    assert_eq!(a.state, "response_requested");
    assert_eq!(a.outcome, None, "a fresh sheet awaits a decision");
    assert_eq!(a.focused_field.as_deref(), Some("username"));
    assert!(!a.username_entered, "fresh sheet has no typed username");
    assert!(!a.password_entered, "fresh sheet has no typed password");
    assert_eq!(p.presentation, Some(AUTH_PRESENTATION_ID));
    assert_eq!(p.host.as_deref(), Some("native-above-web"));
    assert!(p.visible);
    assert_eq!(
        p.ids,
        AUTH_IDS.iter().map(|s| s.to_string()).collect::<Vec<_>>()
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
        "the sheet is its own window, not the browser window"
    );
    assert!(!p.portal_is_key, "showing never makes the portal key");
    assert!(!p.portal_is_main, "showing never makes the portal main");
    assert_eq!(
        p.rect,
        Some(auth_sheet_rect(
            snapshot.window_size[0],
            snapshot.window_size[1]
        )),
        "the sheet resolves centered from the present-time window"
    );
}

/// Assert the credential sheet is fully dismissed: `auth.open` false and the
/// one portal host back to its typed closed default (nothing visible, no
/// ids, still attached, never key/main).
pub(super) fn assert_sheet_closed(snapshot: &Snapshot) {
    let a = &snapshot.auth;
    let p = &snapshot.portal;
    assert!(!a.open, "credential sheet closed");
    assert_eq!(p.presentation, None, "no active presentation");
    assert_eq!(p.host, None, "no active host");
    assert!(!p.visible, "logical visibility false while closed");
    assert_eq!(p.rect, None, "no resolved rect while closed");
    assert!(
        p.ids.is_empty(),
        "no bbr-auth-* ids while closed: {:?}",
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
/// structurally untouched by sheet mutations: the sheet never changes the
/// selected WebViewId, the session count, the active profile, or the active
/// space (the merged portal suite's invariant).
pub(super) fn assert_ownership_preserved(snapshot: &Snapshot, baseline: &Snapshot) {
    assert_eq!(
        snapshot.active_web_view, baseline.active_web_view,
        "sheet mutations must never change the selected WebViewId"
    );
    assert_eq!(
        snapshot.engine_sessions, baseline.engine_sessions,
        "sheet mutations must never create, destroy, or recreate sessions"
    );
    assert_eq!(
        snapshot.active_profile, baseline.active_profile,
        "sheet mutations must never change the active profile"
    );
    assert_eq!(
        snapshot.active_space, baseline.active_space,
        "sheet mutations must never change the active space"
    );
}

/// Poll until a credential sheet is open with the owning `web_view` (the
/// presentation defers into the next update, so the snapshot can observe the
/// pre-open frame). Bounded by the standard deadline.
pub(super) fn wait_sheet_open(
    client: &mut Client,
    web_view: WebViewId,
    deadline: Instant,
) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("auth open snapshot");
        if snapshot.auth.open
            && snapshot.auth.web_view == Some(web_view)
            && snapshot.portal.presentation == Some(AUTH_PRESENTATION_ID)
            && snapshot.portal.visible
            && snapshot.portal.native_visible
            && snapshot.portal.frame_synced
        {
            return snapshot;
        }
        assert!(Instant::now() < deadline, "credential sheet did not open");
        std::thread::yield_now();
    }
}

/// Poll until the credential sheet is fully dismissed ([`assert_sheet_closed`]
/// holds).
pub(super) fn wait_sheet_closed(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("auth closed snapshot");
        if !snapshot.auth.open
            && snapshot.portal.presentation.is_none()
            && !snapshot.portal.visible
            && !snapshot.portal.native_visible
        {
            assert_sheet_closed(&snapshot);
            return snapshot;
        }
        assert!(Instant::now() < deadline, "credential sheet did not close");
        std::thread::yield_now();
    }
}

/// Assert native hit-test ownership at the portal frame center over the
/// parent content and therefore over its WKWebViews (the merged portal
/// suite's owner-classified evidence): the sheet's portal window wins
/// hit testing above the active web view.
pub(super) fn assert_frontmost_ownership(snapshot: &Snapshot) {
    let p = &snapshot.portal;
    assert!(
        p.portal_owns_probe_point,
        "the auth portal owns hit testing at the probe point over the web content"
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
/// (merged portal suite helper).
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

/// The redaction proof: serialize the ENTIRE wire snapshot and assert the
/// typed credential strings appear nowhere — not in the auth fields, not in
/// any other snapshot field. The only representation that may leave the
/// sheet is the contract's redaction (`credential:#<n>`).
pub(super) fn assert_redaction(snapshot: &Snapshot, secrets: &[&str]) {
    let wire = serde_json::to_string(snapshot).expect("snapshot serializes");
    for secret in secrets {
        assert!(
            !wire.contains(secret),
            "the wire snapshot must never contain {secret:?} (redaction invariant)"
        );
    }
    let debug = format!("{snapshot:#?}");
    for secret in secrets {
        assert!(
            !debug.contains(secret),
            "the Debug shape must never contain {secret:?} (redaction invariant)"
        );
    }
}

/// The fixture page URLs for the auth suite (a real WKWebView backs the
/// active tab so tab/session identity is real).
pub(super) fn auth_fixtures() -> (bbr_fixtures::FixtureServer, String, String) {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = bbr_fixtures::FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");
    (fixtures, page_one, page_two)
}
