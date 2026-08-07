//! Production portal probe (task packet `native-portal-host`, step 3).
//!
//! One minimal presentation through the general Portal API that proves the
//! native host end to end **without any U1 palette content**: a centered
//! `NativeAboveWeb` card with the stable §15 ids `bbr-portal-probe`,
//! `bbr-portal-backdrop`, and `bbr-portal-card`. The card renders inside the
//! one native portal window ordered above the browser window's WKWebViews;
//! one outside pointer press on the backdrop drives the exact production
//! outside-dismiss method ([`crate::app::BbrApp::portal_outside_dismiss`]),
//! and a press on the card stops propagation and never dismisses.
//!
//! GPUI owns portal content and state; AppKit owns only native z-order,
//! parent attachment, frame synchronization, visibility, and hit-test
//! routing (see `super::native`). The probe content lives in the portal
//! window's GPUI root ([`PortalRoot`]); the app owns the shared probe state
//! ([`PortalProbeState`]) and drives it through the general
//! [`PortalRequest`] seam.
//!
//! ## Resize behavior (bounded for the probe)
//!
//! The native portal frame follows the parent content rect through the
//! AppKit observer (evidence: `frame_synced` in the snapshot). The probe
//! surface itself resolves its card from the viewport captured at present
//! time, so a parent resize while the probe is open re-pins the native frame
//! but keeps the card at its present-time position; re-resolving on resize
//! is owned by the real-app E2E step together with the resize evidence.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    div, prelude::*, px, transparent_black, Context, Entity, MouseButton, MouseDownEvent, Render,
    Window,
};

use super::geometry::{Edges, Rect, Sz};
use super::state::{
    DismissalPolicy, PortalHost, PortalPlacement, PortalPresentation, PortalRequest,
};
use crate::theme::{Spacing, Theme};

/// The one portal host per BBR browser window (allocated once at window
/// construction; stable for the window's lifetime, mirroring `WebViewId`
/// stability). The snapshot exposes the host id through the presentation.
pub const PORTAL_ID: super::state::PortalId = super::state::PortalId(0xB8);

/// The probe's stable presentation id: re-presenting the probe refreshes it
/// in place ([`super::state::PortalTransition::Updated`]) and the portal
/// state still enforces exactly one active presentation.
pub const PROBE_PRESENTATION_ID: super::state::PresentationId = super::state::PresentationId(0xB9);

/// The probe card's size in points (the DESIGN §11.4 N palette width, 480 pt
/// — deliberately the general centered placement, not U1 content).
pub const PROBE_SIZE: Sz = Sz::new(480.0, 360.0);

/// The probe's edge margin in points (DESIGN §5 spacing / §11.4 compact
/// clamp: wider-than-window probes pin to the margin, never to zero).
pub const PROBE_MARGIN: Edges = Edges::all(8.0);

/// The probe's draw priority (the deferred/native host ordering slot).
pub const PROBE_PRIORITY: usize = 1;

/// The stable §15 ids of the portal probe surface (DESIGN.md §15).
pub const PROBE_SURFACE_ID: &str = "bbr-portal-probe";
pub const PROBE_BACKDROP_ID: &str = "bbr-portal-backdrop";
pub const PROBE_CARD_ID: &str = "bbr-portal-card";

/// The general Portal request that opens the probe: centered, `NativeAboveWeb`,
/// dismissible by one outside pointer action (DESIGN §13).
pub fn probe_request() -> PortalRequest {
    PortalRequest::native(
        PROBE_PRESENTATION_ID,
        PortalPlacement::Centered {
            size: PROBE_SIZE,
            margin: PROBE_MARGIN,
        },
        PROBE_PRIORITY,
        DismissalPolicy::outside_pointer(),
    )
}

/// The portal window's render state, shared between the app (present/dismiss)
/// and the portal window root (render). Plain data; the portal window renders
/// from exactly what the app commanded.
#[derive(Debug, Clone)]
pub struct PortalProbeState {
    /// The presentation the portal window should render; `None` when the
    /// probe is hidden/never shown.
    request: Option<PortalRequest>,
    /// The viewport (points) the request was resolved against at present
    /// time — the main window's content size, which equals the portal
    /// window's content size. Both the render and the testkit snapshot
    /// resolve from this same viewport, so they can never drift.
    viewport: Sz,
}

