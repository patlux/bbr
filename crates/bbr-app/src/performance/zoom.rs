//! Zoom-presentation identity for the presentation probe (queue item 11,
//! part A).
//!
//! The `zoom_step` benchmark measures the production `Cmd-Plus` path and
//! commits only from the app's observed zoom state: the active view's site
//! at exactly the armed level, that level actually applied to the native
//! view (`applied == Some(level)` — the `pageZoom` property write completed,
//! never a fast-but-wrong lifecycle-only change), and no unacknowledged
//! persistence write (`!pending` — the store write was flushed and acked).
//! The observation is recorded by the production mutation path itself
//! (`BbrApp::zoom_step`), so a sample can never commit from state that the
//! app did not genuinely reach.
//!
//! This child module keeps the probe's zoom-specific surface out of
//! `performance.rs` (which stays under the ~500-line budget); it reaches the
//! probe's private `State`/`Pending` through parent-module privacy, exactly
//! like `super::find`.

use super::{Pending, PerformanceProbe, State};

/// The exact zoom state a pending sample must commit from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ZoomExpectation {
    /// The active view's site presented exactly this level, applied natively
    /// to the view, with no pending persistence write.
    Level { level: u16 },
}

/// The zoom state the production mutation path observed after the change.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct ZoomObservation {
    pub level: u16,
    pub applied: Option<u16>,
    pub pending: bool,
}

impl PerformanceProbe {
    /// Arm the exact zoom state the pending sample must commit from.
    /// Callers arm it in the same synchronous update as the production
    /// mutation, before the observation lands. Arming clears any prior
    /// observation: a render/state that showed an earlier level can never
    /// satisfy a newer expectation.
    pub fn expect_zoom(&self, level: u16) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.zoom = Some(ZoomExpectation::Level { level });
            state.latest_zoom = None;
        }
    }

    /// Record the zoom state the production mutation path actually reached
    /// (the milestone). The zoom benchmarks commit only from this
    /// observation: the level changed, the native view carries it, and the
    /// persistence write is acknowledged.
    pub fn observe_zoom_state(&self, level: u16, applied: Option<u16>, pending: bool) {
        self.inner.borrow_mut().latest_zoom = Some(ZoomObservation {
            level,
            applied,
            pending,
        });
    }
}

/// The zoom part of the presentation predicate: true when no zoom
/// expectation is armed, or when the observed state carries exactly the
/// armed level applied to the view with no pending write. An armed
/// expectation with no observation yet can never commit.
pub(super) fn zoom_matches(pending: &Pending, state: &State) -> bool {
    let Some(expected) = pending.zoom.as_ref() else {
        return true;
    };
    let observed = state.latest_zoom.as_ref();
    match (expected, observed) {
        (ZoomExpectation::Level { level }, Some(observed)) => {
            observed.level == *level && observed.applied == Some(*level) && !observed.pending
        }
        (_, None) => false,
    }
}
