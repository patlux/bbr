//! DESIGN.md §7/§11.3 identity-glyph path catalog — the curated
//! [`BbrGlyph`] set drawn in the §7 geometric family (1.5 pt stroke at
//! N/XL, 1.25 pt at S, squared corners, rest luminance at `ink.secondary`).
//!
//! The domain owns the *catalog* (the curated names); this module owns the
//! *paths* — every curated glyph resolves to a drawn path here, so footer
//! Space chips and the space-config glyph picker render real §7 geometry
//! instead of two-letter badges. A non-curated identity string (nothing in
//! `BbrGlyph::ALL`) still falls back to the §7 two-letter badge
//! ([`crate::icons::glyph_fallback_badge`]) — the badge is the
//! missing-glyph fallback, never the curated rendering.
//!
//! All geometry is stroke-only (no fills, no emoji substitutes, §14), kept
//! inside the tile box with the arm/radius conventions the rest of the §7
//! catalog uses.

use bbr_domain::BbrGlyph;
use gpui::{point, px, Bounds, PathBuilder, Pixels, Rgba, Window};

use super::glyphs::icon_stroke;
use super::paint::{paint_filled_circle, paint_path, paint_ring, paint_rounded_square_ring};
use crate::theme::Density;

/// The rendered form of a space identity string (DESIGN.md §11.3/§7): a
/// curated emoji scalar renders directly, a curated [`BbrGlyph`] name
/// renders its drawn §7 path, and any other value renders the §7
/// two-letter badge — the badge is the **missing-glyph fallback** for
/// non-curated values, never the curated rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityForm {
    /// A single curated emoji scalar (rendered directly).
    Emoji,
    /// A curated identity glyph (drawn from its §7 path).
    Glyph(BbrGlyph),
    /// A non-curated value (the §7 two-letter badge fallback).
    Badge,
}

/// Classify a canonical identity string into its rendered form. The one
/// classifier the chips, the picker, and the snapshot share: a single
/// scalar is emoji, a curated catalog name is a glyph path, everything
/// else is the badge fallback.
pub fn classify_identity(identity: &str) -> IdentityForm {
    if identity.chars().count() == 1 {
        IdentityForm::Emoji
    } else if let Some(glyph) = BbrGlyph::from_name(identity) {
        IdentityForm::Glyph(glyph)
    } else {
        IdentityForm::Badge
    }
}

impl IdentityForm {
    /// The stable wire name for the rendered form (the typed snapshot's
    /// `identity_kind`): `"emoji"`, `"glyph"`, or `"badge"`.
    pub fn kind(self) -> &'static str {
        match self {
            IdentityForm::Emoji => "emoji",
            IdentityForm::Glyph(_) => "glyph",
            IdentityForm::Badge => "badge",
        }
    }
}

/// One identity glyph's paint arm: every curated glyph resolves to a drawn
/// §7 path. The caller supplies the tile bounds, the density (for the §7
/// stroke weight) and the rest color.
pub fn paint_identity(
    glyph: BbrGlyph,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    density: Density,
    color: Rgba,
) {
    let stroke = icon_stroke(density);
    let size = f32::from(bounds.size.width);
    let cx = f32::from(bounds.center().x);
    let cy = f32::from(bounds.center().y);
    // The glyph half-extent: every identity glyph stays inside the tile
    // (the shared `h` convention below keeps the widest arm inside the
    // box). 0.34 renders the identity at the reference chip weight — the
    // widest arm (±h) plus the §7 stroke stays inside the tile at every
    // shipped size.
    let h = size * 0.34;
    match glyph {
        BbrGlyph::Code => paint_code(window, stroke, color, cx, cy, h),
        BbrGlyph::Terminal => paint_terminal(window, stroke, color, cx, cy, h),
        BbrGlyph::Folder => paint_folder(window, stroke, color, cx, cy, h),
        BbrGlyph::Person => paint_person(window, stroke, color, cx, cy, h),
        BbrGlyph::Briefcase => paint_briefcase(window, stroke, color, cx, cy, h),
        BbrGlyph::Heart => paint_heart(window, stroke, color, cx, cy, h),
        BbrGlyph::Star => paint_star(window, stroke, color, cx, cy, h),
        BbrGlyph::Leaf => paint_leaf(window, stroke, color, cx, cy, h),
        BbrGlyph::Compass => paint_compass(window, stroke, color, cx, cy, h),
        BbrGlyph::Moon => paint_moon(window, stroke, color, cx, cy, h),
        BbrGlyph::Book => paint_book(window, stroke, color, cx, cy, h),
        BbrGlyph::Camera => paint_camera(window, stroke, color, cx, cy, h),
        BbrGlyph::Palette => paint_palette(window, stroke, color, cx, cy, h),
        BbrGlyph::Globe => paint_globe(window, stroke, color, cx, cy, h),
    }
}

