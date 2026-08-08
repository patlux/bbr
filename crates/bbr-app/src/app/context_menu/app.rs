//! App-side integration of the BBR context menu (task packet
//! `bbr-context-menus-over-web`).
//!
//! The `BbrApp` surface of the menu controller: the exact production
//! methods the portal render, the row element listeners, the root key-down
//! listener, the native bridge's `ContextMenuRequested` event, and the
//! testkit seams call. The deterministic row model, the placement geometry,
//! and the presentation state live in the parent module
//! (`super::ContextMenuController` etc.) — this module owns only the wiring
//! between the app and that domain.
//!
//! Split from `context_menu.rs` by responsibility (app integration vs the
//! pure controller/geometry domain) so both files stay under the ~500-line
//! budget.

use bbr_web::{WebAction, WebRect, WebViewId};
use gpui::{Context, Window};

use crate::app::BbrApp;
use crate::portal::geometry::Sz;
use crate::portal::state::{DismissalPolicy, PortalRequest};

use super::{
    context_menu_placement, ContextMenuController, ContextMenuState, ContextMenuSurface,
    CONTEXT_MENU_PRIORITY, MENU_BACK, MENU_COPY_LINK, MENU_FORWARD, MENU_OPEN_NEW_TAB, MENU_RELOAD,
};

impl BbrApp {
    /// The immutable menu data one portal-window render consumes (mirrors
    /// the find-surface pattern: the portal reads the app's live state per
    /// render, so selection and enablement can never drift from the app).
    pub fn context_menu_surface(&self) -> ContextMenuSurface {
        let state = &self.context_menu.state;
        ContextMenuSurface {
            theme: self.theme(),
            focused: state.open && state.focused,
            selected: state.selected,
            items: state.items.clone(),
        }
    }

    /// The exact production open path the native bridge's
    /// [`WebEvent::ContextMenuRequested`] routes to: present the BBR page
    /// menu anchored at the pointer (window content coordinates, top-left
    /// origin, points) over the **active** view. A request for any other
    /// view (a non-active split pane, a stale id) is a no-op: the menu is
    /// chrome over the active page and every action targets the active
    /// tab's session.
    ///
    /// Opening replaces any open find bar or command surface (exactly one
    /// active portal presentation per window), captures the origin focus
    /// domain for deterministic restoration, and leaves tab/session/
    /// profile/space identity untouched.
    pub fn menu_open_from_event(
        &mut self,
        view: WebViewId,
        point: WebRect,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.controller.snapshot().active_view != Some(view) {
            log::debug!("context menu requested for a non-active view {view:?}; ignored");
            return;
        }
        if self.context_menu.state.open {
            // Re-presenting the same presentation id refreshes it in place;
            // the origin was captured at the first open.
            self.present_context_menu(point, window, cx);
            cx.notify();
            return;
        }
        // Exactly one active portal presentation per window: a page
        // right-click replaces an open find bar or command surface.
        self.find_discard(cx);
        self.dismiss_command_portal(cx);
        let snapshot = self.controller.snapshot();
        let url = snapshot.nav.url.clone();
        let items = ContextMenuController::build_items(
            snapshot.nav.can_go_back,
            snapshot.nav.can_go_forward,
            url.as_deref(),
        );
        let selected = ContextMenuController::first_enabled(&items);
        self.context_menu.state = ContextMenuState {
            open: true,
            focused: false,
            selected,
            items,
            bound_view: Some(view),
            point: [point.x, point.y],
            url,
            origin_chrome_focused: self.chrome_focused(window),
        };
        self.present_context_menu(point, window, cx);
        self.menu_focus_field(window, cx);
        cx.notify();
    }

