//! Tab/Favorite context-menu scenario driving (task packet
//! `tab-favorite-context-menus`) for the warm interaction matrix.
//!
//! Owns the tab-menu matrix block: `tab_context_menu_open` measures the
//! production event-driven open path — the exact seam the sidebar tab
//! row's right-button mouse-down listener routes to
//! (`open_tab_menu_from_row`): the source resolution, the availability
//! matrix, the deterministic row model, the clamped placement, and the
//! focus handoff — on a deterministic fixture loaded before any sample
//! (fixture/network time excluded). The sample commits only from a
//! main-window render that drew the menu open, focused, and bound to the
//! exact source `TabId` at the armed pointer anchor (the in-app probe owns
//! the render-inclusive fast-but-wrong guard). `tab_context_menu_submenu_commit`
//! measures the production hover path that opens the one-level `Move to
//! Space` submenu card (`tab_menu_select`): the sample commits only when
//! the render drew the open submenu card bound to the same source. The
//! block opens a second space in the source profile (unmeasured setup,
//! making the submenu parent row genuinely present), never touches a
//! tab/session permanently, and closes the menu before the dense seeding
//! that follows.
//!
//! Split from `drive.rs` so the tab-menu additions live in a focused
//! module under the ~500-line budget (same pattern as `context_menu_drive.rs`).

use std::collections::BTreeMap;

use bbr_testkit::protocol::Request;
use bbr_testkit::Client;

use super::scenarios::TrialConfig;
use super::tab_menu::{benchmark_tab_menu_open, benchmark_tab_menu_submenu_commit};

/// The `Move to Space` submenu parent row: with the setup's second space
/// and no folders, it is the first enabled row (render order: placement
/// group first).
const MOVE_SPACE_PARENT_ROW: usize = 0;

/// Drive the tab-menu matrix block and return the raw samples. Runs after
/// the context-menu block and before the auth block; state before it:
/// whatever the context-menu block left (transient tab closed), no menu
/// presentation. The block opens a second space (unmeasured setup), then
/// measures the warm open + the warm submenu commit, then restores the
/// matrix state (menu closed, no tab/session touched).
pub(super) fn drive_tab_menu_scenarios(
    client: &mut Client,
    harness: &mut crate::perf_common::harness::AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    // Unmeasured setup: a second space in the active profile makes the
    // `Move to Space` submenu parent row genuinely present (an empty
    // destination group is omitted whole). The tab population and the
    // sessions are untouched.
    // Earlier matrix blocks already created spaces, so only the delta is
    // meaningful: exactly one more space than before this setup.
    let before = client.snapshot().expect("profile snapshot");
    let profile = before.active_profile;
    client
        .send_ok(&Request::CreateSpace {
            profile,
            title: "Perf Second".into(),
        })
        .expect("create perf second space");
    wait_for_space_count(client, before.spaces.len() + 1);

    // tab_context_menu_open × N: each sample is one production event-driven
    // open over the inactive tab row 0 (the deterministic fixture tab,
    // loaded before any sample). Two alternating pointer anchors keep the
    // anchored-placement path genuine across samples (a fixed anchor could
    // mask a degenerate clamp). The menu is closed between samples
    // (unmeasured) so every sample measures the open transition.
    let mut open_samples = Vec::with_capacity(config.interaction_samples);
    // Both anchors must stay in-window for the full card (the probe compares
    // the drawn anchor to the armed pointer): the 12-row menu is 344 pt tall
    // in the 701 pt matrix window, so an anchor at y 400 would legitimately
    // clamp to 349 and reject.
    let anchors = [[180.0, 220.0], [640.0, 300.0]];
    for sample in 0..config.interaction_samples {
        let point = anchors[sample % anchors.len()];
        let measured = benchmark_tab_menu_open(client, since, *operation, 0, point);
        open_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("tab_context_menu_open".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: the production close path restores the origin
        // focus domain for the next open.
        client
            .send_ok(&Request::TabMenuClose)
            .expect("close tab menu");
        wait_menu_closed(client);
    }

    // tab_context_menu_submenu_commit × N: each sample starts at the
    // production hover path that opens the `Move to Space` submenu card
    // against an already-open menu (opened unmeasured between samples) and
    // commits only from a render that drew the open submenu card bound to
    // the same source `TabId`.
    let mut submenu_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let point = anchors[sample % anchors.len()];
        client
            .send_ok(&Request::RightClickRow {
                index: 0,
                x: point[0],
                y: point[1],
            })
            .expect("open tab menu between submenu samples");
        wait_menu_open(client);
        let source_tab = client
            .snapshot()
            .expect("submenu source snapshot")
            .tab_menu
            .bound_tab
            .expect("bound source");
        let measured = benchmark_tab_menu_submenu_commit(
            client,
            since,
            *operation,
            MOVE_SPACE_PARENT_ROW,
            source_tab,
        );
        submenu_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("tab_context_menu_submenu_commit".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        client
            .send_ok(&Request::TabMenuClose)
            .expect("close tab menu");
        wait_menu_closed(client);
    }

    // Restoration: the menu is closed; no tab or session was touched.
    let restored = client.snapshot().expect("tab-menu block restored snapshot");
    assert!(!restored.tab_menu.open, "the menu presentation is closed");

    let mut raw = BTreeMap::new();
    raw.insert("tab_context_menu_open".to_string(), open_samples);
    raw.insert(
        "tab_context_menu_submenu_commit".to_string(),
        submenu_samples,
    );
    raw
}

/// Poll until the tab-menu presentation is open (bounded by the harness
/// deadline).
fn wait_menu_open(client: &mut Client) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("menu open snapshot");
        if snapshot.tab_menu.open {
            return;
        }
        assert!(
            std::time::Instant::now() < until,
            "tab menu did not open: {:?}",
            snapshot.tab_menu
        );
        std::thread::yield_now();
    }
}

/// Poll until the tab-menu presentation is fully closed (bounded by the
/// harness deadline).
fn wait_menu_closed(client: &mut Client) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("menu closed snapshot");
        if !snapshot.tab_menu.open {
            return;
        }
        assert!(
            std::time::Instant::now() < until,
            "tab menu did not close: {:?}",
            snapshot.tab_menu
        );
        std::thread::yield_now();
    }
}

/// Poll until the space collection reaches `count` (the unmeasured setup
/// that makes the `Move to Space` submenu present).
fn wait_for_space_count(client: &mut Client, count: usize) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        client
            .send_ok(&Request::FlushFrame)
            .expect("flush while waiting for spaces");
        let snapshot = client.snapshot().expect("space count snapshot");
        if snapshot.spaces.len() == count {
            return;
        }
        assert!(
            std::time::Instant::now() < until,
            "spaces never reached {count}: {:?}",
            snapshot.spaces.len()
        );
        std::thread::yield_now();
    }
}
