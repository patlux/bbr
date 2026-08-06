//! GPUI rendering for the browser shell.
//!
//! All visuals come from the [`crate::theme::Theme`] tokens (DESIGN.md); no
//! local `rgb(0x…)`/spacing magic values. The first intentional shell slice
//! replaces this placeholder body with the controller-backed tab surface
//! while keeping the brand / command bar / status / toggle anatomy.

use gpui::{
    div, prelude::*, px, Context, MouseButton, MouseDownEvent, Render, SharedString, Window,
};

use crate::slot::NativeWebViewSlot;
use crate::theme::{Density, Radius, Spacing, Theme};

use super::BbrApp;

impl Render for BbrApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let p = theme.palette;
        let chrome_focused = self.chrome_handle.is_focused(window);
        let browser = self.controller.snapshot();
        let status_text: SharedString = crate::shell::status_line(&browser).into();
        #[cfg(feature = "testkit")]
        self.performance.observe_render_state(
            browser.active_view,
            self.sidebar_visible,
            chrome_focused,
            self.command_bar_mode.into(),
        );

        let nav_dot = |label: &'static str, enabled: bool| {
            div()
                .id(label)
                .px_8()
                .py_4()
                .rounded(Radius::small())
                .bg(if enabled {
                    p.surface_field
                } else {
                    p.surface_sidebar
                })
                .text_color(if enabled { p.ink } else { p.ink_faint })
                .text_xs()
                .child(label)
        };

        let sidebar = div()
            .id("bbr-sidebar")
            .w(Density::Normal.sidebar_width())
            .h_full()
            .flex()
            .flex_col()
            .bg(p.surface_sidebar)
            .border_r_1()
            .border_color(p.hairline)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| this.focus_chrome(window, cx)),
            )
            .child(
                div()
                    .id("bbr-brand")
                    .px(Spacing::sp4())
                    .py(Spacing::sp3())
                    .text_size(theme.type_.title.size)
                    .font_weight(theme.type_.title.weight)
                    .text_color(p.ink)
                    .child("BBR"),
            )
            .child(
                div()
                    .id("bbr-nav")
                    .flex()
                    .flex_row()
                    .gap(Spacing::sp2())
                    .px(Spacing::sp3())
                    .py(Spacing::sp2())
                    .child(nav_dot("◀", browser.nav.can_go_back).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _w, _cx| this.go_back()),
                    ))
                    .child(nav_dot("▶", browser.nav.can_go_forward).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _w, _cx| this.go_forward()),
                    ))
                    .child(nav_dot("⟳", true).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _w, _cx| this.reload()),
                    )),
            )
            .child(
                div()
                    .id("bbr-command-bar")
                    .mx(Spacing::sp3())
                    .my(Spacing::sp2())
                    .px(Spacing::sp3())
                    .py(Spacing::sp2())
                    .rounded(Radius::large())
                    .bg(if chrome_focused {
                        p.surface_field
                    } else {
                        p.surface_sidebar
                    })
                    .border_1()
                    .border_color(if chrome_focused {
                        p.accent
                    } else {
                        p.hairline
                    })
                    .flex()
                    .flex_row()
                    .gap(Spacing::sp2())
                    .child(
                        div()
                            .text_size(theme.type_.meta.size)
                            .text_color(p.ink_faint)
                            .child("▶"),
                    )
                    .child(
                        div()
                            .id("bbr-command-text")
                            .flex_1()
                            .text_size(theme.type_.mono.size)
                            .text_color(p.ink)
                            .child(if self.command_bar_text.is_empty() {
                                SharedString::from("Type a URL…")
                            } else {
                                SharedString::from(self.command_bar_text.clone())
                            }),
                    )
                    .when(chrome_focused && self.command_bar_mode.is_active(), |el| {
                        el.child(
                            div()
                                .id("bbr-command-cursor")
                                .w(px(2.0))
                                .h_full()
                                .bg(p.accent),
                        )
                    }),
            )
            .child(
                div()
                    .id("bbr-status")
                    .px(Spacing::sp4())
                    .py(Spacing::sp2())
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .child(status_text),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("bbr-sidebar-toggle")
                    .px(Spacing::sp4())
                    .py(Spacing::sp2())
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, window, cx| {
                            this.toggle_sidebar(window, cx)
                        }),
                    )
                    .child("Sidebar (Cmd-B)"),
            );

        div()
            .id("bbr-root")
            .track_focus(&self.chrome_handle)
            .key_context("bbr_chrome")
            .flex()
            .flex_row()
            .size_full()
            .bg(p.surface_canvas)
            .on_key_down(cx.listener(Self::on_chrome_key_down))
            .when(self.sidebar_visible, |root| root.child(sidebar))
            .child({
                let slot = NativeWebViewSlot::new(self.engine.clone());
                #[cfg(feature = "testkit")]
                let slot = slot.on_presented(
                    self.performance.presentation_pending(),
                    self.performance.presentation_callback(),
                );
                slot
            })
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
