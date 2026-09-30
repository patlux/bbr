//! Productive new-window request execution.
//!
//! Native WebKit and typed testkit injection both enter
//! [`BbrApp::on_new_window_request`]. Portable policy already decided the
//! request; this module validates/executes the plain outcome through the one
//! browser controller session authority and records typed diagnostics.

use bbr_web::new_window::{NewWindowDelivery, NewWindowOutcome};
use gpui::{Context, Window};

use super::BbrApp;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct NewWindowState {
    pub(crate) requests: u64,
    pub(crate) created_tabs: u64,
    pub(crate) created_background_tabs: u64,
    pub(crate) blocked_reported: u64,
}

impl BbrApp {
    /// Execute one fully decided request from the native coordinator or
    /// testkit production-seam injection.
    pub fn on_new_window_request(
        &mut self,
        delivery: NewWindowDelivery,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        self.new_window.get().requests += 1;
        match delivery.outcome {
            NewWindowOutcome::CreateSession { session } => {
                match session.disposition() {
                    bbr_web::new_window::NewWindowDisposition::Preview => {
                        // Transient Peek (Link Preview / Peek packet): the
                        // controller opens a source-profile preview session
                        // above the warm origin; no regular tab is created.
                        // The preview owns browser-command focus while open.
                        let result = self
                            .controller
                            .get()
                            .open_preview_from_new_window(delivery.request.source, &session)
                            .map_err(|error| {
                                log::warn!("peek request rejected: {error}");
                                error
                            })
                            .ok();
                        if let Some(outcome) = &result {
                            self.clear_command_profile_hint();
                            self.find_discard_if_unbound(cx);
                            self.handle_peek_outcome(outcome, window, cx);
                            cx.notify();
                        }
                        result
                    }
                    bbr_web::new_window::NewWindowDisposition::OpenTab => {
                        let result = self
                            .controller
                            .get()
                            .open_tab_from_new_window(delivery.request.source, &session)
                            .map_err(|error| {
                                log::warn!("new-window request rejected: {error}");
                                error
                            })
                            .ok();
                        if let Some(outcome) = &result {
                            self.new_window.get().created_tabs += 1;
                            self.clear_command_profile_hint();
                            self.sync_command_bar();
                            self.find_discard_if_unbound(cx);
                            self.emit_browser(outcome);
                            self.ensure_active_visible();
                            self.sync_error_surface_visibility();
                            // The new regular tab may be reported
                            // unresponsive: re-assert the §11.8a pane
                            // visibility.
                            self.sync_unresponsive_visibility();
                            self.zoom_after_activation();
                            self.persist_structure();
                            cx.notify();
                        }
                        result
                    }
                    bbr_web::new_window::NewWindowDisposition::OpenBackgroundTab => {
                        // Command-primary / middle-click background link
                        // tabs: one regular child opens in the background
                        // immediately after the exact opener. The opener
                        // stays selected, visible, and web-focused, so this
                        // path must NOT zoom, re-assert active visibility,
                        // or touch the unresponsive surface of the active
                        // pane — the hidden child is a warm session and the
                        // selection never moved.
                        let result = self
                            .controller
                            .get()
                            .open_background_tab_from_new_window(delivery.request.source, &session)
                            .map_err(|error| {
                                log::warn!("background new-window request rejected: {error}");
                                error
                            })
                            .ok();
                        if let Some(outcome) = &result {
                            self.new_window.get().created_tabs += 1;
                            self.new_window.get().created_background_tabs += 1;
                            self.clear_command_profile_hint();
                            self.find_discard_if_unbound(cx);
                            self.emit_browser(outcome);
                            self.persist_structure();
                            cx.notify();
                        }
                        result
                    }
                }
            }
            NewWindowOutcome::Blocked => {
                self.new_window.get().blocked_reported += 1;
                // Blueprint D13: say so, with Open and Always Allow.
                self.popup_blocked(delivery.request.clone(), cx);
                log::info!(
                    "blocked script-created new-window request from {:?}",
                    delivery.request.source
                );
                cx.notify();
                None
            }
            NewWindowOutcome::External { url } => {
                // The MVP default policy never selects External. Preserve one
                // explicit outcome without duplicating the external-URL input
                // router or creating a web session.
                log::warn!("new-window external outcome is not enabled for MVP: {url}");
                None
            }
        }
    }

    #[cfg(feature = "testkit")]
    pub(crate) fn new_window_state(&self) -> NewWindowState {
        self.new_window.value()
    }
}
