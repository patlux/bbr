//! §12 profile chip identity and the §11.3 space rows.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled,
    Window,
};

use crate::palette::ProfileHueRamp;
use crate::theme::{Radius, Spacing, Theme};

use super::{ProfileChipSpec, SpaceRowVisuals, SpaceVisualState};

/// The §15 semantic id of a profile chip, keyed to the *profile* index
/// (DESIGN.md §12) — never the space index. The sidebar renders one chip per
/// space row, so a profile's spaces repeat this id; the testkit snapshot
/// exposes each chip id once per profile. This function is the single keying
/// both the render and the snapshot share, so the chip a row renders can
/// never drift from the E2E contract when space and profile indices diverge.
pub fn profile_chip_id(profile_index: usize) -> SharedString {
    SharedString::from(format!("bbr-profile-chip-{profile_index}"))
}

/// §12 profile chip: 20 pt square (18 at S), `radius:sm`, filled with the
/// profile's hue tile and a 1–2 character monogram. The letter derives from
/// the profile title so identity is readable per row without a header.
pub fn profile_chip(theme: &Theme, profile_index: usize, spec: &ProfileChipSpec) -> Stateful<Div> {
    let size = if theme.density == crate::theme::Density::Small {
        px(18.0)
    } else {
        px(20.0)
    };
    let tile = ProfileHueRamp::tile(spec.hue, theme.mode);
    div()
        .id(profile_chip_id(profile_index))
        .size(size)
        .rounded(Radius.small())
        .bg(tile.tile)
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(tile.letter)
        .child(spec.letter.clone())
}

/// §11.3 space row anatomy:
/// `[profile chip 20] · sp:2 · [name, trunc] · [count, type:meta tnum]`,
/// `density:space-row` tall. The active space shows the accent rail exactly
/// like a selected tab; a row a drag currently targets shows the §10.9
/// drag-target outline + tint. `on_select` is the production view listener.
pub fn space_row(
    theme: &Theme,
    visuals: &SpaceRowVisuals,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let selected = visuals.state == SpaceVisualState::Selected;
    let target = visuals.state == SpaceVisualState::DragTarget;
    div()
        .id(("bbr-space-row", visuals.index))
        .h(theme.density.space_row())
        .px(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.medium())
        .bg(if selected {
            p.tint_selected
        } else {
            p.surface_sidebar
        })
        .cursor_pointer()
        .when(target, |el| {
            // §10.9 drag-target: 1 pt accent outline + tint; the destination
            // profile chip (§12.3) is already the row's own chip, so the
            // target identity reads without a second glyph.
            el.border_1().border_color(p.accent).bg(p.tint_selected)
        })
        .when(selected, |el| {
            el.child(
                div()
                    .w(px(3.0))
                    .h(theme.density.space_row() - px(6.0))
                    .rounded(Radius.small())
                    .bg(p.accent),
            )
        })
        .child(profile_chip(theme, visuals.profile_index, &visuals.chip))
        .child(
            div()
                .id(("bbr-space-title", visuals.index))
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(if selected { p.ink } else { p.ink_secondary })
                .child(visuals.title.clone()),
        )
        .child(
            div()
                .id(("bbr-space-count", visuals.index))
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                .child(visuals.count.to_string()),
        )
        .on_mouse_down(MouseButton::Left, on_select)
}

/// §11.3/§11.5 New Space affordance at the foot of the space region: focuses
/// the command bar in NewSpace mode (the pointer half of `Cmd-Shift-N`).
pub fn space_new_row(
    theme: &Theme,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-space-new")
        .h(theme.density.space_row())
        .px(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.medium())
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, on_select)
        .child(
            div()
                .size(px(20.0))
                .rounded(Radius.small())
                .border_1()
                .border_color(theme.palette.hairline)
                .flex()
                .items_center()
                .justify_center()
                .text_size(theme.type_.label.size)
                .text_color(theme.palette.ink_secondary)
                .child("+"),
        )
        .child(
            div()
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .text_color(theme.palette.ink_secondary)
                .child("New Space"),
        )
}
