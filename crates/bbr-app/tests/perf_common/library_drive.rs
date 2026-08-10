//! B3 Saved Pages Library scenario driving (packet `saved-pages-library`)
//! for the interaction matrix.
//!
//! Owns the B3 matrix block: the production Library search
//! (`saved_pages_library_search`) and the production Library scroll
//! (`saved_pages_library_scroll`) measured against the seeded ≥1,000-record
//! provider (the existing `seed_saved_pages_store` preseed — never fixture
//! setup inside a sample). The exact expected query result count/window and
//! the exact scroll anchor/window/selection are asserted from the typed
//! snapshot before any sample is accepted (fast-but-wrong guards); the
//! in-app probe additionally rejects a fast-but-wrong presentation before
//! the sample leaves the app. Split out of `drive.rs` so the B3 additions
//! live in a focused module under the ~500-line budget, mirroring the B2
//! `saved_pages_drive` module.

use std::collections::BTreeMap;

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_measurement;
use super::harness::AppHarness;
use super::scenarios::TrialConfig;

/// The full unfiltered provider size of the seeded ≥1,000-record store.
pub(super) const LIBRARY_TOTAL: usize = 1_000;
/// The deterministic narrow search over the seeded provider: `saved page 99`
/// matches exactly 11 records (titles "Saved Page 99" and "Saved Page 990"
/// … "Saved Page 999", BookmarkIds 100 and 991…=1000). In the provider
/// presentation order (`provider_order` = most-recently-updated first =
/// BookmarkId descending for the seeded store) the full window is [0, 11)
/// anchored on id 1000.
pub(super) const LIBRARY_SEARCH_NARROW: &str = "saved page 99";
pub(super) const LIBRARY_SEARCH_NARROW_TOTAL: usize = 11;
/// The narrow search window anchor: the first matching BookmarkId in the
/// provider order (id 1000).
pub(super) const LIBRARY_SEARCH_NARROW_ANCHOR: u64 = 1_000;
/// The deterministic wide search: `saved page 9` matches 111 records
/// ("Saved Page 9", "Saved Page 90"…"Saved Page 99", and "Saved Page 900"
/// … "Saved Page 999", ids 10, 91…=100, 901…=1000) — a full bounded window
/// [0, 24) anchored on id 1000, so the two queries guard both a changed
/// result count and a changed window edge.
pub(super) const LIBRARY_SEARCH_WIDE: &str = "saved page 9";
pub(super) const LIBRARY_SEARCH_WIDE_TOTAL: usize = 111;
/// The wide search window anchor: the first matching BookmarkId in the
/// provider order (id 1000, the newest record).
pub(super) const LIBRARY_SEARCH_WIDE_ANCHOR: u64 = 1_000;
/// The scroll-loop offsets over the unfiltered 1,000-record provider:
/// offset 0 renders window [0, 24) anchored on id 1000; offset 500 renders
/// window [500, 524) anchored on id 500. Both are valid mid-list positions
/// (never a clamped no-op), so every sample moves the anchor.
pub(super) const LIBRARY_SCROLL_OFFSETS: [(usize, usize, usize, u64); 2] =
    [(0, 0, 24, 1_000), (500, 500, 524, 500)];
/// The deterministic selection the scroll loop must preserve exactly: the
/// first BookmarkId of the seeded provider in presentation order (id 1000,
/// the offset-0 window anchor).
pub(super) const LIBRARY_SELECTED_ROW: u64 = 1_000;

