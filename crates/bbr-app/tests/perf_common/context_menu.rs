//! BBR context-menu benchmark driver (task packet `bbr-context-menus-over-web`)
//! for the real-app performance suites.
//!
//! `benchmark_context_menu_open` measures the production event-driven open
//! path through the `BenchmarkContextMenuOpen` request: the sample starts at
//! the exact seam the native bridge's `ContextMenuRequested` routes to
//! (`menu_open_from_event`) and commits in the app only when a render of the
//! one `NativeAboveWeb` portal window drew the menu open and focused over the
//! same active view at the armed pointer anchor (the in-app probe owns the
//! render-inclusive fast-but-wrong guard). The driver asserts the measured
//! state and the typed snapshot (open presentation, menu focus, bound view,
//! armed anchor, portal presentation id, unchanged session identity).
//!
//! This focused module owns the context-menu additions so `benchmarks.rs`
//! stays at its pre-menu size (the same split pattern as `find.rs`).

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;

/// The menu's presentation id (`PresentationId(0xC4)` in
/// `crate::app::context_menu`), exposed through the snapshot's typed
/// `portal.presentation` field.
pub(crate) const CONTEXT_MENU_PRESENTATION_ID: u64 = 0xC4;

/// The production event-driven menu open through the
/// `BenchmarkContextMenuOpen` request: the sample commits only from a portal
/// render that drew the menu over the same active view at the armed pointer
/// anchor; the same active view stays selected and no tab/session is created
/// or touched. The driver asserts the measured state and the typed snapshot.
pub(crate) fn benchmark_context_menu_open(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    point: [f32; 2],
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkContextMenuOpen {
            operation_id,
            view: expected_view,
            point,
        })
        .expect("benchmark context menu open");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::ContextMenuOpen, operation_id, *since);
    *since = seq;
    assert_eq!(
        measured.active_web_view,
        Some(expected_view),
        "the menu keeps the active view"
    );
    assert!(
        !measured.chrome_focused,
        "the menu owns a non-chrome focus domain while open"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("context menu open snapshot");
    assert!(snapshot.context_menu.open, "the menu presentation is open");
    assert!(
        snapshot.context_menu.focused,
        "the menu owns keyboard focus"
    );
    assert_eq!(
        snapshot.context_menu.bound_view,
        Some(expected_view),
        "the menu is bound to the active view"
    );
    assert_eq!(
        snapshot.context_menu.point, point,
        "the menu is anchored at the armed pointer point"
    );
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert!(!snapshot.chrome_focused);
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Inactive);
    assert_eq!(
        snapshot.portal.presentation,
        Some(CONTEXT_MENU_PRESENTATION_ID),
        "the menu presented through the one portal host"
    );
    measured
}

/// The production bookmark-row activation through the
/// `BenchmarkContextMenuBookmark` request (task packet
/// `page-background-context-menu.md`): the sample commits only from a render
/// that drew the menu closed over the same active view, with the B2 store
/// truth verified at the mutation site. The driver asserts the measured
/// state, the closed menu presentation, the unchanged session identity, and
/// the flipped Saved Page record.
pub(crate) fn benchmark_context_menu_bookmark(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    point: [f32; 2],
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let before = client.snapshot().expect("bookmark-commit before snapshot");
    let saved_before = before.store.saved_pages_count;
    client
        .send_ok(&Request::BenchmarkContextMenuBookmark {
            operation_id,
            view: expected_view,
            point,
        })
        .expect("benchmark context menu bookmark commit");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::ContextMenuBookmarkCommit,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(
        measured.active_web_view,
        Some(expected_view),
        "the bookmark commit keeps the active view"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("bookmark-commit snapshot");
    assert!(
        !snapshot.context_menu.open,
        "the menu is dismissed after the bookmark commit"
    );
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Inactive);
    assert_eq!(
        snapshot.store.saved_pages_count,
        if saved_before == 0 { 1 } else { 0 },
        "the B2 toggle flipped exactly one record"
    );
    assert_eq!(snapshot.engine_sessions, before.engine_sessions);
    measured
}
