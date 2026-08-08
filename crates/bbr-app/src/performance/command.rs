//! Command-result presentation identity for the presentation probe.
//!
//! The §11.5 `command_type` benchmark measures the production text entry,
//! the real ranking debounce, and the *rendered* result presentation. The
//! commit predicate therefore carries command identity: the generation that
//! published, the first and selected result keys, and the typed result
//! state. A sample commits only from a render that actually drew the result
//! palette carrying the exact armed identity — a stale generation, a wrong
//! first/selected row, or a wrong state rejects the sample in the app
//! (fast-but-wrong guard). The observation is render-gated: the ranking
//! publication alone can never emit a measurement, and an observation from
//! an older render is cleared whenever a new expectation is armed.
//!
//! This child module keeps the probe's command-specific surface out of
//! `performance.rs` (which stays under the ~500-line budget); it reaches the
//! probe's private `State`/`Pending` through parent-module privacy.

use bbr_testkit::protocol::{BenchmarkKind, CommandResultState};

use crate::app::palette::CommandSurface;

use super::{Pending, PerformanceProbe, State};

/// The exact command-result identity a pending sample must commit from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CommandExpectation {
    /// The generation whose publication may commit the sample. The app arms
    /// the generation the production entry just scheduled, so any other
    /// publication (stale task or deterministic flush) is rejected.
    pub generation: u64,
    /// The first (correct) result key of the published generation.
    pub first_key: Option<String>,
    /// The selected row's key (the palette's selection).
    pub selected_key: Option<String>,
    /// The typed result state the presentation must show.
    pub result_state: CommandResultState,
    /// The full-search twin ([`BenchmarkKind::CommandBarFullSearch`]): when
    /// `Some`, the complete ranked result-set presentation must carry the
    /// exact row-key window and the truthful total match count — a partial,
    /// reordered, truncated, or stale window, or a wrong total, rejects the
    /// sample. Takes precedence over the first/selected-key guard.
    pub full_set: Option<FullSetExpectation>,
}

/// The exact complete result-set presentation a full-search sample commits
/// from: the limit-16 row-key window plus the total match count before
/// truncation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FullSetExpectation {
    /// The full limit-16 row-key window, in display order.
    pub keys: Vec<String>,
    /// The truthful total match count of the published generation.
    pub total: usize,
}

/// The command-result presentation observed at the ranking publish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CommandObservation {
    pub generation: u64,
    pub first_key: Option<String>,
    pub selected_key: Option<String>,
    pub result_state: CommandResultState,
    /// The full limit-16 row-key window the render actually drew, in display
    /// order (the complete result-set presentation the full-search guard
    /// compares against).
    pub row_keys: Vec<String>,
    /// The total match count the render's published generation reported.
    pub total: usize,
}

