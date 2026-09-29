//! §11.8a unresponsive-pane app composition (renderer-isolation lane).
//!
//! The portable controller owns the renderer-availability policy (the
//! bounded liveness probes, the fail-closed deadline, the closed recovery
//! set Wait / Reload / Close). This module owns the app's presentation side
//! only:
//!
//! - the single render/visibility predicate (`unresponsive_pane_over`): the
//!   pane replaces the **exact active view's** native slot — never a
//!   transient Peek preview (the preview is not the active tab's view) and
//!   never while the Library or the classified error surface is drawn;
//! - the native-view visibility sync: while the pane is drawn the hung
//!   renderer's stale frame is hidden (the WKWebView is a platform subview
//!   above the GPUI content layer); identity is untouched and the view is
//!   restored as soon as the pane clears;
//! - the recovery actions (Wait / Reload / Close) routing through the sole
//!   controller authority, and the bounded cadence pump that arms due
//!   probes on the main thread without ever blocking on the renderer.
//!
//! Focus contract: the pane's Wait → Reload → Close tab buttons own focus
//! handles in tree order; when the pane appears and the user is not
//! operating chrome, the Wait button takes focus (the hung renderer owns no
//! usable focus target).

use std::collections::HashSet;

use bbr_browser::RendererAvailability;
use bbr_web::WebViewId;
use gpui::{Context, Window};

use super::BbrApp;

/// App-side bookkeeping for the §11.8a pane: the native views hidden while
/// their pane is drawn (identity untouched; restored when the pane clears).
#[derive(Default)]
pub(crate) struct UnresponsiveState {
    hidden: HashSet<WebViewId>,
}

impl UnresponsiveState {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl BbrApp {
    /// Whether the §11.8a pane is drawn over `view` (the exact active
    /// view's native slot) — the single predicate the render and the
    /// visibility sync share, so the drawn surface and the hidden native
    /// view can never disagree:
    /// - the Library wins while open (it is the single visibility
    ///   authority);
    /// - the classified error surface wins over the pane;
    /// - the pane replaces only the active tab's view, and only when that
    ///   view is the pane layout's rendered slot (a transient Peek preview
    ///   covers the origin view, so the pane is not drawn over it);
    /// - the session stays alive: `Unresponsive` is a sibling of the crash
    ///   authority, never a destroyed session.
    pub(crate) fn unresponsive_pane_over(&self, view: WebViewId) -> bool {
        if self.library.open {
            return false;
        }
        if self.active_error_surface_spec().is_some() {
            return false;
        }
        let snap = self.controller.snapshot();
        snap.active_view == Some(view)
            && self.controller.renderer_state(view) == RendererAvailability::Unresponsive
            && self
                .controller
                .pane_layout()
                .is_some_and(|layout| layout.primary == view || layout.secondary == Some(view))
    }

    /// Re-assert native-view visibility against the §11.8a pane: while the
    /// pane is drawn over the active view, the hung renderer's WKWebView is
    /// hidden (it is a platform subview above the GPUI content layer and
    /// would otherwise cover the pane with its last frame); once the pane
    /// clears — a completing probe, a recovery, or a different active
    /// view — a view we hid is restored. Never destroys or recreates the
    /// session. Called from every mutation that can change the pane (engine
    /// events, tab actions, the cadence pump, surface lifecycle), so the
    /// visible state never lags the drawn surface.
    pub(crate) fn sync_unresponsive_visibility(&mut self) {
        let Some(view) = self.controller.snapshot().active_view else {
            return;
        };
        let error_hides = self.error_surface.spec_for(view).is_some_and(|spec| {
            spec.incident.reason.category() != bbr_web::web_errors::IncidentCategory::ProcessCrash
        });
        if self.unresponsive_pane_over(view) {
            // Always re-assert the hide (idempotent): the controller's
            // visibility sync re-shows the view when the user switches back
            // to a hung tab, so the stale frame can never cover the pane.
            self.engine.borrow_mut().show(view, false);
            self.unresponsive.hidden.insert(view);
        } else if self.unresponsive.hidden.remove(&view) {
            // Restore only when no other surface is the current hider (the
            // error surface and the Library win while drawn; the three
            // hidden trackers never fight).
            if !error_hides && !self.library.open {
                self.engine.borrow_mut().show(view, true);
            }
        }
    }

    /// Drop the pane bookkeeping for a destroyed view (a closed tab or a
    /// recovery-rebuilt session): the hidden tracker must not outlive the
    /// session it names.
    pub(crate) fn unresponsive_forget_view(&mut self, view: WebViewId) {
        self.unresponsive.hidden.remove(&view);
    }

    /// The Wait recovery action (§11.8a): keep monitoring the unresponsive
    /// session — the state stays `Unresponsive` until a probe answers (a
    /// busy→alive page clears it). Routes through the sole controller
    /// authority.
    pub(crate) fn wait_unresponsive_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(view) = self.controller.snapshot().active_view else {
            return;
        };
        let _outcome = self.controller.wait_unresponsive(view);
        self.sync_unresponsive_visibility();
        cx.notify();
        self.emit_focus(window);
    }

    /// The Reload recovery action (§11.8a): destroy the hung session and
    /// recreate a **fresh** one for the same tab in its owning profile
    /// store (fresh `WebViewId`, same tab identity). A hung renderer may
    /// never honour `Reload`, so recovery routes through the controller's
    /// destroy → recreate seam, never `WebAction::Reload`. Focus returns to
    /// the recovered page.
    pub(crate) fn reload_unresponsive_active(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(view) = self.controller.snapshot().active_view else {
            return;
        };
        match self.controller.recover_unresponsive(view) {
            Ok(outcome) => {
                // The hung session is gone: its pane bookkeeping dies with
                // it. The fresh session is responsive and shown.
                self.unresponsive_forget_view(view);
                if let bbr_browser::ControllerEvent::SessionRecovered { .. } = &outcome {
                    self.sync_error_surface_visibility();
                    self.sync_unresponsive_visibility();
                }
                self.zoom_after_activation();
                self.persist_structure();
                self.sync_peek_surface(cx);
                cx.notify();
                self.focus_web(window, cx);
            }
            Err(error) => {
                log::error!("unresponsive reload rejected: {error}");
            }
        }
    }

    /// The bounded cadence tick (production pump): arm due liveness probes
    /// for the visible sessions through the controller (never blocking,
    /// never waiting on the renderer), apply the portable fail-closed
    /// deadline, and re-assert the pane visibility. Runs on the main
    /// thread on a fixed cadence; an idle window (no due probe) arms
    /// nothing and emits nothing.
    pub(crate) fn pump_responsiveness_cadence(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let transitions = self.controller.pump_responsiveness();
        // Blueprint D24: the media probe rides the same cadence.
        self.pump_media();
        let active_view = self.controller.snapshot().active_view;
        let became_unresponsive = transitions.iter().any(|event| {
            matches!(
                event,
                bbr_browser::ControllerEvent::RendererStateChanged {
                    view,
                    state: RendererAvailability::Unresponsive,
                } if active_view == Some(*view)
            )
        });
        self.sync_unresponsive_visibility();
        if became_unresponsive && !self.chrome_focused(window) {
            // The hung renderer owns no usable focus target: move focus to
            // the pane's first recovery control (Wait). Chrome focus is
            // left untouched so the user keeps operating the sidebar /
            // command bar while the active page hangs.
            window.focus(&self.unresponsive_wait_handle, cx);
            self.emit_focus(window);
        }
        cx.notify();
    }
}
