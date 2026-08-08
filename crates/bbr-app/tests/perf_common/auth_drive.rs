//! Credential-sheet scenario driving (queue item 10) for the warm
//! interaction matrix.
//!
//! Owns the auth matrix block: the production credential-sheet open
//! (`auth_sheet_open`) with `INTERACTION_SAMPLES` accepted samples through
//! the typed benchmark request, committing only from a render of the sheet
//! that carried the armed challenge identity (fast-but-wrong guard).
//! Fixture time is excluded by construction: the awaiting challenge is
//! recorded deterministically before every sample (never through a network
//! exchange), and the measured entry is the production presentation itself.
//! The block restores the matrix state — sheet closed, tabs/sessions
//! untouched (the sheet never changes tab, session, WebView, profile, or
//! space identity) — before the dense seeding that follows.
//!
//! Redaction: the fixture request carries only authority metadata; the
//! sheet is measured empty (no typing), so no credential can exist in any
//! sample, report, or benchmark record.
//!
//! Split from `drive.rs` so the auth additions live in a focused module
//! under the ~500-line budget (same pattern as `command_drive.rs` /
//! `find_drive.rs` / `downloads_drive.rs`).

use std::collections::BTreeMap;

use bbr_testkit::protocol::Request;
use bbr_testkit::Client;

use super::auth::benchmark_auth_sheet_open;
use super::harness::AppHarness;
use super::scenarios::TrialConfig;

/// Drive the auth matrix block and return the raw samples per scenario.
/// Runs after the downloads block and before the dense seeding; state
/// before it: the default space's two warm fixture tabs, no auth surface.
/// The block measures the warm sheet open (30 samples), then restores the
/// matrix state (sheet closed, the same tabs and sessions).
pub(super) fn drive_auth_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    let start = client.snapshot().expect("auth block start snapshot");
    let view = start.active_web_view.expect("active fixture view");
    let sessions_before = start.engine_sessions;
    let tabs_before = start.tabs.len();

    // auth_sheet_open × N: every sample is a fresh production open of the
    // sheet presenting the awaiting challenge (recorded before the sample,
    // never part of the measured duration). The sample commits only when
    // the portal render drew the sheet with the armed challenge id. Escape
    // (unmeasured) cancels the presented challenge between samples so every
    // sample measures the open transition, never a re-focus.
    let mut open_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        // Deterministic awaiting-challenge fixture (authority metadata only).
        client
            .send_ok(&Request::AuthRecordChallenge {
                profile: bbr_web::ProfileId(1),
                web_view: view,
                http_scheme: bbr_web::auth_challenges::HttpScheme::Https,
                scheme: bbr_web::auth_challenges::AuthScheme::Basic,
                host: "intranet.example".to_string(),
                port: 0,
                realm: Some("Finance VPN".to_string()),
                authority: bbr_web::auth_challenges::AuthAuthority::Origin,
                user_initiated: true,
            })
            .expect("record awaiting challenge");
        let measured = benchmark_auth_sheet_open(client, since, *operation, view);
        open_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("auth_sheet_open".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured cancel closes the presented challenge and the sheet;
        // the next sample opens a fresh sheet for a fresh challenge.
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape after auth open sample");
        wait_auth_closed(client);
    }

    // Restoration: the sheet surface never touched tabs, sessions, the
    // active view, or the space, so the matrix state is exactly what the
    // dense seeding expected before the block.
    let restored = client.snapshot().expect("auth block restored snapshot");
    assert!(!restored.auth.open, "sheet closed");
    assert_eq!(
        restored.active_web_view,
        Some(view),
        "the auth surface never changes the active view"
    );
    assert_eq!(restored.engine_sessions, sessions_before);
    assert_eq!(restored.tabs.len(), tabs_before);

    let mut raw = BTreeMap::new();
    raw.insert("auth_sheet_open".to_string(), open_samples);
    raw
}

/// Poll until the credential sheet is fully closed (bounded by the harness
/// deadline).
fn wait_auth_closed(client: &mut Client) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("auth closed snapshot");
        if !snapshot.auth.open {
            return;
        }
        assert!(
            std::time::Instant::now() < until,
            "credential sheet did not close: {:?}",
            snapshot.auth
        );
        std::thread::yield_now();
    }
}
