//! Pure tab-drag target resolution (DESIGN.md §10.9, §11.2).
//!
//! The geometry that turns a pointer displacement into a typed drop target:
//! region-bounded insertion slots for the tab list and the §11.10 favorites
//! grid band above it. Kept separate from the drag state machine in
//! `super::drag` so the resolution math and its unit tests stay small and
//! layout-independent — the only layout constants are the §8 density row
//! heights, derived from the same theme tokens the render uses.
//!
//! Target resolution is anchored at the press: the pointer's distance above
//! the tab list top is `-(dy) - (source * row_h + press_in_row)`, where
//! `press_in_row` is the press's y within the source row (0 at the top edge,
//! `row_h` at the bottom). Tab-slot resolution is exact in row units
//! regardless of the press offset (row-unit rounding from the source row);
//! the favorites band is addressed through the same anchor, so grid
//! targeting is exact for the recorded press offset. When the layout cannot
//! expose the true offset (GPUI 0.2.2 offers no element geometry), the
//! caller anchors at the row center (`row_h / 2`), which bounds the
//! favorites-band error at half a row.
//!
//! The revealed-list drag-to-space band was removed with the legacy §11.4
//! `[Spaces]` toggle: dragging a tab into a space is a later explicit
//! drag-to-chip F2a layer, not part of this geometry.

/// §8 normal-density row height the drag resolution uses. Derived from the
/// same theme token the shell renders with, so a density change is picked
/// up by construction.
pub fn tab_row_h(theme: &crate::theme::Theme) -> f32 {
    f32::from(theme.density.row())
}

/// A drag's current typed target (DESIGN.md §10.9): an insertion slot in
/// the tab list or the §11.10 favorites grid. `Tab { slot }` means "insert
/// before the row at `slot`"; `slot` equal to the source region's end
/// renders at the end of that region. The favorites target toggles the
/// dragged tab in/out of the window-global favorites grid (add/remove —
/// never a copy or a move).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragTarget {
    /// Insertion slot in the tab list, within the source tab's region.
    Tab { slot: usize },
    /// The §11.10 favorites grid: toggle the dragged tab as a favorite.
    Favorites,
}

/// The tab-list geometry a drag resolves against: the pinned/regular
/// boundary (§11.2), the tab count, the §8 row height, and the favorites
/// region above the list (its deterministic height; 0 when the grid is
/// empty). Grouped so the pure resolver stays readable; the drag state
/// machine builds it per move from the same theme tokens and the
/// organization read model.
#[derive(Debug, Clone, Copy)]
pub struct DragGeometry {
    pub pinned: usize,
    pub len: usize,
    pub row_h: f32,
    /// The §11.10 favorites region's deterministic height above the tab
    /// list (0 when the grid is empty/hidden — no droppable band).
    pub favorites_h: f32,
}

/// Resolve the drag target for a source row and a row-unit displacement.
///
/// Pure and unit-testable: `dy` is the pointer's vertical displacement from
/// the press, `press_in_row` the press's y within the source row (0 at the
/// row's top edge, `row_h` at its bottom; the caller passes the true offset
/// when the layout exposes it and the row center `row_h / 2` otherwise), and
/// `geometry` the tab-list layout (see [`DragGeometry`]).
///
/// Semantics (interaction contract):
/// - The pointer over the tab list resolves to an insertion slot inside the
///   source region. A slot outside the region (the pinned/regular boundary
///   crossing) is illegal: the target clamps to the region edge so the tick
///   renders there, but `valid` is false.
/// - The pointer above the tab list resolves to the §11.10 favorites grid
///   band (when the region is rendered — a drop toggles the dragged tab
///   in/out of the window-global favorites). The fixed chrome above the
///   grid is not droppable.
pub fn resolve_target(
    source: usize,
    dy: f32,
    press_in_row: f32,
    geometry: &DragGeometry,
) -> (DragTarget, bool) {
    if geometry.len == 0 {
        return (DragTarget::Tab { slot: 0 }, false);
    }
    let DragGeometry {
        pinned,
        len,
        row_h,
        favorites_h,
    } = *geometry;
    // Distance from the pointer up to the tab list's top edge. The press
    // sits `source * row_h + press_in_row` below it; `dy` is the pointer's
    // displacement from the press.
    let above_list = -(dy) - (source as f32 * row_h + press_in_row);
    if above_list <= 0.0 {
        let raw = source as isize + (dy / row_h).round() as isize;
        let (lo, hi) = if source < pinned {
            (0, pinned)
        } else {
            (pinned, len)
        };
        let slot = raw.clamp(lo as isize, hi as isize) as usize;
        // A raw slot inside the region is legal. Outside it, only the
        // whole-list outer edges are legal drop points: the top of the list
        // (slot 0, the pinned region's own start) and the bottom of the list
        // (slot len, the regular region's own end). Anything else is the
        // pinned/regular boundary crossing, which is rejected whole.
        let valid = (lo..=hi).contains(&(raw as usize))
            || (lo == 0 && raw < 0)
            || (hi == len && raw > len as isize);
        (DragTarget::Tab { slot }, valid)
    } else {
        // Pointer above the tab list: the favorites region occupies the
        // `favorites_h` immediately above it (when rendered). A drop on the
        // favorites band toggles the dragged tab in the window-global grid.
        if favorites_h > 0.0 && above_list <= favorites_h {
            return (DragTarget::Favorites, true);
        }
        // Above the favorites band is the fixed chrome (the command field /
        // brand row) — not droppable.
        (DragTarget::Tab { slot: source }, false)
    }
}