/// The production Library search (`BenchmarkSavedPagesLibrarySearch`): the
/// typed search entry filters the loaded provider and the bounded window
/// re-renders; the sample commits only when the exact armed result count
/// and window materialize (fast-but-wrong guard in the app), and the driver
/// re-asserts the exact total/window/anchor from the typed snapshot before
/// accepting the sample.
#[allow(clippy::too_many_arguments)]
pub(super) fn benchmark_library_search(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    text: &str,
    expected_total: usize,
    expected_first: usize,
    expected_last: usize,
    expected_anchor: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkSavedPagesLibrarySearch {
            operation_id,
            text: text.to_string(),
            expected_total,
            expected_first,
            expected_last,
        })
        .expect("benchmark library search");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::SavedPagesLibrarySearch,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(
        measured.active_web_view,
        Some(expected_view),
        "the measured sample keeps the active view"
    );
    assert!(
        measured.chrome_focused,
        "the Library surface owns the chrome focus domain"
    );
    assert_eq!(
        measured.command_bar_mode,
        CommandBarMode::Inactive,
        "the Library never opens a command surface"
    );
    let snapshot = client.snapshot().expect("library search snapshot");
    let library = &snapshot.library;
    assert!(library.open, "the Library stays open during search samples");
    assert_eq!(
        library.total, expected_total,
        "the exact query result count is presented"
    );
    assert_eq!(
        (library.window_first, library.window_last),
        (expected_first, expected_last),
        "the exact bounded window is presented"
    );
    assert_eq!(
        library.window_anchor,
        Some(expected_anchor),
        "the exact window anchor is presented"
    );
    assert_eq!(
        library.rows.len(),
        expected_last - expected_first,
        "the rendered row count matches the window"
    );
    assert!(
        library.saved_pages_count >= 1_000,
        "the Library corpus carries the seeded provider scale"
    );
    measured
}

/// The production Library scroll (`BenchmarkSavedPagesLibraryScroll`): the
/// tracked scroll offset re-anchors the bounded window over the loaded
/// provider; the sample commits only when the exact armed window/anchor
/// materialize and the exact selection set is preserved (fast-but-wrong
/// guards in the app), and the driver re-asserts the same from the typed
/// snapshot before accepting.
#[allow(clippy::too_many_arguments)]
pub(super) fn benchmark_library_scroll(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    offset: usize,
    expected_first: usize,
    expected_last: usize,
    expected_anchor: u64,
    expected_selected: &[u64],
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkSavedPagesLibraryScroll {
            operation_id,
            offset,
            expected_first,
            expected_last,
            expected_anchor,
        })
        .expect("benchmark library scroll");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::SavedPagesLibraryScroll,
        operation_id,
        *since,
    );
    *since = seq;
    assert_eq!(
        measured.active_web_view,
        Some(expected_view),
        "the measured sample keeps the active view"
    );
    assert!(
        measured.chrome_focused,
        "the Library surface owns the chrome focus domain"
    );
    assert_eq!(
        measured.command_bar_mode,
        CommandBarMode::Inactive,
        "the Library never opens a command surface"
    );
    let snapshot = client.snapshot().expect("library scroll snapshot");
    let library = &snapshot.library;
    assert!(library.open, "the Library stays open during scroll samples");
    assert_eq!(
        (library.window_first, library.window_last),
        (expected_first, expected_last),
        "the exact scroll window is presented"
    );
    assert_eq!(
        library.window_anchor,
        Some(expected_anchor),
        "the exact scroll anchor is presented"
    );
    assert_eq!(
        library.selected, expected_selected,
        "scrolling preserves the exact selection set"
    );
    assert_eq!(
        library.rows.len(),
        expected_last - expected_first,
        "the rendered row count matches the window"
    );
    measured
}