impl PortalProbeState {
    pub fn new() -> Self {
        Self {
            request: None,
            viewport: Sz::new(0.0, 0.0),
        }
    }

    /// Logical visibility: the probe surface is shown when the app
    /// presented a request and has not dismissed it.
    pub fn is_visible(&self) -> bool {
        self.request.is_some()
    }

    /// The presented request, if any.
    pub fn request(&self) -> Option<PortalRequest> {
        self.request
    }

    /// The active presentation's stable id, if any.
    pub fn presentation(&self) -> Option<super::state::PresentationId> {
        self.request.map(|request| request.presentation)
    }

    /// The active presentation's host, if any.
    pub fn host(&self) -> Option<PortalHost> {
        self.request.map(|request| request.host)
    }

    /// The resolved card rect in window coordinates (points, top-left
    /// origin), from the same viewport the render uses.
    pub fn resolved_rect(&self) -> Option<Rect> {
        self.request
            .map(|request| PortalPresentation::from(request).resolve(self.viewport, Edges::none()))
    }

    /// The stable `bbr-portal-*` ids the portal window renders while the
    /// probe is visible (DESIGN §15); empty while hidden.
    pub fn ids(&self) -> Vec<String> {
        if self.request.is_none() {
            return Vec::new();
        }
        vec![
            PROBE_SURFACE_ID.into(),
            PROBE_BACKDROP_ID.into(),
            PROBE_CARD_ID.into(),
        ]
    }

    /// Present (or refresh) the probe for `viewport`.
    pub fn present(&mut self, request: PortalRequest, viewport: Sz) {
        self.request = Some(request);
        self.viewport = viewport;
    }

    /// Dismiss the probe: the portal window renders nothing until the next
    /// presentation.
    pub fn clear(&mut self) {
        self.request = None;
    }
}

impl Default for PortalProbeState {
    fn default() -> Self {
        Self::new()
    }
}

/// The portal window's GPUI root: the probe surface.
///
/// Renders the transparent full-window backdrop with the centered card when
/// the probe is visible, and an empty transparent root otherwise. The
/// backdrop's pointer press drives the exact production outside-dismiss
/// method on the app entity; the card stops propagation so a press on the
/// card is never treated as an outside press.
pub struct PortalRoot {
    /// The probe surface state shared with the app entity.
    probe: Rc<RefCell<PortalProbeState>>,
    /// The app entity the backdrop routes its outside press through. `None`
    /// before the app seam attaches the bridge (or after teardown).
    app: Option<Entity<crate::app::BbrApp>>,
}

impl PortalRoot {
    /// Build the portal root for the probe. `app` is the retained browser
    /// app entity; `probe` is the shared surface state.
    pub fn new(app: Entity<crate::app::BbrApp>, probe: Rc<RefCell<PortalProbeState>>) -> Self {
        Self {
            probe,
            app: Some(app),
        }
    }
}