/// Code `</>`: two angle brackets with the slash between them.
fn paint_code(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    let inner = h * 0.15;
    let tip = h * 0.55;
    let mut path = PathBuilder::stroke(stroke);
    // Left bracket `<`.
    path.move_to(point(px(cx - h), px(cy)));
    path.line_to(point(px(cx - inner), px(cy - tip)));
    path.move_to(point(px(cx - h), px(cy)));
    path.line_to(point(px(cx - inner), px(cy + tip)));
    // Right bracket `>`.
    path.move_to(point(px(cx + h), px(cy)));
    path.line_to(point(px(cx + inner), px(cy - tip)));
    path.move_to(point(px(cx + h), px(cy)));
    path.line_to(point(px(cx + inner), px(cy + tip)));
    // The slash between the brackets.
    path.move_to(point(px(cx - inner), px(cy + tip * 0.9)));
    path.line_to(point(px(cx + inner), px(cy - tip * 0.9)));
    paint_path(window, path, color);
}

/// Terminal `>_`: a right-pointing chevron over an underscore.
fn paint_terminal(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    let mut path = PathBuilder::stroke(stroke);
    // The `>` chevron in the upper-left.
    path.move_to(point(px(cx - h * 0.5), px(cy - h * 0.5)));
    path.line_to(point(px(cx - h * 0.05), px(cy)));
    path.line_to(point(px(cx - h * 0.5), px(cy + h * 0.5)));
    // The `_` underscore at the lower-right.
    path.move_to(point(px(cx - h * 0.05), px(cy + h * 0.55)));
    path.line_to(point(px(cx + h * 0.5), px(cy + h * 0.55)));
    paint_path(window, path, color);
}

/// Folder: the same §7 closed-folder silhouette as [`super::Icon::Folder`]
/// (a folder to switch into; the disclosure state belongs to the folder
/// row anatomy, not the identity chip).
fn paint_folder(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    let tab_depth = h * 0.47;
    let tab_width = h * 0.67;
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(cx - h), px(cy + h)));
    path.line_to(point(px(cx - h), px(cy - h + tab_depth)));
    path.line_to(point(px(cx - h + tab_width), px(cy - h)));
    path.line_to(point(px(cx + h), px(cy - h)));
    path.line_to(point(px(cx + h), px(cy + h)));
    path.line_to(point(px(cx - h), px(cy + h)));
    paint_path(window, path, color);
}

/// Person: a head ring and a shoulder arc.
fn paint_person(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    paint_ring(window, cx, cy - h * 0.38, h * 0.2, stroke, color);
    let shoulder_r = h * 0.45;
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(cx - shoulder_r), px(cy + h * 0.55)));
    path.arc_to(
        point(px(shoulder_r), px(shoulder_r)),
        px(0.0),
        false,
        false,
        point(px(cx + shoulder_r), px(cy + h * 0.55)),
    );
    paint_path(window, path, color);
}

/// Briefcase: a rounded body, the handle arc, and the center latch.
fn paint_briefcase(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    paint_rounded_square_ring(window, cx, cy + h * 0.1, h * 0.5, h * 0.1, stroke, color);
    let mut path = PathBuilder::stroke(stroke);
    // The handle: a semicircular arc from the body top, bulging up.
    path.move_to(point(px(cx - h * 0.2), px(cy - h * 0.4)));
    path.arc_to(
        point(px(h * 0.2), px(h * 0.2)),
        px(0.0),
        false,
        false,
        point(px(cx + h * 0.2), px(cy - h * 0.4)),
    );
    // The center latch from the handle base into the body.
    path.move_to(point(px(cx), px(cy - h * 0.38)));
    path.line_to(point(px(cx), px(cy + h * 0.02)));
    paint_path(window, path, color);
}

