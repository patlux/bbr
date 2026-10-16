//! DESIGN.md §7 glyph geometry — the shared path math for the icon catalog.
//!
//! Every arm, radius, and offset at the §7 stroke weight lives here (pure
//! `f32` math, no GPUI types except nothing) so the catalog file stays
//! under the size budget and the geometry contract tests sit next to the
//! numbers they lock. The `u2_glyphs` geometry (Downloads, Folder,
//! Disclosure, Check, Stopped) moved here verbatim from the app's
//! `components/u2_glyphs.rs`; the catalog's own geometry
//! ([`icon_stroke`], [`close_arm`], [`plus_arm`], [`pin_radius`],
//! [`lifering_radii`], [`arc_radius`], [`chevron_arm`],
//! [`chassis_radii`]) joined it so `bbr-ui::icons` stays under the module
//! budget.

use crate::theme::Density;
use gpui::Pixels;

/// §7 stroke weight: 1.5 pt at N/XL density, 1.25 pt at S.
pub fn icon_stroke(density: Density) -> Pixels {
    gpui::px(match density {
        Density::Small => 1.25,
        Density::Normal | Density::ExtraLarge => 1.5,
    })
}

/// Close ×: half the diagonal stroke length (of the full stroke).
pub fn close_arm(size: f32) -> f32 {
    size * 0.28
}

/// New-space plus: half the stroke length — Paper's `M9 3.75v10.5` (a
/// 5.25/18 arm) scaled to the box, before its round caps.
pub fn plus_arm(size: f32) -> f32 {
    size * 5.25 / 18.0
}

/// The plus: one horizontal and one vertical stroke through the centre.
pub fn plus_segments(size: f32) -> Vec<Segment> {
    let arm = plus_arm(size);
    vec![((-arm, 0.0), (arm, 0.0)), ((0.0, -arm), (0.0, arm))]
}

/// The minus: the plus's horizontal stroke alone.
pub fn minus_segments(size: f32) -> Vec<Segment> {
    let arm = plus_arm(size);
    vec![((-arm, 0.0), (arm, 0.0))]
}

/// Pin dot: the radius of the filled dot / hollow ring.
pub fn pin_radius(size: f32) -> f32 {
    size * 0.30
}

/// Lifering: (outer ring radius, inner ring radius).
pub fn lifering_radii(size: f32) -> (f32, f32) {
    (size * 0.44, size * 0.28)
}

/// Loading arc radius (§10.5: 14 pt arc inside the favicon slot).
pub fn arc_radius(size: f32) -> f32 {
    size * 0.40
}

/// Find-bar step chevron: half the arm length from the apex to either
/// corner — the same §7 geometry as the disclosure chevron, so one arm
/// contract covers both families.
pub fn chevron_arm(size: f32) -> f32 {
    disclosure_arm(size)
}

/// Reload glyph (the §11.14 titlebar control): (arc radius, arrowhead
/// half-arm). A 300° clockwise ring ending at the top (the gap spans the
/// top-right 60°) with the arrowhead at the top end pointing in the
/// travel direction — drawn, never a Unicode substitute.
pub fn reload_geometry(size: f32) -> (f32, f32) {
    (size * 0.32, size * 0.11)
}

/// Sidebar toggle glyph (§11.14): (frame half-side, divider x-offset from
/// the glyph center). The divider sits near the left edge when collapsed
/// and toward the center when expanded, so the glyph reflects the
/// sidebar's state without color alone.
pub fn sidebar_geometry(size: f32, expanded: bool) -> (f32, f32) {
    let half = size * 0.42;
    let divider = if expanded { -size * 0.14 } else { -size * 0.34 };
    (half, divider)
}

/// Chassis mark: (frame corner radius, aperture ring radius). The frame
/// corner keeps the §5 4–8 px chrome radius at the 64 pt mark size.
pub fn chassis_radii(size: f32) -> (f32, f32) {
    (size * 0.125, size * 0.30)
}

/// One stroked segment relative to the glyph centre, in points (y grows
/// downward, like the window).
pub type Segment = ((f32, f32), (f32, f32));

/// The outline of one round-capped stroke of half-width `r` (Paper draws
/// every ported glyph with `stroke-linecap/linejoin: round`). The outline
/// runs along the `+side` edge from start to end, around the end cap
/// through `end_tip`, back along the `-side` edge, and around the start
/// cap through `start_tip`; `along` and `side` are the unit direction and
/// normal scaled to `r`, the tangents of the cap quarter arcs. Every
/// capsule winds the same way, so a non-zero fill of several is their
/// union — round caps and round joins exactly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capsule {
    pub start_plus: (f32, f32),
    pub end_plus: (f32, f32),
    pub end_tip: (f32, f32),
    pub end_minus: (f32, f32),
    pub start_minus: (f32, f32),
    pub start_tip: (f32, f32),
    pub along: (f32, f32),
    pub side: (f32, f32),
}

