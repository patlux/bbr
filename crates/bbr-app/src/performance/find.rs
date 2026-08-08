//! Find-presentation identity for the presentation probe (task packet
//! `find-in-page-portal-bar`).
//!
//! The `find_open`/`find_query_result`/`find_step` benchmarks measure the
//! production find paths and commit only from a *render* of the find bar
//! that carries the exact armed presentation: the bar open with the field
//! focused (`FindOpen`), or the WebKit-reported query/count/ordinal with no
//! request pending and no staleness (`FindQueryResult`/`FindStep`). The
//! observation is render-gated — the portal window's `PortalRoot` records
//! the find state it actually drew, so a completion alone can never emit a
//! measurement and a frame that omitted the bar can never commit one. A
//! pending, stale, or mismatched report rejects the sample in the app
//! (fast-but-wrong guard).
//!
//! This child module keeps the probe's find-specific surface out of
//! `performance.rs` (which stays under the ~500-line budget); it reaches the
//! probe's private `State`/`Pending` through parent-module privacy, exactly
//! like `super::command`.

use bbr_testkit::protocol::BenchmarkKind;

use super::{Pending, PerformanceProbe, State};

/// The exact find-bar presentation a pending sample must commit from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FindExpectation {
    /// The bar is open with the find field focused (`FindOpen`). No report
    /// is required — opening a fresh bar shows no count yet.
    Open,
    /// WebKit reported exactly this query/count/ordinal through the portable
    /// contract, no request is pending, and the document is not stale
    /// (`FindQueryResult`/`FindStep`).
    Report {
        query: String,
        match_count: usize,
        current_match: Option<usize>,
    },
}

/// The find-bar presentation the portal render actually drew.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct FindObservation {
    pub open: bool,
    pub focused: bool,
    pub query: String,
    pub match_count: Option<usize>,
    pub current_match: Option<usize>,
    pub pending: bool,
    pub stale: bool,
}

impl PerformanceProbe {
    /// Arm the exact find-bar presentation the pending sample must commit
    /// from. Callers arm it in the same synchronous update as the production
    /// mutation, before any completion can land. Arming clears any prior
    /// observation: a render that drew an earlier report (or the pre-report
    /// state) can never satisfy a newer expectation.
    pub fn expect_find_open(&self) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.find = Some(FindExpectation::Open);
            state.latest_find = None;
        }
    }

    /// Arm the exact WebKit-reported query/count/ordinal presentation the
    /// pending sample must commit from (fast-but-wrong guard: a pending,
    /// stale, or mismatched report rejects in the app).
    pub fn expect_find_report(
        &self,
        query: &str,
        match_count: usize,
        current_match: Option<usize>,
    ) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.find = Some(FindExpectation::Report {
                query: query.to_string(),
                match_count,
                current_match,
            });
            state.latest_find = None;
        }
    }

    /// Record the find state the portal render actually drew for the find
    /// bar (the render milestone; the portal `PortalRoot` calls this after
    /// consuming the app's live `FindSurface`). No observation is recorded
    /// for a frame that omitted the bar, so a pending sample can never
    /// commit from a render that did not present it.
    #[allow(clippy::too_many_arguments)] // the find-state fields mirror `FindState`
    pub fn observe_find_state(
        &self,
        open: bool,
        focused: bool,
        query: &str,
        match_count: Option<usize>,
        current_match: Option<usize>,
        pending: bool,
        stale: bool,
    ) {
        let mut state = self.inner.borrow_mut();
        state.latest_find = Some(FindObservation {
            open,
            focused,
            query: query.to_string(),
            match_count,
            current_match,
            pending,
            stale,
        });
    }

    /// True while a find benchmark is pending whose sample depends on an
    /// async WebKit completion (`FindQueryResult`/`FindStep`): the
    /// completion path schedules the render-inclusive commit for these
    /// kinds. `FindOpen` commits from the render scheduled at dispatch.
    pub fn find_async_benchmark_pending(&self) -> bool {
        matches!(
            self.pending_kind(),
            Some(BenchmarkKind::FindQueryResult | BenchmarkKind::FindStep)
        )
    }
}

/// The find-presentation part of the presentation predicate: true when no
/// find expectation is armed, or when the observed rendered presentation
/// carries exactly the armed identity. An armed expectation with no render
/// observation yet can never commit.
pub(super) fn find_matches(pending: &Pending, state: &State) -> bool {
    let Some(expected) = pending.find.as_ref() else {
        return true;
    };
    let observed = state.latest_find.as_ref();
    match (expected, observed) {
        (FindExpectation::Open, Some(observed)) => observed.open && observed.focused,
        (
            FindExpectation::Report {
                query,
                match_count,
                current_match,
            },
            Some(observed),
        ) => {
            observed.open
                && observed.focused
                && !observed.pending
                && !observed.stale
                && &observed.query == query
                && observed.match_count == Some(*match_count)
                && observed.current_match == *current_match
        }
        (_, None) => false,
    }
}
