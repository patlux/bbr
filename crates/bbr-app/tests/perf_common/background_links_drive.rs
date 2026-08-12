//! Background-link scenario driving (DD-LINK packet `background-link-tabs`)
//! for the warm interaction matrix.
//!
//! Owns the background-link matrix block: the one required production-path
//! scenario —
//!
//! - `background_link_open`: a Command-primary / middle-click disposition
//!   opens a fresh source-profile regular child immediately after the exact
//!   opener while the opener stays selected, visible, and web-focused
//!   (fresh native-session class, p95 ≤ 100 ms / max ≤ 150 ms), committing
//!   only from the authoritative browser state (fast-but-wrong guard in the
//!   app).
//!
//! Every sample drives the typed benchmark request (the exact production
//! seam); fixture page-load time is excluded by construction (the target is
//! a deterministic local fixture and the samples commit at insertion, not
//! at page ready). Between samples the child is closed (unmeasured, the
//! opener stays active) so every sample measures a fresh open and the block
//! restores the matrix state exactly — the same tabs, sessions, and active
//! view — before the dense seeding that follows.
//!
//! Split from `drive.rs` so the background-link additions live in a focused
//! module under the ~500-line budget (the same pattern as `peek_drive.rs`).

use std::collections::BTreeMap;

use bbr_testkit::protocol::{BrowserChange, Request};
use bbr_testkit::Client;

use bbr_testkit::protocol::Event;

use super::background_links::benchmark_background_link_open;
use super::benchmarks::deadline;
use super::harness::AppHarness;
use super::scenarios::TrialConfig;

/// Drive the background-link matrix block and return the raw samples per
/// scenario. Runs after the Peek block and before the dense seeding; state
/// before it: the default space's fixture tabs with an active warm view, no
/// preview. The block measures the required `background_link_open` scenario
/// (30 accepted samples each), then restores the matrix state (the child
/// closed, the same tabs and sessions, the same active view).
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_background_link_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    target_url: &str,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    let start = client.snapshot().expect("background block start snapshot");
    let opener_view = start.active_web_view.expect("background opener view");
    let opener_index = start
        .tabs
        .iter()
        .position(|t| t.web_view == Some(opener_view))
        .expect("opener index");
    let tabs_before = start.tabs.len();
    let sessions_before = start.engine_sessions;

    // background_link_open × N: every sample is a fresh background
    // disposition through the production seam — the child session is
    // created hidden in the source profile's store immediately after the
    // opener, and the sample commits only from the authoritative state
    // (opener still active/web-focused, child hidden at the insertion
    // index). The child is closed between samples (unmeasured, the opener
    // stays active) so every sample measures the open transition.
    let mut open_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let measured = benchmark_background_link_open(
            client,
            since,
            *operation,
            opener_view,
            opener_index,
            target_url,
        );
        open_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("background_link_open".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: close the child at the opener-relative index
        // (the production close path; the opener stays active) so the next
        // sample opens fresh.
        client
            .send_ok(&Request::CloseTab {
                index: opener_index + 1,
            })
            .expect("close child between background samples");
        let closed = wait_child_closed(client, *since, tabs_before);
        *since = closed;
    }

    // Restore check: the block ends exactly as it started.
    let restored = client.snapshot().expect("background block end snapshot");
    assert_eq!(restored.tabs.len(), tabs_before, "matrix tabs restored");
    assert_eq!(
        restored.engine_sessions, sessions_before,
        "matrix sessions restored"
    );
    assert_eq!(
        restored.active_web_view,
        Some(opener_view),
        "the same active view restored"
    );

    let mut raw = BTreeMap::new();
    raw.insert("background_link_open".to_string(), open_samples);
    raw
}

/// Wait until the child at `opener_index + 1` is gone (the close event)
/// and the tab count is back to `tabs_before`.
fn wait_child_closed(client: &mut Client, since: u64, tabs_before: usize) -> u64 {
    let (seq, _) = client
        .wait_for_event_since(
            |event| {
                matches!(
                    event,
                    Event::BrowserChanged {
                        kind: BrowserChange::TabClosed { .. }
                    }
                )
            },
            since,
            deadline(),
        )
        .expect("child close event");
    let snapshot = client.snapshot().expect("post-close snapshot");
    assert_eq!(snapshot.tabs.len(), tabs_before, "child closed");
    seq
}