/// Splice-index mapping for a committed tab-slot drop (domain reorder uses
/// remove-then-insert semantics): inserting "before original row `slot`"
/// maps to the splice index after removal.
pub fn splice_index(source: usize, slot: usize) -> usize {
    if slot > source {
        slot - 1
    } else {
        slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Row-center press offset for the classic 32 pt row height.
    fn center_press() -> f32 {
        16.0
    }

    /// A 32 pt row geometry over the given tab list.
    fn geometry(pinned: usize, len: usize) -> DragGeometry {
        DragGeometry {
            pinned,
            len,
            row_h: 32.0,
            favorites_h: 0.0,
        }
    }

    /// The same geometry with a rendered favorites band above the list.
    fn geometry_with_favorites(pinned: usize, len: usize, favorites_h: f32) -> DragGeometry {
        DragGeometry {
            pinned,
            len,
            row_h: 32.0,
            favorites_h,
        }
    }

    #[test]
    fn reorder_targets_stay_inside_the_source_region() {
        // Pinned region [0, 2), regular [2, 5).
        let g = geometry(2, 5);
        // Regular tab at 3 dragged one row down: slot 4 (valid).
        let (target, valid) = resolve_target(3, 32.0, center_press(), &g);
        assert_eq!(target, DragTarget::Tab { slot: 4 });
        assert!(valid);
        // …and one row up: slot 2 (still regular — the first regular slot).
        let (target, valid) = resolve_target(3, -32.0, center_press(), &g);
        assert_eq!(target, DragTarget::Tab { slot: 2 });
        assert!(valid);
        // Pinned tab at 0 dragged one row down stays pinned.
        let (target, valid) = resolve_target(0, 32.0, center_press(), &g);
        assert_eq!(target, DragTarget::Tab { slot: 1 });
        assert!(valid);
    }

    #[test]
    fn boundary_crossing_is_illegal_and_clamps_to_the_region_edge() {
        let g = geometry(2, 5);
        // Regular tab at 2 dragged far up: the target clamps to the region
        // start (slot 2, the first regular row) but is invalid.
        let (target, valid) = resolve_target(2, -2000.0, center_press(), &g);
        assert_eq!(target, DragTarget::Tab { slot: 2 });
        assert!(!valid, "boundary crossing is rejected whole");
        // Pinned tab dragged far down clamps to the region end and is invalid.
        let (target, valid) = resolve_target(0, 2000.0, center_press(), &g);
        assert_eq!(target, DragTarget::Tab { slot: 2 });
        assert!(!valid);
    }

    #[test]
    fn dragging_to_the_region_end_is_valid() {
        let g = geometry(2, 5);
        // Regular tab at 4 (last) dragged down: slot 5 = end of the regular
        // region, valid.
        let (target, valid) = resolve_target(4, 200.0, center_press(), &g);
        assert_eq!(target, DragTarget::Tab { slot: 5 });
        assert!(valid);
        // Pinned last tab at 1 dragged down to the boundary edge: slot 2
        // (end of the pinned region) is valid.
        let (target, valid) = resolve_target(1, 32.0, center_press(), &g);
        assert_eq!(target, DragTarget::Tab { slot: 2 });
        assert!(valid);
        // …and one more row is the boundary crossing: invalid.
        let (target, valid) = resolve_target(1, 64.0, center_press(), &g);
        assert_eq!(target, DragTarget::Tab { slot: 2 });
        assert!(!valid);
    }

    #[test]
    fn pointer_over_the_favorites_band_targets_the_grid() {
        // The favorites band occupies the region directly above the tab
        // list; a drop there toggles the dragged tab in/out of the grid.
        // Source row 1's center is 48 pt below the list top; the band is
        // 60 pt tall (a wrapped two-row grid at N density).
        let g = geometry_with_favorites(0, 3, 60.0);
        // 30 pt above the list top → inside the favorites band.
        let (target, valid) = resolve_target(1, -(48.0 + 30.0), center_press(), &g);
        assert_eq!(target, DragTarget::Favorites);
        assert!(valid);
        // The band's top edge boundary still targets favorites.
        let (target, _valid) = resolve_target(1, -(48.0 + 60.0), center_press(), &g);
        assert_eq!(target, DragTarget::Favorites);
        // One point above the band is the fixed chrome: not droppable.
        let (target, valid) = resolve_target(1, -(48.0 + 60.0 + 1.0), center_press(), &g);
        assert_eq!(target, DragTarget::Tab { slot: 1 });
        assert!(
            !valid,
            "fixed chrome above the favorites band is not a drop target"
        );
        // Without a rendered favorites region there is no favorites target.
        let g = geometry(0, 3);
        let (target, _valid) = resolve_target(1, -(48.0 + 30.0), center_press(), &g);
        assert_ne!(target, DragTarget::Favorites);
    }

    #[test]
    fn splice_index_maps_visual_slots_to_domain_indices() {
        // Drag from 1 to before original row 3 → splice insert at 2.
        assert_eq!(splice_index(1, 3), 2);
        // Drag from 3 to before original row 1 → splice insert at 1.
        assert_eq!(splice_index(3, 1), 1);
        // Dropping at the source slot is a no-op.
        assert_eq!(splice_index(2, 2), 2);
    }
}
