//! §18.5 ⌘T after-commit moments: freshly published results fade in with a
//! 20 ms stagger and a 4 pt rise (Reduce Motion: fade only), and the cursor
//! highlight glides between rows in `dur:micro` while the selected result is
//! already committed (Reduce Motion: it snaps). Only the centered palette
//! plays them; the ⌘L overlay renders at rest.

use gpui::{div, prelude::*, px, AnimationExt, AnyElement, Div, SharedString, Stateful};

use crate::theme::motion::{result_entrance_sample, Moment};

use super::model::{CommandModel, CommandPresentation};
use super::render::CommandAnatomy;

fn palette(model: &CommandModel) -> bool {
    model.presentation == CommandPresentation::CenteredPalette
}

/// The running glide plan, if the cursor is gliding right now.
fn glide(model: &CommandModel) -> Option<(usize, u64, crate::theme::motion::MomentPlan)> {
    let (from, key) = model.glide.filter(|_| palette(model))?;
    let plan = Moment::CommandCursor.plan(model.theme.motion_mode)?;
    Some((from, key, plan))
}

/// Whether the selected row paints its own cursor surface (it does unless
/// the gliding highlight is carrying the cursor).
pub(super) fn row_paints_cursor(model: &CommandModel) -> bool {
    glide(model).is_none()
}

/// A result row, entering with the stagger while the entrance plays.
pub(super) fn entering_row(model: &CommandModel, index: usize, row: Stateful<Div>) -> AnyElement {
    let entrance = model.entrance.filter(|_| palette(model));
    let plan = Moment::CommandResults.plan(model.theme.motion_mode);
    let (Some(key), Some(plan)) = (entrance, plan) else {
        return row.into_any_element();
    };
    row.relative()
        .with_animation(
            SharedString::from(format!("bbr-palette-in-{key}-{index}")),
            plan.animation(),
            move |el, t| {
                let (opacity, rise) = result_entrance_sample(t, index, plan.reduced);
                el.opacity(opacity).top(px(rise))
            },
        )
        .into_any_element()
}

/// The gliding cursor highlight: an inset `tint.accent` row surface moving
/// from the previous row to the committed one, beneath the rows.
pub(super) fn glide_highlight(model: &CommandModel, anatomy: CommandAnatomy) -> Option<AnyElement> {
    let (from, key, plan) = glide(model)?;
    let to = model.selected;
    let pad = f32::from(anatomy.list_padding);
    let row = f32::from(anatomy.row_height);
    let colour = model.theme.palette.tint_accent;
    let highlight = move |top: f32| -> Div {
        div()
            .absolute()
            .left(px(pad))
            .right(px(pad))
            .top(px(top))
            .h(px(row))
            .rounded(anatomy.row_radius)
            .bg(colour)
    };
    Some(
        highlight(pad + row * from as f32)
            .with_animation(
                SharedString::from(format!("bbr-palette-glide-{key}")),
                plan.animation(),
                move |_, t| highlight(pad + row * (from as f32 + (to as f32 - from as f32) * t)),
            )
            .into_any_element(),
    )
}