/// The [`Capsule`] around `segment`, or `None` for a zero-length segment.
pub fn capsule(segment: Segment, r: f32) -> Option<Capsule> {
    let ((x0, y0), (x1, y1)) = segment;
    let (dx, dy) = (x1 - x0, y1 - y0);
    let len = (dx * dx + dy * dy).sqrt();
    if len <= f32::EPSILON {
        return None;
    }
    let along = (dx / len * r, dy / len * r);
    let side = (-along.1, along.0);
    let at = |(x, y): (f32, f32), (ax, ay): (f32, f32), k: f32| (x + ax * k, y + ay * k);
    Some(Capsule {
        start_plus: at((x0, y0), side, 1.0),
        end_plus: at((x1, y1), side, 1.0),
        end_tip: at((x1, y1), along, 1.0),
        end_minus: at((x1, y1), side, -1.0),
        start_minus: at((x0, y0), side, -1.0),
        start_tip: at((x0, y0), along, -1.0),
        along,
        side,
    })
}

/// A point of Paper's 18-unit glyph viewBox mapped onto a `size` box,
/// relative to its centre.
fn paper18(size: f32, x: f32, y: f32) -> (f32, f32) {
    let k = size / 18.0;
    ((x - 9.0) * k, (y - 9.0) * k)
}

/// The cancelled-row Delete glyph (Paper M-4): a lid with a handle over a
/// tapering bin.
pub fn trash_segments(size: f32) -> Vec<Segment> {
    let p = |x, y| paper18(size, x, y);
    vec![
        (p(3.5, 5.0), p(14.5, 5.0)),
        (p(7.0, 5.0), p(7.0, 3.25)),
        (p(7.0, 3.25), p(11.0, 3.25)),
        (p(11.0, 3.25), p(11.0, 5.0)),
        (p(5.0, 5.0), p(5.75, 15.0)),
        (p(5.75, 15.0), p(12.25, 15.0)),
        (p(12.25, 15.0), p(13.0, 5.0)),
    ]
}

/// The Downloads glyph (Paper W-1/S-7 `[Downloads]`): a stem dropping to a
/// downward arrowhead above one flat baseline, Paper's 18-unit path
/// scaled to the glyph box.
pub fn downloads_segments(size: f32) -> Vec<Segment> {
    let p = |x, y| paper18(size, x, y);
    vec![
        (p(9.0, 2.75), p(9.0, 11.25)),
        (p(9.0, 11.25), p(5.75, 8.0)),
        (p(9.0, 11.25), p(12.25, 8.0)),
        (p(3.25, 13.75), p(14.75, 13.75)),
    ]
}

/// The titlebar Back/Forward arrow (Paper W-1 `bbr-nav-back`): an 11.5/18
/// shaft with 4.5/18 arms from the tip, Paper's path scaled to the box.
pub fn nav_arrow_segments(size: f32, pointing_right: bool) -> Vec<Segment> {
    let p = |x: f32, y: f32| {
        let (px, py) = paper18(size, x, y);
        (if pointing_right { -px } else { px }, py)
    };
    vec![
        (p(3.5, 9.0), p(8.0, 4.5)),
        (p(3.5, 9.0), p(8.0, 13.5)),
        (p(3.5, 9.0), p(15.0, 9.0)),
    ]
}

/// The S12 Paper folder silhouette: a body wider than tall whose top-left
/// tab rises above the body top and steps down to it on a short slant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FolderShape {
    pub half_w: f32,
    pub half_h: f32,
    /// The tab's flat top width from the left edge.
    pub tab_w: f32,
    /// How far the body top sits below the tab top.
    pub tab_h: f32,
    /// The horizontal run of the step from the tab down to the body top.
    pub slant: f32,
}

/// The folder silhouette geometry (closed and open share it).
pub fn folder_shape(size: f32) -> FolderShape {
    FolderShape {
        half_w: size * 0.36,
        half_h: size * 0.28,
        tab_w: size * 0.26,
        tab_h: size * 0.11,
        slant: size * 0.08,
    }
}

/// The search magnifier: (lens radius, lens offset up-left from the
/// center, handle length).
pub fn search_geometry(size: f32) -> (f32, f32, f32) {
    (size * 0.26, size * 0.08, size * 0.20)
}