impl PerformanceProbe {
    /// Arm the exact command-result presentation identity the pending sample
    /// must commit from (generation, first/selected result keys, typed
    /// state). Callers arm it before the real debounce publishes. Arming
    /// clears any prior observation: a render that drew an older publication
    /// can never satisfy a newer expectation, so only a render that occurs
    /// after this arming can commit the sample.
    pub fn expect_command_presentation(
        &self,
        generation: u64,
        first_key: &str,
        selected_key: &str,
        result_state: CommandResultState,
    ) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.command = Some(CommandExpectation {
                generation,
                first_key: Some(first_key.to_string()),
                selected_key: Some(selected_key.to_string()),
                result_state,
                full_set: None,
            });
            // Only a render after this arming may satisfy the guard; an old
            // render's observation (from a previous sample or a pre-typing
            // frame) must never commit.
            state.latest_command = None;
        }
    }

    /// Arm the exact complete result-set presentation the pending sample
    /// must commit from (generation, the full limit-16 row-key window, the
    /// truthful total match count, and the typed state). The full-search
    /// benchmark arms this instead of the first-key expectation; the commit
    /// predicate then rejects any render whose drawn window or total differs
    /// (a partial, reordered, truncated, or stale window, or a wrong total).
    /// Arming clears any prior observation, exactly like the first-key arm.
    pub fn expect_command_full_set(
        &self,
        generation: u64,
        keys: &[String],
        total: usize,
        result_state: CommandResultState,
    ) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.command = Some(CommandExpectation {
                generation,
                first_key: keys.first().cloned(),
                selected_key: keys.first().cloned(),
                result_state,
                full_set: Some(FullSetExpectation {
                    keys: keys.to_vec(),
                    total,
                }),
            });
            // Only a render after this arming may satisfy the guard.
            state.latest_command = None;
        }
    }

    /// Record the command-result identity the render actually drew after
    /// constructing the result palette (the render milestone). `rendered` is
    /// true only when the result surface with its ranked rows was part of
    /// the frame; otherwise the observation is cleared, so a pending sample
    /// cannot commit from a frame that omitted or did not present the
    /// palette. The ranking publication itself never records an observation
    /// and therefore can never emit a measurement. `row_keys`/`total` carry
    /// the complete result-set presentation the full-search guard compares
    /// against.
    #[allow(clippy::too_many_arguments)] // the flat observed-identity surface
    pub fn observe_rendered_command(
        &self,
        generation: u64,
        rendered: bool,
        first_key: Option<&str>,
        selected_key: Option<&str>,
        result_state: CommandResultState,
        row_keys: &[String],
        total: usize,
    ) {
        let mut state = self.inner.borrow_mut();
        state.latest_command = if rendered {
            Some(CommandObservation {
                generation,
                first_key: first_key.map(str::to_string),
                selected_key: selected_key.map(str::to_string),
                result_state,
                row_keys: row_keys.to_vec(),
                total,
            })
        } else {
            None
        };
    }

    /// The kind of the pending measurement, if any.
    pub fn pending_kind(&self) -> Option<BenchmarkKind> {
        self.inner
            .borrow()
            .pending
            .as_ref()
            .map(|pending| pending.kind)
    }

    /// Require the pending presentation to show `surface` as the §11.4
    /// command-surface presentation (None = closed). The palette-open
    /// benchmark arms the centered palette; the commit predicate then
    /// rejects any render whose observed surface differs (a render that
    /// omitted the palette or drew the wrong presentation).
    pub fn expect_surface(&self, surface: CommandSurface) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.expected_surface = Some(surface);
        }
    }

    /// Record the §11.4 command-surface presentation the render actually
    /// constructed into the frame (the render milestone). `None` when the
    /// frame contained no palette/overlay presentation, so a pending
    /// palette-open sample can never commit from a render that omitted the
    /// surface.
    pub fn observe_rendered_surface(&self, surface: CommandSurface) {
        self.inner.borrow_mut().latest_surface = surface;
    }

    /// True while a command benchmark is pending. The production activation
    /// path suppresses its generic synchronous commit for these kinds: the
    /// benchmark driver must validate caller-specific destination/session
    /// state before any sample can leave the process. `CommandBarFirstResult`
    /// is the history-rich twin of `CommandType` (same command-text entry
    /// body and render-inclusive commit), and `CommandBarFullSearch` is its
    /// complete-result-set sibling — so all three kinds suppress the generic
    /// activation commit while pending.
    pub fn command_benchmark_pending(&self) -> bool {
        matches!(
            self.pending_kind(),
            Some(
                BenchmarkKind::CommandType
                    | BenchmarkKind::CommandBarFirstResult
                    | BenchmarkKind::CommandBarFullSearch
                    | BenchmarkKind::CommandActivate
                    | BenchmarkKind::CommandActivateNewTab
                    | BenchmarkKind::PaletteOpen
            )
        )
    }
}

/// The command-identity part of the presentation predicate: true when no
/// command expectation is armed, or when the observed rendered identity
/// carries exactly the armed generation/keys/state. An armed expectation
/// with no render observation yet can never commit.
pub(super) fn command_matches(pending: &Pending, state: &State) -> bool {
    match (pending.command.as_ref(), state.latest_command.as_ref()) {
        (None, _) => true,
        (Some(expected), Some(observed)) => {
            // The full-search guard compares the complete rendered result-set
            // presentation: the exact row-key window plus the truthful total,
            // still gated on the generation and typed state.
            if let Some(full) = expected.full_set.as_ref() {
                return expected.generation == observed.generation
                    && expected.result_state == observed.result_state
                    && observed.row_keys == full.keys
                    && observed.total == full.total;
            }
            expected.generation == observed.generation
                && expected.first_key.as_deref() == observed.first_key.as_deref()
                && expected.selected_key.as_deref() == observed.selected_key.as_deref()
                && expected.result_state == observed.result_state
        }
        (Some(_), None) => false,
    }
}
