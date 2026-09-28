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
