//! Full-search command-surface benchmark driver (queue item 16,
//! `command_bar_full_search`): the complete-result-set sibling of the
//! first-result path. This focused module owns the full-search expectations
//! and driver so `command.rs` stays at its pre-full-search size; the shared
//! production command-text path, `command_setup`, and the first-result /
//! activation drivers remain in `command.rs`.
//!
//! `benchmark_command_bar_full_search` measures the same production
//! command-text path as `benchmark_command_bar_first_result` — Cmd-L,
//! replace-selection typing, the real ranking debounce, and a render of the
//! result surface committing the sample in the app — against the same
//! 500-tab + 10k-history corpus, but for the *complete* ranked result-set
//! presentation: the query matches far more rows than the limit-16 window,
//! so the in-app probe arms the exact full row-key window plus the truthful
//! total match count and rejects any partial, reordered, truncated, or stale
//! window or a wrong total before the sample leaves. The driver then asserts
//! the typed snapshot carries the full window and the truthful total on the
//! wire.

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;

/// The deterministic expected complete-result-set presentation for the
/// full-search query against the seeded history corpus (see
/// `seed_history_store`): every entry id `N` carries `last_visited = N-1`, so
/// the query "history" (which matches every entry plus the navigate/search
/// fallback row) ranks the limit-16 window as the most recently visited ids
/// `count..=count-15` and reports the truthful total `count + 1`. The driver
/// derives the exact window from the seed contract so the in-app probe can
/// reject any partial/reordered/stale window or wrong total.
pub(crate) fn full_search_expectations(history_count: usize) -> (Vec<String>, usize) {
    let keys = (0..16usize)
        .map(|i| format!("history:{}", history_count - i))
        .collect();
    (keys, history_count + 1)
}

/// The history-rich complete-result-set path (queue item 16,
/// `command_bar_full_search`): the same production command-text path as
/// `benchmark_command_bar_first_result` — Cmd-L, replace-selection typing,
/// the real ranking debounce, and a render of the result surface committing
/// the sample in the app — measured against the same 500-tab + 10k-history
/// corpus, but for the *complete* ranked result-set presentation: the query
/// matches far more rows than the limit-16 window, so the in-app probe arms
/// the exact full row-key window plus the truthful total match count and
/// rejects any partial, reordered, truncated, or stale window or a wrong
/// total before the sample leaves. The driver then asserts the typed
/// snapshot carries the full window and the truthful total on the wire.
pub(crate) fn benchmark_command_bar_full_search(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    text: &str,
    expected_keys: &[String],
    expected_total: usize,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkCommandBarFullSearch {
            operation_id,
            text: text.to_string(),
            expected_keys: expected_keys.to_vec(),
            expected_total,
        })
        .expect("benchmark command bar full search");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::CommandBarFullSearch,
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
    let snapshot = client.snapshot().expect("command full search snapshot");
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Location);
    assert!(snapshot.chrome_focused);
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(
        snapshot.command_results.len(),
        expected_keys.len(),
        "the complete limit-16 result window is presented"
    );
    assert_eq!(
        snapshot
            .command_results
            .iter()
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>(),
        expected_keys.iter().map(String::as_str).collect::<Vec<_>>(),
        "the presented window carries exactly the expected ranked keys"
    );
    assert_eq!(
        snapshot
            .command_results
            .first()
            .map(|row| row.group.as_str()),
        Some("history")
    );
    assert_eq!(
        snapshot.command_result_total, expected_total,
        "the truthful total match count is presented on the wire"
    );
    assert!(
        snapshot.store.history_count >= 10_000,
        "the history-rich corpus must carry the seeded provider scale"
    );
    measured
}
