//! GPUI rendering for the browser shell.

use gpui::{
    div, prelude::*, px, rgb, Context, MouseButton, MouseDownEvent, Render, SharedString, Window,
};

use crate::slot::NativeWebViewSlot;

use super::{BbrApp, SIDEBAR_WIDTH_PX};

impl Render for BbrApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            let (color, bg) = if enabled {
                (rgb(0xe8e8e8), rgb(0x333333))
            } else {
                (rgb(0x555555), rgb(0x222222))
            };
            div()
                .id(label)
                .px_8()
                .py_4()
                .rounded_md()
                .bg(bg)
                .text_color(color)
                .child(label)
        };

        let sidebar = div()
            .id("bbr-sidebar")
            .w(px(SIDEBAR_WIDTH_PX))
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(0x1e1e1e))
            .border_r_1()
            .border_color(rgb(0x333333))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| this.focus_chrome(window, cx)),
            )
            .child(
                div()
                    .id("bbr-brand")
                    .px_16()
                    .py_12()
                    .text_xl()
                    .text_color(rgb(0xf2f2f2))
                    .child("BBR"),
            )
            .child(
                div()
                    .id("bbr-nav")
                    .flex()
                    .flex_row()
                    .gap_8()
                    .px_12()
                    .py_8()
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
                    .mx_12()
                    .my_8()
                    .px_12()
                    .py_8()
                    .rounded_md()
                    .bg(if chrome_focused {
                        rgb(0x2c2c2c)
                    } else {
                        rgb(0x232323)
                    })
                    .border_1()
                    .border_color(if chrome_focused {
                        rgb(0x6aa5ff)
                    } else {
                        rgb(0x3c3c3c)
                    })
                    .flex()
                    .flex_row()
                    .gap_8()
                    .child(div().text_color(rgb(0x9a9a9a)).child("▶"))
                    .child(
                        div()
                            .id("bbr-command-text")
                            .flex_1()
                            .text_color(rgb(0xe8e8e8))
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
                                .bg(rgb(0x6aa5ff)),
                        )
                    }),
            )
            .child(
                div()
                    .id("bbr-status")
                    .px_16()
                    .py_8()
                    .text_color(rgb(0x9a9a9a))
                    .text_xs()
                    .child(status_text),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("bbr-sidebar-toggle")
                    .px_16()
                    .py_8()
                    .text_color(rgb(0x9a9a9a))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, _w, cx| this.toggle_sidebar(cx)),
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
            .bg(rgb(0x141414))
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
