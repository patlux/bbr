//! §18.5 favorite pop (S12, after the Paper `micro_favorite` study): after a
//! favorite is committed, its tile surface pops 0.86 → 1 with overshoot, an
//! `accent` outline breathes out from the tile, and ten specks in the
//! accent and curated space hues burst past the tile's edge, shrinking as
//! they fade (Reduce Motion: a `dur:fine` `tint.accent` flash). The tile's
//! layout box, icon and listeners never move — only the decoration layers
//! animate, so the committed tile is present on the commit frame.

use gpui::{div, prelude::*, px, relative, AnimationExt, AnyElement, Rgba, SharedString};

use crate::theme::motion::{Easing, Moment};
use crate::theme::{Radius, Theme};

/// Ten specks around the tile centre, the first 0.3 rad past east.
const SPECKS: usize = 10;
const SPECK_OFFSET: f32 = 0.3;

/// One favorite-pop frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopSample {
    /// The tile surface's scale.
    pub scale: f32,
    /// The specks' distance from the tile centre (pt, horizontal; the
    /// burst is 0.8 as tall as wide).
    pub travel: f32,
    /// The specks' radius (pt).
    pub speck: f32,
    /// The specks' opacity.
    pub opacity: f32,
    /// The accent outline's outset (pt) and opacity.
    pub ring: (f32, f32),
    /// The reduced variant's `tint.accent` flash opacity.
    pub flash: f32,
}

/// §18.5 favorite pop at (linear) moment time `t`. Full motion: the tile
/// pops 0.86 → 1 with a back-ease overshoot over the first 60 %; the specks
/// burst 18 → 44 pt (`ease:standard`; they start clear of the icon) while
/// shrinking 3 → 0.6 pt and fading; the outline grows 0 → 3 pt from 60 %
/// opacity to clear. Reduced: only the flash, fading 1 → 0 at scale 1.
pub fn pop_sample(t: f32, reduced: bool) -> PopSample {
    let t = t.clamp(0.0, 1.0);
    if reduced {
        return PopSample {
            scale: 1.0,
            travel: 0.0,
            speck: 0.0,
            opacity: 0.0,
            ring: (0.0, 0.0),
            flash: 1.0 - t,
        };
    }
    // Back-ease-out (overshoot ≈ 10 %), mapped 0.86 → 1.
    let u = (t / 0.6).min(1.0);
    let (c1, c3) = (1.701_58_f32, 2.701_58_f32);
    let back = 1.0 + c3 * (u - 1.0).powi(3) + c1 * (u - 1.0).powi(2);
    PopSample {
        scale: 0.86 + 0.14 * back,
        travel: 18.0 + 26.0 * Easing::Standard.apply(t),
        speck: 2.4 * (1.0 - t) + 0.6,
        opacity: 1.0 - t,
        ring: (3.0 * t, 0.6 * (1.0 - t)),
        flash: 0.0,
    }
}

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
                    let sample = pop_sample(t, plan.reduced);
                    layer(sample.scale, sample.flash)
                },
            )
            .into_any_element(),
        _ => layer(1.0, 0.0).into_any_element(),
    }
}

/// The burst layer of a popping tile (full motion only): the accent
/// outline and ten specks travelling out past the tile and fading.
pub(super) fn speck_layer(
    theme: &Theme,
    radius: gpui::Pixels,
    pop: Option<u64>,
) -> Option<AnyElement> {
    let key = pop?;
    let plan = Moment::FavoritePop.plan(theme.motion_mode)?;
    if plan.reduced {
        return None;
    }
    let accent = theme.palette.accent;
    let hue =
        |token| crate::theme::space_tint(token, crate::theme::TintIntensity::Standard, theme).hue;
    use bbr_domain::SpaceTheme as S;
    let palette: [Rgba; 8] = [
        accent,
        hue(S::Cobalt),
        hue(S::Amber),
        hue(S::Plum),
        hue(S::Pine),
        hue(S::Fuchsia),
        hue(S::Violet),
        hue(S::Terracotta),
    ];
    Some(
        div()
            .absolute()
            .size_full()
            .with_animation(
                SharedString::from(format!("bbr-fav-pop-specks-{key}")),
                plan.animation(),
                move |el, t| {
                    let sample = pop_sample(t, false);
                    let (outset, ring_opacity) = sample.ring;
                    let outline = div()
                        .absolute()
                        .left(px(-outset))
                        .right(px(-outset))
                        .top(px(-outset))
                        .bottom(px(-outset))
                        .rounded(radius + px(2.0))
                        .border(px(1.5))
                        .border_color(accent)
                        .opacity(ring_opacity);
                    let specks = (0..SPECKS).map(move |index| {
                        let angle =
                            index as f32 / SPECKS as f32 * std::f32::consts::TAU + SPECK_OFFSET;
                        let r = sample.speck;
                        div()
                            .absolute()
                            .left(relative(0.5))
                            .top(relative(0.5))
                            .ml(px(sample.travel * angle.cos() - r))
                            .mt(px(0.8 * sample.travel * angle.sin() - r))
                            .size(px(2.0 * r))
                            .rounded(Radius.full())
                            .bg(palette[index % palette.len()])
                            .opacity(sample.opacity)
                    });
                    el.child(outline).children(specks)
                },
            )
            .into_any_element(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn the_pop_overshoots_and_the_burst_leaves_the_tile() {
        let start = pop_sample(0.0, false);
        assert!(close(start.scale, 0.86) && close(start.travel, 18.0));
        assert!(close(start.opacity, 1.0) && close(start.speck, 3.0));
        assert_eq!(start.ring, (0.0, 0.6));
        assert_eq!(start.flash, 0.0);
        let peak = (1..20)
            .map(|step| pop_sample(step as f32 / 20.0, false).scale)
            .fold(0.0, f32::max);
        assert!(peak > 1.0, "the pop overshoots");
        let end = pop_sample(1.0, false);
        assert!(close(end.scale, 1.0) && close(end.travel, 44.0));
        assert!(close(end.opacity, 0.0) && close(end.speck, 0.6));
        assert!(close(end.ring.0, 3.0) && close(end.ring.1, 0.0));
        // The specks end past a favorite tile's half-width (≤ 38 pt).
        assert!(end.travel > 38.0);
        // Reduced: only the flash, at rest scale.
        let reduced = pop_sample(0.0, true);
        assert_eq!(
            (reduced.scale, reduced.opacity, reduced.flash),
            (1.0, 0.0, 1.0)
        );
    }
}
