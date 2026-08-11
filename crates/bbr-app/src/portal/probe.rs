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
    div, prelude::*, px, transparent_black, App, Context, Div, Entity, MouseButton, MouseDownEvent,
    Render, Window,
};

use super::geometry::{Edges, Rect, Sz};
use super::state::{
    DismissalPolicy, PortalHost, PortalPlacement, PortalPresentation, PortalRequest,
};
use bbr_ui::surfaces::auth::{
    AUTH_CANCEL_ID, AUTH_HEADER_ID, AUTH_PASSWORD_ID, AUTH_SHEET_ID, AUTH_SUBMIT_ID,
    AUTH_USERNAME_ID,
};
use bbr_ui::surfaces::menu::{context_menu_card, ContextMenuRect};
use bbr_ui::theme::{Spacing, Theme};

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

/// Which GPUI content the one native portal window renders.
#[derive(Debug, Clone)]
pub enum PortalContent {
    Probe,
    Command(crate::app::palette::CommandSurface),
    /// The find-in-page bar (task packet `find-in-page-portal-bar`). The
    /// portal render derives the live surface from the app entity each
    /// frame, so the bar carries no data of its own.
    Find,
    /// The BBR context menu over web content (task packet
    /// `bbr-context-menus-over-web`). The portal render derives the live
    /// surface from the app entity each frame, so the menu carries no data
    /// of its own.
    ContextMenu,
    /// The conventional macOS credential sheet (queue item 10). The portal
    /// render derives the redacted surface from the app entity each frame;
    /// the raw credential never exists in any serializable shape.
    Auth,
    /// The transient Peek preview bar (Link Preview / Peek packet). The
    /// portal render derives the live surface from the app entity each
    /// frame, so the bar carries no data of its own.
    Peek,
}

/// The portal window's render state, shared between the app (present/dismiss)
/// and the portal window root (render). Plain data; the portal window renders
/// from exactly what the app commanded.
#[derive(Debug, Clone)]
pub struct PortalProbeState {
    /// The presentation the portal window should render; `None` when hidden.
    request: Option<PortalRequest>,
    content: Option<PortalContent>,
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
            content: None,
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

    pub fn content(&self) -> Option<&PortalContent> {
        self.content.as_ref()
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
        match self.content.as_ref() {
            None => Vec::new(),
            Some(PortalContent::Probe) => vec![
                PROBE_SURFACE_ID.into(),
                PROBE_BACKDROP_ID.into(),
                PROBE_CARD_ID.into(),
            ],
            Some(PortalContent::Find) => vec![
                bbr_ui::surfaces::find::FIND_BAR_ID.into(),
                bbr_ui::surfaces::find::FIND_FIELD_ID.into(),
                bbr_ui::surfaces::find::FIND_TEXT_ID.into(),
                bbr_ui::surfaces::find::FIND_COUNT_ID.into(),
                bbr_ui::surfaces::find::FIND_PREVIOUS_ID.into(),
                bbr_ui::surfaces::find::FIND_NEXT_ID.into(),
                bbr_ui::surfaces::find::FIND_CLOSE_ID.into(),
            ],
            Some(PortalContent::Auth) => vec![
                AUTH_SHEET_ID.into(),
                AUTH_HEADER_ID.into(),
                AUTH_USERNAME_ID.into(),
                AUTH_PASSWORD_ID.into(),
                AUTH_SUBMIT_ID.into(),
                AUTH_CANCEL_ID.into(),
            ],
            Some(PortalContent::Peek) => vec![
                crate::app::peek::PEEK_BAR_ID.into(),
                crate::app::peek::PEEK_URL_ID.into(),
                crate::app::peek::PEEK_STATE_ID.into(),
                crate::app::peek::PEEK_BACK_ID.into(),
                crate::app::peek::PEEK_FORWARD_ID.into(),
                crate::app::peek::PEEK_RELOAD_ID.into(),
                crate::app::peek::PEEK_OPEN_AS_TAB_ID.into(),
                crate::app::peek::PEEK_CLOSE_ID.into(),
            ],
            Some(PortalContent::Command(surface)) => vec![match surface {
                crate::app::palette::CommandSurface::Palette => {
                    crate::app::palette::PALETTE_ID.into()
                }
                crate::app::palette::CommandSurface::Overlay => {
                    crate::app::palette::OVERLAY_ID.into()
                }
                crate::app::palette::CommandSurface::None => return Vec::new(),
            }],
            Some(PortalContent::ContextMenu) => {
                // The menu's stable surface ids: the backdrop (app-owned)
                // and the card + deterministic row ids (the five fixed
                // entries; the card ids moved to `bbr_ui::surfaces::menu`
                // with the card render).
                let mut ids = vec![
                    crate::app::context_menu::CONTEXT_MENU_BACKDROP_ID.into(),
                    bbr_ui::surfaces::menu::CONTEXT_MENU_ID.into(),
                ];
                for index in 0..5 {
                    ids.push(format!(
                        "{}-{index}",
                        bbr_ui::surfaces::menu::CONTEXT_MENU_ROW_PREFIX
                    ));
                }
                ids
            }
        }
    }

