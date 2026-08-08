//! Context-menu scenario driving (task packet `bbr-context-menus-over-web`)
//! for the warm interaction matrix.
//!
//! Owns the context-menu matrix block: `context_menu_open` measures the
//! production event-driven open path — the row-model build, the anchored
//! placement, the portal presentation, and the focus handoff — on a
//! deterministic fixture loaded before any sample (fixture/network time
//! excluded). The sample commits only from a render of the one
//! `NativeAboveWeb` portal window that drew the menu over the same active
//! view at the armed pointer anchor (fast-but-wrong guard in the app; the
//! driver asserts the measured state and the typed snapshot). The block
//! restores the matrix state exactly — menu closed, the transient menu tab
//! closed — before the dense seeding that follows.
//!
//! Split from `drive.rs` so the context-menu additions live in a focused
//! module under the ~500-line budget while the pre-existing matrix content
//! stays where it was (same pattern as `zoom_drive.rs`).

use std::collections::BTreeMap;

use bbr_testkit::protocol::{BrowserChange, Request};
use bbr_testkit::Client;

use super::benchmarks::{wait_browser_change, wait_title};
use super::context_menu::benchmark_context_menu_open;
use super::harness::AppHarness;
use super::scenarios::{TrialConfig, PAGE_ONE_TITLE};

/// Drive the context-menu matrix block and return the raw samples. Runs
/// after the zoom block and before the dense seeding; state before it:
/// whatever the zoom block left (transient tab closed), no menu
/// presentation. The block opens a transient fixture tab, measures the warm
/// event-driven open, then restores the matrix state (menu closed,
/// transient tab closed).
pub(super) fn drive_context_menu_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    page_one: &str,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    // Unmeasured setup: a transient tab on the deterministic page-one
    // fixture, active in web focus. The fixture is loaded before any sample,
    // so fixture/network time never enters the measured durations.
    client
        .send_ok(&Request::OpenTab {
            url: page_one.to_string(),
        })
        .expect("open context-menu fixture tab");
    let opened = client.snapshot().expect("context-menu fixture snapshot");
    let menu_view = opened.active_web_view.expect("context-menu fixture view");
    if opened.title.as_deref() != Some(PAGE_ONE_TITLE) {
        *since = wait_title(client, menu_view, PAGE_ONE_TITLE, *since);
    }
    let sessions_before = opened.engine_sessions;

    // context_menu_open × N: each sample is one production event-driven open
    // (the exact seam the native bridge's `ContextMenuRequested` routes to).
    // Two alternating pointer anchors keep the anchored-placement path
    // genuine across samples (a fixed anchor could mask a degenerate clamp).
    // The menu is closed between samples (unmeasured) so every sample
    // measures the open transition, never a re-present no-op.
    let mut menu_samples = Vec::with_capacity(config.interaction_samples);
    let anchors = [[200.0, 240.0], [760.0, 420.0]];
    for sample in 0..config.interaction_samples {
        let point = anchors[sample % anchors.len()];
        let measured = benchmark_context_menu_open(client, since, *operation, point, menu_view);
        menu_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("context_menu_open".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: the production close path (the same method the
        // backdrop press / Escape / activation call) restores the origin
        // focus domain for the next open.
        client
            .send_ok(&Request::ContextMenuClose)
            .expect("close context menu between samples");
        wait_menu_closed(client);
    }

    // Restoration: Cmd-W closes the transient menu tab so the matrix state
    // (tab population, sessions) is exactly what the dense seeding expected.
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: "cmd-w".into(),
        })
        .expect("close transient context-menu tab");
    *since = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::TabClosed { .. }),
        *since,
    )
    .0;
    let restored = client
        .snapshot()
        .expect("context-menu block restored snapshot");
    assert!(
        !restored.context_menu.open,
        "the menu presentation is closed after the block"
    );
    assert!(
        restored.engine_sessions <= sessions_before,
        "closing the transient tab never grows the session population"
    );
    assert!(
        restored.active_web_view != Some(menu_view),
        "the transient menu tab is gone"
    );

    let mut raw = BTreeMap::new();
    raw.insert("context_menu_open".to_string(), menu_samples);
    raw
}

/// Poll until the context-menu presentation is fully closed (bounded by the
/// harness deadline).
fn wait_menu_closed(client: &mut Client) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("menu closed snapshot");
        if !snapshot.context_menu.open {
            return;
        }
        assert!(
            std::time::Instant::now() < until,
            "context menu did not close: {:?}",
            snapshot.context_menu
        );
        std::thread::yield_now();
    }
}