/// Drive the B3 Saved Pages Library matrix block and return the raw samples
/// per scenario. Runs after the B2 Saved Page block; state before it: the
/// seeded ≥1,000-record provider loaded at startup and web focus on the
/// active view. The Library opens through the production entry (unmeasured
/// setup — the surface is fixed app state the samples run against, never a
/// timed region), then the search loop alternates the two deterministic
/// queries with an unmeasured clear restoring the unfiltered state between
/// samples, and the scroll loop alternates the two mid-list offsets with a
/// deterministic selection preserved exactly. The block runs last — the
/// corpus state after it does not matter.
pub(super) fn drive_library_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    // The seeded ≥1,000-record provider is loaded (enforced here so a
    // missing seed silently degrades the measurement to an empty provider).
    let state = client.snapshot().expect("library pre-block snapshot");
    assert!(
        state.store.saved_pages_count >= 1_000,
        "the Saved Page corpus must carry the seeded provider scale"
    );

    // Open the Library through the production entry (unmeasured setup).
    client
        .send_ok(&Request::OpenSavedPagesLibrary)
        .expect("open library");
    client
        .send_ok(&Request::FlushFrame)
        .expect("flush library open frame");
    let opened = client.snapshot().expect("library opened snapshot");
    assert!(opened.library.open, "the Library opened");
    assert_eq!(opened.library.total, LIBRARY_TOTAL);
    assert_eq!(opened.library.sort, "updated_newest");
    assert_eq!(
        opened.library.window_anchor,
        Some(LIBRARY_SELECTED_ROW),
        "the offset-0 window anchors on the first provider row"
    );
    // The active view the Library hid stays the app's active view (the
    // samples never touch the session); every measured sample asserts it.
    let active_view = opened.active_web_view.expect("library active view");

    // Discarded warm-up: the first search presentation after the seeded
    // provider pays the one-time window/presentation cost. It is unmeasured
    // setup — the same discarded-warm-up practice the other matrix blocks
    // use — so every accepted sample measures the steady production
    // interaction.
    let _ = benchmark_library_search(
        client,
        since,
        *operation,
        LIBRARY_SEARCH_NARROW,
        LIBRARY_SEARCH_NARROW_TOTAL,
        0,
        LIBRARY_SEARCH_NARROW_TOTAL,
        LIBRARY_SEARCH_NARROW_ANCHOR,
        active_view,
    );
    *operation += 1;
    client
        .send_ok(&Request::SetSavedPagesLibrarySearch {
            text: String::new(),
        })
        .expect("clear search after warm-up");
    client
        .send_ok(&Request::FlushFrame)
        .expect("settle clear after warm-up");

    // The search loop: each sample filters from the unfiltered state (the
    // unmeasured clear restores the uniform window between samples). The
    // two deterministic queries guard both the exact result count and the
    // exact window edge.
    let mut search_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let (text, total, last, anchor) = if sample % 2 == 0 {
            (
                LIBRARY_SEARCH_NARROW,
                LIBRARY_SEARCH_NARROW_TOTAL,
                LIBRARY_SEARCH_NARROW_TOTAL,
                LIBRARY_SEARCH_NARROW_ANCHOR,
            )
        } else {
            (
                LIBRARY_SEARCH_WIDE,
                LIBRARY_SEARCH_WIDE_TOTAL,
                24,
                LIBRARY_SEARCH_WIDE_ANCHOR,
            )
        };
        let measured = benchmark_library_search(
            client,
            since,
            *operation,
            text,
            total,
            0,
            last,
            anchor,
            active_view,
        );
        search_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("saved_pages_library_search".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: clear the search to restore the unfiltered
        // uniform window for the next sample.
        client
            .send_ok(&Request::SetSavedPagesLibrarySearch {
                text: String::new(),
            })
            .expect("clear search after sample");
        client
            .send_ok(&Request::FlushFrame)
            .expect("settle clear after search sample");
    }

    // Deterministic selection (unmeasured setup): the offset-0 window
    // anchor row; every scroll sample must preserve exactly this set.
    client
        .send_ok(&Request::ToggleSavedPagesLibrarySelect {
            id: LIBRARY_SELECTED_ROW,
        })
        .expect("select anchor row");
    client
        .send_ok(&Request::FlushFrame)
        .expect("flush selection frame");
    let selected = client.snapshot().expect("selection snapshot");
    assert_eq!(selected.library.selected, vec![LIBRARY_SELECTED_ROW]);

    // Discarded warm-up: one scroll sample pays the one-time window cost.
    let (warm_offset, warm_first, warm_last, warm_anchor) = LIBRARY_SCROLL_OFFSETS[0];
    let _ = benchmark_library_scroll(
        client,
        since,
        *operation,
        warm_offset,
        warm_first,
        warm_last,
        warm_anchor,
        &[LIBRARY_SELECTED_ROW],
        active_view,
    );
    *operation += 1;

    // The scroll loop: each sample moves the anchor mid-list (offset 0 and
    // 500 are both valid positions, never a clamped no-op) and must
    // preserve the exact selection set.
    let mut scroll_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let (offset, first, last, anchor) = LIBRARY_SCROLL_OFFSETS[sample % 2];
        let measured = benchmark_library_scroll(
            client,
            since,
            *operation,
            offset,
            first,
            last,
            anchor,
            &[LIBRARY_SELECTED_ROW],
            active_view,
        );
        scroll_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("saved_pages_library_scroll".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }
    let final_state = client.snapshot().expect("final library snapshot");
    assert_eq!(
        final_state.library.selected,
        vec![LIBRARY_SELECTED_ROW],
        "the scroll loop preserved the selection"
    );

    let mut raw = BTreeMap::new();
    raw.insert("saved_pages_library_search".to_string(), search_samples);
    raw.insert("saved_pages_library_scroll".to_string(), scroll_samples);
    raw
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_persistence::SavedPageRecord;

    /// The deterministic `seed_saved_pages_store` contract (scenarios.rs):
    /// id 1 is "Alpha Saved Target" over the fixture URL; every other id
    /// `k` is "Saved Page {k-1}" over `https://saved-{k-1:05}.example/`.
    fn seeded_pages(count: usize) -> Vec<SavedPageRecord> {
        (0..count)
            .map(|index| SavedPageRecord {
                id: (index + 1) as u64,
                canonical_url: if index == 0 {
                    "https://fixture.example/page1.html".into()
                } else {
                    format!("https://saved-{index:05}.example/")
                },
                captured_title: if index == 0 {
                    "Alpha Saved Target".into()
                } else {
                    format!("Saved Page {index}")
                },
                user_label: None,
                created_at: index as u64,
                updated_at: index as u64,
                collection_id: None,
            })
            .collect()
    }

    /// The deterministic model filter (the `record_matches` mirror: a
    /// case-insensitive literal substring over title/URL; the empty query
    /// matches everything).
    fn matches(page: &SavedPageRecord, query: &str) -> bool {
        let needle = query.to_lowercase();
        needle.is_empty()
            || page.captured_title.to_lowercase().contains(&needle)
            || page
                .user_label
                .as_deref()
                .is_some_and(|label| label.to_lowercase().contains(&needle))
            || page.canonical_url.to_lowercase().contains(&needle)
    }

    /// Locks the exact expectations the B3 driver guards against the
    /// seeded-shape provider: the presentation order (`provider_order` =
    /// most-recently-updated first, id-descending for the seeded store),
    /// the exact query result counts/windows/anchors, and the exact scroll
    /// windows/anchors. A future seed or model drift fails here before it
    /// can silently degrade the measured scenarios.
    #[test]
    fn seeded_provider_expectations_are_exact() {
        let pages = seeded_pages(LIBRARY_TOTAL);
        // `provider_order`: updated_at descending, then id descending.
        let mut order = pages.iter().collect::<Vec<_>>();
        order.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(b.id.cmp(&a.id)));
        assert_eq!(order[0].id, 1_000, "the newest record leads");
        assert_eq!(order.last().unwrap().id, 1, "the oldest record trails");
        assert_eq!(
            order.first().unwrap().id,
            LIBRARY_SELECTED_ROW,
            "the offset-0 window anchor is the first presented record"
        );

        // Narrow search: exact result count and window anchor.
        let narrow = order
            .iter()
            .copied()
            .filter(|page| matches(page, LIBRARY_SEARCH_NARROW))
            .collect::<Vec<_>>();
        assert_eq!(narrow.len(), LIBRARY_SEARCH_NARROW_TOTAL);
        assert_eq!(narrow[0].id, LIBRARY_SEARCH_NARROW_ANCHOR);

        // Wide search: exact result count and window anchor.
        let wide = order
            .iter()
            .copied()
            .filter(|page| matches(page, LIBRARY_SEARCH_WIDE))
            .collect::<Vec<_>>();
        assert_eq!(wide.len(), LIBRARY_SEARCH_WIDE_TOTAL);
        assert_eq!(wide[0].id, LIBRARY_SEARCH_WIDE_ANCHOR);

        // Scroll windows: the exact `visible_window` math over the
        // unfiltered provider, including the window edge (last - first).
        for (offset, expected_first, expected_last, expected_anchor) in LIBRARY_SCROLL_OFFSETS {
            let (first, last) = bbr_ui::components::visible_window(LIBRARY_TOTAL, 24, offset);
            assert_eq!((first, last), (expected_first, expected_last));
            assert_eq!(order[first].id, expected_anchor);
        }
        // The windowed search rows stay bounded (the 1k bounded-render
        // contract the samples run against).
        let (first, last) = bbr_ui::components::visible_window(LIBRARY_SEARCH_WIDE_TOTAL, 24, 0);
        assert_eq!((first, last), (0, 24));
    }
}