impl Render for PortalRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let probe = self.probe.borrow();
        let Some(request) = probe.request() else {
            // Hidden: an empty transparent root. No ids and no hitboxes —
            // a dismissed portal must not capture pointer events.
            return div().size_full().bg(transparent_black()).into_any();
        };
        let theme = Theme::dark();
        let p = theme.palette;
        // The card is resolved from the viewport captured at present time so
        // the render and the testkit snapshot always agree (see module docs
        // for the bounded resize behavior).
        let Some(rect) = probe.resolved_rect() else {
            return div().size_full().bg(transparent_black()).into_any();
        };
        drop(probe);

        let surface: gpui::AnyElement = div()
            .id(PROBE_SURFACE_ID)
            .size_full()
            .bg(transparent_black())
            .child(
                div()
                    .id(PROBE_BACKDROP_ID)
                    .size_full()
                    .bg(transparent_black())
                    // DESIGN §13 one-outside-pointer-action: an outside press
                    // on the backdrop runs the exact production outside-
                    // dismiss method — the same method the testkit
                    // `PortalOutsideDismiss` request drives.
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                            if let Some(app) = &this.app {
                                cx.update_entity(app, |view, cx| {
                                    view.portal_outside_dismiss(cx);
                                });
                            }
                        }),
                    )
                    .child(
                        div()
                            .id(PROBE_CARD_ID)
                            .absolute()
                            .left(px(rect.x))
                            .top(px(rect.y))
                            .w(px(rect.width))
                            .h(px(rect.height))
                            .rounded(theme.radius.large())
                            .bg(p.surface_overlay)
                            .border_1()
                            .border_color(p.hairline)
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(Spacing::sp2())
                            // The card stops propagation: a press on the card
                            // is never an outside press, so the backdrop's
                            // dismiss handler must not run.
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_this, _e: &MouseDownEvent, _window, cx| {
                                    cx.stop_propagation();
                                }),
                            )
                            .child(
                                div()
                                    .text_size(theme.type_.title.size)
                                    .text_color(p.ink)
                                    .child("Portal probe"),
                            )
                            .child(
                                div()
                                    .text_size(theme.type_.meta.size)
                                    .text_color(p.ink_faint)
                                    .child(format!(
                                        "{} · {}",
                                        request.presentation.0,
                                        request.host.label()
                                    )),
                            ),
                    ),
            )
            .into_any();
        surface
    }
}

#[cfg(test)]
mod tests {
    use super::super::geometry::Pt;
    use super::*;

    #[test]
    fn probe_state_starts_hidden_and_clears() {
        let mut state = PortalProbeState::new();
        assert!(!state.is_visible());
        assert_eq!(state.request(), None);
        assert_eq!(state.presentation(), None);
        assert_eq!(state.host(), None);
        assert_eq!(state.resolved_rect(), None);
        assert!(state.ids().is_empty());

        state.present(probe_request(), Sz::new(1080.0, 700.0));
        assert!(state.is_visible());
        assert_eq!(state.presentation(), Some(PROBE_PRESENTATION_ID));
        assert_eq!(state.host(), Some(PortalHost::NativeAboveWeb));
        assert_eq!(
            state.ids(),
            vec!["bbr-portal-probe", "bbr-portal-backdrop", "bbr-portal-card"]
        );

        state.clear();
        assert!(!state.is_visible());
        assert_eq!(state.resolved_rect(), None);
        assert!(state.ids().is_empty());
    }

    #[test]
    fn probe_state_resolves_the_centered_card() {
        let mut state = PortalProbeState::new();
        state.present(probe_request(), Sz::new(1080.0, 700.0));
        // (1080 - 480) / 2 = 300; (700 - 360) / 2 = 170.
        assert_eq!(
            state.resolved_rect(),
            Some(Rect::new(Pt::new(300.0, 170.0), PROBE_SIZE))
        );
    }

    #[test]
    fn probe_request_uses_the_general_native_portal_api() {
        let request = probe_request();
        assert_eq!(request.host, PortalHost::NativeAboveWeb);
        assert!(request.host.crosses_native_boundary());
        assert_eq!(request.presentation, PROBE_PRESENTATION_ID);
        assert_eq!(request.priority, PROBE_PRIORITY);
        assert!(request.dismissal.outside_pointer);
        assert!(!request.dismissal.escape);
        assert!(matches!(
            request.placement,
            PortalPlacement::Centered { size, margin }
                if size == PROBE_SIZE && margin == PROBE_MARGIN
        ));
    }

    #[test]
    fn probe_state_refreshes_in_place_with_new_viewport() {
        let mut state = PortalProbeState::new();
        state.present(probe_request(), Sz::new(1080.0, 700.0));
        state.present(probe_request(), Sz::new(640.0, 480.0));
        assert_eq!(state.presentation(), Some(PROBE_PRESENTATION_ID));
        assert_eq!(
            state.resolved_rect(),
            Some(Rect::new(Pt::new(80.0, 60.0), PROBE_SIZE)),
            "(640-480)/2=80; (480-360)/2=60"
        );
    }
}
