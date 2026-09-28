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

/// New-space plus: half the stroke length.
pub fn plus_arm(size: f32) -> f32 {
    size * 0.28
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

/// Navigation back/forward arrow: (head half-arm, stem length). The head
/// apex sits at the pointing side, the arms open to the corners, and the
/// stem runs from the head base outward — the §11.14 titlebar navigation
/// glyphs.
pub fn nav_arrow_geometry(size: f32) -> (f32, f32) {
    (size * 0.20, size * 0.26)
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

/// Downloads arrow-into-tray geometry: (stem half-length, arrowhead
/// half-arm, tray half-width, tray base offset below center, tray side
/// length). Calibrated to the reference footer glyph: a clearly readable
/// arrow dropping into an open tray with visible side walls.
pub fn downloads_geometry(size: f32) -> (f32, f32, f32, f32, f32) {
    (
        size * 0.30,
        size * 0.14,
        size * 0.32,
        size * 0.34,
        size * 0.12,
    )
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
            // Downloads: the stem and the tray base plus the stroke fit.
            let (arm, _arrow, tray_half, tray_depth, tray_side) = downloads_geometry(size);
            assert!(arm < size / 2.0, "downloads stem fits at {size}pt");
            assert!(
                tray_half + stroke <= size / 2.0,
                "downloads tray fits at {size}pt"
            );
            assert!(tray_side < tray_depth, "tray side shorter than depth");
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
            let (head, stem) = nav_arrow_geometry(size);
            assert!(head <= size / 2.0, "nav head fits at {size}pt");
            assert!(stem + head <= size / 2.0, "nav stem fits at {size}pt");
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
}
