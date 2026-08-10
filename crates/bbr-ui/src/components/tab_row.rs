//! §11.1/§11.2 vertical tab row anatomy and its controls: favicon tile
//! (monogram fallback), row title, state slot, close control, pin control,
//! and the §10.9 drag insertion tick.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, Rgba, SharedString, Stateful,
    Styled, Window,
};

use crate::icons::Icon;
use crate::palette::Palette;
use crate::state::TabVisualState;
use crate::theme::{Radius, Spacing, Theme, ThemeMode};

use super::favicon::{favicon_tile, FaviconSpec};

/// §15 stable ids of the §11.1/§11.2 vertical tab row (DESIGN.md §11.1/
/// §11.2 assert ids). The components render these exact strings; the tests
/// pin them so a label edit can never shift an id.
pub const TAB_ROW_ID: &str = "bbr-tab-row";
pub const TAB_TITLE_ID: &str = "bbr-tab-title";
pub const TAB_CLOSE_ID: &str = "bbr-tab-close";
pub const TAB_STATE_ID: &str = "bbr-tab-state";
pub const TAB_PIN_ID: &str = "bbr-tab-pin";
pub const DRAG_TICK_ID: &str = "bbr-drag-tick";

/// One row's rendered identity, derived from the controller snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RowVisuals {
    pub index: usize,
    pub title: SharedString,
    pub state: TabVisualState,
    pub favicon: Option<FaviconSpec>,
    /// §11.2: whether the row is in the pinned region (drives the pin
    /// control's region identity).
    pub pinned: bool,
    pub show_close: bool,
    /// §10.2 hover-reveal: the close control is visible when the pointer
    /// hovers or presses the row, the row is selected (keyboard users), or
    /// the row is being dragged — never at rest, so the row stays quiet
    /// until interaction.
    pub reveal_close: bool,
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
        .id((TAB_TITLE_ID, index))
        // Test-support only (a release no-op): the stable §15 id as a
        // gallery debug target, the same addressing the app's testkit uses.
        .debug_selector(move || format!("{TAB_TITLE_ID}-{index}"))
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
pub fn row_state_slot(theme: &Theme, index: usize, state: TabVisualState) -> Stateful<Div> {
    div()
        .id((TAB_STATE_ID, index))
        .debug_selector(move || format!("{TAB_STATE_ID}-{index}"))
        .size(theme.density.state_slot())
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_faint)
        .when(attention_visible(state), |el| {
            el.child(attention_dot(theme))
        })
}

/// §10.6: whether the attention dot shows in the trailing state slot. The
/// dot appears only while the composed §10 winner is `Attention` (the
/// caller already resolved error > loading > pressed > selected > hover >
/// default, so attention never rides another state's row).
pub fn attention_visible(state: TabVisualState) -> bool {
    state == TabVisualState::Attention
}

/// §10.6 attention dot: an 8 pt `status.attention` dot in the row's
/// trailing state slot. Static — the two-beat opacity pulse and the
/// one-frame amber surface flash stay later polish (the same honesty as
/// the static loading arc); the dot itself is the attention visual
/// contract, and the §10.6 badge always comes with its reason line in the
/// status area (app-owned).
fn attention_dot(theme: &Theme) -> Div {
    div()
        .size(px(8.0))
        .rounded(Radius.full())
        .bg(theme.palette.status_attention)
}

/// §11.1 close control: the §7 drawn close glyph (two 1.5 pt diagonal
/// strokes) at the density icon size, optically centered in the
/// `density:hit-min.close` hit square (§7: close controls ≥ S 24 / N 28 /
/// XL 32). Always interactive from the first slice so the semantic id and
/// the keyboard equivalent exist; the hover-reveal polish landed with the
/// hover slice.
pub fn close_control(theme: &Theme, index: usize) -> Stateful<Div> {
    div()
        .id((TAB_CLOSE_ID, index))
        .debug_selector(move || format!("{TAB_CLOSE_ID}-{index}"))
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
        .id((TAB_PIN_ID, index))
        .debug_selector(move || format!("{TAB_PIN_ID}-{index}"))
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
        .id((DRAG_TICK_ID, index))
        .debug_selector(move || format!("{DRAG_TICK_ID}-{index}"))
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
    let bg = row_wash(&p, visuals.state);
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
        .id((TAB_ROW_ID, visuals.index))
        .debug_selector(move || format!("{TAB_ROW_ID}-{}", visuals.index))
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
                .shadow(drag_shadow(theme))
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
        .child(row_state_slot(theme, visuals.index, visuals.state))
        .child(close)
        .on_mouse_down(MouseButton::Left, on_select)
        .on_mouse_down(MouseButton::Left, on_begin_drag)
}

/// §10.2–§10.4/§10.7/§10.9: the row's inset-surface wash per §10 state.
/// The more specific composed state already won in the caller (the app's
/// `row_state` composition — error > loading > active > selected > hover >
/// default — and the drag state machine replacing the composition with
/// `Dragging`/`DragTarget`); this function resolves the winner's visuals
/// so the render and the tests share one resolution.
pub fn row_wash(p: &Palette, state: TabVisualState) -> Rgba {
    match state {
        TabVisualState::Selected => p.tint_selected,
        TabVisualState::Error => p.status_error_surface,
        TabVisualState::DragTarget => p.tint_selected,
        // §10.2/§10.3: the pointer-state washes blend over the sidebar
        // material exactly as DESIGN.md defines (token `wash_hover` /
        // `wash_active`); these arms never fight selection/error/loading
        // because the caller already composed a single winning state.
        TabVisualState::Hover => p.wash_hover,
        TabVisualState::Active => p.wash_active,
        _ => p.surface_sidebar,
    }
}

