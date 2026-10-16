//! Spatial detail windows derived only from the stored exact diff pixels.
//! Row bands remain comparison metadata, not claims about connected regions.
use crate::{
    compare::{Finding, CHANGED},
    store,
};
use image::RgbaImage;
use std::path::Path;

const WIDTH: u32 = 304;
const HEIGHT: u32 = 96;
const PAD: u32 = 8;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Detail {
    /// Exact changed bounds in this cell, with exclusive right/bottom edges.
    pub bounds: [u32; 4],
    /// Same unscaled window for Alt, Neu, Diff, stage and the numbered mark.
    pub crop: [u32; 4],
    pub changed: u64,
}

pub(super) fn load(root: &Path, asset: &str, item: &Finding) -> Option<Vec<Detail>> {
    if item.status != "changed" || item.reason.is_some() || item.rows.is_empty() {
        return None;
    }
    let size = item.actual_size?;
    let path = root.join(asset).join("diff.png");
    store::no_symlinks(&path).ok()?;
    let diff = store::read_png(&path).ok()?;
    if [diff.width(), diff.height()] != size || item.expected_size != Some(size) {
        return None;
    }
    let details = windows(&diff, &item.rows);
    // Missing/inconsistent assets never fabricate an unchanged detail preview.
    if details.is_empty() || Some(details.iter().map(|d| d.changed).sum()) != item.changed_pixels {
        return None;
    }
    Some(details)
}

fn windows(diff: &RgbaImage, bands: &[[u32; 2]]) -> Vec<Detail> {
    let (width, height) = diff.dimensions();
    let mut details = Vec::new();
    for &[top, bottom] in bands {
        // Bound vertical strips, then pack occupied columns from their first
        // changed pixel. Unlike a fixed grid, this does not split a thin edge
        // just because it straddles an arbitrary horizontal cell boundary.
        for strip in (top..bottom.min(height)).step_by((HEIGHT - 2 * PAD) as usize) {
            let end = strip
                .saturating_add(HEIGHT - 2 * PAD)
                .min(bottom)
                .min(height);
            let mut cells: Vec<([u32; 4], u64)> = Vec::new();
            for x in 0..width {
                for y in strip..end {
                    if diff.get_pixel(x, y).0 != CHANGED {
                        continue;
                    }
                    if cells
                        .last()
                        .is_none_or(|(b, _)| x - b[0] >= WIDTH - 2 * PAD)
                    {
                        cells.push(([x, y, x + 1, y + 1], 0));
                    }
                    let (bounds, count) = cells.last_mut().expect("changed pixel starts a cell");
                    bounds[1] = bounds[1].min(y);
                    bounds[2] = x + 1;
                    bounds[3] = bounds[3].max(y + 1);
                    *count += 1;
                }
            }
            for (bounds, changed) in cells {
                let [x1, y1, x2, y2] = bounds;
                let w = WIDTH.min(width);
                let h = HEIGHT.min(height);
                let x = x1.saturating_sub((w - (x2 - x1)) / 2).min(width - w);
                let y = y1.saturating_sub((h - (y2 - y1)) / 2).min(height - h);
                details.push(Detail {
                    bounds,
                    crop: [x, y, w, h],
                    changed,
                });
            }
        }
    }
    details
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn verify(diff: &RgbaImage, bands: &[[u32; 2]]) -> Vec<Detail> {
        let details = windows(diff, bands);
        let mut count = 0;
        for d in &details {
            let [x, y, w, h] = d.crop;
            assert!(w > 0 && h > 0 && w <= WIDTH && h <= HEIGHT);
            assert!(x + w <= diff.width() && y + h <= diff.height());
            let [x1, y1, x2, y2] = d.bounds;
            assert!(x <= x1 && y <= y1 && x + w >= x2 && y + h >= y2);
            assert!(d.changed > 0);
            count += d.changed;
        }
        let changed: Vec<_> = diff
            .enumerate_pixels()
            .filter(|(_, _, p)| p.0 == CHANGED)
            .collect();
        assert_eq!(count, changed.len() as u64);
        for (x, y, _) in changed {
            assert!(details.iter().any(|d| {
                let [cx, cy, w, h] = d.crop;
                (cx..cx + w).contains(&x) && (cy..cy + h).contains(&y)
            }));
        }
        assert_eq!(details, windows(diff, bands));
        details
    }

    #[test]
    fn isolated_edges_dense_fields_and_empty_images_have_exact_bounded_windows() {
        for (w, h) in [(1, 1), (20, 12), (608, 2814), (1500, 120)] {
            let mut diff = RgbaImage::new(w, h);
            assert!(windows(&diff, &[[0, h]]).is_empty());
            for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
                diff.put_pixel(x, y, Rgba(CHANGED));
            }
            verify(&diff, &[[0, h]]);
            let dense = RgbaImage::from_pixel(w, h, Rgba(CHANGED));
            verify(&dense, &[[0, h]]);
        }
    }

    #[test]
    fn distant_changes_in_one_row_band_do_not_choose_an_empty_union_corner() {
        let mut diff = RgbaImage::new(608, 2814);
        for y in 0..2814 {
            diff.put_pixel(600, y, Rgba(CHANGED));
        }
        diff.put_pixel(20, 2780, Rgba(CHANGED));
        diff.put_pixel(410, 22, Rgba(CHANGED));
        let windows = verify(&diff, &[[0, 2814]]);
        assert!(windows.len() > 2);
        assert!(windows[0].crop[0] > 20);
        assert!(windows[0].bounds[0] >= 410);
    }

    #[test]
    fn separate_row_bands_stay_separate_even_inside_one_cell_height() {
        let mut diff = RgbaImage::new(20, 12);
        diff.put_pixel(2, 1, Rgba(CHANGED));
        diff.put_pixel(2, 11, Rgba(CHANGED));
        assert_eq!(verify(&diff, &[[1, 2], [11, 12]]).len(), 2);
    }
}
