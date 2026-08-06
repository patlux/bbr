//! §11.1/§11.2 vertical tab row anatomy and its controls: favicon tile
//! (monogram fallback), row title, state slot, close control, pin control,
//! and the §10.9 drag insertion tick.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled,
    Window,
};

use crate::palette::FaviconSlate;
use crate::theme::{Radius, Spacing, Theme, ThemeMode};

use super::{FaviconSpec, RowVisuals, TabVisualState};

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
/// and the §4 separator-aware truncation have already been applied by the
/// caller. Loading dims to `ink.secondary`.
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

/// §11.1 state slot: the fixed 8 pt trailing slot that later holds the
/// `attention` dot (§10.6). Present on every row so the `bbr-tab-state-<n>`
/// assert id (§15) exists; empty until an attention surface ships.
pub fn row_state_slot(theme: &Theme, index: usize) -> Stateful<Div> {
    div()
        .id(("bbr-tab-state", index))
        .size(px(8.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_faint)
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

/// §11.2 pin control: the pointer half of the pin/unpin command (`Cmd-Shift-P`).
/// A pinned row renders the filled dot in `accent`; an unpinned row the
/// hollow dot in `ink.faint`. `pinned` is the row's current region identity,
/// `disabled` suppresses interaction (e.g. while the row is being dragged).
pub fn pin_control(theme: &Theme, index: usize, pinned: bool, disabled: bool) -> Stateful<Div> {
    div()
        .id(("bbr-tab-pin", index))
        .size(theme.density.icon())
        .rounded(Radius.small())
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.meta.size)
        .text_color(if disabled {
            theme.palette.ink_faint
        } else if pinned {
            theme.palette.accent
        } else {
            theme.palette.ink_secondary
        })
        .cursor(if disabled {
            gpui::CursorStyle::Arrow
        } else {
            gpui::CursorStyle::PointingHand
        })
        .child(if pinned { "●" } else { "○" })
}

/// §10.9 drag insertion tick: a 2 pt vertical accent rule at the target slot
/// spanning the row tile. `invalid` renders the muted hairline variant so an
/// illegal boundary drop is clearly signaled without asserting a committed
/// target. The stable §15 id lets E2E assert the tick's presence and state.
pub fn drag_tick(theme: &Theme, index: usize, invalid: bool) -> Stateful<Div> {
    div()
        .id(("bbr-drag-tick", index))
        .h(px(2.0))
        .w_full()
        .flex_shrink_0()
        .bg(if invalid {
            theme.palette.ink_faint
        } else {
            theme.palette.accent
        })
}

/// §11.1 vertical tab row anatomy:
/// `[favicon 16] · [title flex, trunc] · [pin] · [state slot 8] · [close 16]`,
/// tile per §8 with the selected rail at the leading edge.
///
/// `on_select`/`on_close`/`on_pin` are the production view listeners (built
/// with `cx.listener` in the sidebar surface), so the pointer path stays in
/// the app layer. The close and pin controls are frontmost child hitboxes:
/// in the bubble phase their handlers run before the row's and stop
/// propagation, so a click on a control does not first select the row.
/// `on_begin_drag` is the pointer-drag entry (DESIGN.md §10.9): pressing the
/// row starts the drag; the controls' stop-propagation means a press on them
/// never drags.
///
/// Drag states: a `Dragging` row renders at 92% opacity with an elev:2
/// shadow and a strong outline (§10.9) and disables its controls; a
/// `DragTarget` row adds the accent outline + tint; `Disabled` suppresses
/// the wash. These derive from the drag state the caller passes via
/// `RowVisuals.state`.
pub fn tab_row(
    theme: &Theme,
    visuals: &RowVisuals,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_pin: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_begin_drag: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let bg = match visuals.state {
        TabVisualState::Selected => p.tint_selected,
        TabVisualState::Error => p.status_error_surface,
        TabVisualState::DragTarget => p.tint_selected,
        _ => p.surface_sidebar,
    };
    let dragging = visuals.state == TabVisualState::Dragging;
    let controls_disabled = visuals.state == TabVisualState::Disabled || dragging;
    let close = close_control(theme, visuals.index)
        .on_mouse_down(MouseButton::Left, on_close)
        .when(visuals.show_close && !controls_disabled, |el| {
            el.cursor_pointer()
        });
    let pin = pin_control(theme, visuals.index, visuals.pinned, controls_disabled)
        .on_mouse_down(MouseButton::Left, on_pin);
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
        .when(dragging, |el| {
            // §10.9: the dragged row recedes (92% opacity) with an elev:2
            // shadow + strong outline, and its own controls are disabled.
            el.opacity(0.92)
                .shadow(elev2_shadow(theme))
                .border_1()
                .border_color(p.border_strong)
        })
        .when(visuals.state == TabVisualState::DragTarget, |el| {
            el.border_1().border_color(p.accent)
        })
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
        .child(pin)
        .child(row_state_slot(theme, visuals.index))
        .child(close)
        .on_mouse_down(MouseButton::Left, on_select)
        .on_mouse_down(MouseButton::Left, on_begin_drag)
}

/// §10.9 elev:2 shadow for the dragged row: bounded GPUI shadow, no blur.
fn elev2_shadow(theme: &Theme) -> Vec<gpui::BoxShadow> {
    let color = if theme.mode == ThemeMode::Dark {
        gpui::rgba(0x00000073).into()
    } else {
        gpui::rgba(0x171a1d1a).into()
    };
    vec![gpui::BoxShadow {
        color,
        offset: gpui::point(px(0.0), px(12.0)),
        blur_radius: px(24.0),
        spread_radius: px(0.0),
    }]
}
