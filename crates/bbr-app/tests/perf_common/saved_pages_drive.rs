//! B2 Saved Page scenario driving (packet `saved-pages-actions-command`)
//! for the interaction matrix.
//!
//! Owns the B2 matrix block: the production command-text path measured
//! against the seeded ≥1,000-record Saved Page provider
//! (`saved_page_search_first_result`) and the production fresh-tab open of
//! a Saved Page result (`saved_page_open`, session-recreation class). The
//! expected first-result key is the stable `saved_page:1` row of the unique
//! seeded target; the in-app probe rejects a fast-but-wrong first result (a
//! tab/fallback row) or a stale generation before the sample leaves, and
//! the open guard rejects anything but exactly one fresh regular tab/session
//! in the active Space/Profile. Split out of `drive.rs` so the B2 additions
//! live in a focused module under the ~500-line budget.

use std::collections::BTreeMap;

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, BrowserChange, CommandBarMode, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::{wait_browser_change, wait_measurement};
use super::command::wait_active_navigation_idle;
use super::harness::AppHarness;
use super::scenarios::TrialConfig;

/// The unique seeded Saved Page target: entry 1 of the ≥1,000-record
/// provider ("Alpha Saved Target" over the deterministic fixture page URL).
/// The query `alpha saved` matches only it (the title word pair; the
/// navigable fixture URL is the record's canonical destination), so the
/// correct first result is the ranked Saved Page row `saved_page:1`.
pub(super) const SAVED_PAGE_TARGET_ID: u64 = 1;
pub(super) const SAVED_PAGE_QUERY: &str = "alpha saved";
pub(super) const SAVED_PAGE_FIRST_KEY: &str = "saved_page:1";

/// The production command-text path against the Saved Page provider: Cmd-L,
/// replace-selection typing, the real ranking debounce, and a render of the
/// result surface committing the sample in the app. The expected first
/// result is the ranked Saved Page row (`saved_page:<id>`); the in-app
/// probe rejects a fast-but-wrong first result (a tab or fallback row), a
/// stale generation, or a missing palette before the sample leaves.
pub(super) fn benchmark_saved_page_search_first_result(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    text: &str,
    expected_first_key: &str,
    expected_view: WebViewId,
) -> bbr_testkit::protocol::PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkSavedPageSearchFirstResult {
            operation_id,
            text: text.to_string(),
            expected_first_key: expected_first_key.to_string(),
        })
        .expect("benchmark saved page search first result");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::SavedPageSearchFirstResult,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        measured.chrome_focused,
        "the typed surface keeps the chrome focus domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Location);
    let snapshot = client.snapshot().expect("saved page search snapshot");
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Location);
    assert!(snapshot.chrome_focused);
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(
        snapshot.command_results.first().map(|row| row.key.as_str()),
        Some(expected_first_key),
        "the published first result is the expected ranked Saved Page row"
    );
    assert_eq!(
        snapshot
            .command_results
            .first()
            .map(|row| row.group.as_str()),
        Some("saved_page")
    );
    // The seeded provider scale is in effect: the corpus the sample ranked
    // carried the seeded ≥1,000 records (enforced here so a missing seed
    // silently degrades the measurement to an empty provider).
    assert!(
        snapshot.store.saved_pages_count >= 1_000,
        "the Saved Page corpus must carry the seeded provider scale"
    );
    measured
}

