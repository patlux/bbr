//! The §11.3 paper row of the configuration surface (v2 D6/D7): the three
//! curated intensity steps and the static-grain toggle, beneath the theme
//! swatches. Only curated values exist here: no free strength, no colour
//! field. Each step previews the rail it paints — the draft theme resolved
//! at that step through the one central [`crate::theme::space_tint`] — and
//! carries its label, so the choice is never colour-only.

use bbr_domain::TintIntensity;
use gpui::{div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, Stateful, Window};

use crate::theme::{Radius, Spacing, Theme};

use super::callbacks::{SpaceConfigCallbacks, SpaceConfigSelectIntensity, SpaceConfigToggleGrain};
use super::ids::{space_config_intensity_id, SPACE_CONFIG_GRAIN_ID, SPACE_CONFIG_PAPER_LABEL_ID};
use super::model::SpaceConfigModel;

/// The paper row: `bbr-space-config-paper-label`, one
/// `bbr-space-config-intensity-<step>` option per step, and the
/// `bbr-space-config-grain` toggle.
pub(super) fn paper_row(
    theme: &Theme,
    model: &SpaceConfigModel,
    callbacks: &SpaceConfigCallbacks,
) -> Div {
    let steps = TintIntensity::ALL
        .map(|step| intensity_option(theme, model, step, callbacks.on_select_intensity.clone()));
    div()
        .flex()
        .flex_col()
        .gap(Spacing::sp1())
        .child(
            div()
                .id(SPACE_CONFIG_PAPER_LABEL_ID)
                .debug_selector(|| SPACE_CONFIG_PAPER_LABEL_ID.to_string())
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(theme.palette.ink_faint)
                .child("PAPER"),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(Spacing::sp1())
                .children(steps)
                .child(grain_toggle(
                    theme,
                    model.paper.grain,
                    callbacks.on_toggle_grain.clone(),
                )),
        )
}

/// One intensity step: a rail preview swatch plus the step label; the
/// selected step carries the accent outline + `tint.accent`.
fn intensity_option(
    theme: &Theme,
    model: &SpaceConfigModel,
    step: TintIntensity,
    on_select: SpaceConfigSelectIntensity,
) -> Stateful<Div> {
    let p = theme.palette;
    let selected = model.paper.intensity == step;
    let preview = crate::theme::space_tint(model.theme, step, theme).chrome;
    let id = space_config_intensity_id(step);
    option(theme, id, selected)
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                on_select(step, window, cx)
            },
        )
        .child(
            div()
                .size(px(12.0))
                .rounded(Radius.small())
                .bg(preview)
                .border_1()
                .border_color(p.border_strong),
        )
        .child(label(theme, step.label(), selected))
}

/// The static-grain toggle: an option tile reading `Grain · On/Off`.
fn grain_toggle(theme: &Theme, on: bool, on_toggle: SpaceConfigToggleGrain) -> Stateful<Div> {
    option(theme, SPACE_CONFIG_GRAIN_ID.to_string(), on)
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| on_toggle(window, cx),
        )
        .child(label(
            theme,
            if on { "Grain · On" } else { "Grain · Off" },
            on,
        ))
}

/// The shared option tile of the paper row (the identity option family).
fn option(theme: &Theme, id: String, selected: bool) -> Stateful<Div> {
    let p = theme.palette;
    let selector = id.clone();
    div()
        .id(id)
        .debug_selector(move || selector.clone())
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp1())
        .px(Spacing::sp2())
        .py(Spacing::sp1())
        .rounded(Radius.small())
        .cursor_pointer()
        .bg(if selected {
            p.tint_accent
        } else {
            p.surface_field
        })
        .border_1()
        .border_color(if selected { p.accent } else { p.hairline })
}

fn label(theme: &Theme, text: &'static str, selected: bool) -> Div {
    let p = theme.palette;
    div()
        .text_size(theme.type_.label.size)
        .font_weight(theme.type_.label.weight)
        .text_color(if selected { p.ink } else { p.ink_secondary })
        .child(text)
}
