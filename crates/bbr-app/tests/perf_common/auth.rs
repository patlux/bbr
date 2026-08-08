//! Credential-sheet benchmark driver (queue item 10) for the real-app
//! performance suites.
//!
//! `benchmark_auth_sheet_open` measures the production presentation of the
//! credential sheet through the `BenchmarkAuthSheetOpen` request: the sample
//! starts at `auth_present_sheet` (the exact path a challenge arrival
//! drives) and commits in the app only when a render of the one
//! `NativeAboveWeb` portal window drew the sheet with the armed challenge id
//! (fast-but-wrong guard). The awaiting challenge is the fixture, recorded
//! before the sample, so fixture/network time is excluded by construction.
//! The driver asserts the measured state and the typed snapshot.

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;

/// The production credential-sheet open through the `BenchmarkAuthSheetOpen`
/// request: the sheet presents centered in the one `NativeAboveWeb` portal
/// window showing the awaiting challenge, the same active view stays
/// selected, the sidebar stays visible, and no tab/session is created or
/// touched. The driver asserts the measured state and the typed snapshot
/// (open, owning view, masked edit state, no credential anywhere).
pub(crate) fn benchmark_auth_sheet_open(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkAuthSheetOpen { operation_id })
        .expect("benchmark auth sheet open");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::AuthSheetOpen, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        !measured.chrome_focused,
        "the sheet owns keyboard focus (chrome released)"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("auth open snapshot");
    let a = &snapshot.auth;
    assert!(a.open, "the credential sheet is open");
    assert_eq!(
        a.web_view,
        Some(expected_view),
        "the sheet presents the owning view's challenge"
    );
    assert_eq!(a.state, "response_requested");
    assert_eq!(a.focused_field.as_deref(), Some("username"));
    assert!(
        !a.username_entered && !a.password_entered,
        "fresh masked sheet"
    );
    assert_eq!(
        snapshot.portal.presentation,
        Some(0xC4),
        "the auth portal presentation"
    );
    assert!(
        snapshot.portal.ids.iter().any(|id| id == "bbr-auth-sheet"),
        "the sheet rendered its stable ids"
    );
    measured
}