    /// Present (or refresh) the probe for `viewport`.
    pub fn present(&mut self, request: PortalRequest, viewport: Sz) {
        self.request = Some(request);
        self.content = Some(PortalContent::Probe);
        self.viewport = viewport;
    }

    pub fn present_command(
        &mut self,
        request: PortalRequest,
        viewport: Sz,
        surface: crate::app::palette::CommandSurface,
    ) {
        self.request = Some(request);
        self.content = Some(PortalContent::Command(surface));
        self.viewport = viewport;
    }

    /// Present the find-in-page bar (the render derives the live surface
    /// from the app entity, so no surface data is stored here).
    pub fn present_find(&mut self, request: PortalRequest, viewport: Sz) {
        self.request = Some(request);
        self.content = Some(PortalContent::Find);
        self.viewport = viewport;
    }

    /// Present the BBR context menu (the render derives the live surface
    /// from the app entity, so no surface data is stored here).
    pub fn present_context_menu(&mut self, request: PortalRequest, viewport: Sz) {
        self.request = Some(request);
        self.content = Some(PortalContent::ContextMenu);
        self.viewport = viewport;
    }

    /// Present the credential sheet (the render derives the redacted
    /// surface from the app entity each frame; the raw credential never
    /// enters this state).
    pub fn present_auth(&mut self, request: PortalRequest, viewport: Sz) {
        self.request = Some(request);
        self.content = Some(PortalContent::Auth);
        self.viewport = viewport;
    }

    /// Present the transient Peek preview bar (the render derives the live
    /// surface from the app entity, so no surface data is stored here).
    pub fn present_peek(&mut self, request: PortalRequest, viewport: Sz) {
        self.request = Some(request);
        self.content = Some(PortalContent::Peek);
        self.viewport = viewport;
    }

    /// Clear only when the active presentation is the Peek bar (a later
    /// surface owns the portal otherwise — never wipe another surface).
    pub fn clear_peek(&mut self) {
        if matches!(self.content, Some(PortalContent::Peek)) {
            self.request = None;
            self.content = None;
        }
    }

    /// Dismiss the active surface: the portal window renders nothing until
    /// the next presentation.
    pub fn clear(&mut self) {
        self.request = None;
        self.content = None;
    }
}

impl Default for PortalProbeState {
    fn default() -> Self {
        Self::new()
    }
}

/// The portal window's GPUI root. It observes the one browser-app entity and
/// re-renders U1 chrome from that authoritative state; the portal owns no
/// browser/session/profile/space lifetime.
pub struct PortalRoot {
    probe: Rc<RefCell<PortalProbeState>>,
    /// `pub(crate)`: the portal surface modules (command/find/context menu)
    /// route pointer actions through the same deferred app contract.
    pub(crate) browser_window: gpui::WindowHandle<gpui_component::Root>,
    /// `pub(crate)`: see `browser_window`.
    pub(crate) app: Option<Entity<crate::app::BbrApp>>,
    _app_subscription: gpui::Subscription,
}

impl PortalRoot {
    pub fn new(
        browser_window: gpui::WindowHandle<gpui_component::Root>,
        app: Entity<crate::app::BbrApp>,
        probe: Rc<RefCell<PortalProbeState>>,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscription = cx.observe(&app, |_root, _app, cx| cx.notify());
        Self {
            probe,
            browser_window,
            app: Some(app),
            _app_subscription: subscription,
        }
    }

