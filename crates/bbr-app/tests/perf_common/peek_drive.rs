//! Transient Peek scenario driving (Link Preview / Peek packet) for the
//! warm interaction matrix.
//!
//! Owns the Peek matrix block: the three required production-path
//! scenarios —
//!
//! - `peek_request_to_present`: an eligible direct new-window request
//!   presents a fresh source-profile preview session above its warm origin
//!   (fresh native-session class, p95 ≤ 100 ms / max ≤ 150 ms), committing
//!   only from a render of the one `NativeAboveWeb` portal window that drew
//!   the Peek bar for the armed preview identity above the armed origin
//!   (fast-but-wrong guard in the app);
//! - `peek_dismiss_to_origin`: the production dismissal destroys the
//!   preview session and restores the exact warm origin, active and focused
//!   (warm interaction class, p95 ≤ 50 ms / max ≤ 100 ms);
//! - `peek_promote_to_tab`: the production "Open as Tab" adopts the SAME
//!   preview session as a regular tab without reload or recreation
//!   (warm interaction class, p95 ≤ 50 ms / max ≤ 100 ms).
//!
//! Every sample drives the typed benchmark request (the exact production
//! seam); fixture/network time is excluded by construction (the preview
//! target is a deterministic local fixture and the samples commit at
//! presentation, not at page ready). The block restores the matrix state
//! exactly — preview closed, no extra tab, no extra session, the same
//! active view — before the dense seeding that follows.
//!
//! Split from `drive.rs` so the Peek additions live in a focused module
//! under the ~500-line budget (the same pattern as `find_drive.rs` /
//! `auth_drive.rs` / `context_menu_drive.rs`).

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use bbr_testkit::protocol::Request;
use bbr_testkit::Client;
use bbr_web::new_window::NewWindowIntent;
use bbr_web::{ProfileId, WebViewId};

use super::harness::AppHarness;
use super::peek::{
    benchmark_peek_dismiss_to_origin, benchmark_peek_promote_to_tab,
    benchmark_peek_request_to_present,
};
use super::scenarios::TrialConfig;

/// Drive the Peek matrix block and return the raw samples per scenario.
/// Runs after the auth block and before the dense seeding; state before it:
/// the default space's fixture tabs with an active warm view, no preview.
/// The block measures the three required Peek scenarios (30 accepted
/// samples each), then restores the matrix state (preview closed, the same
/// tabs and sessions, the same active view).
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_peek_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    peek_target: &str,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    let start = client.snapshot().expect("peek block start snapshot");
    let source_view = start.active_web_view.expect("peek source view");
    let source_profile = start.active_profile;
    let tabs_before = start.tabs.len();
    let sessions_before = start.engine_sessions;

    // peek_request_to_present × N: every sample is a fresh eligible
    // new-window request through the production seam — the preview session
    // is created in the source profile's store and raised above the warm
    // origin, and the sample commits only from the portal render that drew
    // the Peek bar. The preview is dismissed between samples (unmeasured
    // setup through the same production close path) so every sample
    // measures the open transition, never a replacement or a no-op.
    let mut request_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let measured =
            benchmark_peek_request_to_present(client, since, *operation, source_view, peek_target);
        request_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("peek_request_to_present".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: dismiss the preview (the production close path)
        // so the next sample opens fresh.
        client
            .send_ok(&Request::PeekClose)
            .expect("close preview between request samples");
        wait_preview_closed(client);
    }

    // peek_dismiss_to_origin × N: every sample starts from an open preview
    // (unmeasured setup through the production new-window seam) and
    // measures the production dismissal — preview session destroyed, the
    // exact warm origin active/focused again.
    let mut dismiss_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        open_preview(client, since, source_view, source_profile, peek_target);
        let measured = benchmark_peek_dismiss_to_origin(client, since, *operation, source_view);
        dismiss_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("peek_dismiss_to_origin".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    // peek_promote_to_tab × N: every sample starts from an open preview
    // (unmeasured setup) and measures the production "Open as Tab" — the
    // SAME preview session adopted as a regular tab, active and focused, no
    // reload, no recreation. Between samples the promoted tab is closed
    // (unmeasured, the deterministic neighbor becomes active again) so the
    // matrix state never grows and every sample opens a fresh preview.
    let mut promote_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        open_preview(client, since, source_view, source_profile, peek_target);
        let opened = client.snapshot().expect("peek promote setup snapshot");
        let preview_view = opened
            .preview
            .as_ref()
            .expect("preview open for promotion")
            .view;
        let measured = benchmark_peek_promote_to_tab(client, since, *operation, preview_view);
        promote_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("peek_promote_to_tab".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: close the promoted tab (the appended regular
        // row) so the next sample starts from the same matrix state. The
        // deterministic neighbor selection makes the origin active again.
        let promoted = client.snapshot().expect("promoted tab snapshot");
        let promoted_index = promoted
            .tabs
            .iter()
            .position(|tab| tab.web_view == Some(preview_view))
            .expect("promoted tab is a regular tab row");
        client
            .send_ok(&Request::CloseTab {
                index: promoted_index,
            })
            .expect("close promoted tab between promote samples");
        let closed = client.snapshot().expect("promote teardown snapshot");
        assert_eq!(
            closed.tabs.len(),
            tabs_before,
            "closing the promoted tab restores the tab population"
        );
        assert_eq!(
            closed.active_web_view,
            Some(source_view),
            "the deterministic neighbor selection returns the origin"
        );
    }

    // Restoration: the block must leave the matrix exactly as it found it —
    // no preview, the same tabs, the same sessions, the same active view.
    let restored = client.snapshot().expect("peek block restored snapshot");
    assert!(
        restored.preview.is_none(),
        "no preview left open after the peek block"
    );
    assert!(!restored.peek_presented, "the Peek bar is gone");
    assert_eq!(
        restored.tabs.len(),
        tabs_before,
        "the peek block never grows the tab population"
    );
    assert_eq!(
        restored.engine_sessions, sessions_before,
        "the peek block never grows the session population"
    );
    assert_eq!(
        restored.active_web_view,
        Some(source_view),
        "the origin is the active view again"
    );

    let mut raw = BTreeMap::new();
    raw.insert("peek_request_to_present".to_string(), request_samples);
    raw.insert("peek_dismiss_to_origin".to_string(), dismiss_samples);
    raw.insert("peek_promote_to_tab".to_string(), promote_samples);
    raw
}

/// Unmeasured setup: open one transient preview through the exact
/// production `BbrApp::on_new_window_request` seam (the default intent —
/// eligible direct activation → transient preview) and wait until the
/// snapshot reports it.
fn open_preview(
    client: &mut Client,
    _since: &mut u64,
    source: WebViewId,
    source_profile: ProfileId,
    target_url: &str,
) {
    client
        .send_ok(&Request::NewWindow {
            source,
            source_profile,
            url: target_url.to_string(),
            user_initiated: true,
            intent: NewWindowIntent::Default,
        })
        .expect("open peek preview for benchmark setup");
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("peek setup snapshot");
        if snapshot.preview.is_some() && snapshot.peek_presented {
            return;
        }
        assert!(
            Instant::now() < until,
            "peek preview did not open: {:?}",
            snapshot.preview
        );
        std::thread::yield_now();
    }
}

/// Poll until no preview is open (bounded by the harness deadline).
fn wait_preview_closed(client: &mut Client) {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("peek closed snapshot");
        if snapshot.preview.is_none() {
            return;
        }
        assert!(
            Instant::now() < until,
            "peek preview did not close: {:?}",
            snapshot.preview
        );
        std::thread::yield_now();
    }
}
