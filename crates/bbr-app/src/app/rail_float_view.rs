//! Blueprint D02: the floating rail's rendering seams — the Portal layer
//! (the unchanged rail composition inside the floating panel, plus the
//! root pointer listeners the docked rail gets from the browser root), the
//! browser-window keyboard twin, and the callback retargeting that makes
//! every rail action run against the browser window.

use std::rc::Rc;

use bbr_ui::sidebar::model::Callbacks;
use gpui::{
    div, prelude::*, AnyElement, App, Context, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ScrollWheelEvent, Window, WindowHandle,
};

use super::BbrApp;
use crate::window_seam::with_app;

type Browser = WindowHandle<gpui_component::Root>;

/// Wrap one callback slot so it runs against the browser window.
macro_rules! via_browser {
    ($slot:expr, $browser:expr; $($arg:ident: $ty:ty),*) => {{
        let inner = $slot.clone();
        let browser: Browser = $browser;
        $slot = Rc::new(move |$($arg: $ty,)* _window: &mut Window, cx: &mut App| {
            let _ = browser.update(cx, |_root, window, cx| inner($($arg,)* window, cx));
        });
    }};
}

/// Retarget every rail callback to the browser window. The Portal delivers
/// the pointer; focus, the command field and the native first responder
/// live in the browser window, exactly as for the docked rail.
pub(crate) fn retarget(mut c: Callbacks, b: Browser) -> Callbacks {
    use bbr_domain::FolderId;
    use bbr_ui::titlebar::TitlebarControl;
    via_browser!(c.on_rail_mouse_down, b; e: &MouseDownEvent);
    via_browser!(c.on_rail_scroll_wheel, b; e: &ScrollWheelEvent);
    via_browser!(c.titlebar.on_toggle_sidebar, b; e: &MouseDownEvent);
    via_browser!(c.titlebar.on_back, b; e: &MouseDownEvent);
    via_browser!(c.titlebar.on_forward, b; e: &MouseDownEvent);
    via_browser!(c.titlebar.on_reload, b; e: &MouseDownEvent);
    via_browser!(c.titlebar.on_titlebar_mouse_down, b; e: &MouseDownEvent);
    via_browser!(c.titlebar.on_control_hover, b; k: TitlebarControl, h: bool);
    via_browser!(c.titlebar.on_control_press, b; k: TitlebarControl);
    via_browser!(c.titlebar.on_control_release, b;);
    via_browser!(c.titlebar.on_control_secondary, b; k: TitlebarControl, e: &MouseDownEvent);
    via_browser!(c.on_command_hover, b; h: &bool);
    via_browser!(c.on_command_open, b; e: &MouseDownEvent);
    via_browser!(c.on_address_zoom_reset, b; e: &MouseDownEvent);
    via_browser!(c.on_address_site, b; e: &MouseDownEvent);
    via_browser!(c.on_select_row, b; i: usize, e: &MouseDownEvent);
    via_browser!(c.on_close_row, b; i: usize, e: &MouseDownEvent);
    via_browser!(c.on_toggle_pin, b; i: usize, e: &MouseDownEvent);
    via_browser!(c.on_begin_drag, b; i: usize, e: &MouseDownEvent);
    via_browser!(c.on_press_row, b; i: usize, e: &MouseDownEvent);
    via_browser!(c.on_release_row, b; i: usize, e: &MouseUpEvent);
    via_browser!(c.on_hover_row, b; i: usize, h: bool);
    via_browser!(c.on_new_tab, b; e: &MouseDownEvent);
    via_browser!(c.on_tab_list_key_down, b; e: &KeyDownEvent);
    via_browser!(c.on_activate_favorite, b; i: usize, e: &MouseDownEvent);
    via_browser!(c.on_context_menu_row, b; i: usize, e: &MouseDownEvent);
    via_browser!(c.on_context_menu_favorite, b; i: usize, e: &MouseDownEvent);
    via_browser!(c.on_focus_folder, b; f: FolderId, e: &MouseDownEvent);
    via_browser!(c.on_context_menu_folder, b; f: FolderId, e: &MouseDownEvent);
    via_browser!(c.on_hover_folder, b; f: FolderId, h: bool);
    via_browser!(c.on_toggle_folder, b; f: FolderId, e: &MouseDownEvent);
    via_browser!(c.on_select_folder_member, b; f: FolderId, m: usize, e: &MouseDownEvent);
    via_browser!(c.on_close_folder_member, b; f: FolderId, m: usize, e: &MouseDownEvent);
    via_browser!(c.on_folders_key_down, b; e: &KeyDownEvent);
    via_browser!(c.on_plus_menu_key_down, b; e: &KeyDownEvent);
    via_browser!(c.on_plus_menu_commit, b; i: usize, e: &MouseDownEvent);
    via_browser!(c.on_plus_menu_inside_press, b; e: &MouseDownEvent);
    via_browser!(c.on_downloads, b; e: &MouseDownEvent);
    via_browser!(c.on_plus, b; e: &MouseDownEvent);
    via_browser!(c.on_select_space, b; s: bbr_browser::SpaceId, e: &MouseDownEvent);
    via_browser!(c.on_context_menu_space, b; s: bbr_browser::SpaceId, e: &MouseDownEvent);
    via_browser!(c.on_space_strip_key_down, b; e: &KeyDownEvent);
    via_browser!(c.on_divider_mouse_down, b; e: &MouseDownEvent);
    via_browser!(c.on_divider_key_down, b; e: &KeyDownEvent);
    via_browser!(c.on_media, b; a: bbr_ui::components::media_card::MediaCardAction);
    c
}

