//! §18.5 download finished: the succeeded row's `status.ok` check draws its
//! stroke and one ring expands and fades, within `dur:emphasis`, starting
//! only after the success is committed (Reduce Motion: the check simply
//! appears, no ring).

use gpui::{div, prelude::*, px, AnimationExt, AnyElement, Pixels, SharedString};

use crate::icons::spinner::check_progress_canvas;
use crate::icons::Icon;
use crate::theme::motion::{download_check_sample, Moment};
use crate::theme::{Radius, Theme};

/// The succeeded row's status glyph: the static check, or the drawing
/// check with its ring while the moment keyed `check` plays.
pub(super) fn succeeded_glyph(theme: &Theme, size: Pixels, check: Option<u64>) -> AnyElement {
    let colour = theme.palette.status_ok;
    let (Some(key), Some(plan)) = (check, Moment::DownloadCheck.plan(theme.motion_mode)) else {
        return Icon::Check
            .canvas_with(theme, size, colour)
            .into_any_element();
    };
    let theme = *theme;
    div()
        .size(size)
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .with_animation(
            SharedString::from(format!("bbr-download-check-{key}")),
            plan.animation(),
            move |el, t| {
                let (stroke, growth, ring_opacity) = download_check_sample(t);
                let ring = f32::from(size) + growth;
                let offset = (f32::from(size) - ring) / 2.0;
                el.child(check_progress_canvas(&theme, size, colour, stroke))
                    .child(
                        div()
                            .absolute()
                            .left(px(offset))
                            .top(px(offset))
                            .size(px(ring))
                            .rounded(Radius.full())
                            .border_1()
                            .border_color(colour)
                            .opacity(ring_opacity),
                    )
            },
        )
        .into_any_element()
}
