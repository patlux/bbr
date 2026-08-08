//! Downloads-panel benchmark drivers (task packet `downloads-sidebar-panel`)
//! for the real-app performance suites.
//!
//! `benchmark_downloads_panel_open` measures the production footer
//! `[Downloads]` open through the `BenchmarkDownloadsPanelOpen` request: the
//! sample commits in the app only when a render of the anchored panel drew
//! it open with the keyboard cursor over the same active view.
//! `benchmark_downloads_progress` measures the production L6 event path
//! (a deterministic `DownloadEvent::Progress` applied through the same
//! main-thread pump the native adapter's sink uses): the sample commits only
//! when the panel render drew the running row at the armed received/total
//! fraction — fixture/network time is excluded by construction (the event is
//! applied in-process; no transfer runs). `benchmark_downloads_row_retry`
//! measures the production retry seam: the fresh attempt must be presented
//! in the Active group linked via `retry_of`. The drivers assert the
//! measured state and the typed snapshot; the in-app probe owns the
//! render-inclusive guard.

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, DownloadGroup, DownloadMachineState,
    DownloadRowAction, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;

/// The production footer `[Downloads]` open through the
/// `BenchmarkDownloadsPanelOpen` request: the panel presents open with the
/// keyboard cursor (chrome focus), the same active view stays selected, and
/// no tab/session is created or touched. The driver asserts the measured
/// state and the typed snapshot.
pub(crate) fn benchmark_downloads_panel_open(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDownloadsPanelOpen { operation_id })
        .expect("benchmark downloads panel open");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::DownloadsPanelOpen,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(measured.chrome_focused, "the panel owns chrome focus");
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("downloads open snapshot");
    assert!(snapshot.downloads.open, "the anchored panel is open");
    assert!(snapshot.chrome_focused);
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Inactive);
    measured
}

/// The production L6 progress path (`BenchmarkDownloadsProgress`): the
/// sample commits only when the panel render drew the running row `id` at
/// exactly `received`/`total` (fast-but-wrong guard in the app). The driver
/// asserts the measured state and the snapshot row.
pub(crate) fn benchmark_downloads_progress(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    id: u64,
    received: u64,
    total: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDownloadsProgress {
            operation_id,
            id,
            received,
            total,
        })
        .expect("benchmark downloads progress");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::DownloadsProgressCommit,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(measured.chrome_focused, "the panel owns chrome focus");
    let snapshot = client.snapshot().expect("downloads progress snapshot");
    assert!(snapshot.downloads.open);
    let row = snapshot
        .download_rows
        .iter()
        .find(|row| row.id == id)
        .unwrap_or_else(|| panic!("no row for attempt {id} in {:#?}", snapshot.download_rows));
    assert_eq!(row.machine_state, DownloadMachineState::Running);
    assert_eq!(row.group, DownloadGroup::Active);
    assert_eq!(
        row.progress_fraction,
        Some(received as f64 / total as f64),
        "the armed fraction is presented"
    );
    assert_eq!(row.action, DownloadRowAction::Cancel);
    measured
}

/// The production retry row action (`BenchmarkDownloadsRowRetry`): the
/// sample commits only when the panel render drew the fresh attempt in the
/// Active group — `Waiting`, linked via `retry_of` to the failed source row,
/// with the Cancel action. The driver asserts the measured state and that
/// the snapshot gained exactly one fresh linked row.
pub(crate) fn benchmark_downloads_row_retry(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    row: usize,
    retry_of: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let before = client.snapshot().expect("pre retry snapshot");
    let linked_before = before
        .download_rows
        .iter()
        .filter(|row| row.retry_of == Some(retry_of))
        .count();
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDownloadsRowRetry { operation_id, row })
        .expect("benchmark downloads row retry");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::DownloadsRowRetry,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(measured.chrome_focused, "the panel owns chrome focus");
    let snapshot = client.snapshot().expect("downloads retry snapshot");
    assert!(snapshot.downloads.open);
    let linked = snapshot
        .download_rows
        .iter()
        .filter(|row| row.retry_of == Some(retry_of))
        .count();
    assert_eq!(
        linked,
        linked_before + 1,
        "the retry presents exactly one fresh linked attempt"
    );
    let fresh = snapshot
        .download_rows
        .iter()
        .find(|row| {
            row.retry_of == Some(retry_of) && row.machine_state == DownloadMachineState::Waiting
        })
        .expect("fresh waiting retry row");
    assert_eq!(fresh.group, DownloadGroup::Active);
    assert_eq!(fresh.action, DownloadRowAction::Cancel);
    measured
}

