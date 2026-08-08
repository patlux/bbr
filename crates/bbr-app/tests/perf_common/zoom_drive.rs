//! Zoom-in scenario driving (queue item 11, part A) for the warm
//! interaction matrix.
//!
//! Owns the zoom matrix block: `zoom_step` measures the production `Cmd-Plus`
//! path — the lifecycle change, the persistence write, and the native
//! `pageZoom` application on the active view — on a deterministic fixture
//! loaded before any sample (fixture/network time excluded). The sample
//! commits only from the app's observed zoom state at the armed level,
//! applied to the view, with no pending write (fast-but-wrong guard); the
//! driver asserts the measured state and the typed snapshot. The block
//! restores the matrix state exactly — the transient zoom tab closed —
//! before the dense seeding that follows.
//!
//! Split from `drive.rs` so the zoom additions live in a focused module
//! under the ~500-line budget while the pre-existing matrix content stays
//! where it was (same pattern as `find_drive.rs`).

use std::collections::BTreeMap;

use bbr_testkit::protocol::{BrowserChange, Request};
use bbr_testkit::Client;

use super::benchmarks::{wait_browser_change, wait_title};
use super::harness::AppHarness;
use super::scenarios::{TrialConfig, PAGE_ONE_TITLE};
use super::zoom::benchmark_zoom_step;

/// Drive the zoom matrix block and return the raw samples. Runs after the
/// downloads block and before the dense seeding; state before it: whatever
/// the downloads block left (panel closed, matrix tabs restored), no zoom
/// presentation. The block opens a transient zoom tab, measures the warm
/// `Cmd-Plus` path, then restores the matrix state (transient tab closed).
pub(super) fn drive_zoom_scenarios(
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
        .expect("open zoom fixture tab");
    let opened = client.snapshot().expect("zoom fixture snapshot");
    let zoom_view = opened.active_web_view.expect("zoom fixture view");
    if opened.title.as_deref() != Some(PAGE_ONE_TITLE) {
        *since = wait_title(client, zoom_view, PAGE_ONE_TITLE, *since);
    }
    let sessions_before = opened.engine_sessions;

    // zoom_step × N: each sample is one production Cmd-Plus step. The level
    // climbs the contract ladder (125 → 400 over 12 steps) and then clamps
    // deterministically at the maximum — a step at the bound is still a
    // genuine measured Cmd-Plus no-op, and the in-app probe requires the
    // observed level/applied/pending state every time.
    let mut zoom_samples = Vec::with_capacity(config.interaction_samples);
    let mut expected: u16 = 125;
    for _ in 0..config.interaction_samples {
        let measured = benchmark_zoom_step(client, since, *operation, expected, zoom_view);
        zoom_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("zoom_step".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        expected = (expected + 25).min(400);
    }

    // Restoration: Cmd-W closes the transient zoom tab so the matrix state
    // (tab population, sessions) is exactly what the dense seeding expected.
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: "cmd-w".into(),
        })
        .expect("close transient zoom tab");
    *since = wait_browser_change(
        client,
        |kind| matches!(kind, BrowserChange::TabClosed { .. }),
        *since,
    )
    .0;
    let restored = client.snapshot().expect("zoom block restored snapshot");
    assert!(
        restored.engine_sessions <= sessions_before,
        "closing the transient tab never grows the session population"
    );
    assert!(
        restored.active_web_view != Some(zoom_view),
        "the transient zoom tab is gone"
    );

    let mut raw = BTreeMap::new();
    raw.insert("zoom_step".to_string(), zoom_samples);
    raw
}
