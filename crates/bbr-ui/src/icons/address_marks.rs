//! Blueprint D03 (DESIGN §7/§11.5) address identity marks: the not-secure
//! warning triangle, the Settings gear and the local-file document. Same
//! geometric family as the rest of §7 — one stroke weight, square joins,
//! drawn paths scaled from the slot size.

use gpui::{point, px, PathBuilder, Pixels, Rgba, Window};

use super::paint::{paint_filled_circle, paint_path, paint_ring};

/// The warning triangle (apex up) with its exclamation mark.
pub(super) fn paint_warning(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let (half, top, bottom) = (size * 0.42, cy - size * 0.36, cy + size * 0.34);
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(cx), px(top)));
    path.line_to(point(px(cx + half), px(bottom)));
    path.line_to(point(px(cx - half), px(bottom)));
    path.close();
    path.move_to(point(px(cx), px(cy - size * 0.12)));
    path.line_to(point(px(cx), px(cy + size * 0.08)));
    paint_path(window, path, color);
    paint_filled_circle(window, cx, cy + size * 0.2, f32::from(stroke) * 0.7, color);
}

/// The gear: a hub ring with six radial teeth.
pub(super) fn paint_gear(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    paint_ring(window, cx, cy, size * 0.26, stroke, color);
    paint_ring(window, cx, cy, size * 0.09, stroke, color);
    let mut path = PathBuilder::stroke(stroke);
    for tooth in 0..6 {
        let angle = (tooth as f32 * 60.0).to_radians();
        let (sin, cos) = angle.sin_cos();
        path.move_to(point(px(cx + cos * size * 0.3), px(cy + sin * size * 0.3)));
        path.line_to(point(
            px(cx + cos * size * 0.42),
            px(cy + sin * size * 0.42),
        ));
    }
    paint_path(window, path, color);
}

/// The document: a page with a folded top-right corner.
pub(super) fn paint_document(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let (left, right) = (cx - size * 0.28, cx + size * 0.28);
    let (top, bottom) = (cy - size * 0.38, cy + size * 0.38);
    let fold = size * 0.2;
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(left), px(top)));
    path.line_to(point(px(right - fold), px(top)));
    path.line_to(point(px(right), px(top + fold)));
    path.line_to(point(px(right), px(bottom)));
    path.line_to(point(px(left), px(bottom)));
    path.close();
    path.move_to(point(px(right - fold), px(top)));
    path.line_to(point(px(right - fold), px(top + fold)));
    path.line_to(point(px(right), px(top + fold)));
    paint_path(window, path, color);
}
