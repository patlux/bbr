//! Tab/Favorite context-menu benchmark drivers (task packet
//! `tab-favorite-context-menus`) for the real-app performance suites.
//!
//! `benchmark_tab_menu_open` measures the production event-driven open
//! path through the `BenchmarkTabMenuOpen` request: the sample starts at
//! the exact app method the sidebar tab row's right-button mouse-down
//! listener routes to (`open_tab_menu_from_row` at the invoking pointer)
//! and commits in the app only when a main-window render drew the menu
//! open, focused, bound to the exact source `TabId` at the armed pointer
//! anchor (the in-app probe owns the render-inclusive fast-but-wrong
//! guard). `benchmark_tab_menu_submenu_commit` measures the production
//! hover path that opens the one-level submenu card (`tab_menu_select`)
//! against an already-open menu; the sample commits only when the render
//! drew the open submenu card bound to the same source `TabId`. Neither
//! path ever selects, moves, reloads, closes, recreates, or changes the
//! profile/space/session of the source tab.

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement, Request,
};
use bbr_testkit::Client;

use super::benchmarks::wait_measurement;

/// The production event-driven Tab/Favorite menu open over the tab row at
/// `row`: the sample commits only from a main-window render that drew the
/// menu open and focused bound to the exact source `TabId` at the armed
/// pointer anchor; the source tab stays selected-free and no tab/session is
/// created or touched. The driver asserts the measured state and the typed
/// snapshot.
pub(crate) fn benchmark_tab_menu_open(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    row: usize,
    point: [f32; 2],
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkTabMenuOpen {
            operation_id,
            row,
            point,
        })
        .expect("benchmark tab menu open");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::TabMenuOpen, operation_id, *since);
    *since = seq;
    assert_eq!(
        measured.command_bar_mode,
        CommandBarMode::Inactive,
        "the menu never opens a command surface"
    );
    let snapshot = client.snapshot().expect("tab menu open snapshot");
    assert!(snapshot.tab_menu.open, "the menu presentation is open");
    assert!(snapshot.tab_menu.focused, "the menu owns keyboard focus");
    assert_eq!(snapshot.tab_menu.source_key, "bbr-tab-menu");
    assert_eq!(snapshot.active_web_view, measured.active_web_view);
    measured
}

/// The production submenu commit against an already-open menu: the sample
/// commits only from a render that drew the open submenu card for
/// `parent_row` bound to the same source `TabId`. The driver asserts the
/// measured state and the typed snapshot (open submenu, bound source,
/// cursor on the first enabled submenu row).
pub(crate) fn benchmark_tab_menu_submenu_commit(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    parent_row: usize,
    expected_source: u64,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkTabMenuSubmenuCommit {
            operation_id,
            parent_row,
        })
        .expect("benchmark tab menu submenu commit");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::TabMenuSubmenuCommit,
        operation_id,
        *since,
    );
    *since = seq;
    let snapshot = client.snapshot().expect("submenu commit snapshot");
    assert!(snapshot.tab_menu.open, "the menu stays open");
    assert_eq!(
        snapshot.tab_menu.bound_tab,
        Some(expected_source),
        "the submenu commit preserves the exact source TabId"
    );
    assert_eq!(snapshot.tab_menu.open_submenu, Some(parent_row));
    assert!(
        snapshot.tab_menu.submenu_selected.is_some(),
        "the submenu cursor lands on an enabled row"
    );
    measured
}
