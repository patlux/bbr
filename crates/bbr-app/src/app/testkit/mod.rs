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

    /// Open the two-pane split over the two active-space tab indices (E2E
    /// seam into the production controller split path; the split-core lane
    /// owns the user-facing creation UI).
    pub fn set_split(
        &mut self,
        primary: usize,
        secondary: usize,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self.controller.open_split(primary, secondary);
        self.apply_mutation(result, cx)
    }

    /// Close the active space's split layout; both tabs and their warm
    /// sessions stay (E2E seam into the production controller path).
    pub fn close_split(&mut self, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let result = self.controller.close_split();
        self.apply_mutation(result, cx)
    }

    /// Deterministic dense-list seeding (testkit-only setup, DESIGN.md
    /// §11.6): open `tab_count` more tabs through the production controller
    /// open path, pin the first `pinned_count` identities, immediately
    /// suspend every tab outside the keep set (first `pinned_count` +
    /// global index ≥ `recent_start`), then activate the pinned tab 0.
    ///
    /// The keep set is computed from the *global* index (`existing_tabs +
    /// i`), so the harness may call this in batches and every batch keeps the
    /// same final identities live; each transient session is destroyed before
    /// the next open, so the concurrent live-session population never
    /// exceeds `pinned_count + recent + 1` — never `tab_count` WKWebViews at
    /// once. The deterministic session budget converges the exact `cap`
    /// afterwards (`enforce_session_budget`). The state is in-memory:
    /// persisted-restore acceptance is deliberately open until the
    /// persistence repository.
    pub fn seed_dense_state(
        &mut self,
        tab_count: usize,
        pinned_count: usize,
        recent_start: usize,
        url: &str,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let start = self.controller.snapshot().tabs.len();
        // Pin existing leading identities that belong in the pinned region
        // (the app's initial tab sits at global index 0 when `pinned_count`
        // > 0). Pinning appends at the end of the pinned region, so it never
        // shifts later identities.
        for global in 0..pinned_count.min(start) {
            let tab = self.controller.snapshot().tabs.get(global).cloned();
            if tab.is_some_and(|tab| !tab.pinned) {
                self.pin_tab(global, cx);
            }
        }
        for i in 0..tab_count {
            let global = start + i;
            let keep = global < pinned_count || global >= recent_start;
            if global < pinned_count {
                // Pin before hiding: the fresh tab is still the active tab at
                // its own index, so pinning appends it at the end of the
                // pinned region without shifting later identities.
                self.open_tab(&format!("{url}{global}"), cx);
                self.pin_tab(global, cx);
                continue;
            }
            // Open through the production path (a transient session is
            // created), then hide the fresh tab by selecting the pinned tab 0
            // and suspend it: the legal Warm → Suspended path the budget
            // policy uses. The session is destroyed before the next open, so
            // the transient live peak stays at pinned + recent + 1.
            self.open_tab(&format!("{url}{global}"), cx);
            if !keep {
                self.select_tab(0, cx);
                self.suspend_tab(global, cx);
            }
        }
        // The last opened tab is active; select the pinned tab 0 so the
        // active identity is inside the keep set (DESIGN.md §11.6 "active,
        // pinned, and recently used identities").
        self.select_tab(0, cx)
    }

    /// Deterministically converge the active space's live-session population
    /// to `cap` through the controller's session-budget policy (testkit-only
    /// setup; the policy itself is the production lifecycle path).
    pub fn enforce_session_budget(
        &mut self,
        cap: usize,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self.controller.enforce_session_budget(cap);
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
