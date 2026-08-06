//! Testkit-only mutation seams of the root view (testkit feature).
//!
//! The E2E gateway dispatches typed mutation requests onto these app
//! methods; benchmark drivers live in `super::benchmarks` and the semantic
//! snapshot in `super::snapshot`, so no module outgrows the ~500-line
//! budget.

use bbr_browser::{ControllerError, ControllerEvent, SpaceId};
use bbr_web::{ProfileId, WebAction};
use gpui::Context;

use super::{BbrApp, BrowserSink, FocusSink};

mod snapshot;

/// Create a profile with its first space (E2E seam; production uses the
/// startup profile).
impl BbrApp {
    pub fn create_profile_space(
        &mut self,
        profile_title: &str,
        space_title: &str,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self
            .controller
            .create_profile_space(profile_title, space_title);
        self.apply_mutation(result, cx)
    }

    /// Create a space in an existing profile (E2E seam).
    pub fn create_space(
        &mut self,
        profile: ProfileId,
        title: &str,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self.controller.create_space_event(profile, title);
        self.apply_mutation(result, cx)
    }

    /// Move a tab to another space (E2E seam; follows the profile-boundary
    /// recreation policy inside the controller).
    pub fn move_tab(
        &mut self,
        index: usize,
        to_space: SpaceId,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self.controller.move_tab(index, to_space);
        self.apply_mutation(result, cx)
    }

    /// Suspend a tab's native session (E2E seam).
    pub fn suspend_tab(&mut self, index: usize, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let result = self.controller.suspend_tab(index);
        self.apply_mutation(result, cx)
    }

    /// Route a generic [`WebAction`] through the focused controller session.
    pub fn web_action(&mut self, action: WebAction) -> Option<ControllerEvent> {
        self.controller
            .navigate(action)
            .map_err(|error| {
                log::error!("could not route web action: {error}");
                error
            })
            .ok()
    }

    /// Install a focus-change sink (used by the E2E gateway).
    pub fn set_focus_sink(&mut self, sink: FocusSink) {
        *self.focus_sink.borrow_mut() = Some(sink);
    }

    /// Install a semantic browser-mutation sink (used by the E2E gateway).
    pub fn set_browser_sink(&mut self, sink: BrowserSink) {
        *self.browser_sink.borrow_mut() = Some(sink);
    }

    fn apply_mutation(
        &mut self,
        result: Result<ControllerEvent, ControllerError>,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let outcome = result
            .map_err(|error| {
                log::error!("browser mutation failed: {error}");
                error
            })
            .ok();
        if let Some(event) = &outcome {
            self.emit_browser(event);
            cx.notify();
        }
        outcome
    }
}