    fn dismiss_command_surface(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.close_command_surface(window, cx)
            });
        });
    }

    /// The find bar's §13 one-pointer-action dismissal: a press on the
    /// portal backdrop closes the find presentation through the production
    /// close path (restores the origin focus domain, clears highlights).
    fn dismiss_find_surface(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.find_close(window, cx)
            });
        });
    }

    /// The find bar's previous-button pointer path (production app method).
    pub(super) fn find_previous_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                view.find_previous(cx)
            });
        });
    }

    /// The find bar's next-button pointer path (production app method).
    pub(super) fn find_next_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                view.find_next(cx)
            });
        });
    }

    /// The find bar's close-button pointer path (production close path).
    pub(super) fn find_close_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.find_close(window, cx)
            });
        });
    }

    /// The context menu's §13 one-pointer-action dismissal: a press on the
    /// portal backdrop closes the menu through the production close path
    /// (restores the origin focus domain).
    fn dismiss_context_menu_surface(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.menu_close(window, cx)
            });
        });
    }

    /// Row hover: the production `on_hover` listener path into the shared
    /// menu cursor (the exact app method a real pointer hover calls — the
    /// `bbr_ui::surfaces::menu` card routes its enabled-row hover
    /// listeners here).
    pub(crate) fn context_menu_row_hovered(&self, index: usize, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                view.menu_select(index, cx)
            });
        });
    }

    /// Row press: the production mouse-down listener path (select +
    /// activate + dismiss; the exact app method a real pointer press calls
    /// — the `bbr_ui::surfaces::menu` card routes its enabled-row press
    /// listeners here).
    pub(crate) fn context_menu_row_pressed(&self, index: usize, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.menu_activate(index, window, cx)
            });
        });
    }

    /// The credential sheet's Log In button path (the production submit
    /// seam the keyboard Enter routes to): the decision is applied through
    /// the merged auth contract by `BbrApp::auth_submit`.
    pub(super) fn auth_submit_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.auth_submit(window, cx)
            });
        });
    }

    /// The credential sheet's Cancel button path (the production cancel
    /// seam the keyboard Escape routes to): the challenge is cancelled
    /// through the contract — the request fails and never retries.
    pub(super) fn auth_cancel_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.auth_cancel(window, cx)
            });
        });
    }

    /// The Peek bar's close-control pointer path (the production dismissal
    /// seam: destroy the preview session and restore focus to the warm
    /// origin).
    pub(super) fn peek_close_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.peek_close(window, cx)
            });
        });
    }

    /// The Peek bar's "Open as Tab" pointer path (the production promotion
    /// seam: adopt the SAME preview session as a regular tab without reload).
    pub(super) fn peek_promote_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.peek_promote(window, cx)
            });
        });
    }

    /// The Peek bar's Back pointer path (routes to the preview session).
    pub(super) fn peek_back_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, _cx| {
                view.peek_action(bbr_web::WebAction::Back)
            });
        });
    }

    /// The Peek bar's Forward pointer path (routes to the preview session).
    pub(super) fn peek_forward_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, _cx| {
                view.peek_action(bbr_web::WebAction::Forward)
            });
        });
    }

    /// The Peek bar's Reload pointer path (routes to the preview session).
    pub(super) fn peek_reload_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, _cx| {
                view.peek_action(bbr_web::WebAction::Reload)
            });
        });
    }
}

