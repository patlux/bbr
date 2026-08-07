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
}

/// The command-result presentation observed at the ranking publish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CommandObservation {
    pub generation: u64,
    pub first_key: Option<String>,
    pub selected_key: Option<String>,
    pub result_state: CommandResultState,
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
            });
            // Only a render after this arming may satisfy the guard; an old
            // render's observation (from a previous sample or a pre-typing
            // frame) must never commit.
            state.latest_command = None;
        }
    }

    /// Record the command-result identity the render actually drew after
    /// constructing the result palette (the render milestone). `rendered` is
    /// true only when the result surface with its ranked rows was part of
    /// the frame; otherwise the observation is cleared, so a pending sample
    /// cannot commit from a frame that omitted or did not present the
    /// palette. The ranking publication itself never records an observation
    /// and therefore can never emit a measurement.
    pub fn observe_rendered_command(
        &self,
        generation: u64,
        rendered: bool,
        first_key: Option<&str>,
        selected_key: Option<&str>,
        result_state: CommandResultState,
    ) {
        let mut state = self.inner.borrow_mut();
        state.latest_command = if rendered {
            Some(CommandObservation {
                generation,
                first_key: first_key.map(str::to_string),
                selected_key: selected_key.map(str::to_string),
                result_state,
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

    /// True while a command benchmark is pending. The production activation
    /// path suppresses its generic synchronous commit for these kinds: the
    /// benchmark driver must validate caller-specific destination/session
    /// state before any sample can leave the process.
    pub fn command_benchmark_pending(&self) -> bool {
        matches!(
            self.pending_kind(),
            Some(
                BenchmarkKind::CommandType
                    | BenchmarkKind::CommandActivate
                    | BenchmarkKind::CommandActivateNewTab
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
            expected.generation == observed.generation
                && expected.first_key.as_deref() == observed.first_key.as_deref()
                && expected.selected_key.as_deref() == observed.selected_key.as_deref()
                && expected.result_state == observed.result_state
        }
        (Some(_), None) => false,
    }
}
