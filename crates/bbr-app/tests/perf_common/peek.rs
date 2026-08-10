//! Transient Peek benchmark drivers (Link Preview / Peek packet) for the
//! real-app performance suites.
//!
//! `benchmark_peek_request_to_present` measures the production eligible
//! new-window → Peek path through the `BenchmarkPeekRequestToPresent`
//! request: the sample starts inside the app when the request enters
//! `BbrApp::on_new_window_request` (the exact seam the native coordinator
//! delivery pump and the typed testkit injection share) and commits only
//! from a render of the one `NativeAboveWeb` portal window that drew the
//! Peek bar for the preview session raised above its warm origin
//! (fast-but-wrong guard owned by the in-app probe). The preview is the
//! fresh native-session class (p95 ≤ 100 ms; max ≤ 150 ms); fixture/network
//! time is excluded by construction (the deterministic local target is
//! loaded after presentation, and the sample commits at presentation, not
//! at page ready).
//!
//! `benchmark_peek_dismiss_to_origin` and `benchmark_peek_promote_to_tab`
//! measure the production dismissal/promotion paths (`peek_close` /
//! `peek_promote`) through the typed benchmark requests: the samples commit
//! from the authoritative post-mutation browser state (the warm origin
//! active/focused again; the SAME preview `WebViewId` as a regular tab,
//! active and focused, no reload) — the warm interaction class
//! (p95 ≤ 50 ms; max ≤ 100 ms). Neither path creates, recreates, or
//! reloads a native session. Each driver asserts the measured state and the
//! typed snapshot so a fast-but-wrong sample can never pass.

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;

/// The stable Peek bar id (DESIGN §15 / `crate::app::peek::PEEK_BAR_ID`),
/// exposed through the snapshot's typed `portal.ids`.
pub(crate) const PEEK_BAR_ID: &str = "bbr-peek-bar";

/// The production eligible new-window → Peek presentation through the
/// `BenchmarkPeekRequestToPresent` request: the covered warm origin stays
/// the active view, the preview owns browser-command focus (chrome
/// released), and no regular tab is created. The driver asserts the
/// measured state and the typed snapshot (preview present over the origin,
/// Peek bar drawn in the portal with its stable ids).
pub(crate) fn benchmark_peek_request_to_present(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    source_view: WebViewId,
    target_url: &str,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkPeekRequestToPresent {
            operation_id,
            url: target_url.to_string(),
        })
        .expect("benchmark peek request to present");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::PeekRequestToPresent,
        operation_id,
        *since,
    );
    *since = seq;
    // The covered warm origin stays the active view; the preview owns
    // browser-command focus while open.
    assert_eq!(measured.active_web_view, Some(source_view));
    assert!(
        !measured.chrome_focused,
        "the preview owns focus while Peek is open"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("peek request-to-present snapshot");
    let preview = snapshot.preview.as_ref().expect("preview presented");
    assert_eq!(
        preview.source, source_view,
        "the preview covers the exact warm origin"
    );
    assert_eq!(preview.url, target_url, "the preview target is verbatim");
    assert!(snapshot.peek_presented, "the Peek bar is presented");
    assert!(
        snapshot.portal.ids.iter().any(|id| id == PEEK_BAR_ID),
        "the Peek bar rendered its stable ids"
    );
    assert_eq!(
        snapshot.active_web_view,
        Some(source_view),
        "the origin stays the active view"
    );
    measured
}

/// The production Peek dismissal through the `BenchmarkPeekDismissToOrigin`
/// request: the preview session is destroyed, the Peek bar is gone, and the
/// exact warm origin is again the active, focused view. The driver asserts
/// the measured state and the typed snapshot.
pub(crate) fn benchmark_peek_dismiss_to_origin(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    source_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkPeekDismissToOrigin { operation_id })
        .expect("benchmark peek dismiss to origin");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::PeekDismissToOrigin,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(
        measured.active_web_view,
        Some(source_view),
        "the warm origin is active again"
    );
    assert!(
        !measured.chrome_focused,
        "dismissal hands native focus back to the origin"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("peek dismiss snapshot");
    assert!(
        snapshot.preview.is_none(),
        "the preview session is destroyed on dismissal"
    );
    assert!(!snapshot.peek_presented, "the Peek bar is gone");
    assert!(
        !snapshot.portal.ids.iter().any(|id| id == PEEK_BAR_ID),
        "the Peek bar ids are gone after dismissal"
    );
    assert_eq!(snapshot.active_web_view, Some(source_view));
    measured
}

/// The production Peek promotion through the `BenchmarkPeekPromoteToTab`
/// request: the SAME preview session becomes a regular tab in the source
/// space/profile without reload or recreation, active and focused. The
/// driver asserts the measured state and the typed snapshot.
pub(crate) fn benchmark_peek_promote_to_tab(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    preview_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkPeekPromoteToTab { operation_id })
        .expect("benchmark peek promote to tab");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::PeekPromoteToTab,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(
        measured.active_web_view,
        Some(preview_view),
        "the promoted session is the active view"
    );
    assert!(
        !measured.chrome_focused,
        "promotion hands native focus to the promoted page"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("peek promote snapshot");
    assert!(
        snapshot.preview.is_none(),
        "the transient preview state is gone after promotion"
    );
    assert!(
        snapshot
            .tabs
            .iter()
            .any(|tab| tab.web_view == Some(preview_view)),
        "the SAME preview session is now a regular tab (no reload/recreation)"
    );
    assert_eq!(
        snapshot.active_web_view,
        Some(preview_view),
        "the promoted tab is active"
    );
    measured
}