impl Render for PortalRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let probe = self.probe.borrow();
        let Some(content) = probe.content().cloned() else {
            return div().size_full().bg(transparent_black()).into_any();
        };
        let Some(rect) = probe.resolved_rect() else {
            return div().size_full().bg(transparent_black()).into_any();
        };

        match content {
            PortalContent::Probe => {
                let request = probe.request().expect("visible probe request");
                drop(probe);
                probe_surface(self, request, rect, cx).into_any()
            }
            PortalContent::Command(surface) => {
                drop(probe);
                let Some(app) = self.app.as_ref() else {
                    return div().size_full().bg(transparent_black()).into_any();
                };
                let model = app.read(cx).command_portal_model(surface);
                #[cfg(feature = "testkit")]
                app.read(cx).observe_portal_command_render(surface);
                // The card carries its own §13 stop-propagation handler (it
                // is the surface, never an outside press); the backdrop
                // below owns the one-pointer-action dismissal.
                let card = crate::portal::command::command_card(&model, rect);
                div()
                    .id("bbr-command-portal-surface")
                    .size_full()
                    .bg(transparent_black())
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                            this.dismiss_command_surface(cx);
                        }),
                    )
                    .child(card)
                    .into_any()
            }
            PortalContent::Find => {
                drop(probe);
                let Some(app) = self.app.as_ref() else {
                    return div().size_full().bg(transparent_black()).into_any();
                };
                let theme = app.read(cx).theme();
                let surface = app.read(cx).find_surface();
                // The find-bar render milestone: record the find state this
                // render actually drew into the probe, so a pending find
                // benchmark can commit only from a frame that presented the
                // armed open/report identity (fast-but-wrong guard).
                #[cfg(feature = "testkit")]
                app.read(cx).observe_portal_find_render();
                // The production `bbr-ui` find bar: the portal root passes
                // the resolved rect and the `cx.listener` closures routing
                // the controls through the production app methods, exactly
                // like the pre-cutover portal render.
                let card = bbr_ui::surfaces::find::find_bar(
                    &theme,
                    &surface,
                    bbr_ui::surfaces::find::FindBarRect {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    },
                    cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                        this.find_previous_clicked(cx);
                    }),
                    cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                        this.find_next_clicked(cx);
                    }),
                    cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                        this.find_close_clicked(cx);
                    }),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|_this, _e: &MouseDownEvent, _window, cx| {
                        cx.stop_propagation();
                    }),
                );
                // §13 one-pointer-action dismissal: a press on the portal
                // backdrop (the whole window over the glass) closes the find
                // bar through the production close path.
                div()
                    .id("bbr-find-portal-surface")
                    .size_full()
                    .bg(transparent_black())
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                            this.dismiss_find_surface(cx);
                        }),
                    )
                    .child(card)
                    .into_any()
            }
            PortalContent::ContextMenu => {
                drop(probe);
                let Some(app) = self.app.as_ref() else {
                    return div().size_full().bg(transparent_black()).into_any();
                };
                let surface = app.read(cx).context_menu_surface();
                #[cfg(feature = "testkit")]
                app.read(cx).observe_portal_context_menu_render();
                // The production card render lives in `bbr_ui::surfaces::menu`
                // (G2 Group 9 extraction); this branch builds the DTO from
                // the app's live surface and routes the row listeners to the
                // exact `PortalRoot` methods a real pointer hover/press
                // calls (the same downgraded-handle seam `cx.listener`
                // uses).
                let portal_root = cx.entity().downgrade();
                let on_row_hover = {
                    let portal_root = portal_root.clone();
                    move |index: usize, _hovered: &bool, _window: &mut Window, cx: &mut App| {
                        if let Some(root) = portal_root.upgrade() {
                            root.update(cx, |this, cx| this.context_menu_row_hovered(index, cx));
                        }
                    }
                };
                let on_row_press = {
                    move |index: usize, _e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                        if let Some(root) = portal_root.upgrade() {
                            root.update(cx, |this, cx| this.context_menu_row_pressed(index, cx));
                        }
                    }
                };
                let card = context_menu_card(
                    &surface.theme,
                    &surface.model,
                    ContextMenuRect::new(rect.x, rect.y, rect.width, rect.height),
                    on_row_hover,
                    on_row_press,
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|_this, _e: &MouseDownEvent, _window, cx| {
                        cx.stop_propagation();
                    }),
                );
                div()
                    .id(crate::app::context_menu::CONTEXT_MENU_BACKDROP_ID)
                    .size_full()
                    .bg(transparent_black())
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                            this.dismiss_context_menu_surface(cx);
                        }),
                    )
                    .child(card)
                    .into_any()
            }
            PortalContent::Auth => {
                drop(probe);
                let Some(app) = self.app.as_ref() else {
                    return div().size_full().bg(transparent_black()).into_any();
                };
                let Some(surface) = app.read(cx).auth_surface() else {
                    return div().size_full().bg(transparent_black()).into_any();
                };
                #[cfg(feature = "testkit")]
                app.read(cx).observe_portal_auth_render(&surface);
                // The extracted `bbr_ui::surfaces::auth` presentation
                // (Group 8): the app builds the DTO from the redacted
                // surface (records.rs) and wires the production
                // submit/cancel seams; the portal owns the sheet geometry
                // (absolute rect) and the backdrop policy — no outside
                // dismissal for credentials.
                let model = surface.model();
                let card = div()
                    .absolute()
                    .left(px(rect.x))
                    .top(px(rect.y))
                    .w(px(rect.width))
                    .h(px(rect.height))
                    .child(bbr_ui::surfaces::auth::auth_sheet(
                        &model,
                        cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                            this.auth_submit_clicked(cx);
                        }),
                        cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                            this.auth_cancel_clicked(cx);
                        }),
                    ));
                div()
                    .id("bbr-auth-portal-surface")
                    .size_full()
                    .bg(transparent_black())
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _e: &MouseDownEvent, _window, cx| {
                            cx.stop_propagation();
                        }),
                    )
                    .child(card)
                    .into_any()
            }
            PortalContent::Peek => {
                drop(probe);
                let Some(app) = self.app.as_ref() else {
                    return div().size_full().bg(transparent_black()).into_any();
                };
                let Some(surface) = app.read(cx).peek_surface() else {
                    // The controller no longer holds a preview (teardown or
                    // staleness): render nothing; the app's teardown path
                    // dismisses the presentation deterministically.
                    return div().size_full().bg(transparent_black()).into_any();
                };
                // The Peek-bar render milestone: record the Peek state this
                // render actually drew into the probe, so a pending
                // `peek_request_to_present` benchmark can commit only from a
                // frame that presented the armed preview identity above its
                // warm origin (fast-but-wrong guard).
                #[cfg(feature = "testkit")]
                app.read(cx).observe_portal_peek_render();
                let card = crate::portal::peek::peek_bar(&surface, rect, cx);
                div()
                    .id("bbr-peek-portal-surface")
                    .size_full()
                    .bg(transparent_black())
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _e: &MouseDownEvent, _window, cx| {
                            cx.stop_propagation();
                        }),
                    )
                    .child(card)
                    .into_any()
            }
        }
    }
}

