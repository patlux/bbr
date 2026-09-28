//! Transient Peek preview app composition (Link Preview / Peek packet).
//!
//! The controller owns the transient preview lifecycle (session creation,
//! one-preview-per-pane replacement, dismissal, no-reload promotion); this
//! module owns the app's presentation and focus choreography around it:
//!
//! - Presenting the Peek bar (a `NativeAboveWeb` portal surface) and routing
//!   browser-command focus to the preview web view.
//! - Deterministic dismissal (Escape / close control) with exact focus
//!   restoration to the warm origin.
//! - "Open as Tab" promotion routed through the controller's same-session
//!   adoption.
//! - Staleness/teardown: whenever a controller mutation ends the preview
//!   (origin removal, space switch, split change, window close), the app
//!   dismisses the portal surface without forcing focus.
//!
//! The preview itself is never a persisted `Tab`, recently-closed row,
//! sidebar row, or history item: it exists only while the controller holds
//! it and dies with its session.

use bbr_web::{WebAction, WebViewId};
use gpui::{Context, Window};

use crate::portal::geometry::{Edges, FitMode, PortalAnchor, Pt, Sz};
use crate::portal::state::{AnchoredPlacement, DismissalPolicy, PortalPlacement, PortalRequest};
use bbr_ui::theme::Theme;

use super::BbrApp;

/// The one transient Peek presentation id (portal identity; re-presenting it
/// refreshes the active bar in place).
pub const PEEK_PRESENTATION_ID: crate::portal::state::PresentationId =
    crate::portal::state::PresentationId(0x70);

/// The Peek bar's draw priority (the deferred/native host ordering slot;
/// the find bar uses the same slot family).
pub const PEEK_PORTAL_PRIORITY: usize = 1;

/// Stable §15 ids of the Peek preview surface (DESIGN.md §15).
pub const PEEK_BAR_ID: &str = "bbr-peek-bar";
pub const PEEK_URL_ID: &str = "bbr-peek-url";
pub const PEEK_STATE_ID: &str = "bbr-peek-state";
pub const PEEK_BACK_ID: &str = "bbr-peek-back";
pub const PEEK_FORWARD_ID: &str = "bbr-peek-forward";
pub const PEEK_RELOAD_ID: &str = "bbr-peek-reload";
pub const PEEK_OPEN_AS_TAB_ID: &str = "bbr-peek-open-as-tab";
pub const PEEK_CLOSE_ID: &str = "bbr-peek-close";

/// App-owned Peek presentation state. The preview *session* state lives in
/// the controller snapshot (`preview`); this is only the presentation/
/// focus-restore bookkeeping the app must own.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PeekState {
    /// The Peek bar portal is currently presented.
    pub presented: bool,
    /// The warm origin session the preview covered: captured at open so an
    /// explicit dismissal can restore focus to the exact origin control/web
    /// view even after the transient state is released.
    pub source_view: Option<WebViewId>,
    /// Blueprint D09: the content card's inner box of the last main-window
    /// frame while a preview is open (the portal washes it around the inset
    /// preview); `None` at rest.
    pub card: Option<([f32; 4], f32)>,
}

/// The read-only Peek surface model the portal bar renders (derived each
/// render from the controller snapshot's live preview state).
#[derive(Debug, Clone, PartialEq)]
pub struct PeekSurface {
    pub id: u64,
    pub view: WebViewId,
    pub source: WebViewId,
    pub profile: bbr_web::ProfileId,
    pub url: String,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub crashed: bool,
    /// Blueprint D09: the bar/preview/card geometry of the last main-window
    /// frame (`None` until the card has been laid out with the preview).
    pub frame: Option<bbr_ui::card::PeekFrame>,
}

impl BbrApp {
    /// Whether the Peek bar is currently presented.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // testkit snapshot + probe only; production callers arrive with the peek controls
    pub fn peek_presented(&self) -> bool {
        self.peek.presented
    }

    /// The live Peek surface model, when a preview is open.
    pub fn peek_surface(&self) -> Option<PeekSurface> {
        let preview = self.controller.snapshot().preview?;
        Some(PeekSurface {
            id: preview.id,
            view: preview.view,
            source: preview.source,
            profile: preview.profile,
            url: preview.url,
            title: preview.title.clone(),
            loading: preview.loading,
            can_go_back: preview.can_go_back,
            can_go_forward: preview.can_go_forward,
            crashed: preview.crashed,
            frame: self.peek.card.map(|(card, radius)| {
                bbr_ui::card::PeekFrame::resolve(
                    card,
                    radius,
                    f32::from(self.theme().density.row()),
                )
            }),
        })
    }

