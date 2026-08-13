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
/// half-arm). A near-full clockwise arc with the arrowhead at the end
/// pointing in the travel direction — drawn, never a Unicode substitute.
pub fn reload_geometry(size: f32) -> (f32, f32) {
    (size * 0.34, size * 0.12)
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
/// length).
pub fn downloads_geometry(size: f32) -> (f32, f32, f32, f32, f32) {
    (
        size * 0.26,
        size * 0.10,
        size * 0.28,
        size * 0.30,
        size * 0.08,
    )
}

/// Folder silhouette geometry: (body half-width/height, tab height above
/// the body, tab width).
pub fn folder_geometry(size: f32) -> (f32, f32, f32) {
    (size * 0.30, size * 0.14, size * 0.20)
}

/// Open-folder geometry: (body half-width/height, tab height above the
/// body, tab width, mouth). `mouth` is the distance the open body's top
/// edge sits below the closed top (`cy - mouth`): the lid flap lifts
/// above it, so the open variant reads as the folder's lid raised off the
/// body (the §11.11 expanded disclosure state).
pub fn folder_open_geometry(size: f32) -> (f32, f32, f32, f32) {
    (size * 0.30, size * 0.14, size * 0.20, size * 0.105)
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

/// §10.11 clock companion geometry: (ring radius, hour-hand half-length,
/// minute-hand half-length) of the unresponsive row's clock glyph — a
/// static ring with two short hands, drawn, never a Unicode substitute.
pub fn clock_geometry(size: f32) -> (f32, f32, f32) {
    (size * 0.34, size * 0.12, size * 0.22)
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
            // Folder: the body plus the stroke fits, the tab stays inside.
            let (fhalf, tab_depth, tab_width) = folder_geometry(size);
            assert!(fhalf + stroke <= size / 2.0, "folder body fits");
            assert!(tab_depth < fhalf, "folder tab below the body top");
            assert!(tab_width < fhalf * 2.0, "folder tab inside the width");
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
            // Reload (§11.14): the arc radius and the arrowhead extent fit.
            let (radius, head) = reload_geometry(size);
            assert!(radius + stroke <= size / 2.0, "reload arc fits at {size}pt");
            assert!(
                radius * 0.866 + head + stroke <= size / 2.0,
                "reload arrowhead fits at {size}pt"
            );
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
