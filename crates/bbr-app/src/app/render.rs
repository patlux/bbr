//! GPUI rendering for the browser shell.
//!
//! The root renders the sidebar rail (surface composition lives in
//! `app::sidebar_view`) next to the native web-view slot. All visuals come
//! from the Chassis tokens and components (DESIGN.md); the tab rows are
//! built from the controller snapshot, never from a second navigation-state
//! copy. The root also carries the production sidebar-drag listeners:
//! mouse-move follows an active drag anywhere over the window and mouse-up
//! commits it, so the 4 pt divider keeps working when the cursor leaves it.

use gpui::{div, prelude::*, Context, MouseButton, Render, Window};

use crate::components;
use crate::slot::NativeWebViewSlot;
use crate::theme::Theme;

use super::BbrApp;

impl Render for BbrApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let p = theme.palette;
        let chrome_focused = self.chrome_handle.is_focused(window);
        let browser = self.controller.snapshot();
        #[cfg(feature = "testkit")]
        self.performance.observe_render_state(
            browser.active_view,
            self.sidebar_visible,
            chrome_focused,
            self.command_bar_mode.into(),
        );

        let window_width = f32::from(window.bounds().size.width);
        let sidebar_width = if self.sidebar_visible {
            self.sidebar.effective_width(window_width)
        } else {
            0.0
        };
        let sidebar = self.sidebar_surface(&theme, sidebar_width, chrome_focused, window, cx);

        let content: gpui::AnyElement = if browser.active_view.is_some() {
            let slot = NativeWebViewSlot::new(self.engine.clone());
            #[cfg(feature = "testkit")]
            let slot = slot
                .on_geometry(self.performance.slot_geometry_callback())
                .on_presented(
                    self.performance.presentation_pending(),
                    self.performance.presentation_callback(),
                );
            slot.into_any()
        } else if browser
            .active_index
            .and_then(|index| browser.tabs.get(index))
            .is_some_and(|tab| tab.crashed)
        {
            // The crash destroyed the session (and with it the session nav
            // state), so the error surface derives from the tab's persistent
            // crash flag, never from `browser.nav` (DESIGN.md §10.7).
            components::crash_panel(
                &theme,
                cx.listener(|this, _e, window, cx| this.reload_active_from_panel(window, cx)),
                cx.listener(|this, _e, window, cx| {
                    let _ = this.close_active_tab(window, cx);
                }),
            )
            .into_any()
        } else {
            components::empty_space_pane(
                &theme,
                &crate::shell::profile_line(
                    browser.active_profile_title.as_deref().unwrap_or("Default"),
                ),
                cx.listener(|this, _e, window, cx| this.activate_new_tab(window, cx)),
            )
            .into_any()
        };

        div()
            .id("bbr-root")
            .track_focus(&self.chrome_handle)
            .key_context("bbr_chrome")
            .flex()
            .flex_row()
            .size_full()
            .bg(p.surface_canvas)
            .on_key_down(cx.listener(Self::on_chrome_key_down))
            .on_mouse_move(cx.listener(Self::on_sidebar_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_sidebar_drag_end))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_sidebar_drag_end))
            // The tab-drag listeners run alongside the sidebar-drag ones on
            // the same root; each handler is a no-op unless its own drag is
            // active, so the two drag surfaces never interfere.
            .on_mouse_move(cx.listener(Self::on_tab_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::end_tab_drag))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::end_tab_drag))
            .when(self.sidebar_visible, |root| root.child(sidebar))
            .child(content)
    }
}

impl BbrApp {
    /// The resolved theme tokens for the current shell mode. Fixed
    /// dark/standard/normal density until the light, contrast, and density
    /// slices introduce mode switching; the §15 mode fields are already
    /// reported by the snapshot.
    pub fn theme(&self) -> Theme {
        Theme::dark()
    }
}
