//! B3 Saved Pages Library presentation identity for the presentation probe
//! (packet `saved-pages-library`).
//!
//! The `SavedPagesLibrarySearch`/`SavedPagesLibraryScroll` benchmarks
//! measure the production Library search/scroll entries and commit only
//! from the exact armed presentation: the query result count and the
//! bounded window for search; the window/anchor and the preserved
//! multi-select set for scroll. The fast-but-wrong guard rejects a
//! missing/wrong filter, a stale window, a wrong anchor, or a corrupted
//! selection before the sample leaves the app. The observation is recorded
//! by the benchmark driver from the authoritative post-mutation model (the
//! same model the render and the typed snapshot share), exactly like the
//! dense/error-surface drivers; the generic probe state
//! (view/sidebar/focus/mode) rides the existing fields.
//!
//! This child module keeps the probe's Library identity out of
//! `performance.rs` (which stays under the ~500-line budget); it reaches
//! the probe's private `State`/`Pending` through parent-module privacy,
//! exactly like `super::u2` and `super::error_surface`.

use super::{Pending, PerformanceProbe, State};

/// The Library presentation the main-window render/benchmark observed
/// (embedded in [`State`]). The empty default means no Library sample has
/// observed a presentation yet.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct LibraryObserved {
    /// The matching-record count behind the window (the search guard).
    pub total: usize,
    /// The bounded window `[first, last)` the render built.
    pub window_first: usize,
    pub window_last: usize,
    /// The stable scroll anchor (the `BookmarkId` of the first visible
    /// row); `None` for an empty window.
    pub anchor: Option<u64>,
    /// The multi-select set the presentation must preserve (the scroll
    /// guard).
    pub selected: Vec<u64>,
}

/// The Library presentation the pending sample must commit from (embedded
/// in [`Pending`]); `None` fields mean no expectation for that benchmark
/// class.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct LibraryExpected {
    /// The exact matching-record count the search sample must present.
    pub total: Option<usize>,
    /// The exact bounded window the sample must present.
    pub window_first: Option<usize>,
    pub window_last: Option<usize>,
    /// The exact scroll anchor (`Some(Some(id))` = that row, `Some(None)` =
    /// an empty window).
    pub anchor: Option<Option<u64>>,
    /// The exact multi-select set the sample must preserve.
    pub selected: Option<Vec<u64>>,
}

impl LibraryExpected {
    /// Whether any armed expectation differs from the observed Library
    /// presentation (the fast-but-wrong guard: a missing/wrong filter, a
    /// stale window, a wrong anchor, or a corrupted selection is rejected
    /// before it leaves the app).
    pub fn mismatches(&self, observed: &LibraryObserved) -> bool {
        self.total
            .is_some_and(|expected| expected != observed.total)
            || self
                .window_first
                .is_some_and(|expected| expected != observed.window_first)
            || self
                .window_last
                .is_some_and(|expected| expected != observed.window_last)
            || self
                .anchor
                .is_some_and(|expected| expected != observed.anchor)
            || self
                .selected
                .as_deref()
                .is_some_and(|expected| expected != observed.selected.as_slice())
    }
}

impl PerformanceProbe {
    /// Record the Library presentation the production mutation produced
    /// (the same model the render and the typed snapshot share), so the
    /// pending sample commits only against the truthful post-mutation
    /// state.
    pub fn observe_library(
        &self,
        total: usize,
        window_first: usize,
        window_last: usize,
        anchor: Option<u64>,
        selected: Vec<u64>,
    ) {
        self.inner.borrow_mut().library = LibraryObserved {
            total,
            window_first,
            window_last,
            anchor,
            selected,
        };
    }

    /// Arm the exact Library presentation the pending sample must commit
    /// from (the fast-but-wrong guard: the armed count/window/anchor/
    /// selection must match the observed presentation, or the commit is
    /// rejected in the app).
    pub fn expect_library(
        &self,
        total: usize,
        window_first: usize,
        window_last: usize,
        anchor: Option<u64>,
        selected: Vec<u64>,
    ) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.library = Some(LibraryExpected {
                total: Some(total),
                window_first: Some(window_first),
                window_last: Some(window_last),
                anchor: Some(anchor),
                selected: Some(selected),
            });
        }
    }
}

/// Whether the pending sample's armed Library expectation matches the
/// observed presentation. `None` expectations pass (other benchmark kinds
/// never arm the Library identity).
pub(super) fn library_matches(pending: &Pending, state: &State) -> bool {
    pending
        .library
        .as_ref()
        .is_none_or(|expected| !expected.mismatches(&state.library))
}
