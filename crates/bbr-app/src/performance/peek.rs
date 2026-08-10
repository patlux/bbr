//! Transient Peek presentation identity for the presentation probe (Link
//! Preview / Peek packet).
//!
//! The `peek_request_to_present` benchmark measures the production eligible
//! new-window → Peek path and commits only from a *render* of the Peek bar
//! that carried the exact armed presentation: the preview session drawn in
//! the one `NativeAboveWeb` portal window, raised above its covered warm
//! origin (truthful z-order), with the origin still the active view. The
//! observation is render-gated — the portal window's `PortalRoot` records
//! the Peek state it actually drew, so the preview creation alone can never
//! emit a measurement and a frame that omitted the bar can never commit one.
//!
//! `peek_dismiss_to_origin` and `peek_promote_to_tab` need no portal-render
//! observation: their semantic milestone is the authoritative browser state
//! (the warm origin active/focused again, or the same preview session as a
//! regular tab), which the probe's general browser-state guard checks and
//! the drivers assert on the typed snapshot. This child module keeps the
//! probe's Peek-specific surface out of `performance.rs` (which stays under
//! the ~500-line budget); it reaches the probe's private `State`/`Pending`
//! through parent-module privacy, exactly like `super::find`.

use bbr_web::WebViewId;

use super::{Pending, PerformanceProbe, State};

/// The exact Peek presentation a pending sample must commit from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PeekExpectation {
    /// The preview session is presented: the Peek bar is drawn for `view`
    /// with the covered origin `source` still the active view, and the
    /// preview's native subview sits above the origin's (`peek_request_to_present`).
    Presented { view: WebViewId, source: WebViewId },
}

/// The Peek presentation the portal render actually drew.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct PeekObservation {
    /// The Peek bar was drawn in the portal window this frame.
    pub presented: bool,
    /// The preview session the bar names (`None` when no preview is open).
    pub view: Option<WebViewId>,
    /// The covered warm origin the bar names.
    pub source: Option<WebViewId>,
    /// Truthful native z-order evidence: the preview's AppKit subview index
    /// exceeds its covered origin's (the preview is raised above it).
    pub raised_above_origin: bool,
}

impl PerformanceProbe {
    /// Arm the exact Peek presentation the pending sample must commit from.
    /// Callers arm it in the same synchronous update as the production
    /// mutation, before the portal render can land. Arming clears any prior
    /// observation: a render that drew an earlier presentation can never
    /// satisfy a newer expectation.
    pub fn expect_peek_presented(&self, view: WebViewId, source: WebViewId) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.peek = Some(PeekExpectation::Presented { view, source });
            state.latest_peek = None;
        }
    }

    /// Record the Peek state the portal render actually drew for the Peek
    /// bar (the render milestone; the portal `PortalRoot` calls this after
    /// consuming the app's live Peek surface). No observation is recorded
    /// for a frame that omitted the bar, so a pending sample can never
    /// commit from a render that did not present it.
    #[allow(clippy::too_many_arguments)] // the peek-state fields mirror the live Peek surface
    pub fn observe_peek_state(
        &self,
        presented: bool,
        view: Option<WebViewId>,
        source: Option<WebViewId>,
        raised_above_origin: bool,
    ) {
        let mut state = self.inner.borrow_mut();
        state.latest_peek = Some(PeekObservation {
            presented,
            view,
            source,
            raised_above_origin,
        });
    }
}

/// The Peek-presentation part of the presentation predicate: true when no
/// Peek expectation is armed, or when the observed rendered presentation
/// carries exactly the armed identity (bar drawn for the armed view above
/// the armed source). An armed expectation with no render observation yet
/// can never commit.
pub(super) fn peek_matches(pending: &Pending, state: &State) -> bool {
    let Some(expected) = pending.peek.as_ref() else {
        return true;
    };
    let observed = state.latest_peek.as_ref();
    match (expected, observed) {
        (PeekExpectation::Presented { view, source }, Some(observed)) => {
            observed.presented
                && observed.view == Some(*view)
                && observed.source == Some(*source)
                && observed.raised_above_origin
        }
        (_, None) => false,
    }
}