    /// Record the card's inner box for the portal's wash while a preview is
    /// open (blueprint D09). A changed box re-notifies so the portal redraws
    /// from this frame's geometry, never a stale one.
    pub(super) fn record_peek_card(
        &mut self,
        card: bbr_ui::card::CardInsets,
        rail_width: f32,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let inner = self.controller.preview_open().then(|| {
            let size = window.viewport_size();
            let rect = card.inner_rect([f32::from(size.width), f32::from(size.height)], rail_width);
            (rect, card.radius)
        });
        if self.peek.card != inner {
            self.peek.card = inner;
            if inner.is_some() {
                cx.notify();
            }
        }
    }

    /// Present (or refresh) the Peek bar over the web content area and route
    /// native keyboard focus to the preview session. Called when the
    /// controller reports `PreviewOpened`/`PreviewNavigated`.
    pub(super) fn present_peek(
        &mut self,
        view: WebViewId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let theme = self.theme();
        let viewport = Sz::new(
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        );
        let request = PortalRequest::native(
            PEEK_PRESENTATION_ID,
            peek_placement(viewport, &theme, self.sidebar_visible_width(window)),
            PEEK_PORTAL_PRIORITY,
            DismissalPolicy::none(),
        );
        self.portal_probe
            .borrow_mut()
            .present_peek(request, viewport);
        self.activate_portal_request(request, cx);
        self.peek.presented = true;
        // Route native keyboard focus to the preview session (the preview
        // owns browser-command focus while open; the covered origin stays
        // warm and receives no input).
        self.focus_web_view(view, window, cx);
        cx.notify();
    }

    /// The current sidebar width in points (0 when collapsed).
    fn sidebar_visible_width(&self, window: &Window) -> f32 {
        let window_width = f32::from(window.bounds().size.width);
        if self.sidebar_visible {
            self.sidebar.effective_width(window_width)
        } else {
            0.0
        }
    }

    /// Production dismissal path (Escape / the close control): destroy the
    /// preview session through the controller and restore focus to the exact
    /// warm origin (when the origin is still the active view).
    pub fn peek_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(outcome) = self
            .controller
            .close_preview()
            .map_err(|error| log::warn!("peek close rejected: {error}"))
            .ok()
        else {
            return false;
        };
        self.handle_peek_outcome(&outcome, window, cx);
        cx.notify();
        true
    }

    /// Production promotion path ("Open as Tab"): adopt the SAME preview
    /// session as a regular tab in the source space/profile without reload
    /// or state loss. A crashed preview refuses (the controller rejects).
    pub fn peek_promote(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(outcome) = self
            .controller
            .promote_preview()
            .map_err(|error| log::warn!("peek promote rejected: {error}"))
            .ok()
        else {
            return false;
        };
        self.handle_peek_outcome(&outcome, window, cx);
        cx.notify();
        true
    }

    /// Route one browser navigation action to the preview session (it owns
    /// browser-command focus while open). Back/Forward/Reload/Stop/Navigate.
    pub(crate) fn peek_action(&mut self, action: WebAction) {
        let _ = self.controller.preview_action(action);
    }

    /// Handle one controller outcome that names the transient preview:
    /// presents/refreshes the bar, promotes, or dismisses with focus
    /// restoration. Returns `true` when the outcome was a preview event.
    pub(super) fn handle_peek_outcome(
        &mut self,
        outcome: &bbr_browser::ControllerEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match outcome {
            bbr_browser::ControllerEvent::PreviewOpened { view, .. } => {
                self.peek.source_view = self.controller.snapshot().preview.map(|p| p.source);
                self.present_peek(*view, window, cx);
                self.emit_browser(outcome);
                // The preview covers the origin view: the §11.8a pane is no
                // longer drawn over it, so a hidden origin is restored.
                self.sync_unresponsive_visibility();
                true
            }
            bbr_browser::ControllerEvent::PreviewNavigated { .. } => {
                // The bar derives live state each render; refresh it.
                if self.peek.presented {
                    cx.notify();
                }
                self.emit_browser(outcome);
                true
            }
            bbr_browser::ControllerEvent::PreviewClosed { .. } => {
                self.dismiss_peek_presentation(cx);
                self.peek.presented = false;
                // Exact focus restoration: the warm origin session was never
                // touched; hand native focus back to it when it is still the
                // active view (a staleness close already moved focus with the
                // mutation that ended the preview).
                let source = self.peek.source_view;
                self.peek.source_view = None;
                if let Some(source) = source {
                    if self.controller.snapshot().active_view == Some(source) {
                        self.focus_web_view(source, window, cx);
                    }
                }
                self.emit_browser(outcome);
                // The preview closed: the origin view is the pane again — a
                // still-unresponsive origin re-hides under the §11.8a pane.
                self.sync_unresponsive_visibility();
                true
            }
            bbr_browser::ControllerEvent::PreviewPromoted { view, .. } => {
                self.dismiss_peek_presentation(cx);
                self.peek.presented = false;
                self.peek.source_view = None;
                self.clear_command_profile_hint();
                self.sync_command_bar();
                self.ensure_active_visible();
                self.persist_structure();
                self.focus_web_view(*view, window, cx);
                self.emit_browser(outcome);
                // The promoted session is the active view now: re-assert the
                // §11.8a pane visibility.
                self.sync_unresponsive_visibility();
                true
            }
            _ => false,
        }
    }

    /// Teardown reconciliation after any controller mutation that can end
    /// the preview (origin removal, space switch, split change, new tab):
    /// when the controller no longer holds a preview but the bar is still
    /// presented, dismiss it. Focus is NOT forced here — the mutation that
    /// ended the preview already routed focus deterministically.
    pub(super) fn sync_peek_surface(&mut self, cx: &mut Context<Self>) {
        if !self.peek.presented {
            return;
        }
        if self.controller.preview_open() {
            // Still open (e.g. a same-space tab list refresh): refresh.
            cx.notify();
            return;
        }
        self.dismiss_peek_presentation(cx);
        self.peek.presented = false;
        self.peek.source_view = None;
    }

    /// Dismiss the Peek bar portal presentation without touching the
    /// preview session (used by the teardown path; dismissal of the session
    /// itself goes through `peek_close`).
    fn dismiss_peek_presentation(&mut self, cx: &mut Context<Self>) {
        let _ = self.portal_dismiss(cx);
        self.portal_probe.borrow_mut().clear_peek();
        cx.notify();
    }

    /// Route native keyboard focus to `view` (a preview or a regular tab
    /// session): blur GPUI's focus tree and make the native web view first
    /// responder, exactly like the command-surface exit path.
    fn focus_web_view(&mut self, view: WebViewId, window: &mut Window, cx: &mut Context<Self>) {
        window.blur();
        self.engine.borrow_mut().focus_native_web(view);
        cx.notify();
        self.emit_focus(window);
    }
}