    /// Focus the menu (main-window GPUI focus; the portal window is
    /// non-activating and never key by design, so keystrokes are handled by
    /// the main window's root listener in menu mode).
    fn menu_focus_field(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.context_menu.state.focused = true;
        window.focus(&self.menu_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Pointer-hover/select path into the open menu: the exact app method
    /// the row's `on_hover` element listener calls. The keyboard cursor and
    /// the pointer hover share one selection. No-op when the menu is closed.
    pub fn menu_select(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.context_menu.state.open {
            return;
        }
        if self.context_menu.state.selected != Some(index) {
            self.context_menu.state.selected = Some(index);
            cx.notify();
        }
    }

    /// Pointer-press path into the open menu: the exact app method the
    /// row's mouse-down listener calls — select the row, activate it, and
    /// dismiss the menu with the origin focus restored. A press on a
    /// disabled row never activates; like a native menu it dismisses the
    /// menu (the row does not stop propagation, so the backdrop's
    /// one-pointer-action closes it).
    pub fn menu_activate(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.context_menu.state.open {
            return;
        }
        let enabled = self
            .context_menu
            .state
            .items
            .get(index)
            .is_some_and(|item| item.enabled);
        if !enabled {
            self.menu_close(window, cx);
            return;
        }
        self.context_menu.state.selected = Some(index);
        self.menu_activate_selected(window, cx);
    }

    /// Escape / backdrop-press / close-seam: dismiss the menu and restore
    /// the focus domain captured at open. No-op when no menu is open.
    pub fn menu_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.context_menu.state.open {
            return;
        }
        let origin_chrome_focused = self.context_menu.state.origin_chrome_focused;
        self.menu_discard(cx);
        if origin_chrome_focused {
            self.focus_command_bar(window, cx);
        } else {
            self.focus_web(window, cx);
        }
    }

    /// Close the menu WITHOUT touching focus (the invoking mutation — a
    /// command-surface entry or a find open — owns the focus choreography
    /// afterwards).
    pub(crate) fn menu_discard(&mut self, cx: &mut Context<Self>) {
        if !self.context_menu.state.open {
            return;
        }
        self.context_menu.state = ContextMenuState::default();
        self.portal_dismiss(cx);
        cx.notify();
    }

    /// Handle menu-mode keystrokes from the root key-down listener:
    /// Up/Down move the keyboard cursor skipping disabled rows, Enter
    /// activates the selected row, Escape dismisses.
    pub(crate) fn on_menu_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.context_menu.state.open {
            return;
        }
        let ks = &event.keystroke;
        match ks.key.as_str() {
            "escape" => {
                cx.stop_propagation();
                self.menu_close(window, cx);
            }
            "enter" => {
                cx.stop_propagation();
                self.menu_activate_selected(window, cx);
            }
            "up" | "down" => {
                cx.stop_propagation();
                let delta = if ks.key == "up" { -1 } else { 1 };
                if self.context_menu.menu_move_selection(delta) {
                    cx.notify();
                }
            }
            _ => {}
        }
        cx.notify();
    }

    /// Activate the selected row through its production seam and dismiss the
    /// menu with the origin focus restored. A disabled selection is a no-op
    /// (the traversal never lands there; the pointer path checks first).
    fn menu_activate_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.context_menu.state.selected else {
            return;
        };
        let Some(item) = self.context_menu.state.items.get(index).cloned() else {
            return;
        };
        if !item.enabled {
            return;
        }
        let bound_view = self.context_menu.state.bound_view;
        let url = self.context_menu.state.url.clone();
        match item.key {
            MENU_BACK => self.go_back(),
            MENU_FORWARD => self.go_forward(),
            MENU_RELOAD => self.reload(),
            MENU_COPY_LINK => {
                if let Some(view) = bound_view {
                    self.engine.borrow_mut().action(view, WebAction::CopyLink);
                }
            }
            MENU_OPEN_NEW_TAB => {
                if let Some(url) = url {
                    // Open exactly one new tab/session with the menu's page
                    // URL; the new tab becomes active (the canonical
                    // production open path).
                    let _ = self.open_tab(&url, cx);
                }
            }
            _ => {}
        }
        self.menu_close(window, cx);
    }

    /// Present the menu through the one native portal host, anchored at the
    /// pointer and snapped fully on screen (the pure geometry clamp).
    fn present_context_menu(
        &mut self,
        point: WebRect,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let viewport = Sz::new(
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        );
        let theme = self.theme();
        let request = PortalRequest::native(
            super::CONTEXT_MENU_PRESENTATION_ID,
            context_menu_placement(point, &theme),
            CONTEXT_MENU_PRIORITY,
            DismissalPolicy::any(),
        );
        self.portal_probe
            .borrow_mut()
            .present_context_menu(request, viewport);
        self.activate_portal_request(request, cx);
    }
    /// The typed E2E snapshot state of the menu (wire mirror + the native
    /// bridge's ownership diagnostics).
    #[cfg(feature = "testkit")]
    pub fn context_menu_snapshot(&self) -> bbr_testkit::protocol::ContextMenuSnapshot {
        let state = &self.context_menu.state;
        bbr_testkit::protocol::ContextMenuSnapshot {
            open: state.open,
            focused: state.focused,
            selected: state.selected,
            items: state
                .items
                .iter()
                .map(|item| bbr_testkit::protocol::ContextMenuItemSnapshot {
                    key: item.key.to_string(),
                    title: item.title.to_string(),
                    enabled: item.enabled,
                })
                .collect(),
            bound_view: state.bound_view,
            point: state.point,
            url: state.url.clone(),
            origin_chrome_focused: state.origin_chrome_focused,
            native: self.native_context_menu_diagnostics(),
        }
    }

    /// The native context-menu bridge diagnostics (macOS; the engine handle
    /// is attached by the composition root). Empty defaults when no bridge
    /// is attached or on non-macOS hosts.
    #[cfg(feature = "testkit")]
    fn native_context_menu_diagnostics(&self) -> bbr_testkit::protocol::ContextMenuNativeSnapshot {
        #[cfg(target_os = "macos")]
        {
            self.menu_engine
                .as_ref()
                .and_then(|engine| engine.borrow().context_menu_diagnostics())
                .map(|native| bbr_testkit::protocol::ContextMenuNativeSnapshot {
                    monitor_installed: native.monitor_installed,
                    right_clicks_over_web: native.right_clicks_over_web,
                    page_owned: native.page_owned,
                    element_owned: native.element_owned,
                })
                .unwrap_or_default()
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = self;
            bbr_testkit::protocol::ContextMenuNativeSnapshot::default()
        }
    }
}
