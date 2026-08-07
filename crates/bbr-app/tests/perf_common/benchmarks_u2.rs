//! U2 sidebar benchmark drivers (DESIGN.md §11.4/§11.10/§11.11) for the
//! real-app performance suites: the plus menu toggle, the favorites tile
//! add/remove, and the folder disclosure, each through the production
//! gateway path with the in-app probe guard. Split from `super::benchmarks`
//! so no driver module outgrows the ~500-line budget.

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, PresentationMeasurement, Request,
};
use bbr_testkit::Client;

use super::benchmarks::wait_measurement;

/// The §11.4 plus menu through the production footer `[+]` path: each sample
/// toggles the menu (alternating open/close). The sample commits only when
/// the presented rail shows the menu in the expected open state (the in-app
/// probe guard rejects a fast-but-wrong presentation without the menu
/// surface and its two items).
pub(crate) fn benchmark_plus_menu(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkPlusMenu { operation_id })
        .expect("benchmark plus menu");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::SidebarPlusMenu, operation_id, *since);
    *since = seq;
    let snapshot = client.snapshot().expect("plus menu snapshot");
    assert_eq!(
        snapshot.plus_menu_open, measured.plus_menu_open,
        "the sample and the snapshot agree on the menu state"
    );
    if snapshot.plus_menu_open {
        assert!(
            snapshot.visible_ids.iter().any(|id| id == "bbr-plus-menu"),
            "open sample renders the menu surface"
        );
    }
    measured
}

/// The §11.10 favorites toggle through the production path (the exact app
/// method the drag-into-grid drop commits through): the sample commits only
/// when the grid renders the expected tile count (fast-but-wrong guard), and
/// the toggled tab keeps its space membership and session.
pub(crate) fn benchmark_favorite_toggle(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    index: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkFavoriteToggle {
            operation_id,
            index,
        })
        .expect("benchmark favorite toggle");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::SidebarFavorites,
        operation_id,
        *since,
    );
    *since = seq;
    let snapshot = client.snapshot().expect("favorites snapshot");
    assert_eq!(
        snapshot.favorites.len(),
        measured.favorites_len,
        "the sample and the snapshot agree on the tile count"
    );
    if measured.favorites_len > 0 {
        assert!(
            snapshot
                .visible_ids
                .iter()
                .any(|id| id == "bbr-favorites" || id == "bbr-fav-0"),
            "a non-empty sample renders the grid and its tiles"
        );
    }
    measured
}

/// The §11.11 folder disclosure through the production toggle path: the
/// sample commits only when the folder's expanded/collapsed value matches
/// the toggle (guard — a presentation without the nested rows is rejected).
pub(crate) fn benchmark_folder_toggle(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    index: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkFolderToggle {
            operation_id,
            index,
        })
        .expect("benchmark folder toggle");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::SidebarFolderToggle,
        operation_id,
        *since,
    );
    *since = seq;
    let snapshot = client.snapshot().expect("folder toggle snapshot");
    assert_eq!(
        snapshot.folders[index].expanded, measured.folder_expanded,
        "the sample and the snapshot agree on the disclosure"
    );
    measured
}
