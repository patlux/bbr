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
use crate::theme::{ContrastMode, Theme, ThemeMode};
use bbr_web::WebViewId;

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

        let content: gpui::AnyElement = if let Some(layout) = self.controller.pane_layout() {
            // The authoritative pane layout drives the native surface:
            // one slot per visible pane, each addressing its own
            // `WebViewId`. Single-pane geometry is exactly the previous
            // sidebar/content contract; split renders two equal,
            // adjacent, strictly positive horizontal panes with no
            // divider or polish in this anchor.
            #[cfg(feature = "testkit")]
            {
                let mut keep = vec![layout.primary];
                if let Some(secondary) = layout.secondary {
                    keep.push(secondary);
                }
                self.performance.prune_pane_rects(&keep);
            }
            let build_slot = |this: &BbrApp, id: WebViewId, _focused: bool| {
                let slot = NativeWebViewSlot::new(this.engine.clone(), id);
                #[cfg(feature = "testkit")]
                let slot = slot.on_keyed_geometry(this.performance.keyed_geometry_callback());
                #[cfg(feature = "testkit")]
                let slot = if _focused {
                    // The focused/active slot feeds the legacy
                    // `content_rect` and the presentation-commit path; a
                    // secondary pane must never satisfy either.
                    slot.on_geometry(this.performance.slot_geometry_callback())
                        .on_presented(
                            this.performance.presentation_pending(),
                            this.performance.presentation_callback(),
                        )
                } else {
                    slot
                };
                slot.into_any()
            };
            match layout.secondary {
                Some(_) => div()
                    .flex()
                    .flex_row()
                    .flex_grow()
                    .size_full()
                    .child(build_slot(
                        self,
                        layout.primary,
                        layout.focused == layout.primary,
                    ))
                    .child(build_slot(
                        self,
                        layout.secondary.expect("split has two panes"),
                        layout.focused == layout.secondary.expect("split has two panes"),
                    ))
                    .into_any(),
                None => build_slot(self, layout.primary, true),
            }
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
            // §10.3 release path: a pointer press ends with the primary
            // button up anywhere over (or out of) the window.
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _e: &gpui::MouseUpEvent, _window, cx| {
                    this.clear_row_pressed(cx)
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _e: &gpui::MouseUpEvent, _window, cx| {
                    this.clear_row_pressed(cx)
                }),
            )
            .on_hover(cx.listener(|this, hovered: &bool, _window, cx| {
                // The pointer left the whole GPUI chrome (e.g. into the
                // native web view): clear the row hover. Entering the root
                // never clears a row the pointer is already over.
                if !*hovered {
                    this.clear_row_hover(cx);
                }
            }))
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
    /// The resolved theme tokens for the current shell mode. Density is
    /// automatic from the regular-tab collection size (DESIGN.md §8); the
    /// §15 mode fields are already reported by the snapshot.
    pub fn theme(&self) -> Theme {
        let browser = self.controller.snapshot();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let regular = browser.tabs.len().saturating_sub(pinned);
        Theme::for_collection(ThemeMode::Dark, ContrastMode::Standard, regular)
    }
}
