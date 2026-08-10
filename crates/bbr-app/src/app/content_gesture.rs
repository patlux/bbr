//! Web-content gesture commit execution (task packet
//! `web-content-navigation-gestures`).
//!
//! The native content-gesture bridge reporter pump and typed testkit
//! injection both enter [`BbrApp::on_content_gesture_commit`] — the exact
//! production method the native reporter sink calls. The controller owns the
//! logical decision matrix (exactly-once history dispatch vs. the atomic
//! Back-at-settled-root child-close/opener-activate transaction); this
//! module only executes the decided outcome through the one
//! browser-controller authority and records typed diagnostics.

use bbr_web::ContentGestureCommit;
use gpui::Context;

use super::BbrApp;

/// Typed diagnostics for the testkit snapshot: how the app classified each
/// committed logical gesture.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ContentGestureState {
    /// Commits received from the native bridge or testkit injection.
    pub(crate) commits: u64,
    /// Commits that dispatched exactly one real history action.
    pub(crate) history_routed: u64,
    /// Commits that ran the atomic Back-at-root opener return.
    pub(crate) opener_returns: u64,
    /// Commits that were deterministic no-ops (disabled direction, stale
    /// view, missing/stale opener, provisional load, hidden target).
    pub(crate) no_ops: u64,
}

impl BbrApp {
    /// Execute one committed logical content gesture against the exact bound
    /// view. Returns the typed controller outcome (`GestureHistoryRouted`,
    /// `OpenerReturn`, or `None` for a no-op) and applies the presentation
    /// side effects the outcome requires.
    pub fn on_content_gesture_commit(
        &mut self,
        commit: ContentGestureCommit,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        self.content_gesture.commits += 1;
        let outcome = self.controller.on_content_gesture(commit);
        let is_opener_return = matches!(
            outcome,
            Some(bbr_browser::ControllerEvent::OpenerReturn { .. })
        );
        match &outcome {
            Some(bbr_browser::ControllerEvent::GestureHistoryRouted { .. }) => {
                self.content_gesture.history_routed += 1;
            }
            Some(bbr_browser::ControllerEvent::OpenerReturn { .. }) => {
                self.content_gesture.opener_returns += 1;
            }
            Some(_) | None => self.content_gesture.no_ops += 1,
        }
        if let Some(outcome) = &outcome {
            self.emit_browser(outcome);
            // The opener return activates a different tab (and possibly a
            // different Space/Profile): it is a tab-context mutation, so the
            // same post-activation side effects as select/open apply. A
            // plain history dispatch keeps focus in the same native view and
            // mutates no tab context.
            if is_opener_return {
                self.clear_command_profile_hint();
                self.sync_command_bar();
                self.find_discard_if_unbound(cx);
                self.ensure_active_visible();
                self.sync_error_surface_visibility();
                self.zoom_after_activation();
                self.persist_structure();
            }
        }
        cx.notify();
        outcome
    }

    #[cfg(feature = "testkit")]
    pub(crate) fn content_gesture_state(&self) -> ContentGestureState {
        self.content_gesture
    }
}