/// The Peek bar's placement: a full web-area-width bar anchored to the top
/// of the web content area (right of the sidebar), staying inside the
/// window margins on overflow.
fn peek_placement(viewport: Sz, theme: &Theme, sidebar_width: f32) -> PortalPlacement {
    let gap = f32::from(bbr_ui::theme::Spacing::sp2());
    // §11.17: the bar is `density:row` tall (blueprint G-8).
    let bar_height = f32::from(theme.density.row());
    let web_left = sidebar_width + gap;
    let web_width = (viewport.width - web_left - gap).max(1.0);
    PortalPlacement::Anchored(AnchoredPlacement {
        anchor_point: Pt::new(web_left, gap),
        anchor: PortalAnchor::TopLeft,
        size: Sz::new(web_width, bar_height),
        offset: Pt::new(0.0, 0.0),
        fit: FitMode::SnapToWindowWithMargin(Edges::all(gap)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal::state::PortalHost;

    #[test]
    fn peek_state_defaults_to_not_presented() {
        let state = PeekState::default();
        assert!(!state.presented);
        assert_eq!(state.source_view, None);
    }

    #[test]
    fn peek_placement_anchors_to_the_web_area_top() {
        let theme = Theme::dark();
        let placement = peek_placement(Sz::new(1200.0, 800.0), &theme, 260.0);
        match placement {
            PortalPlacement::Anchored(placement) => {
                assert_eq!(placement.anchor, PortalAnchor::TopLeft);
                assert!(placement.size.width > 0.0);
                assert!(placement.anchor_point.x > 260.0);
            }
            other => panic!("expected anchored placement, got {other:?}"),
        }
    }

    #[test]
    fn peek_uses_the_native_above_web_host() {
        let theme = Theme::dark();
        let request = PortalRequest::native(
            PEEK_PRESENTATION_ID,
            peek_placement(Sz::new(1200.0, 800.0), &theme, 0.0),
            PEEK_PORTAL_PRIORITY,
            DismissalPolicy::none(),
        );
        assert_eq!(request.host, PortalHost::NativeAboveWeb);
    }
}
