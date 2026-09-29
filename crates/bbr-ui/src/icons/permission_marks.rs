//! Blueprint D11 (boards M-8, M-9, DESIGN §7): the permission marks — the
//! camera (a body with a lens ring and a viewfinder wedge) and the
//! microphone (a capsule on a cradle). A blocked mark adds the §7 slash.
//! Same geometric family: one stroke weight, drawn paths scaled from the
//! slot size.

use gpui::{point, px, PathBuilder, Pixels, Rgba, Window};

use super::paint::{paint_path, paint_ring};

/// The camera body, lens and viewfinder.
pub(super) fn paint_camera(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let (l, r, t, b) = (
        cx - size * 0.42,
        cx + size * 0.14,
        cy - size * 0.24,
        cy + size * 0.24,
    );
    let c = size * 0.06;
    let mut body = PathBuilder::stroke(stroke);
    body.move_to(point(px(l + c), px(t)));
    body.line_to(point(px(r - c), px(t)));
    body.line_to(point(px(r), px(t + c)));
    body.line_to(point(px(r), px(b - c)));
    body.line_to(point(px(r - c), px(b)));
    body.line_to(point(px(l + c), px(b)));
    body.line_to(point(px(l), px(b - c)));
    body.line_to(point(px(l), px(t + c)));
    body.close();
    paint_path(window, body, color);
    let mut finder = PathBuilder::stroke(stroke);
    finder.move_to(point(px(r), px(cy - size * 0.06)));
    finder.line_to(point(px(cx + size * 0.42), px(cy - size * 0.2)));
    finder.line_to(point(px(cx + size * 0.42), px(cy + size * 0.2)));
    finder.line_to(point(px(r), px(cy + size * 0.06)));
    paint_path(window, finder, color);
    paint_ring(window, (l + r) / 2.0, cy, size * 0.1, stroke, color);
}

/// The microphone capsule on its cradle and stand.
pub(super) fn paint_microphone(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let (half, top, bottom) = (size * 0.13, cy - size * 0.4, cy + size * 0.08);
    let mut capsule = PathBuilder::stroke(stroke);
    capsule.move_to(point(px(cx - half), px(top + half)));
    capsule.cubic_bezier_to(
        point(px(cx + half), px(top + half)),
        point(px(cx - half), px(top - half * 0.3)),
        point(px(cx + half), px(top - half * 0.3)),
    );
    capsule.line_to(point(px(cx + half), px(bottom - half)));
    capsule.cubic_bezier_to(
        point(px(cx - half), px(bottom - half)),
        point(px(cx + half), px(bottom + half * 0.3)),
        point(px(cx - half), px(bottom + half * 0.3)),
    );
    capsule.close();
    paint_path(window, capsule, color);
    let arm = size * 0.27;
    let mut cradle = PathBuilder::stroke(stroke);
    cradle.move_to(point(px(cx - arm), px(cy - size * 0.06)));
    cradle.cubic_bezier_to(
        point(px(cx + arm), px(cy - size * 0.06)),
        point(px(cx - arm), px(cy + size * 0.28)),
        point(px(cx + arm), px(cy + size * 0.28)),
    );
    cradle.move_to(point(px(cx), px(cy + size * 0.2)));
    cradle.line_to(point(px(cx), px(cy + size * 0.4)));
    paint_path(window, cradle, color);
}

/// The blocked slash across the slot.
pub(super) fn paint_slash(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let mut slash = PathBuilder::stroke(stroke);
    slash.move_to(point(px(cx - size * 0.4), px(cy - size * 0.4)));
    slash.line_to(point(px(cx + size * 0.4), px(cy + size * 0.4)));
    paint_path(window, slash, color);
}
