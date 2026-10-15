//! D4 (§18.1, §18.5): after a committed space switch the chrome tint
//! cross-fades from the source to the destination over `dur:standard`
//! (`ease:standard`). The surface already paints the committed destination
//! tint; this layer — the source tint fading out above that paint and
//! beneath every child — is the whole cross-fade. Geometry, lists, the
//! selected tab and the native view never animate.

use gpui::{div, prelude::*, AnimationExt, AnyElement, Rgba, SharedString};

use crate::theme::motion::Moment;
use crate::theme::Theme;

/// The fading source-tint layer for `surface` (a stable name per chrome
/// surface), or `None` when no fade plays (Reduce Motion: instant).
pub fn tint_fade_layer(
    theme: &Theme,
    surface: &'static str,
    fade: Option<(Rgba, u64)>,
) -> Option<AnyElement> {
    let (from, key) = fade?;
    let plan = Moment::SpaceTintFade.plan(theme.motion_mode)?;
    Some(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(from)
            .with_animation(
                SharedString::from(format!("bbr-space-tint-fade-{surface}-{key}")),
                plan.animation(),
                |el, t| el.opacity(1.0 - t),
            )
            .into_any_element(),
    )
}

/// The same cross-fade for a paint that is not a plain background: `paint`
/// receives the fading source tint and returns the absolute layer drawn in
/// it (the card's corner covers, DESIGN §2), which then fades out over the
/// same plan as the surface's own [`tint_fade_layer`] so both read as one
/// chassis. `None` when no fade plays.
pub fn tint_fade_painted(
    theme: &Theme,
    surface: &'static str,
    fade: Option<(Rgba, u64)>,
    paint: impl FnOnce(Rgba) -> AnyElement,
) -> Option<AnyElement> {
    let (from, key) = fade?;
    let plan = Moment::SpaceTintFade.plan(theme.motion_mode)?;
    Some(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(paint(from))
            .with_animation(
                SharedString::from(format!("bbr-space-tint-fade-{surface}-{key}")),
                plan.animation(),
                |el, t| el.opacity(1.0 - t),
            )
            .into_any_element(),
    )
}
