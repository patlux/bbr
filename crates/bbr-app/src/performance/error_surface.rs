//! Error-surface presentation identity for the presentation probe (task
//! packet `error-surface-contract`).
//!
//! The `ErrorSurfacePresent` benchmark measures the production
//! incident-presentation path (`on_incident` → the contract `present`
//! lifecycle) and commits only from a *render* of the §11.8-style error
//! surface that carried the armed category over the unchanged active view
//! (fast-but-wrong guard — a frame that omitted the surface, drew a
//! different category, or changed the view/session can never commit). The
//! observation is render-gated: the main-window render records the surface
//! state it actually drew, so a model mutation alone can never emit a
//! measurement.
//!
//! Fixture/network time is excluded by construction: the classified
//! incident is applied in-process through the exact seam the native
//! delegate's incident sink drives (`on_incident`) — never through a
//! network transfer. The native-code → taxonomy mapping is proven by the
//! `bbr-web-macos::error_mapping` unit suite; this module gates the
//! *presentation commit* of the surface under the warm budget.
//!
//! This child module keeps the probe's error-surface surface out of
//! `performance.rs` (which stays under the ~500-line budget); it reaches
//! the probe's private `State`/`Pending` through parent-module privacy,
//! exactly like `super::find` and `super::downloads`.

use bbr_web::web_errors::IncidentCategory;

use super::{Pending, PerformanceProbe, State};

/// The exact error-surface presentation a pending sample must commit from.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ErrorSurfaceExpectation {
    /// The §11.8-style error surface is drawn in the content area for the
    /// armed closed category (TLS, navigation, or offline — the process-
    /// crash class is presented by the §11.8 crash panel, never here).
    Present { category: IncidentCategory },
}

/// The error-surface presentation the main-window render actually drew (the
/// render milestone; the content-area render records it after selecting the
/// surface). No observation is recorded for a frame that omitted the
/// surface, so a pending sample can never commit from a render that did not
/// present it.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct ErrorSurfaceObservation {
    pub visible: bool,
    pub category: Option<IncidentCategory>,
}

impl PerformanceProbe {
    /// Arm the exact error-surface presentation the pending sample must
    /// commit from (the surface drawn for `category`). Arming clears any
    /// prior observation: a render that omitted the surface (or an older
    /// presentation) can never satisfy the newer expectation.
    pub fn expect_error_surface(&self, category: IncidentCategory) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.error_surface = Some(ErrorSurfaceExpectation::Present { category });
            state.latest_error_surface = ErrorSurfaceObservation::default();
        }
    }

    /// Record the error-surface presentation the render actually drew for
    /// the content area (the render milestone; the render calls this after
    /// selecting the surface). `visible == false` (or a frame without the
    /// surface) records the empty observation so a stale surface can never
    /// satisfy a later arm.
    pub fn observe_error_surface(&self, visible: bool, category: Option<IncidentCategory>) {
        self.inner.borrow_mut().latest_error_surface =
            ErrorSurfaceObservation { visible, category };
    }
}

/// Whether the pending sample's armed error-surface expectation matches the
/// latest render observation. A frame that omitted the surface (or drew a
/// different category) fails the guard; `None` expectations pass (other
/// benchmark kinds never arm the error-surface identity).
pub(super) fn error_surface_matches(pending: &Pending, state: &State) -> bool {
    let Some(expectation) = pending.error_surface.as_ref() else {
        return true;
    };
    match expectation {
        ErrorSurfaceExpectation::Present { category } => {
            state.latest_error_surface.visible
                && state.latest_error_surface.category == Some(*category)
        }
    }
}