/// Heart: two lobe arcs meeting at the top dip, falling to the bottom tip.
fn paint_heart(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    let lobe_r = h * 0.347;
    let lobe_x = h * 0.6;
    let lobe_y = h * 0.05;
    let mut path = PathBuilder::stroke(stroke);
    // From the bottom tip up the left side to the left lobe.
    path.move_to(point(px(cx), px(cy + h * 0.6)));
    path.line_to(point(px(cx - lobe_x), px(cy + lobe_y)));
    // Left lobe arc up to the top dip (counterclockwise, bulging up-left).
    path.arc_to(
        point(px(lobe_r), px(lobe_r)),
        px(0.0),
        false,
        false,
        point(px(cx), px(cy - h * 0.3)),
    );
    // Right lobe arc down to the right lobe (clockwise, mirror).
    path.arc_to(
        point(px(lobe_r), px(lobe_r)),
        px(0.0),
        false,
        true,
        point(px(cx + lobe_x), px(cy + lobe_y)),
    );
    // Down to the bottom tip.
    path.line_to(point(px(cx), px(cy + h * 0.6)));
    paint_path(window, path, color);
}

/// Star: the five-point star as straight strokes (outer/inner radius
/// alternation), no fills.
fn paint_star(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    let outer = h * 0.55;
    let inner = h * 0.25;
    // The ten vertices in traversal order (outer/inner alternating, starting
    // at the top point): the classic five-point star.
    let points: [(f32, f32); 10] = [
        (0.0, -1.0),
        (0.5878, -0.8090),
        (0.9511, -0.3090),
        (0.9511, 0.3090),
        (0.5878, 0.8090),
        (0.0, 1.0),
        (-0.5878, 0.8090),
        (-0.9511, 0.3090),
        (-0.9511, -0.3090),
        (-0.5878, -0.8090),
    ];
    let mut path = PathBuilder::stroke(stroke);
    for (index, (dx, dy)) in points.iter().enumerate() {
        let radius = if index % 2 == 0 { outer } else { inner };
        let x = cx + dx * radius;
        let y = cy + dy * radius;
        if index == 0 {
            path.move_to(point(px(x), px(y)));
        } else {
            path.line_to(point(px(x), px(y)));
        }
    }
    path.line_to(point(px(cx), px(cy - outer)));
    paint_path(window, path, color);
}

/// Leaf: a pointed lens along the diagonal plus a short stem.
fn paint_leaf(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    let dx = h * 0.45;
    let dy = h * 0.35;
    let r = h * 0.57;
    let mut path = PathBuilder::stroke(stroke);
    // Lower arc (bulging down-left) then upper arc (bulging up-right).
    path.move_to(point(px(cx - dx), px(cy + dy)));
    path.arc_to(
        point(px(r), px(r)),
        px(0.0),
        false,
        true,
        point(px(cx + dx), px(cy - dy)),
    );
    path.arc_to(
        point(px(r), px(r)),
        px(0.0),
        false,
        false,
        point(px(cx - dx), px(cy + dy)),
    );
    // The stem from the leaf base.
    path.move_to(point(px(cx - dx), px(cy + dy)));
    path.line_to(point(px(cx - dx - h * 0.2), px(cy + dy + h * 0.2)));
    paint_path(window, path, color);
}

/// Compass: a ring with the diamond needle.
fn paint_compass(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    paint_ring(window, cx, cy, h * 0.5, stroke, color);
    let mut path = PathBuilder::stroke(stroke);
    path.move_to(point(px(cx), px(cy - h * 0.4)));
    path.line_to(point(px(cx + h * 0.12), px(cy)));
    path.line_to(point(px(cx), px(cy + h * 0.4)));
    path.line_to(point(px(cx - h * 0.12), px(cy)));
    path.line_to(point(px(cx), px(cy - h * 0.4)));
    paint_path(window, path, color);
}

