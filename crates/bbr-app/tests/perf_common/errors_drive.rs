//! Error-surface scenario driving (task packet `error-surface-contract`)
//! for the warm interaction matrix.
//!
//! Owns the error-surface matrix block: the production incident-presentation
//! (`error_surface_present`) — 30 accepted samples through the typed
//! `BenchmarkErrorSurfacePresent` request, each committing only from a
//! main-window render of the content area that drew the error surface for
//! the armed navigation category over the unchanged active view
//! (fast-but-wrong guard). Fixture/network time is excluded by
//! construction: the classified incident is applied in-process through the
//! exact seam the native delegate's incident sink drives (`on_incident`),
//! never through a network transfer — presenting the surface is ordinary
//! GPUI chrome in the warm next-frame presentation class (the same budget
//! as the downloads panel open). The block dismisses between samples
//! (unmeasured) so every sample measures the presentation transition, and
//! restores the matrix state — surface cleared, tabs/sessions untouched —
//! before the dense seeding that follows.
//!
//! Split from `drive.rs` so the error-surface additions live in a focused
//! module under the ~500-line budget (same pattern as `command_drive.rs` /
//! `find_drive.rs` / `downloads_drive.rs`).

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, ErrorSurfaceCategory,
    PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;
use super::harness::AppHarness;
use super::scenarios::TrialConfig;

/// Drive the error-surface matrix block and return the raw samples per
/// scenario. Runs after the downloads block and before the dense seeding;
/// state before it: the default space's two warm fixture tabs, no error
/// surface. The block measures the warm presentation 30 times (each sample
/// a fresh presentation from the cleared surface), then restores the matrix
/// state (surface dismissed, the same tabs and sessions).
pub(super) fn drive_error_surface_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    let start = client
        .snapshot()
        .expect("error-surface block start snapshot");
    let view = start.active_web_view.expect("active fixture view");
    let sessions_before = start.engine_sessions;
    let tabs_before = start.tabs.len();

    let mut samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let measured =
            benchmark_error_surface_present(client, since, *operation, view, sessions_before);
        samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("error_surface_present".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured dismissal between samples: every sample measures the
        // presentation transition, never a re-presentation of an already
        // visible surface (which the in-app probe would reject anyway).
        client
            .send_ok(&Request::ClickErrorDismiss)
            .expect("dismiss between error-surface samples");
        wait_surface_cleared(client);
    }

    // Restoration: the surface is dismissed; presenting it never touched
    // tabs, sessions, the active view, or the space, so the matrix state is
    // exactly what the dense seeding expected before the block.
    let restored = client
        .snapshot()
        .expect("error-surface block restored snapshot");
    assert!(!restored.error_surface.visible, "surface cleared");
    assert_eq!(
        restored.active_web_view,
        Some(view),
        "the error surface never changes the active view"
    );
    assert_eq!(restored.engine_sessions, sessions_before);
    assert_eq!(restored.tabs.len(), tabs_before);

    let mut raw = BTreeMap::new();
    raw.insert("error_surface_present".to_string(), samples);
    raw
}

/// Measure the production error-surface presentation: the classified
/// incident enters through the exact seam the native incident sink drives
/// (`on_incident` → the contract `present` lifecycle) and the sample
/// commits only from a main-window render that drew the error surface for
/// the armed navigation category over the unchanged active view. The
/// driver asserts the measured state and the typed snapshot.
fn benchmark_error_surface_present(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_view: WebViewId,
    sessions: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkErrorSurfacePresent { operation_id })
        .expect("benchmark error-surface present");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::ErrorSurfacePresent,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("error-surface present snapshot");
    assert!(snapshot.error_surface.visible, "the surface is presented");
    assert_eq!(snapshot.error_surface.view, Some(expected_view));
    assert_eq!(
        snapshot.error_surface.category,
        Some(ErrorSurfaceCategory::Navigation)
    );
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(
        snapshot.engine_sessions, sessions,
        "the presentation never touches sessions"
    );
    measured
}

/// Poll until no error surface is shown (bounded by the suite deadline).
fn wait_surface_cleared(client: &mut Client) {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("error-surface cleared snapshot");
        if !snapshot.error_surface.visible {
            return;
        }
        assert!(
            Instant::now() < until,
            "error surface did not clear: {:?}",
            snapshot.error_surface
        );
        std::thread::yield_now();
    }
}