impl BbrApp {
    /// The Portal's floating-rail layer (`None` while the rail is hidden):
    /// the panel with the rail inside, the open Tab/Folder/Space menu (it
    /// must sit above the page too), and the root pointer listeners the
    /// docked rail gets from the browser root — drag moves and releases,
    /// the divider drag, the pressed-state resets. Built against the
    /// browser window (`window`), whose focus the rail reflects.
    pub(crate) fn floating_rail_layer(
        &mut self,
        browser: Browser,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let geometry = self.floating_rail.geometry?;
        let theme = self.theme();
        let chrome_focused = self.chrome_handle.is_focused(window);
        let rail = self.rail_surface(
            &theme,
            geometry.rect[2],
            chrome_focused,
            Some(browser),
            window,
            cx,
        );
        let panel = bbr_ui::sidebar::floating::floating_rail(
            &theme,
            rail.into_any_element(),
            geometry,
            self.active_chrome_tint(),
            self.floating_rail.entrance,
        );
        let menu = self
            .tab_menu
            .state
            .open
            .then(|| self.tab_menu_overlay(&theme, window, cx));
        let app = cx.entity();
        let on_move = {
            let app = app.clone();
            move |event: &MouseMoveEvent, _w: &mut Window, cx: &mut App| {
                let _ = with_app(&browser, &app, cx, |view, window, cx| {
                    view.on_sidebar_drag_move(event, window, cx);
                    view.on_tab_drag_move(event, window, cx);
                });
            }
        };
        let on_up = {
            let app = app.clone();
            move |event: &MouseUpEvent, _w: &mut Window, cx: &mut App| {
                let _ = with_app(&browser, &app, cx, |view, window, cx| {
                    view.on_sidebar_drag_end(event, window, cx);
                    view.end_tab_drag(event, window, cx);
                    view.clear_row_pressed(cx);
                    view.end_nav_press(cx);
                    view.clear_titlebar_pressed(cx);
                });
            }
        };
        let on_down = move |_e: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
            let _ = with_app(&browser, &app, cx, |view, window, cx| {
                view.on_root_left_press(window, cx);
            });
        };
        Some(
            div()
                .id("bbr-floating-rail-layer")
                .absolute()
                .inset_0()
                .on_mouse_down(MouseButton::Left, on_down)
                .on_mouse_move(on_move)
                .on_mouse_up(MouseButton::Left, on_up.clone())
                .on_mouse_up_out(MouseButton::Left, on_up)
                .child(panel)
                .children(menu)
                .into_any_element(),
        )
    }

    /// The panel sliding out after a hide (blueprint D02 exit): the same
    /// rail inside the leaving panel, with no listeners — the Portal no
    /// longer captures the pointer there, so the page has it.
    pub(crate) fn floating_rail_leaving_layer(
        &mut self,
        browser: Browser,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (geometry, key) = self.floating_rail.leaving?;
        let theme = self.theme();
        let rail = self.rail_surface(&theme, geometry.rect[2], false, Some(browser), window, cx);
        Some(bbr_ui::sidebar::floating::floating_rail_leaving(
            &theme,
            rail.into_any_element(),
            geometry,
            self.active_chrome_tint(),
            key,
        ))
    }

    /// The browser-window keyboard twin while the rail floats: the rail's
    /// focus handles, tracked with the same key listeners, so a focused tab
    /// list, folder list, strip, plus menu or divider takes the keys the
    /// docked rail would.
    pub(crate) fn floating_rail_keyboard(&mut self, cx: &mut Context<Self>) -> AnyElement {
        use bbr_ui::sidebar::callbacks::key;
        let c = self.sidebar_callbacks(cx);
        fn twin(
            handle: &gpui::FocusHandle,
            listener: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
        ) -> gpui::Div {
            div().size_0().track_focus(handle).on_key_down(listener)
        }
        div()
            .id("bbr-floating-rail-keyboard")
            .absolute()
            .size_0()
            .child(twin(&self.tab_list_handle, key(&c.on_tab_list_key_down)))
            .child(twin(&self.u2.folders_handle, key(&c.on_folders_key_down)))
            .child(twin(
                &self.u2.plus_menu_handle,
                key(&c.on_plus_menu_key_down),
            ))
            .child(twin(
                &self.space_strip_handle,
                key(&c.on_space_strip_key_down),
            ))
            .child(twin(&self.divider_handle, key(&c.on_divider_key_down)))
            .into_any_element()
    }
}
