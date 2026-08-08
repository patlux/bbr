//! U2 sidebar performance scenarios (DESIGN.md §11.3/§11.4/§11.10/§11.11)
//! for the warm interaction matrix: the permanent §11.3 space strip is
//! always part of the rail (its chip ids render unconditionally, so the
//! space-switch samples assert them without any reveal step), then drive
//! the plus-menu toggle, the favorites tile add/remove, and the folder
//! disclosure through the production gateway paths with the in-app probe
//! guard. Split from `super` so the driving matrix stays under the ~500-line
//! budget.

use bbr_testkit::protocol::Request;
use bbr_testkit::Client;

use super::super::benchmarks_u2::{
    benchmark_favorite_toggle, benchmark_folder_toggle, benchmark_plus_menu,
};
use super::super::harness::AppHarness;

/// §11.3/§11.4: the space-switch samples assert the rendered
/// `bbr-space-chip-<persistent-space-id>`/`bbr-space-identity-<persistent-
/// space-id>` ids (§11.3/§15), which the permanent footer strip always
/// renders (the legacy `[Spaces]` reveal toggle is gone). The seam verifies
/// the always-present strip before the space benchmarks drive.
pub(super) fn strip_presence(client: &mut Client) {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
    let present = client.snapshot().expect("spaces strip snapshot");
    assert!(
        present.visible_ids.iter().any(|id| id == "bbr-space-strip"),
        "the permanent space strip always renders"
    );
}

/// U2 sidebar surfaces (DESIGN.md §11.4/§11.10/§11.11): the plus menu
/// toggle, the favorites grid tile add/remove, and the folder disclosure.
/// Each is a warm next-frame presentation commit through the production
/// paths (footer `[+]` / drag-drop toggle / disclosure pointer), measured at
/// the exact semantic state the in-app probe requires — a fast-but-wrong
/// sample (missing menu, wrong tile count, wrong expanded value) is rejected
/// before it leaves the app. Returns the three raw sample series.
pub(super) fn drive_u2_sidebar(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    operation: &mut u64,
    samples: usize,
) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let mut plus_menu_samples = Vec::with_capacity(samples);
    for _ in 0..samples {
        let measured = benchmark_plus_menu(client, since, *operation);
        plus_menu_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("sidebar_plus_menu".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }
    // The alternation toggles the closed rest state 30 times, so it
    // ends with the menu closed — the favorites block runs with the
    // menu dismissed, and the folder setup below opens it fresh through
    // the same production footer path.

    let mut favorites_samples = Vec::with_capacity(samples);
    let favorites_index = 0;
    for _ in 0..samples {
        let measured = benchmark_favorite_toggle(client, since, *operation, favorites_index);
        favorites_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("sidebar_favorites".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }
    // The favorites alternation ends with the grid empty (30 toggles):
    // keep the tile for the folder block's nested-row setup.
    client
        .send_ok(&Request::ToggleFavoriteTab {
            index: favorites_index,
        })
        .expect("re-add the favorite tile");

    // Folder setup through the production paths: create the folder via
    // the plus menu, then move the favorite tab into it via the L5
    // `MoveIntoFolder` command so the disclosure has nested rows.
    client
        .send_ok(&Request::ClickFooterPlus)
        .expect("open plus menu for folder");
    client
        .send_ok(&Request::ClickPlusMenuItem { index: 1 })
        .expect("create folder");
    let folder_snapshot = client.snapshot().expect("folder created");
    assert!(
        folder_snapshot.folders[0].id > 0,
        "folder carries its domain id"
    );
    client
        .send_ok(&Request::MoveTabIntoFolder {
            index: favorites_index,
            folder: 0,
        })
        .expect("move tab into folder");
    let mut folder_toggle_samples = Vec::with_capacity(samples);
    for _ in 0..samples {
        let measured = benchmark_folder_toggle(client, since, *operation, 0);
        folder_toggle_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("sidebar_folder_toggle".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }
    // Leave the disclosure collapsed (the alternation ends collapsed)
    // and drop the favorite tile so the dense seeding below starts from
    // the base rail.
    client
        .send_ok(&Request::ToggleFavoriteTab {
            index: favorites_index,
        })
        .expect("remove the favorite tile");

    (plus_menu_samples, favorites_samples, folder_toggle_samples)
}