fn probe_surface(
    root: &PortalRoot,
    request: PortalRequest,
    rect: Rect,
    cx: &mut Context<PortalRoot>,
) -> gpui::Stateful<Div> {
    let theme = Theme::dark();
    let p = theme.palette;
    let app = root.app.clone();
    div()
        .id(PROBE_SURFACE_ID)
        .size_full()
        .bg(transparent_black())
        .child(
            div()
                .id(PROBE_BACKDROP_ID)
                .size_full()
                .bg(transparent_black())
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |_this, _e: &MouseDownEvent, _window, cx| {
                        if let Some(app) = &app {
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
    fn command_state_tracks_the_u1_presentation_and_clears() {
        let mut state = PortalProbeState::new();
        let request = PortalRequest::native(
            crate::app::palette::PALETTE_PRESENTATION_ID,
            PortalPlacement::Centered {
                size: Sz::new(480.0, 420.0),
                margin: Edges::all(8.0),
            },
            crate::app::palette::COMMAND_PORTAL_PRIORITY,
            DismissalPolicy::any(),
        );
        state.present_command(
            request,
            Sz::new(1080.0, 700.0),
            crate::app::palette::CommandSurface::Palette,
        );
        assert!(state.is_visible());
        assert_eq!(
            state.presentation(),
            Some(crate::app::palette::PALETTE_PRESENTATION_ID)
        );
        assert_eq!(state.host(), Some(PortalHost::NativeAboveWeb));
        assert_eq!(state.ids(), vec![crate::app::palette::PALETTE_ID]);
        assert_eq!(
            state.resolved_rect(),
            Some(Rect::new(Pt::new(300.0, 140.0), Sz::new(480.0, 420.0)))
        );

        state.clear();
        assert!(!state.is_visible());
        assert!(state.content().is_none());
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
