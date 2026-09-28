//! §18.5 favorite pop: after a favorite is committed, its tile surface pops
//! 0.86 → 1 with overshoot while eight specks in the curated space hues
//! radiate (Reduce Motion: a `dur:fine` `tint.accent` flash). The tile's
//! layout box, icon and listeners never move — only the decoration layers
//! animate, so the committed tile is present on the commit frame.

use gpui::{div, prelude::*, px, relative, AnimationExt, AnyElement, Rgba, SharedString};

use crate::theme::motion::{favorite_pop_sample, Moment};
use crate::theme::Theme;

/// Eight speck directions around the tile centre.
const SPECKS: usize = 8;
/// Speck diameter (pt).
const SPECK: f32 = 3.0;

/// The tile's surface layer (the §18.2 veil) at rest, or popping when
/// `pop` carries the moment key.
pub(super) fn surface_layer(theme: &Theme, radius: gpui::Pixels, pop: Option<u64>) -> AnyElement {
    let veil = theme.palette.surface_favorite;
    let flash_colour = theme.palette.tint_accent;
    let layer = move |scale: f32, flash: f32| {
        let inset = relative((1.0 - scale) / 2.0);
        div()
            .absolute()
            .left(inset)
            .right(inset)
            .top(inset)
            .bottom(inset)
            .rounded(radius)
            .bg(veil)
            .child(
                div()
                    .absolute()
                    .size_full()
                    .rounded(radius)
                    .bg(flash_colour)
                    .opacity(flash),
            )
    };
    let plan = Moment::FavoritePop.plan(theme.motion_mode);
    match (pop, plan) {
        (Some(key), Some(plan)) => layer(1.0, 0.0)
            .with_animation(
                SharedString::from(format!("bbr-fav-pop-surface-{key}")),
                plan.animation(),
                move |_, t| {
                    let (scale, _, _, flash) = favorite_pop_sample(t, plan.reduced);
                    layer(scale, flash)
                },
            )
            .into_any_element(),
        _ => layer(1.0, 0.0).into_any_element(),
    }
}

/// The speck layer of a popping tile (full motion only): eight dots in the
/// curated space identity hues travelling out from the centre and fading.
pub(super) fn speck_layer(theme: &Theme, pop: Option<u64>) -> Option<AnyElement> {
    let key = pop?;
    let plan = Moment::FavoritePop.plan(theme.motion_mode)?;
    if plan.reduced {
        return None;
    }
    let hues: Vec<Rgba> = bbr_domain::SpaceTheme::ALL
        .iter()
        .take(SPECKS)
        .map(|token| {
            crate::theme::space_tint(*token, crate::theme::TintIntensity::Standard, theme).hue
        })
        .collect();
    Some(
        div()
            .absolute()
            .size_full()
            .with_animation(
                SharedString::from(format!("bbr-fav-pop-specks-{key}")),
                plan.animation(),
                move |el, t| {
                    let (_, travel, opacity, _) = favorite_pop_sample(t, false);
                    el.children(hues.iter().enumerate().map(|(index, hue)| {
                        let angle = index as f32 / SPECKS as f32 * std::f32::consts::TAU;
                        div()
                            .absolute()
                            .left(relative(0.5))
                            .top(relative(0.5))
                            .ml(px(travel * angle.cos() - SPECK / 2.0))
                            .mt(px(travel * angle.sin() - SPECK / 2.0))
                            .size(px(SPECK))
                            .rounded(px(SPECK))
                            .bg(*hue)
                            .opacity(opacity)
                    }))
                },
            )
            .into_any_element(),
    )
}