/// Moon: a crescent — the outer arc and the inner arc both bulging right,
/// opening left (the two cusps are a shared chord).
fn paint_moon(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    let outer_r = h * 0.5;
    let inner_r = h * 0.36;
    let cusp_x = h * 0.25;
    let cusp_y = h * 0.36;
    let mut path = PathBuilder::stroke(stroke);
    // Outer arc from the top cusp around the right to the bottom cusp.
    path.move_to(point(px(cx - cusp_x), px(cy - cusp_y)));
    path.arc_to(
        point(px(outer_r), px(outer_r)),
        px(0.0),
        true,
        true,
        point(px(cx - cusp_x), px(cy + cusp_y)),
    );
    // Inner arc back to the top cusp (the inner edge of the crescent).
    path.arc_to(
        point(px(inner_r), px(inner_r)),
        px(0.0),
        false,
        true,
        point(px(cx - cusp_x), px(cy - cusp_y)),
    );
    paint_path(window, path, color);
}

/// Book: an open book — the two page arcs from the top center to the bottom
/// corners plus the center spine.
fn paint_book(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    let mut path = PathBuilder::stroke(stroke);
    let r = h * 0.55;
    // Left page from the top center to the bottom-left corner.
    path.move_to(point(px(cx), px(cy - h * 0.45)));
    path.arc_to(
        point(px(r), px(r)),
        px(0.0),
        false,
        false,
        point(px(cx - h * 0.55), px(cy + h * 0.45)),
    );
    // Right page (mirror).
    path.move_to(point(px(cx), px(cy - h * 0.45)));
    path.arc_to(
        point(px(r), px(r)),
        px(0.0),
        false,
        true,
        point(px(cx + h * 0.55), px(cy + h * 0.45)),
    );
    // The center spine.
    path.move_to(point(px(cx), px(cy - h * 0.45)));
    path.line_to(point(px(cx), px(cy + h * 0.45)));
    paint_path(window, path, color);
}

/// Camera: the rounded body ring, the lens ring, and the top bump.
fn paint_camera(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    paint_rounded_square_ring(window, cx, cy + h * 0.05, h * 0.5, h * 0.12, stroke, color);
    paint_ring(window, cx, cy + h * 0.05, h * 0.22, stroke, color);
    let mut path = PathBuilder::stroke(stroke);
    // The viewfinder bump on the body top.
    path.move_to(point(px(cx - h * 0.12), px(cy - h * 0.45)));
    path.line_to(point(px(cx + h * 0.12), px(cy - h * 0.45)));
    paint_path(window, path, color);
}

/// Palette: the body ring, three paint dots, and the thumb-hole ring.
fn paint_palette(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    paint_ring(window, cx, cy, h * 0.5, stroke, color);
    // Three paint dots along the upper rim.
    for (dx, dy) in [(-0.38, -0.32), (0.0, -0.42), (0.38, -0.32)] {
        paint_filled_circle(window, cx + dx * h, cy + dy * h, h * 0.08, color);
    }
    // The thumb hole on the lower right.
    paint_ring(window, cx + h * 0.34, cy + h * 0.3, h * 0.1, stroke, color);
}

/// Globe: the ring, the equator, and the meridian arc.
fn paint_globe(window: &mut Window, stroke: Pixels, color: Rgba, cx: f32, cy: f32, h: f32) {
    paint_ring(window, cx, cy, h * 0.5, stroke, color);
    let mut path = PathBuilder::stroke(stroke);
    // The equator.
    path.move_to(point(px(cx - h * 0.5), px(cy)));
    path.line_to(point(px(cx + h * 0.5), px(cy)));
    // The meridian arc from pole to pole, bulging right.
    path.move_to(point(px(cx), px(cy - h * 0.5)));
    path.arc_to(
        point(px(h * 0.5), px(h * 0.5)),
        px(0.0),
        false,
        true,
        point(px(cx), px(cy + h * 0.5)),
    );
    paint_path(window, path, color);
}
