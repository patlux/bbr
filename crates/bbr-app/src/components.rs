//! Reusable typed GPUI Chassis components (DESIGN.md §11).
//!
//! Every shell surface composes these instead of inventing local layout:
//! tab rows (pinned/regular anatomy), favicon tiles (monogram fallback),
//! section labels, the close control, the empty-space pane, and the crash
//! panel. Visual state comes from the §10 vocabulary; listeners are passed
//! in so any view can wire these without reimplementing the anatomy.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, ParentElement, SharedString,
    Stateful, Styled, Window,
};

use crate::palette::FaviconSlate;
use crate::theme::{Radius, Spacing, Theme, ThemeMode};

/// §10 row states the shell renders and E2E asserts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabVisualState {
    Default,
    Selected,
    Loading,
    Error,
}

/// The favicon tile: the deterministic monogram (§7.2) until a real favicon
/// exists; while loading the slot shows the accent spinner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaviconSpec {
    pub letter: char,
    pub hue: usize,
}

/// One row's rendered identity, derived from the controller snapshot.
#[derive(Debug, Clone)]
pub struct RowVisuals {
    pub index: usize,
    pub title: SharedString,
    pub state: TabVisualState,
    pub favicon: Option<FaviconSpec>,
    pub show_close: bool,
}

/// §11.2 section label ("Pinned"/"Tabs"), uppercase `type:badge`; the caller
/// hides the label when its section is empty.
pub fn section_label(theme: &Theme, text: &str) -> Div {
    div()
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(theme.palette.ink_faint)
        .child(text.to_uppercase())
}

/// §7.2 favicon tile: 16 pt (S: 14) square, `radius:sm`, monogram letter on
/// the FNV-slate ramp; loading swaps the letter for the accent spinner dot.
pub fn favicon_tile(
    theme: &Theme,
    index: usize,
    spec: Option<FaviconSpec>,
    state: TabVisualState,
) -> Stateful<Div> {
    let size = theme.density.icon();
    let (bg, fg) = match spec {
        Some(spec) => {
            let bg = match theme.mode {
                ThemeMode::Dark => FaviconSlate::dark(spec.hue),
                ThemeMode::Light => FaviconSlate::light(spec.hue),
            };
            (bg, theme.palette.surface_glass)
        }
        None => (theme.palette.surface_field, theme.palette.ink_faint),
    };
    let letter = spec.map(|s| s.letter).unwrap_or('?');
    div()
        .id(("bbr-tab-favicon", index))
        .size(size)
        .rounded(Radius.small())
        .bg(bg)
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(fg)
        .child(if state == TabVisualState::Loading {
            SharedString::from("●")
        } else {
            SharedString::from(letter.to_string())
        })
}

/// §11.1 row title: one line, end-ellipsis, `type:title`; the §11.9 fallback
/// has already been applied by the caller. Loading dims to `ink.secondary`.
pub fn row_title(theme: &Theme, index: usize, text: &str, state: TabVisualState) -> Stateful<Div> {
    let color = if state == TabVisualState::Loading {
        theme.palette.ink_secondary
    } else {
        theme.palette.ink
    };
    div()
        .id(("bbr-tab-title", index))
        .flex_1()
        .overflow_hidden()
        .text_ellipsis()
        .text_size(theme.type_.title.size)
        .font_weight(theme.type_.title.weight)
        .text_color(color)
        .child(SharedString::from(text.to_string()))
}

/// §11.1 close control: 16 pt glyph in a ≥ 24 pt hit square. Always
/// interactive from the first slice so the semantic id and the keyboard
/// equivalent exist; the hover-reveal polish lands with the hover slice.
pub fn close_control(theme: &Theme, index: usize) -> Stateful<Div> {
    div()
        .id(("bbr-tab-close", index))
        .size(theme.density.icon())
        .rounded(Radius.small())
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_secondary)
        .child("×")
}

/// §11.1 vertical tab row anatomy:
/// `[favicon 16] · [title flex, trunc] · [state slot 8] · [close 16]`,
/// tile per §8 with the selected rail at the leading edge.
///
/// `on_select`/`on_close` are the production view listeners (built with
/// `cx.listener` in `render.rs`), so the pointer path stays in the app layer.
/// The close control is a frontmost child hitbox: in the bubble phase its
/// handler runs before the row's and stops propagation, so a click on the
/// close glyph closes without first selecting the row.
pub fn tab_row(
    theme: &Theme,
    visuals: &RowVisuals,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let bg = match visuals.state {
        TabVisualState::Selected => p.tint_selected,
        TabVisualState::Error => p.status_error_surface,
        _ => p.surface_sidebar,
    };
    let close = close_control(theme, visuals.index)
        .on_mouse_down(MouseButton::Left, on_close)
        .when(visuals.show_close, |el| el.cursor_pointer());
    div()
        .id(("bbr-tab-row", visuals.index))
        .h(theme.density.row())
        .px(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.medium())
        .bg(bg)
        .cursor_pointer()
        .when(visuals.state == TabVisualState::Selected, |el| {
            el.child(
                div()
                    .w(px(3.0))
                    .h(theme.density.row_tile())
                    .rounded(Radius.small())
                    .bg(p.accent),
            )
        })
        .child(favicon_tile(
            theme,
            visuals.index,
            visuals.favicon,
            visuals.state,
        ))
        .child(row_title(
            theme,
            visuals.index,
            &visuals.title,
            visuals.state,
        ))
        .child(close)
        .on_mouse_down(MouseButton::Left, on_select)
}

/// §11.8 empty space pane: `material:canvas` mark, `type:display` headline,
/// profile line, and a primary "New Tab ⌘T" button that focuses the command
/// bar in NewTab mode.
pub fn empty_space_pane(
    theme: &Theme,
    on_new_tab: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-empty-new-tab")
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(Spacing::sp3())
        .bg(theme.palette.surface_canvas)
        .child(
            div()
                .size(px(64.0))
                .rounded(theme.radius.large())
                .bg(theme.palette.surface_sidebar)
                .flex()
                .items_center()
                .justify_center()
                .text_size(theme.type_.display.size)
                .font_weight(theme.type_.display.weight)
                .text_color(theme.palette.ink_secondary)
                .child("B"),
        )
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
                .child("in Profile · Default"),
        )
        .child(
            div()
                .id("bbr-empty-new-tab-button")
                .mt(Spacing::sp3())
                .px(Spacing::sp4())
                .py(Spacing::sp2())
                .rounded(theme.radius.medium())
                .bg(theme.palette.accent)
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .text_color(theme.palette.surface_glass)
                .on_mouse_down(MouseButton::Left, on_new_tab)
                .child("New Tab  ⌘T"),
        )
}

/// §11.8 crash panel: replaces the glass area of a crashed tab with the
/// lifering glyph, copy, and Reload / Close tab actions. The crashed tab row
/// stays selected and listed; the native view is hidden.
pub fn crash_panel(
    theme: &Theme,
    on_reload: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-crash-panel")
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
                .rounded(theme.radius.full())
                .border_2()
                .border_color(theme.palette.status_error)
                .flex()
                .items_center()
                .justify_center()
                .text_size(theme.type_.label.size)
                .text_color(theme.palette.status_error)
                .child("!"),
        )
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
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .bg(theme.palette.accent)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.surface_glass)
                        .on_mouse_down(MouseButton::Left, on_reload)
                        .child("Reload"),
                )
                .child(
                    div()
                        .id("bbr-crash-close")
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
