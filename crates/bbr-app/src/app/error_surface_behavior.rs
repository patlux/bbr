//! App wiring for the web-error surface (task packet `error-surface-contract`).
//!
//! The classified incidents arrive through the exact seam the native
//! delegate's incident sink drives ([`BbrApp::on_incident`], drained by the
//! main-thread pump like find/downloads events). Presenting an incident,
//! recovering, or dismissing never changes tab, session, profile, or space
//! identity: the incident lifecycle is scoped to the owning `WebViewId`, the
//! native web view is only ever hidden (never destroyed) while the surface
//! replaces it, and the renderer-crash class stays on the DESIGN.md §11.8
//! crash panel (the controller-driven `bbr-crash-*` surface whose Reload
//! recreates the session).

use bbr_web::web_errors::{IncidentCategory, RecoveryAction};
use bbr_web::WebAction;
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Attach the macOS error-surface composition: the concrete engine's
    /// classified-incident sink forwards every native failure (TLS,
    /// navigation, offline, renderer termination) into `incident_tx`, which
    /// the main-thread pump drains into [`Self::on_incident`]. Set once at
    /// composition, like the find/downloads adapters.
    #[cfg(target_os = "macos")]
    pub fn install_error_surface(
        &mut self,
        engine: crate::engine::NativeEngineRef,
        incident_tx: smol::channel::Sender<bbr_web_macos::error_mapping::ClassifiedIncident>,
    ) {
        engine.borrow_mut().set_incidents(Box::new(move |incident| {
            let _ = incident_tx.try_send(incident);
        }));
    }

    /// Present a classified web-error incident (the exact seam the native
    /// delegate's incident sink drives). The contract allocates a monotonic
    /// per-view incident id and supersedes any older incident on the same
    /// web view; showing the surface hides the native view (never destroys
    /// it — identity is untouched).
    #[cfg(target_os = "macos")]
    pub fn on_incident(
        &mut self,
        incident: bbr_web_macos::error_mapping::ClassifiedIncident,
        cx: &mut Context<Self>,
    ) {
        self.error_surface.present(
            incident.profile,
            incident.web_view,
            incident.reason,
            incident.context,
            incident.requested_url,
        );
        self.sync_error_surface_visibility();
        cx.notify();
    }

    /// The error surface's primary recovery action (the `bbr-error-action`
    /// button): requests the contract-legal action for the active view's
    /// incident and dispatches the engine action. For the visible classes
    /// (TLS/navigation/offline) the primary action is `Retry`, which
    /// re-issues the failing URL; the surface clears when the retried
    /// navigation starts (see `on_web_event`). The renderer-crash class is
    /// presented by the §11.8 crash panel, so its `Restore` never routes
    /// through this button in v1 (handled defensively anyway).
    pub fn error_recover(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(view) = self.controller.snapshot().active_view else {
            return;
        };
        let Some(spec) = self.error_surface.spec_for(view) else {
            return;
        };
        let action = spec.action;
        if self.error_surface.request_recovery(view, action).is_none() {
            return;
        }
        match action {
            RecoveryAction::Retry => {
                // Re-attempt the failed navigation to the same URL (the
                // app-side retry target captured from the engine report).
                if let Some(url) = self.error_surface.retry_url(view) {
                    let _ = self.controller.navigate(WebAction::Load { url });
                }
            }
            RecoveryAction::Restore => {
                // Defensive: the §11.8 crash panel owns the visible crash
                // surface, but a Restore request still recreates the session
                // through the same recovery the panel's Reload uses.
                self.reload_active_from_panel(_window, cx);
                return;
            }
            _ => {}
        }
        cx.notify();
    }

    /// Dismiss the error surface (the `bbr-error-dismiss` button): a
    /// lifecycle event — identity is never touched. The native view is
    /// restored (it was only hidden while the surface replaced it).
    pub fn error_dismiss(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(view) = self.controller.snapshot().active_view else {
            return;
        };
        if !self.error_surface.dismiss(view) {
            return;
        }
        self.sync_error_surface_visibility();
        cx.notify();
    }

    /// Re-assert native-view visibility against the surface: while the
    /// active view's incident surface is drawn, the native view stays hidden
    /// (the WKWebView is a platform subview above the GPUI content layer);
    /// once the surface clears, a view we hid is restored. Called after
    /// every incident presentation, recovery, dismissal, and controller
    /// mutation that re-syncs view visibility.
    pub(super) fn sync_error_surface_visibility(&mut self) {
        let Some(view) = self.controller.snapshot().active_view else {
            return;
        };
        let surface_visible = self
            .error_surface
            .spec_for(view)
            .is_some_and(|spec| spec.incident.reason.category() != IncidentCategory::ProcessCrash);
        if surface_visible {
            if !self.error_surface.is_hidden(view) {
                self.engine.borrow_mut().show(view, false);
                self.error_surface.mark_hidden(view);
            }
        } else if self.error_surface.take_hidden(view) {
            self.engine.borrow_mut().show(view, true);
        }
    }

    /// The surface's presentation spec for the active view, when the render
    /// should draw it (the crash class is excluded — the DESIGN.md §11.8
    /// crash panel is its surface).
    pub(super) fn active_error_surface_spec(
        &self,
    ) -> Option<super::error_surface::ErrorSurfaceSpec> {
        let view = self.controller.snapshot().active_view?;
        let spec = self.error_surface.spec_for(view)?;
        if spec.incident.reason.category() == IncidentCategory::ProcessCrash {
            return None;
        }
        Some(spec)
    }
}
