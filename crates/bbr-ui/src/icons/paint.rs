//! DESIGN.md §7 icon paint primitives — the GPUI vector/quad painters the
//! catalog uses (rotated diagonal strokes, open paths, rings, filled
//! circles, and the rounded-square chassis ring). Split from `icons.rs` so
//! the catalog file stays under the ~500-line module budget, mirroring the
//! pure-geometry split that [`glyphs`](super::glyphs) already provides.
//!
//! The static geometries tessellate unconditionally; on an unexpected
//! failure the glyph degrades away instead of panicking the paint phase.

use gpui::{
    point, px, quad, size, transparent_black, BorderStyle, Bounds, FillOptions, FillRule,
    PathBuilder, PathStyle, Pixels, Rgba, Window,
};

/// Cubic control distance of a quarter circle (4/3 · (√2 − 1)).
const QUARTER_ARC: f32 = 0.552_284_8;

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

/// Stroke centre-relative segments (the pure [`super::glyphs`] geometry)
/// with Paper's round caps and joins. GPUI's stroker only reaches lyon's
/// butt/miter defaults from `bbr-ui`'s allowed dependencies, so the strokes
/// are filled as a non-zero union of [`super::glyphs::Capsule`]s instead.
pub(super) fn paint_segments(
    window: &mut Window,
    stroke: Pixels,
    color: Rgba,
    cx: f32,
    cy: f32,
    segments: &[super::glyphs::Segment],
) {
    let mut path = PathBuilder::fill().with_style(PathStyle::Fill(
        FillOptions::default().with_fill_rule(FillRule::NonZero),
    ));
    let r = f32::from(stroke) / 2.0;
    let at = |(x, y): (f32, f32)| point(px(cx + x), px(cy + y));
    let off = |(x, y): (f32, f32), (dx, dy): (f32, f32), k: f32| at((x + dx * k, y + dy * k));
    for &segment in segments {
        let Some(c) = super::glyphs::capsule(segment, r) else {
            continue;
        };
        let k = QUARTER_ARC;
        path.move_to(at(c.start_plus));
        path.line_to(at(c.end_plus));
        path.cubic_bezier_to(
            at(c.end_tip),
            off(c.end_plus, c.along, k),
            off(c.end_tip, c.side, k),
        );
        path.cubic_bezier_to(
            at(c.end_minus),
            off(c.end_tip, c.side, -k),
            off(c.end_minus, c.along, k),
        );
        path.line_to(at(c.start_minus));
        path.cubic_bezier_to(
            at(c.start_tip),
            off(c.start_minus, c.along, -k),
            off(c.start_tip, c.side, -k),
        );
        path.cubic_bezier_to(
            at(c.start_plus),
            off(c.start_tip, c.side, k),
            off(c.start_plus, c.along, -k),
        );
        path.close();
    }
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
