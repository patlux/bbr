//! Zoom-in benchmark drivers (queue item 11, part A) for the real-app
//! performance suites.
//!
//! `benchmark_zoom_step` measures the production `Cmd-Plus` path through the
//! `BenchmarkZoomStep` request: the sample starts at the exact seam the
//! `ZoomIn` key binding routes to and commits in the app only when the
//! observed zoom state carries the armed level — the level actually applied
//! to the active view (`applied == Some(level)`, the `pageZoom` property
//! write completed) with no pending persistence write (the in-app probe owns
//! the fast-but-wrong guard; the driver asserts the measured state and the
//! typed snapshot). The deterministic fixture page is loaded before any
//! sample, so fixture/network time is never part of the measured duration.

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;

/// The production `Cmd-Plus` zoom step through the `BenchmarkZoomStep`
/// request: the lifecycle change, the persistence write, and the native
/// `pageZoom` application on the active view. The driver asserts the
/// measured state and the typed snapshot (effective level, applied level,
/// no pending write; active view unchanged).
pub(crate) fn benchmark_zoom_step(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    level: u16,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkZoomStep {
            operation_id,
            level,
        })
        .expect("benchmark zoom step");
    let (seq, measured) = wait_measurement(client, BenchmarkKind::ZoomStep, operation_id, *since);
    *since = seq;
    assert_eq!(
        measured.active_web_view,
        Some(expected_view),
        "the zoom step keeps the active view"
    );
    let snapshot = client.snapshot().expect("zoom step snapshot");
    assert_eq!(
        snapshot.zoom.level, level,
        "the zoom snapshot reports the level"
    );
    assert_eq!(
        snapshot.zoom.applied,
        Some(level),
        "the level is applied to the native view"
    );
    assert!(
        !snapshot.zoom.pending,
        "the persistence write is acknowledged"
    );
    assert_eq!(
        snapshot.active_web_view,
        Some(expected_view),
        "no session identity changed"
    );
    measured
}
