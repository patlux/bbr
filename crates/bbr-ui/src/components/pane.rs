//! §11.8 empty and crash glass panes.

use gpui::{
    div, prelude::*, px, App, Div, FocusHandle, MouseButton, MouseDownEvent, Stateful, Styled,
    Window,
};

use crate::icons::Icon;
use crate::theme::{Spacing, Theme};

/// §11.8 empty space pane: the D10 drawn illustration (tab pills floating
/// around a ⌘ key), `type:display` headline, profile line, and a primary
/// "New Tab ⌘T" button that focuses the command bar in NewTab mode.
/// `profile_line` names the owning profile (§11.8); `illustration` is the
/// pane-appearance key while the illustration plays (`None`: rest frame).
pub fn empty_space_pane(
    theme: &Theme,
    profile_line: &str,
    illustration: Option<u64>,
    on_new_tab: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-empty-new-tab")
        .debug_selector(|| "bbr-empty-new-tab".to_string())
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(Spacing::sp3())
        .bg(theme.palette.surface_canvas)
        .child(super::illustration::illustration(
            theme,
            super::illustration::Illustration::EmptySpace,
            illustration,
        ))
        .child(
            div()
                .text_size(theme.type_.display.size)
                .font_weight(theme.type_.display.weight)
                .text_color(theme.palette.ink)
                .child("No tabs in this space"),
        )
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink_secondary)
                .child(profile_line.to_string()),
        )
        .child(
            div()
                .id("bbr-empty-new-tab-button")
                .debug_selector(|| "bbr-empty-new-tab-button".to_string())
                .mt(Spacing::sp3())
                .px(Spacing::sp4())
                .py(Spacing::sp2())
                .rounded(theme.radius.medium())
                .bg(theme.palette.accent)
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .text_color(theme.palette.on_accent)
                .on_mouse_down(MouseButton::Left, on_new_tab)
                .child("New Tab  ⌘T"),
        )
}

/// §11.8 crash panel: replaces the glass area of a crashed tab with the D10
/// heartbeat illustration (in `status.error`; the row's lifering stays the
/// non-color companion), copy, and Reload / Close tab actions. The crashed
/// tab row stays selected and listed; the native view is hidden.
/// `illustration` is the pane-appearance key while it plays.
pub fn crash_panel(
    theme: &Theme,
    illustration: Option<u64>,
    on_reload: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-crash-panel")
        .debug_selector(|| "bbr-crash-panel".to_string())
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(Spacing::sp3())
        .bg(theme.palette.surface_field)
        .child(super::illustration::illustration(
            theme,
            super::illustration::Illustration::Crashed,
            illustration,
        ))
        .child(
            div()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(theme.palette.ink)
                .child("This page crashed"),
        )
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink_secondary)
                .child("The tab stays open; reload to restore it."),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap(Spacing::sp3())
                .mt(Spacing::sp2())
                .child(
                    div()
                        .id("bbr-crash-reload")
                        .debug_selector(|| "bbr-crash-reload".to_string())
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .bg(theme.palette.accent)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.on_accent)
                        .on_mouse_down(MouseButton::Left, on_reload)
                        .child("Reload"),
                )
                .child(
                    div()
                        .id("bbr-crash-close")
                        .debug_selector(|| "bbr-crash-close".to_string())
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .border_1()
                        .border_color(theme.palette.border_strong)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.ink)
                        .on_mouse_down(MouseButton::Left, on_close)
                        .child("Close tab"),
                ),
        )
}

/// §11.8a unresponsive pane: replaces the glass area of an unresponsive
/// tab (the renderer stopped answering liveness probes; the session and
/// tab stay alive) with the 32 pt lifering + static clock companions, copy,
/// and the closed recovery set Wait / Reload / Close tab. Focus order:
/// Wait → Reload → Close tab → sidebar (each button tracks its own focus
/// handle in tree order; the sidebar follows in the global tab order). The
/// tab row stays selected and listed; the native view is hidden while the
/// pane is up (the hung renderer paints nothing useful).
pub fn unresponsive_panel(
    theme: &Theme,
    wait_handle: &FocusHandle,
    reload_handle: &FocusHandle,
    close_handle: &FocusHandle,
    on_wait: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_reload: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-unresponsive-panel")
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(Spacing::sp3())
        .bg(theme.palette.surface_field)
        .child(
            div()
                .size(px(32.0))
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(Spacing::sp1())
                .child(Icon::Lifering.canvas(theme, px(32.0)))
                .child(Icon::Clock.canvas(theme, px(20.0))),
        )
        .child(
            div()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(theme.palette.ink)
                .child("This page is not responding"),
        )
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink_secondary)
                .child("The page is busy; the tab stays open."),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap(Spacing::sp3())
                .mt(Spacing::sp2())
                .child(
                    div()
                        .id("bbr-unresponsive-wait")
                        .track_focus(wait_handle)
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .border_1()
                        .border_color(theme.palette.border_strong)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.ink)
                        .on_mouse_down(MouseButton::Left, on_wait)
                        .child("Wait"),
                )
                .child(
                    div()
                        .id("bbr-unresponsive-reload")
                        .track_focus(reload_handle)
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .bg(theme.palette.accent)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.on_accent)
                        .on_mouse_down(MouseButton::Left, on_reload)
                        .child("Reload"),
                )
                .child(
                    div()
                        .id("bbr-unresponsive-close")
                        .track_focus(close_handle)
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .border_1()
                        .border_color(theme.palette.border_strong)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.ink)
                        .on_mouse_down(MouseButton::Left, on_close)
                        .child("Close tab"),
                ),
        )
}