/// The production open of a Saved Page result: the record is revalidated in
/// the in-memory provider and exactly one fresh regular tab/session opens in
/// the currently active Space/Profile through the existing NewTab authority
/// — the session-recreation class. The in-app probe rejects anything but
/// exactly one new tab and live session, a fresh active view, and the
/// surface closed in web focus; the driver asserts the measured state and
/// the typed snapshot.
pub(super) fn benchmark_saved_page_open(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    bookmark_id: u64,
    destination_url: &str,
) -> bbr_testkit::protocol::PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let before = client.snapshot().expect("before saved page open snapshot");
    let before_tabs = before.tabs.len();
    let before_sessions = before.engine_sessions;
    let previous_view = before.active_web_view.expect("active view before open");
    client
        .send_ok(&Request::BenchmarkSavedPageOpen {
            operation_id,
            bookmark_id,
        })
        .expect("benchmark saved page open");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::SavedPageOpen, operation_id, *since);
    *since = seq;
    let after = client.snapshot().expect("saved page open snapshot");
    assert_eq!(after.tabs.len(), before_tabs + 1, "exactly one new tab");
    assert_eq!(
        after.engine_sessions,
        before_sessions + 1,
        "exactly one new live session"
    );
    assert_eq!(after.command_bar_mode, CommandBarMode::Inactive);
    assert!(!after.chrome_focused, "the fresh page owns focus");
    let new_view = after.active_web_view.expect("fresh view");
    assert_ne!(new_view, previous_view, "the fresh view is not the source");
    assert_eq!(measured.active_web_view, Some(new_view));
    assert!(!measured.chrome_focused);
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    assert_eq!(
        after.command_bar_text, destination_url,
        "the opened destination is the record's canonical URL"
    );
    measured
}

/// Drive the B2 Saved Page matrix block and return the raw samples per
/// scenario. Runs after the history-rich command block of the interaction
/// matrix; state before it: the seeded ≥1,000-record Saved Page provider
/// loaded at startup, the history-rich corpus, and web focus on the active
/// view. The search loop restores the uniform web origin with Escape after
/// every sample; the open loop closes the fresh tab through the production
/// Cmd-W path after every sample.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_saved_pages_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    page_one: &str,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    // The active view the command-text samples commit against (typing never
    // changes the active view; the open loop restores it after each sample).
    let state = client.snapshot().expect("saved page state");
    let active_view = state.active_web_view.expect("saved page active view");
    assert!(
        state.store.saved_pages_count >= 1_000,
        "the seeded Saved Page provider is loaded"
    );

    // Discarded warm-up: the first command presentation after the seeded
    // provider pays the one-time corpus build/presentation cost. It is
    // unmeasured setup — the same discarded-warm-up practice the release
    // benchmarks use — so every accepted sample measures the steady
    // production interaction.
    let _ = benchmark_saved_page_search_first_result(
        client,
        since,
        *operation,
        SAVED_PAGE_QUERY,
        SAVED_PAGE_FIRST_KEY,
        active_view,
    );
    *operation += 1;
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: "escape".into(),
        })
        .expect("escape after saved page warm-up");
    client
        .send_ok(&Request::FlushFrame)
        .expect("settle escape after saved page warm-up");

    let mut search_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let measured = benchmark_saved_page_search_first_result(
            client,
            since,
            *operation,
            SAVED_PAGE_QUERY,
            SAVED_PAGE_FIRST_KEY,
            active_view,
        );
        search_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("saved_page_search_first_result".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: Escape after every sample restores the web
        // origin (each sample starts from the uniform web focus).
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape after saved page search sample");
        client
            .send_ok(&Request::FlushFrame)
            .expect("settle escape after saved page search sample");
    }

    // The open loop: each sample opens exactly one fresh regular tab/session
    // for the seeded target record, then the unmeasured Cmd-W closes the
    // fresh tab to restore the uniform web origin for the next sample.
    let mut open_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let measured =
            benchmark_saved_page_open(client, since, *operation, SAVED_PAGE_TARGET_ID, page_one);
        open_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("saved_page_open".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: close the fresh tab through the production
        // Cmd-W path (the fresh tab is the active tab in web focus).
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "cmd-w".into(),
            })
            .expect("close fresh saved page tab");
        *since = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::TabClosed { .. }),
            *since,
        )
        .0;
        wait_active_navigation_idle(client);
    }
    let restored = client.snapshot().expect("saved page restored snapshot");
    assert_eq!(
        restored.tabs.len(),
        state.tabs.len(),
        "the open loop restored the original tab count"
    );

    let mut raw = BTreeMap::new();
    raw.insert("saved_page_search_first_result".to_string(), search_samples);
    raw.insert("saved_page_open".to_string(), open_samples);
    raw
}