/// Disclosure chevron: half the arm length from the apex to either corner.
pub fn disclosure_arm(size: f32) -> f32 {
    size * 0.22
}

/// Check mark geometry: (inner stroke half-length, outer stroke half-length)
/// — the short ascending stroke and the long ascending stroke of the ✓.
pub fn check_geometry(size: f32) -> (f32, f32) {
    (size * 0.12, size * 0.26)
}

/// Stopped bar: half the horizontal bar length of the neutral cancelled
/// marker.
pub fn stopped_half(size: f32) -> f32 {
    size * 0.26
}

/// §10.11 clock companion geometry: (ring radius, hour-hand length,
/// minute-hand length) of the unresponsive row's clock glyph — a static
/// ring with two hands measured from the center, drawn, never a Unicode
/// substitute.
pub fn clock_geometry(size: f32) -> (f32, f32, f32) {
    (size * 0.34, size * 0.16, size * 0.22)
}

/// The two clock hands as `(from, to)` segments: both start at the glyph
/// center; the short hour hand points up and the longer minute hand points
/// right (the conventional clock-icon reading, never a centered cross).
pub fn clock_hands(size: f32, cx: f32, cy: f32) -> [((f32, f32), (f32, f32)); 2] {
    let (_ring, hour, minute) = clock_geometry(size);
    [((cx, cy), (cx, cy - hour)), ((cx, cy), (cx + minute, cy))]
}

/// §11.16 bookmark ribbon: (half width, top offset above center, bottom
/// offset below center, notch depth). An outlined ribbon with a V notch at
/// the bottom — the drawn Saved Page mark, never a `★` character.
pub fn bookmark_geometry(size: f32) -> (f32, f32, f32, f32) {
    (size * 0.21, size * 0.32, size * 0.34, size * 0.16)
}

