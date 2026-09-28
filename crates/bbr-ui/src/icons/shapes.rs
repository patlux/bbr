//! S12 Paper glyph shapes: the folder silhouette (a body wider than tall
//! with a raised tab stepping down to the top edge) in its closed and open
//! variants, and the search magnifier the command field leads with. Pure
//! geometry lives in [`super::glyphs`]; these paint it.

use gpui::{point, px, PathBuilder, Pixels, Rgba, Window};

use super::glyphs::{folder_shape, search_geometry, FolderShape};
use super::paint::{paint_path, paint_ring};

/// The closed folder: tab on the top-left, stepping down to the body top.
pub(super) fn paint_folder_closed(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let FolderShape {
        half_w,
        half_h,
        tab_w,
        tab_h,
        slant,
    } = folder_shape(size);
    let (left, right, top, bottom) = (cx - half_w, cx + half_w, cy - half_h, cy + half_h);
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(left), px(bottom)));
    path.line_to(point(px(left), px(top)));
    path.line_to(point(px(left + tab_w), px(top)));
    path.line_to(point(px(left + tab_w + slant), px(top + tab_h)));
    path.line_to(point(px(right), px(top + tab_h)));
    path.line_to(point(px(right), px(bottom)));
    path.close();
    paint_path(window, path, color);
}

/// The open folder: the back of the body with its tab, and the front flap
/// tilted open over it (the §11.11 expanded state, never color-only).
pub(super) fn paint_folder_open(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let FolderShape {
        half_w,
        half_h,
        tab_w,
        tab_h,
        slant,
    } = folder_shape(size);
    let (left, right, top, bottom) = (cx - half_w, cx + half_w, cy - half_h, cy + half_h);
    let flap_top = cy - half_h * 0.05;
    let back_right = right - half_w * 0.22;
    // Back: left edge, the tab, the body top down to the flap.
    let mut back = PathBuilder::stroke(stroke);
    back.move_to(point(px(left), px(bottom)));
    back.line_to(point(px(left), px(top)));
    back.line_to(point(px(left + tab_w), px(top)));
    back.line_to(point(px(left + tab_w + slant), px(top + tab_h)));
    back.line_to(point(px(back_right), px(top + tab_h)));
    back.line_to(point(px(back_right), px(flap_top)));
    paint_path(window, back, color);
    // Front flap: a parallelogram leaning right.
    let lean = half_w * 0.28;
    let mut flap = PathBuilder::stroke(stroke);
    flap.move_to(point(px(left), px(bottom)));
    flap.line_to(point(px(left + lean), px(flap_top)));
    flap.line_to(point(px(right), px(flap_top)));
    flap.line_to(point(px(right - lean), px(bottom)));
    flap.close();
    paint_path(window, flap, color);
}

/// The search magnifier: a lens ring up-left and the handle down-right.
pub(super) fn paint_search(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    size: f32,
) {
    let (radius, offset, handle) = search_geometry(size);
    let (lx, ly) = (cx - offset, cy - offset);
    paint_ring(window, lx, ly, radius, stroke, color);
    let diagonal = std::f32::consts::FRAC_1_SQRT_2;
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(
        px(lx + radius * diagonal),
        px(ly + radius * diagonal),
    ));
    path.line_to(point(
        px(lx + (radius + handle) * diagonal),
        px(ly + (radius + handle) * diagonal),
    ));
    paint_path(window, path, color);
}
