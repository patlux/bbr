//! DESIGN.md §7 U2 glyph geometry — the icon-system slice carry-over.
//!
//! The four U2 catalog glyphs (`Downloads`, `Spaces`, `Folder`,
//! `Disclosure`) keep their geometry here so the catalog file stays under
//! the size budget: pure §7 math (arms, radii, offsets at the 1.5 pt stroke
//! weight), consumed by the `Icon` paint arms in [`super::icons`]. The fit
//! contract tests live next to the numbers they lock.

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

/// Spaces stacked-squares geometry: (half side, diagonal offset, corner
/// radius of each square ring).
pub fn spaces_geometry(size: f32) -> (f32, f32, f32) {
    (size * 0.26, size * 0.13, size * 0.06)
}

/// Folder silhouette geometry: (body half-width/height, tab height above
/// the body, tab width).
pub fn folder_geometry(size: f32) -> (f32, f32, f32) {
    (size * 0.30, size * 0.14, size * 0.20)
}

/// Disclosure chevron: half the arm length from the apex to either corner.
pub fn disclosure_arm(size: f32) -> f32 {
    size * 0.22
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
            // Spaces: the front square plus the stroke fits, the back square
            // stays offset inside, and the corner radius stays within it.
            let (half, offset, corner) = spaces_geometry(size);
            assert!(half + stroke <= size / 2.0, "spaces square fits");
            assert!(offset > 0.0 && offset < half, "spaces offset ordered");
            assert!(corner < half, "spaces corner inside the square");
            // Folder: the body plus the stroke fits, the tab stays inside.
            let (fhalf, tab_depth, tab_width) = folder_geometry(size);
            assert!(fhalf + stroke <= size / 2.0, "folder body fits");
            assert!(tab_depth < fhalf, "folder tab below the body top");
            assert!(tab_width < fhalf * 2.0, "folder tab inside the width");
            assert!(
                disclosure_arm(size) * 1.5 <= size / 2.0,
                "disclosure chevron fits at {size}pt"
            );
        }
    }
}