/// §10.9 the dragged row's elev:2 shadow (the distinct single-shadow
/// variant the dragged row uses — not the §2 floating-surface pair, which
/// lives in [`crate::theme::elevation::elev2`]): bounded GPUI shadow, no
/// blur beyond the documented radius.
fn drag_shadow(theme: &Theme) -> Vec<gpui::BoxShadow> {
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

#[cfg(test)]
mod tests {
    use super::super::favicon::TAB_FAVICON_ID;
    use super::*;

    #[test]
    fn ids_are_the_stable_section_15_ids() {
        // DESIGN.md §11.1/§11.2/§10.9 assert ids — unchangeable contract.
        assert_eq!(TAB_ROW_ID, "bbr-tab-row");
        assert_eq!(TAB_TITLE_ID, "bbr-tab-title");
        assert_eq!(TAB_FAVICON_ID, "bbr-tab-favicon");
        assert_eq!(TAB_CLOSE_ID, "bbr-tab-close");
        assert_eq!(TAB_STATE_ID, "bbr-tab-state");
        assert_eq!(TAB_PIN_ID, "bbr-tab-pin");
        assert_eq!(DRAG_TICK_ID, "bbr-drag-tick");
    }

    #[test]
    fn row_visuals_round_trip_their_fields() {
        let visuals = RowVisuals {
            index: 3,
            title: "Example".into(),
            state: TabVisualState::Selected,
            favicon: Some(FaviconSpec {
                letter: 'E',
                hue: 2,
            }),
            pinned: true,
            show_close: true,
            reveal_close: true,
        };
        assert_eq!(visuals.index, 3);
        assert_eq!(visuals.title, "Example");
        assert_eq!(visuals.state, TabVisualState::Selected);
        assert_eq!(
            visuals.favicon,
            Some(FaviconSpec {
                letter: 'E',
                hue: 2
            })
        );
        assert!(visuals.pinned);
        assert!(visuals.show_close);
        assert!(visuals.reveal_close);
        // Clone/equality: the model is plain data (anchor model contract).
        assert_eq!(visuals, visuals.clone());
        assert_ne!(
            visuals,
            RowVisuals {
                index: 3,
                title: "Example".into(),
                state: TabVisualState::Default,
                favicon: Some(FaviconSpec {
                    letter: 'E',
                    hue: 2
                }),
                pinned: true,
                show_close: true,
                reveal_close: true,
            }
        );
    }

    #[test]
    fn state_composition_wash_resolution_matches_section_10() {
        // The §10 winner resolution: the more specific state already won in
        // the caller (error > loading > active > selected > hover > default,
        // with the drag machine replacing the composition). The component
        // resolves the winner's inset-surface wash:
        let theme = Theme::new(
            ThemeMode::Dark,
            crate::theme::ContrastMode::Standard,
            crate::theme::Density::Normal,
        );
        let p = &theme.palette;
        // §10.7 error surface wins over every other surface.
        assert_eq!(row_wash(p, TabVisualState::Error), p.status_error_surface);
        // §10.4 selected and §10.9 drag-target share the selection tint.
        assert_eq!(row_wash(p, TabVisualState::Selected), p.tint_selected);
        assert_eq!(row_wash(p, TabVisualState::DragTarget), p.tint_selected);
        // §10.3 active wash beats §10.2 hover wash.
        assert_eq!(row_wash(p, TabVisualState::Active), p.wash_active);
        assert_eq!(row_wash(p, TabVisualState::Hover), p.wash_hover);
        // Loading keeps the sidebar material (its visual is the arc +
        // dimmed title); default/dragging/disabled/attention/focused stay
        // on the sidebar material.
        for state in [
            TabVisualState::Default,
            TabVisualState::Loading,
            TabVisualState::Attention,
            TabVisualState::Disabled,
            TabVisualState::Dragging,
            TabVisualState::Focused,
        ] {
            assert_eq!(row_wash(p, state), p.surface_sidebar, "{state:?}");
        }
        assert_ne!(p.wash_active, p.surface_sidebar);
        assert_ne!(p.wash_hover, p.surface_sidebar);
    }

    #[test]
    fn attention_dot_renders_only_for_the_attention_state() {
        // §10.6: the amber dot lives in the trailing state slot and only
        // while the row is in the attention state (the §10 composition
        // winner — attention never rides error/loading/selected).
        let theme = Theme::new(
            ThemeMode::Dark,
            crate::theme::ContrastMode::Standard,
            crate::theme::Density::Normal,
        );
        assert!(attention_visible(TabVisualState::Attention));
        for state in [
            TabVisualState::Default,
            TabVisualState::Hover,
            TabVisualState::Active,
            TabVisualState::Selected,
            TabVisualState::Loading,
            TabVisualState::Error,
            TabVisualState::Disabled,
            TabVisualState::Dragging,
            TabVisualState::DragTarget,
            TabVisualState::Focused,
        ] {
            assert!(!attention_visible(state), "{state:?}");
        }
        // The slot is always present (the stable §15 id) with the dot only
        // for attention; the slot size is the fixed density state slot.
        let _slot = row_state_slot(&theme, 0, TabVisualState::Attention);
        let _quiet = row_state_slot(&theme, 0, TabVisualState::Default);
    }
}