/// The production restart row action on a cancelled row
/// (`BenchmarkDownloadsRowRestart`, the settled §17.8 decision): the sample
/// commits only when the panel render drew the fresh attempt in the Active
/// group — `Waiting`, linked via `retry_of` to the cancelled source row,
/// with the Cancel action (the same machine path as a failed retry). The
/// driver asserts the measured state and that the snapshot gained exactly
/// one fresh linked attempt.
pub(crate) fn benchmark_downloads_row_restart(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    row: usize,
    retry_of: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let before = client.snapshot().expect("pre restart snapshot");
    let linked_before = before
        .download_rows
        .iter()
        .filter(|row| row.retry_of == Some(retry_of))
        .count();
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDownloadsRowRestart { operation_id, row })
        .expect("benchmark downloads row restart");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::DownloadsRowRestart,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(measured.chrome_focused, "the panel owns chrome focus");
    let snapshot = client.snapshot().expect("downloads restart snapshot");
    assert!(snapshot.downloads.open);
    let linked = snapshot
        .download_rows
        .iter()
        .filter(|row| row.retry_of == Some(retry_of))
        .count();
    assert_eq!(
        linked,
        linked_before + 1,
        "the restart presents exactly one fresh linked attempt"
    );
    let fresh = snapshot
        .download_rows
        .iter()
        .find(|row| {
            row.retry_of == Some(retry_of) && row.machine_state == DownloadMachineState::Waiting
        })
        .expect("fresh waiting restart row");
    assert_eq!(fresh.group, DownloadGroup::Active);
    assert_eq!(fresh.action, DownloadRowAction::Cancel);
    measured
}

/// The production row Delete action on a cancelled row
/// (`BenchmarkDownloadsRowDelete`, the settled §17.8 decision): the sample
/// commits only when the panel render no longer draws the deleted row — the
/// durable one-row removal. The driver asserts the measured state and that
/// the snapshot lost exactly that one row.
pub(crate) fn benchmark_downloads_row_delete(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    row: usize,
    deleted_id: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDownloadsRowDelete { operation_id, row })
        .expect("benchmark downloads row delete");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::DownloadsRowDelete,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(measured.chrome_focused, "the panel owns chrome focus");
    let snapshot = client.snapshot().expect("downloads delete snapshot");
    assert!(snapshot.downloads.open);
    assert!(
        snapshot
            .download_rows
            .iter()
            .all(|row| row.id != deleted_id),
        "the deleted row is gone from the rendered panel"
    );
    measured
}

/// The production Clear panel action (`BenchmarkDownloadsClear`, the settled
/// §17.7 decision): the sample commits only when the panel render draws the
/// empty state — open, focused, zero rows. The driver asserts the measured
/// state and that the snapshot reports an empty panel.
pub(crate) fn benchmark_downloads_clear(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDownloadsClear { operation_id })
        .expect("benchmark downloads clear");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::DownloadsClear, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(measured.chrome_focused, "the panel owns chrome focus");
    let snapshot = client.snapshot().expect("downloads clear snapshot");
    assert!(snapshot.downloads.open);
    assert_eq!(
        snapshot.downloads.total_rows, 0,
        "the panel is empty after Clear"
    );
    assert!(snapshot.download_rows.is_empty(), "no rows after Clear");
    measured
}
