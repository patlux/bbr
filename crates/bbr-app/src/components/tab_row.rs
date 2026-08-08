//! §11.1/§11.2 vertical tab row anatomy and its controls: favicon tile
//! (monogram fallback), row title, state slot, close control, pin control,
//! and the §10.9 drag insertion tick.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled,
    Window,
};

use crate::palette::FaviconSlate;
use crate::theme::{Radius, Spacing, Theme, ThemeMode};

use super::{FaviconSpec, Icon, RowVisuals, TabVisualState};

/// §7.2 favicon tile: the §8 density icon size (S 16 / N 18 / XL 20)
/// square, `radius:sm`, monogram letter on the FNV-slate ramp; while
/// loading the slot shows the §10.5 accent arc (drawn, static — rotation
/// is a later polish pass).
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
        .when(state == TabVisualState::Loading, |el| {
            el.child(Icon::LoadingArc.canvas(theme, size))
        })
        .when(state != TabVisualState::Loading, |el| {
            el.child(SharedString::from(letter.to_string()))
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

/// §11.1 state slot: the fixed `density:state-slot` trailing slot (S 8 /
/// N 10 / XL 10) that later holds the `attention` dot (§10.6). Present on
/// every row so the `bbr-tab-state-<n>` assert id (§15) exists; empty
/// until an attention surface ships.
pub fn row_state_slot(theme: &Theme, index: usize) -> Stateful<Div> {
    div()
        .id(("bbr-tab-state", index))
        .size(theme.density.state_slot())
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_faint)
}

/// §11.1 close control: the §7 drawn close glyph (two 1.5 pt diagonal
/// strokes) at the density icon size, optically centered in the
/// `density:hit-min.close` hit square (§7: close controls ≥ S 24 / N 28 /
/// XL 32). Always interactive from the first slice so the semantic id and
/// the keyboard equivalent exist; the hover-reveal polish landed with the
/// hover slice.
pub fn close_control(theme: &Theme, index: usize) -> Stateful<Div> {
    div()
        .id(("bbr-tab-close", index))
        .size(theme.density.hit_min_close())
        .rounded(Radius.small())
        .flex()
        .items_center()
        .justify_center()
        .child(Icon::Close.canvas(theme, theme.density.icon()))
}

/// §11.2 pin control: the pointer half of the pin/unpin command (`Cmd-Shift-P`).
/// A pinned row renders the §7 filled dot in `accent`; an unpinned row the
/// hollow ring in `ink.secondary`; `disabled` drops the glyph to `ink.faint`
/// (e.g. while the row is being dragged). `pinned` is the row's current
/// region identity.
pub fn pin_control(theme: &Theme, index: usize, pinned: bool, disabled: bool) -> Stateful<Div> {
    let color = if disabled {
        theme.palette.ink_faint
    } else if pinned {
        theme.palette.accent
    } else {
        theme.palette.ink_secondary
    };
    div()
        .id(("bbr-tab-pin", index))
        .size(theme.density.icon())
        .rounded(Radius.small())
        .flex()
        .items_center()
        .justify_center()
        .cursor(if disabled {
            gpui::CursorStyle::Arrow
        } else {
            gpui::CursorStyle::PointingHand
        })
        .child(Icon::Pin { pinned }.canvas_with(theme, theme.density.icon(), color))
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
/// `[favicon 18] · [title flex, trunc] · [pin] · [state slot 10] ·
/// [close hit-min.close]`, tile per §8 with the selected rail at the
/// leading edge. The §10.2–§10.4 hover/active/selected wash paints an
/// inset rounded surface (`density:row-inset` from the rail edges,
/// `density:radius`) behind the content.
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
        // §10.2/§10.3: the pointer-state washes blend over the sidebar
        // material exactly as DESIGN.md defines (token `wash_hover` /
        // `wash_active`); the more specific composed state already won in
        // the caller, so these arms never fight selection/error/loading.
        TabVisualState::Hover => p.wash_hover,
        TabVisualState::Active => p.wash_active,
        _ => p.surface_sidebar,
    };
    let dragging = visuals.state == TabVisualState::Dragging;
    let controls_disabled = visuals.state == TabVisualState::Disabled || dragging;
    let close = close_control(theme, visuals.index)
        .on_mouse_down(MouseButton::Left, on_close)
        .when(visuals.show_close && !controls_disabled, |el| {
            el.cursor_pointer()
        })
        // §10.2 hover-reveal: the close control fades in on interaction and
        // fades out at rest (transient indication; the row keeps its hit
        // square so the id and the pointer path are stable).
        .opacity(if visuals.reveal_close { 1.0 } else { 0.0 });
    let pin = pin_control(theme, visuals.index, visuals.pinned, controls_disabled)
        .on_mouse_down(MouseButton::Left, on_pin);
    div()
        .id(("bbr-tab-row", visuals.index))
        .h(theme.density.row())
        .relative()
        .px(theme.density.row_padding_x())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .cursor_pointer()
        .when(dragging, |el| {
            // §10.9: the dragged row recedes (92% opacity) with an elev:2
            // shadow + strong outline, and its own controls are disabled.
            el.opacity(0.92)
                .shadow(elev2_shadow(theme))
                .border_1()
                .border_color(p.border_strong)
        })
        // §10.2–§10.4: the wash paints an inset rounded row surface
        // (`density:row-inset` from the rail edges, `density:radius`)
        // behind the content instead of a full-bleed band, so the trailing
        // close control sits inside that surface. The more specific
        // composed state already won in the caller, so the wash never
        // fights selection/error/loading.
        .child(
            div()
                .absolute()
                .left(theme.density.row_inset())
                .right(theme.density.row_inset())
                .top(theme.density.row_inset())
                .bottom(theme.density.row_inset())
                .rounded(theme.density.density_radius())
                .bg(bg)
                // §10.3: a pressed row deepens its wash with a 1 px inner
                // hairline (no translation, no "press down" motion).
                .when(visuals.state == TabVisualState::Active, |el| {
                    el.border_1().border_color(p.hairline)
                })
                // §10.9: the drop-target surface gets the accent outline
                // and tint on the same inset surface.
                .when(visuals.state == TabVisualState::DragTarget, |el| {
                    el.border_1().border_color(p.accent)
                }),
        )
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
        // GPUI-CE added the `inset` field to `BoxShadow`.
        inset: false,
    }]
}
