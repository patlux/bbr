//! Background-link benchmark driver (DD-LINK packet `background-link-tabs`)
//! for the real-app performance suites.
//!
//! `benchmark_background_link_open` measures the production
//! Command-primary / middle-click → background-child path through the
//! `BenchmarkBackgroundLinkOpen` request: the sample starts inside the app
//! when the request enters `BbrApp::on_new_window_request` (the exact seam
//! the native coordinator delivery pump and the typed testkit injection
//! share) and commits only from the authoritative post-mutation browser
//! state — the exact opener still the active, web-focused view and the new
//! child a live, hidden session at the grouped insertion index directly
//! after the opener (fast-but-wrong guard owned by the in-app probe). The
//! child session is the fresh native-session class (p95 ≤ 100 ms;
//! max ≤ 150 ms); fixture page-load time is excluded by construction (the
//! sample commits at insertion, not at page ready).

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;

/// The production background-link open through the
/// `BenchmarkBackgroundLinkOpen` request: the exact opener stays the
/// active, web-focused view and the new child is a live hidden session at
/// index `opener_index + 1` immediately after the opener — never a
/// foreground activation. The driver asserts the measured state and the
/// typed snapshot so a fast-but-wrong sample can never pass.
pub(crate) fn benchmark_background_link_open(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    opener_view: WebViewId,
    opener_index: usize,
    target_url: &str,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkBackgroundLinkOpen {
            operation_id,
            url: target_url.to_string(),
        })
        .expect("benchmark background link open");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::BackgroundLinkOpen,
        operation_id,
        *since,
    );
    *since = seq;
    // The exact opener stays the active, web-focused view (no selection
    // move, no focus bounce, no foreground activation).
    assert_eq!(
        measured.active_web_view,
        Some(opener_view),
        "the opener stays the active view"
    );
    assert!(
        !measured.chrome_focused,
        "the opener keeps web focus (no focus bounce)"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("background-link snapshot");
    let child = snapshot
        .tabs
        .get(opener_index + 1)
        .unwrap_or_else(|| panic!("child must sit after the opener at {}", opener_index + 1));
    assert_eq!(
        child.web_view.map(|_| true),
        Some(true),
        "the child owns a live session"
    );
    assert!(!child.active, "the child is never foreground-activated");
    assert_eq!(
        snapshot.active_web_view,
        Some(opener_view),
        "the opener stays selected"
    );
    assert!(
        !snapshot.chrome_focused,
        "the opener keeps web focus throughout"
    );
    measured
}
