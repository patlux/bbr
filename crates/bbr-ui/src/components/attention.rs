//! §10.6/§18.5 attention: the trailing-slot dot of a row in the
//! `attention` state and its one-shot two-beat pulse (split from the tab row
//! anatomy to keep both files within the size budget).

use gpui::{div, prelude::*, px, AnimationExt, Styled};

use crate::state::TabVisualState;
use crate::theme::{Radius, Theme};

/// §10.6: whether the attention dot shows in the trailing state slot. The
/// dot appears only while the composed §10 winner is `Attention` (the
/// caller already resolved error > loading > pressed > selected > hover >
/// default, so attention never rides another state's row).
pub fn attention_visible(state: TabVisualState) -> bool {
    state == TabVisualState::Attention
}

/// §10.6/§18.5 attention dot: an 8 pt `status.attention` dot in the row's
/// trailing state slot that plays the two-beat pulse once when the row
/// enters `attention` (dot scale 1 → 1.45 → 1 plus one ring 4 → 16 pt
/// fading out; Reduce Motion: opacity 1 → 0.45 → 1). The pulse is a
/// one-shot keyed to the row, so it replays only when the state is entered
/// again, and rests on the static dot. The §10.6 badge always comes with
/// its reason line in the status area (app-owned).
pub(crate) fn attention_dot(theme: &Theme, index: usize) -> gpui::AnyElement {
    const DOT: f32 = 8.0;
    let color = theme.palette.status_attention;
    let dot = move |scale: f32, opacity: f32| {
        div()
            .size(px(DOT * scale))
            .rounded(Radius.full())
            .bg(color)
            .opacity(opacity)
    };
    let Some(plan) = crate::theme::motion::Moment::AttentionPulse.plan(theme.motion_mode) else {
        return dot(1.0, 1.0).into_any_element();
    };
    div()
        .size(px(DOT))
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .with_animation(
            ("bbr-attention-pulse", index),
            plan.animation(),
            move |el, t| {
                let (scale, opacity, ring, ring_opacity) =
                    crate::theme::motion::attention_sample(t, plan.reduced);
                el.child(dot(scale, opacity)).when(ring > 0.0, |el| {
                    let offset = (DOT - ring) / 2.0;
                    el.child(
                        div()
                            .absolute()
                            .left(px(offset))
                            .top(px(offset))
                            .size(px(ring))
                            .rounded(Radius.full())
                            .border_1()
                            .border_color(color)
                            .opacity(ring_opacity),
                    )
                })
            },
        )
        .into_any_element()
}
