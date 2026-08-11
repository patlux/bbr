//! DESIGN.md §7 icon paint primitives — the GPUI vector/quad painters the
//! catalog uses (rotated diagonal strokes, open paths, rings, filled
//! circles, and the rounded-square chassis ring). Split from `icons.rs` so
//! the catalog file stays under the ~500-line module budget, mirroring the
//! pure-geometry split that [`glyphs`](super::glyphs) already provides.
//!
//! The static geometries tessellate unconditionally; on an unexpected
//! failure the glyph degrades away instead of panicking the paint phase.

use gpui::{
    point, px, quad, size, transparent_black, BorderStyle, Bounds, PathBuilder, Pixels, Rgba,
    Window,
};

/// One rotated diagonal stroke (close × half). The line is built in local
/// coordinates, rotated about the glyph center, then translated there.
pub(super) fn paint_stroke(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    arm: f32,
    angle: f32,
) {
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(-arm), px(0.0)));
    path.line_to(point(px(arm), px(0.0)));
    path.rotate(angle);
    path.translate(point(px(cx), px(cy)));
    paint_path(window, path, color);
}

pub(super) fn paint_path(window: &mut Window, path: PathBuilder, color: Rgba) {
    // The static geometries tessellate unconditionally; on an unexpected
    // failure the glyph degrades away instead of panicking the paint phase.
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

/// A 1.5 pt ring at `radius` around the given center (border stroke centered
/// on the circle outline, transparent hole).
pub(super) fn paint_ring(
    window: &mut Window,
    cx: f32,
    cy: f32,
    radius: f32,
    stroke: Pixels,
    color: Rgba,
) {
    let diameter = radius * 2.0;
    window.paint_quad(quad(
        Bounds::new(
            point(px(cx - radius), px(cy - radius)),
            size(px(diameter), px(diameter)),
        ),
        px(radius),
        transparent_black(),
        stroke,
        color,
        BorderStyle::default(),
    ));
}

/// A filled circle at `radius` around the given center.
pub(super) fn paint_filled_circle(window: &mut Window, cx: f32, cy: f32, radius: f32, color: Rgba) {
    let diameter = radius * 2.0;
    window.paint_quad(quad(
        Bounds::new(
            point(px(cx - radius), px(cy - radius)),
            size(px(diameter), px(diameter)),
        ),
        px(radius),
        color,
        px(0.0),
        transparent_black(),
        BorderStyle::default(),
    ));
}

/// A rounded-square ring (the chassis frame): `half` is half the side, the
/// border stroke is centered on the outline.
pub(super) fn paint_rounded_square_ring(
    window: &mut Window,
    cx: f32,
    cy: f32,
    half: f32,
    corner_radius: f32,
    stroke: Pixels,
    color: Rgba,
) {
    window.paint_quad(quad(
        Bounds::new(
            point(px(cx - half), px(cy - half)),
            size(px(half * 2.0), px(half * 2.0)),
        ),
        px(corner_radius),
        transparent_black(),
        stroke,
        color,
        BorderStyle::default(),
    ));
}
