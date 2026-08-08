//! Context-menu presentation identity for the presentation probe (queue
//! item 12, part B).
//!
//! The `context_menu_open` benchmark measures the production event-driven
//! open path (the exact seam the native bridge's `ContextMenuRequested`
//! routes to) and commits only from a render of the one `NativeAboveWeb`
//! portal window that drew the menu over the same active view with the
//! armed pointer anchor — a fast-but-wrong guard: a render that omitted the
//! menu, changed the bound view, or lost the anchor can never commit. The
//! observation is recorded by the portal render itself
//! (`BbrApp::observe_portal_context_menu_render`), so a sample can never
//! commit from state the app did not genuinely present.
//!
//! This child module keeps the probe's context-menu-specific surface out of
//! `performance.rs` (which stays under the ~500-line budget); it reaches the
//! probe's private `State`/`Pending` through parent-module privacy, exactly
//! like `super::find`.

use bbr_web::WebViewId;

use super::{Pending, PerformanceProbe, State};

/// The exact menu presentation a pending sample must commit from.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ContextMenuExpectation {
    /// The menu is open over this bound view, anchored at this pointer
    /// point (window content coordinates).
    Open {
        bound_view: WebViewId,
        point: [f32; 2],
    },
}

/// The menu state the portal render actually drew.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct ContextMenuObservation {
    pub open: bool,
    pub focused: bool,
    pub bound_view: Option<WebViewId>,
    pub point: [f32; 2],
}

impl PerformanceProbe {
    /// Arm the exact menu presentation the pending sample must commit from.
    /// Arming clears any prior observation: a render that showed an earlier
    /// open (or nothing) can never satisfy a newer expectation.
    pub fn expect_context_menu_open(&self, bound_view: WebViewId, point: [f32; 2]) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.context_menu = Some(ContextMenuExpectation::Open { bound_view, point });
            state.latest_context_menu = None;
        }
    }

    /// Record the menu state the portal render actually drew (the
    /// milestone). The context-menu benchmarks commit only from this
    /// observation: the menu is open, focused, over the bound view, at the
    /// armed anchor.
    pub fn observe_context_menu_state(
        &self,
        open: bool,
        focused: bool,
        bound_view: Option<WebViewId>,
        point: [f32; 2],
    ) {
        self.inner.borrow_mut().latest_context_menu = Some(ContextMenuObservation {
            open,
            focused,
            bound_view,
            point,
        });
    }
}

/// The context-menu part of the presentation predicate: true when no
/// context-menu expectation is armed, or when the observed state carries
/// exactly the armed open identity over the bound view at the armed anchor.
/// An armed expectation with no observation yet can never commit.
pub(super) fn context_menu_matches(pending: &Pending, state: &State) -> bool {
    let Some(expected) = pending.context_menu.as_ref() else {
        return true;
    };
    let observed = state.latest_context_menu.as_ref();
    match (expected, observed) {
        (ContextMenuExpectation::Open { bound_view, point }, Some(observed)) => {
            observed.open
                && observed.focused
                && observed.bound_view == Some(*bound_view)
                && observed.point == *point
        }
        (_, None) => false,
    }
}