/// §11.16 rename pencil along the rising diagonal: (half length, half
/// width, tip length, band offset from the top end). The outline is a
/// narrow body ending in a pointed tip at the lower left, with one band
/// line near the upper-right end.
pub fn pencil_geometry(size: f32) -> (f32, f32, f32, f32) {
    (size * 0.33, size * 0.09, size * 0.14, size * 0.12)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u2_glyph_geometry_stays_inside_its_box() {
        // Every arm/radius plus the 1.5 pt stroke must fit the glyph's
        // bounding box at every shipped size (16/18/20 rows, 32 crash,
        // 64 chassis mark) — the same contract as the other catalog glyphs.
        for size in [16.0, 18.0, 20.0, 32.0, 64.0] {
            let stroke = 1.5 / 2.0;
            // Downloads: the stem, arrowhead and baseline plus the stroke fit.
            assert!(inside(&downloads_segments(size), size), "downloads fits");
            // Folder: the body plus the stroke fits, the tab steps down
            // inside the width and above the body middle.
            let folder = folder_shape(size);
            assert!(folder.half_w + stroke <= size / 2.0, "folder body fits");
            assert!(folder.half_h + stroke <= size / 2.0, "folder height fits");
            assert!(folder.half_w > folder.half_h, "wider than tall");
            assert!(folder.tab_h < folder.half_h, "tab above the middle");
            assert!(
                folder.tab_w + folder.slant < folder.half_w * 2.0,
                "tab inside the width"
            );
            // Search: lens plus handle stay inside the box.
            let (radius, offset, handle) = search_geometry(size);
            assert!(radius + offset + stroke <= size / 2.0, "lens fits");
            let reach = (radius + handle) * std::f32::consts::FRAC_1_SQRT_2 - offset;
            assert!(reach + stroke <= size / 2.0, "handle fits at {size}pt");
            assert!(
                disclosure_arm(size) * 1.5 <= size / 2.0,
                "disclosure chevron fits at {size}pt"
            );
            // Check: the outer stroke plus the stroke width fits; the inner
            // stroke stays inside the outer one.
            let (inner, outer) = check_geometry(size);
            assert!(inner < outer, "check strokes ordered");
            assert!(outer * 1.5 + stroke <= size / 2.0, "check fits at {size}pt");
            // Stopped: the bar half plus the stroke fits.
            assert!(
                stopped_half(size) + stroke <= size / 2.0,
                "stopped bar fits at {size}pt"
            );
            // Clock (§10.11): the ring plus the stroke fits; the minute
            // hand stays inside the ring, the hour hand inside the minute.
            let (ring, hour, minute) = clock_geometry(size);
            assert!(ring + stroke <= size / 2.0, "clock ring fits at {size}pt");
            assert!(minute < ring, "minute hand inside the clock ring");
            assert!(hour < minute, "hour hand shorter than the minute hand");
            // Navigation arrow (§11.14): the head arms and the stem stay
            // inside the box at every shipped size.
            assert!(inside(&nav_arrow_segments(size, false), size), "nav fits");
            // Reload (§11.14): the arc radius fits and the arrowhead's
            // backward arms (at the top end, ±head around the tip) stay
            // inside the box.
            let (radius, head) = reload_geometry(size);
            assert!(radius + stroke <= size / 2.0, "reload arc fits at {size}pt");
            assert!(
                radius + head + stroke <= size / 2.0,
                "reload arrowhead fits at {size}pt"
            );
            assert!(head < radius, "reload arrowhead shorter than the arc");
            // Sidebar toggle (§11.14): the frame and both divider offsets
            // stay inside the box.
            for expanded in [true, false] {
                let (half, divider) = sidebar_geometry(size, expanded);
                assert!(half + stroke <= size / 2.0, "sidebar frame fits");
                assert!(
                    divider.abs() <= half,
                    "sidebar divider inside the frame at {size}pt"
                );
            }
        }
    }

    fn inside(segments: &[Segment], size: f32) -> bool {
        let limit = size / 2.0 - 1.5 / 2.0;
        segments
            .iter()
            .flat_map(|(a, b)| [*a, *b])
            .all(|(x, y)| x.abs() <= limit + 1e-4 && y.abs() <= limit + 1e-4)
    }

    /// Paper W-1/S-7 `[Downloads]`: the arrowhead points down (its arms rise
    /// from the tip) and lands above one flat baseline — never an upward
    /// "⊥" tent. Proportions follow Paper's 18-unit path.
    #[test]
    fn downloads_glyph_points_down_onto_a_flat_baseline() {
        for size in [16.0, 18.0, 22.0] {
            let segments = downloads_segments(size);
            assert!(inside(&segments, size), "downloads fits at {size}pt");
            let stem = segments[0];
            assert_eq!(stem.0 .0, 0.0, "vertical stem");
            assert_eq!(stem.1 .0, 0.0, "vertical stem");
            let tip = stem.1;
            assert!(tip.1 > stem.0 .1, "the stem runs downward to the tip");
            let arms: Vec<_> = segments
                .iter()
                .filter(|(a, b)| *a == tip && b.0 != 0.0)
                .collect();
            assert_eq!(arms.len(), 2, "two arrowhead arms at the tip");
            for (_, end) in &arms {
                assert!(end.1 < tip.1, "arms rise from the tip at {size}pt");
            }
            let horizontal: Vec<_> = segments.iter().filter(|(a, b)| a.1 == b.1).collect();
            assert_eq!(horizontal.len(), 1, "one flat baseline, no tray walls");
            let base = horizontal[0];
            assert!(base.0 .1 > tip.1, "the baseline sits below the tip");
            assert_eq!(segments.len(), 4, "stem, two arms, baseline");
            let k = size / 18.0;
            assert!(((tip.1 - stem.0 .1) - 8.5 * k).abs() < 0.01, "stem 8.5/18");
            assert!(
                ((base.1 .0 - base.0 .0).abs() - 11.5 * k).abs() < 0.01,
                "base 11.5/18"
            );
        }
    }

    /// Paper W-1 titlebar arrows: an 11.5/18 shaft with 4.5/18 arms.
    #[test]
    fn nav_arrows_follow_the_paper_proportions() {
        for size in [16.0, 18.0, 20.0] {
            for right in [false, true] {
                let segments = nav_arrow_segments(size, right);
                assert!(inside(&segments, size), "nav arrow fits at {size}pt");
                let xs: Vec<f32> = segments.iter().flat_map(|(a, b)| [a.0, b.0]).collect();
                let span = xs.iter().cloned().fold(f32::MIN, f32::max)
                    - xs.iter().cloned().fold(f32::MAX, f32::min);
                let k = size / 18.0;
                assert!((span - 11.5 * k).abs() < 0.01, "shaft 11.5/18 at {size}pt");
                let arm = segments[0];
                assert!(
                    ((arm.1 .1 - arm.0 .1).abs() - 4.5 * k).abs() < 0.01,
                    "arm 4.5/18"
                );
            }
        }
    }

    /// Paper M-4 Delete: the trash glyph stays inside its box.
    #[test]
    fn trash_glyph_fits_its_box() {
        for size in [16.0, 18.0, 20.0] {
            assert!(
                inside(&trash_segments(size), size),
                "trash fits at {size}pt"
            );
        }
    }
}
