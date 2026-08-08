//! Find-in-page benchmark drivers (task packet `find-in-page-portal-bar`)
//! for the real-app performance suites.
//!
//! `benchmark_find_open` measures the production `Cmd-F` path through the
//! `BenchmarkFindOpen` request: the sample commits in the app only when a
//! render of the one `NativeAboveWeb` portal window drew the find bar open
//! with the field focused over the same active view. `benchmark_find_query_result`
//! and `benchmark_find_step` measure the production query/step paths: the
//! sample commits only when WebKit's content process reported the armed
//! count/ordinal through the portable contract and the bar re-rendered it
//! (the in-app probe owns the render-inclusive guard; the driver asserts the
//! measured state and the typed snapshot). The deterministic fixture page is
//! loaded before any sample, so fixture/network time is never part of the
//! measured duration.
//!
//! This focused module owns the find additions so `benchmarks.rs` stays at
//! its pre-find size (same split as `command.rs`).

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;

/// The find bar's presentation id (`PresentationId(0xC3)` in
/// `crate::app::find`), exposed through the snapshot's typed
/// `portal.presentation` field.
pub(crate) const FIND_PRESENTATION_ID: u64 = 0xC3;

/// The production `Cmd-F` open through the `BenchmarkFindOpen` request: the
/// bar presents in the portal window with the field focused, the same active
/// view stays selected, and no tab/session is created or touched. The
/// driver asserts the measured state and the typed snapshot.
pub(crate) fn benchmark_find_open(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkFindOpen { operation_id })
        .expect("benchmark find open");
    let (seq, measured) = wait_measurement(client, BenchmarkKind::FindOpen, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        !measured.chrome_focused,
        "the find field owns focus while the bar is open"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("find open snapshot");
    assert!(snapshot.find.open, "the find presentation is open");
    assert!(snapshot.find.focused, "the find field owns GPUI focus");
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert!(!snapshot.chrome_focused);
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Inactive);
    assert_eq!(snapshot.portal.presentation, Some(FIND_PRESENTATION_ID));
    measured
}

/// The production find-query path (`find_set_query`): the sample commits
/// only when WebKit reported `expected_count`/`expected_ordinal` for
/// `query` through the portable contract and the bar re-rendered the report
/// (fast-but-wrong guard in the app). The driver asserts the measured state
/// and the typed snapshot report.
pub(crate) fn benchmark_find_query_result(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    query: &str,
    expected_count: usize,
    expected_ordinal: Option<usize>,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkFindQueryResult {
            operation_id,
            query: query.to_string(),
            expected_count,
            expected_ordinal,
        })
        .expect("benchmark find query result");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::FindQueryResult, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(!measured.chrome_focused);
    let snapshot = client.snapshot().expect("find query snapshot");
    assert_eq!(snapshot.find.query, query);
    assert_eq!(snapshot.find.match_count, Some(expected_count));
    assert_eq!(snapshot.find.current_match, expected_ordinal);
    assert!(!snapshot.find.pending, "the report landed");
    assert!(!snapshot.find.stale);
    assert!(snapshot.find.open);
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    measured
}

/// The production find step (`find_next`/`find_previous`): the sample
/// commits only when WebKit reported the armed ordinal and the bar
/// re-rendered it.
pub(crate) fn benchmark_find_step(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    next: bool,
    expected_count: usize,
    expected_ordinal: Option<usize>,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkFindStep {
            operation_id,
            next,
            expected_count,
            expected_ordinal,
        })
        .expect("benchmark find step");
    let (seq, measured) = wait_measurement(client, BenchmarkKind::FindStep, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(!measured.chrome_focused);
    let snapshot = client.snapshot().expect("find step snapshot");
    assert_eq!(snapshot.find.match_count, Some(expected_count));
    assert_eq!(snapshot.find.current_match, expected_ordinal);
    assert!(!snapshot.find.pending);
    assert!(!snapshot.find.stale);
    assert!(snapshot.find.open);
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    measured
}
