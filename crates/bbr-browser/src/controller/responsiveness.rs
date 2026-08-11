//! Renderer-availability integration on the [`super::BrowserController`]
//! (renderer-isolation lane): the portable monitor's lifecycle is driven
//! here — probes are armed only for visible sessions, `UnresponsiveChanged`
//! engine events are applied with the monitor's stale-id/generation
//! discipline, the crash authority marks `Crashed` + reconciles siblings,
//! and the closed recovery set (Wait / Reload / Close) routes through the
//! existing session authorities. Split from `controller.rs` so the gateway
//! module stays under the ~500-line budget.

use bbr_web::{WebEvent, WebViewId};

use crate::outcome::ControllerEvent;
use crate::responsiveness::RendererAvailability;
use crate::ControllerError;

use super::BrowserController;

impl BrowserController {
    /// Advance the liveness probes of every visible live session (the
    /// bounded cadence): arm due probes through the engine seam, apply the
    /// portable fail-closed deadline, and report every state transition as
    /// a typed observation. Hidden sessions are never probed; an idle
    /// window (no due probe) arms nothing.
    pub fn pump_responsiveness(&mut self) -> Vec<ControllerEvent> {
        let mut events = Vec::new();
        let views: Vec<WebViewId> = self.registry.views().collect();
        for view in views {
            let visible = self.visibility.get(&view) == Some(&true);
            let mut arm = |view: WebViewId| {
                self.engine.borrow_mut().probe_responsiveness(view);
            };
            if let Some(state) = self.responsiveness.pump(view, visible, &mut arm) {
                events.push(ControllerEvent::RendererStateChanged { view, state });
            }
        }
        events
    }

    /// Apply an adapter `UnresponsiveChanged` event to the owning tab's
    /// monitor record. The monitor rejects stale/late completions and
    /// unknown ids exactly like the `UnknownWebView` path; a transition is
    /// reported as a typed observation.
    pub fn on_unresponsive_changed(
        &mut self,
        view: WebViewId,
        unresponsive: bool,
    ) -> ControllerEvent {
        let Some(_tab) = self.registry.tab_for_view(view) else {
            return ControllerEvent::UnknownWebView { view };
        };
        let generation = self.responsiveness.generation(view).unwrap_or(0);
        let transition = if unresponsive {
            self.responsiveness.probe_timed_out(view, generation)
        } else {
            self.responsiveness.probe_completed(view, generation)
        };
        // Advance the cadence: a completion re-arms after the probe
        // interval, a timeout after the backoff (events drive re-arms).
        let _ = self.pump_responsiveness();
        match transition {
            Some(state) => ControllerEvent::RendererStateChanged { view, state },
            None => ControllerEvent::WebEvent {
                event: WebEvent::UnresponsiveChanged(view, unresponsive),
            },
        }
    }

    /// Handle a WebKit process termination: mark the owning tab discarded and
    /// destroy its session; the next activation recreates it. The tab's crash
    /// flag survives so the shell can render the §10.7 error state.
    ///
    /// The crash authority is dispatched from the sibling `events` module
    /// (`PageCrashed`), so it is `pub(super)` — visible across the
    /// `controller` subtree, never part of the crate's public API.
    pub(super) fn on_page_crash(&mut self, view: WebViewId) -> ControllerEvent {
        let Some(tab) = self.registry.tab_for_view(view) else {
            return ControllerEvent::UnknownWebView { view };
        };
        // The crash authority owns the session's availability: mark it
        // Crashed and re-arm every other live session (shared-process
        // reconciliation sweep) before the session is destroyed.
        self.responsiveness.on_crash(view);
        let _ = self.browser.discard_tab(tab);
        self.destroy_session_of(tab);
        self.crashed_tabs.insert(tab);
        self.check_invariants();
        ControllerEvent::TabDiscarded { tab }
    }

    /// The Wait recovery action: keep monitoring the unresponsive session.
    /// The state stays `Unresponsive` until a probe answers (a busy→alive
    /// page clears it); the closed recovery set is exactly Wait / Reload /
    /// Close.
    pub fn wait_unresponsive(&mut self, view: WebViewId) -> ControllerEvent {
        let state = if self.registry.tab_for_view(view).is_none() {
            return ControllerEvent::UnknownWebView { view };
        } else {
            self.responsiveness.wait(view);
            self.responsiveness.state(view)
        };
        let _ = self.pump_responsiveness();
        ControllerEvent::RendererStateChanged { view, state }
    }

    /// The Reload recovery action: destroy the hung session and recreate a
    /// **fresh** one for the same tab in its owning profile store (fresh
    /// `WebViewId`, same tab identity, cookie isolation preserved). A hung
    /// renderer may never honour `Reload`, so recovery never routes through
    /// `WebAction::Reload` — it reuses the crash panel's destroy → recreate
    /// seam. Refuses sessions that are not reported unresponsive.
    pub fn recover_unresponsive(
        &mut self,
        view: WebViewId,
    ) -> Result<ControllerEvent, ControllerError> {
        let tab = self
            .registry
            .tab_for_view(view)
            .ok_or(ControllerError::UnknownWebView(view))?;
        if !self.responsiveness.is_unresponsive(view) {
            return Err(ControllerError::NotUnresponsive(view));
        }
        let profile = self
            .session_profiles
            .get(&view)
            .copied()
            .ok_or(ControllerError::NoSession(tab))?;
        let url = self
            .browser
            .tab(tab)
            .map(|state| state.url.clone())
            .unwrap_or_default();
        // Destroy the hung session (its availability record is forgotten)
        // and recreate a fresh one in the same profile store.
        self.destroy_session_of(tab);
        let fresh = self.create_session(tab, profile, &url);
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::SessionRecovered { view, fresh, tab })
    }

    /// The testkit `ProbeResponsiveness` seam: force exactly one immediate
    /// liveness probe for one live view through the production monitor path
    /// (the packet's "refresh the adapter's bounded liveness probe for one
    /// live view"). Unknown/destroyed ids are no-ops.
    pub fn probe_view(&mut self, view: WebViewId) -> bool {
        if self.registry.tab_for_view(view).is_none() {
            return false;
        }
        let mut arm = |view: WebViewId| {
            self.engine.borrow_mut().probe_responsiveness(view);
        };
        self.responsiveness.arm_now(view, &mut arm)
    }

    /// The availability state of one live session (the snapshot and the
    /// recovery surface consume this).
    pub fn renderer_state(&self, view: WebViewId) -> RendererAvailability {
        self.responsiveness.state(view)
    }
}
