//! Pure tab-drag target resolution (DESIGN.md §10.9, §11.2).
//!
//! The geometry that turns a pointer displacement into a typed drop target:
//! region-bounded insertion slots for the tab list and visible space rows for
//! moves. Kept separate from the drag state machine in `super::drag` so the
//! resolution math and its unit tests stay small and layout-independent — the
//! only layout constants are the §8 density row heights, derived from the
//! same theme tokens the render uses.
//!
//! Target resolution is anchored at the press: the pointer's distance above
//! the tab list top is `-(dy) - (source * row_h + press_in_row)`, where
//! `press_in_row` is the press's y within the source row (0 at the top edge,
//! `row_h` at the bottom). Tab-slot resolution is exact in row units
//! regardless of the press offset (row-unit rounding from the source row);
//! the space surface is addressed through the same anchor, so space-row
//! targeting is exact for the recorded press offset. When the layout cannot
//! expose the true offset (GPUI 0.2.2 offers no element geometry), the caller
//! anchors at the row center (`row_h / 2`), which bounds the space-row error
//! at half a row.

use bbr_browser::SpaceId;

/// §8 normal-density row heights the drag resolution uses. Derived from the
/// same theme tokens the shell renders with, so a density change is picked
/// up by construction; the drag math only ever uses these two.
pub fn tab_row_h(theme: &crate::theme::Theme) -> f32 {
    f32::from(theme.density.row())
}

/// §8 space-row height (`density:space-row`).
pub fn space_row_h(theme: &crate::theme::Theme) -> f32 {
    f32::from(theme.density.space_row())
}

/// A drag's current typed target (DESIGN.md §10.9): either an insertion
/// slot in the tab list, a visible space row, or the §11.10 favorites grid.
/// `Tab { slot }` means "insert before the row at `slot`"; `slot` equal to
/// the source region's end renders at the end of that region. Space targets
/// move the tab; the favorites target toggles the dragged tab in/out of the
/// window-global favorites grid (add/remove — never a copy or a move).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragTarget {
    /// Insertion slot in the tab list, within the source tab's region.
    Tab { slot: usize },
    /// A visible space row: move the dragged tab into that space.
    Space { space: SpaceId },
    /// The §11.10 favorites grid: toggle the dragged tab as a favorite.
    Favorites,
}

/// One rendered space row's position inside the space surface, top-down.
/// The offsets mirror the sidebar's `space_surface` composition (profile
/// groups with hairlines, then the New Space row), so the drag can resolve
/// a pointer above the tab list into the exact space row it hovers.
#[derive(Debug, Clone, Copy)]
pub struct SpaceRowSpan {
    pub space: SpaceId,
    /// Distance from the space surface's top edge (points).
    pub y_top: f32,
    pub height: f32,
}

/// The tab-list geometry a drag resolves against: the pinned/regular
/// boundary (§11.2), the tab count, the §8 row height, the favorites region
/// above the list (its deterministic height; 0 when the grid is empty), and
/// the space surface above that (its top-down rows plus total height).
/// Grouped so the pure resolver stays readable; the drag state machine
/// builds it per move from the same theme tokens, the organization read
/// model, and `shell::space_groups` the render uses.
#[derive(Debug, Clone, Copy)]
pub struct DragGeometry<'a> {
    pub pinned: usize,
    pub len: usize,
    pub row_h: f32,
    /// The §11.10 favorites region's deterministic height above the tab
    /// list (0 when the grid is empty/hidden — no droppable band).
    pub favorites_h: f32,
    pub space_h: f32,
    pub spaces: &'a [SpaceRowSpan],
}

/// Resolve the drag target for a source row and a row-unit displacement.
///
/// Pure and unit-testable: `dy` is the pointer's vertical displacement from
/// the press, `press_in_row` the press's y within the source row (0 at the
/// row's top edge, `row_h` at its bottom; the caller passes the true offset
/// when the layout exposes it and the row center `row_h / 2` otherwise), and
/// `geometry` the tab-list/space-surface layout (see [`DragGeometry`]).
///
/// Semantics (interaction contract):
/// - The pointer over the tab list resolves to an insertion slot inside the
///   source region. A slot outside the region (the pinned/regular boundary
///   crossing) is illegal: the target clamps to the region edge so the tick
///   renders there, but `valid` is false.
/// - The pointer above the tab list resolves to the §11.10 favorites grid
///   band first (when the region is rendered — a drop toggles the dragged
///   tab in/out of the window-global favorites), then to the visible space
///   row it hovers (move). The New Space row and the fixed chrome above the
///   space surface are not droppable.
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
        space_h,
        spaces,
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
        // `favorites_h` immediately above it (when rendered), then the
        // space surface above that (rows top-down, then the New Space row
        // at the bottom). A drop on the favorites band toggles the dragged
        // tab in the window-global grid; `from_bottom` inverts the pointer
        // distance inside the space surface.
        if favorites_h > 0.0 && above_list <= favorites_h {
            return (DragTarget::Favorites, true);
        }
        let from_bottom = space_h - (above_list - favorites_h);
        for span in spaces {
            if span.y_top <= from_bottom && from_bottom < span.y_top + span.height {
                return (DragTarget::Space { space: span.space }, true);
            }
        }
        (DragTarget::Tab { slot: source }, false)
    }
}

/// Build the rendered space rows' top-down offsets, mirroring
/// `shell::space_groups` plus the §11.3 New Space row at the foot. Returns
/// the spans and the surface's total height. Pure so the drag resolution and
/// its unit tests never depend on layout state.
pub fn space_row_spans(rows: &[(SpaceId, usize)], space_row_h: f32) -> (Vec<SpaceRowSpan>, f32) {
    let mut spans = Vec::with_capacity(rows.len());
    let mut y = 0.0;
    for (i, (space, _)) in rows.iter().enumerate() {
        // A hairline separates profile groups (§11.3); `rows` is already
        // grouped, so the group boundary is where the profile changes.
        if i > 0 && rows[i - 1].1 != rows[i].1 {
            y += 1.0;
        }
        spans.push(SpaceRowSpan {
            space: *space,
            y_top: y,
            height: space_row_h,
        });
        y += space_row_h;
    }
    // The New Space row at the foot is part of the surface but never a
    // drop target.
    (spans, y + space_row_h)
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

    fn spans_for(spaces: &[(u64, usize)]) -> (Vec<SpaceRowSpan>, f32) {
        let rows: Vec<(SpaceId, usize)> = spaces
            .iter()
            .map(|(id, profile)| (SpaceId(*id), *profile))
            .collect();
        space_row_spans(&rows, 34.0)
    }

    /// Row-center press offset for the classic 32 pt row height.
    fn center_press() -> f32 {
        16.0
    }

    /// A 32 pt row geometry over the given space spans.
    fn geometry<'a>(
        pinned: usize,
        len: usize,
        space_h: f32,
        spaces: &'a [SpaceRowSpan],
    ) -> DragGeometry<'a> {
        DragGeometry {
            pinned,
            len,
            row_h: 32.0,
            favorites_h: 0.0,
            space_h,
            spaces,
        }
    }

    /// The same geometry with a rendered favorites band above the list.
    fn geometry_with_favorites<'a>(
        pinned: usize,
        len: usize,
        favorites_h: f32,
        space_h: f32,
        spaces: &'a [SpaceRowSpan],
    ) -> DragGeometry<'a> {
        DragGeometry {
            pinned,
            len,
            row_h: 32.0,
            favorites_h,
            space_h,
            spaces,
        }
    }

    #[test]
    fn space_row_spans_stack_groups_with_hairlines_and_new_space_foot() {
        // One profile, two spaces: no hairline, New Space row at the foot.
        let (spans, h) = spans_for(&[(1, 0), (2, 0)]);
        assert_eq!(h, 3.0 * 34.0, "two spaces + New Space row");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].y_top, 0.0);
        assert_eq!(spans[1].y_top, 34.0);
        // Two profiles: a 1 pt hairline separates the groups.
        let (spans, h) = spans_for(&[(1, 0), (2, 1)]);
        assert_eq!(h, 2.0 * 34.0 + 1.0 + 34.0);
        assert_eq!(spans[0].y_top, 0.0);
        assert_eq!(spans[1].y_top, 35.0, "hairline pushes the second group");
    }

    #[test]
    fn reorder_targets_stay_inside_the_source_region() {
        // Pinned region [0, 2), regular [2, 5).
        let (spaces, space_h) = spans_for(&[(1, 0)]);
        let g = geometry(2, 5, space_h, &spaces);
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
        let (spaces, space_h) = spans_for(&[(1, 0)]);
        let g = geometry(2, 5, space_h, &spaces);
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
        let (spaces, space_h) = spans_for(&[(1, 0)]);
        let g = geometry(2, 5, space_h, &spaces);
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
    fn pointer_above_the_tab_list_targets_the_hovered_space_row() {
        let (spaces, space_h) = spans_for(&[(1, 0), (2, 0)]);
        let g = geometry(0, 3, space_h, &spaces);
        // Source row 1: its center is 48 pt below the list top. The second
        // space row (s=1) sits `space_h - (1*34 + 17)` = 102 - 51 = 51 pt
        // above the list top → dy = -(48 + 51) = -99.
        let (target, valid) = resolve_target(1, -99.0, center_press(), &g);
        assert_eq!(target, DragTarget::Space { space: SpaceId(2) });
        assert!(valid);
        // Higher up: the first space row (s=0) at 102 - 17 = 85 above →
        // dy = -(48 + 85) = -133.
        let (target, valid) = resolve_target(1, -133.0, center_press(), &g);
        assert_eq!(target, DragTarget::Space { space: SpaceId(1) });
        assert!(valid);
        // Into the New Space row (the foot of the surface): not droppable.
        let (target, valid) = resolve_target(1, -48.0 - 3.0, center_press(), &g);
        assert!(!valid, "New Space row is not a drop target");
        assert_eq!(
            target,
            DragTarget::Tab { slot: 1 },
            "falls back to the source slot, invalid"
        );
    }

    #[test]
    fn space_surface_hit_is_exact_for_off_center_presses() {
        let (spaces, space_h) = spans_for(&[(1, 0), (2, 0)]);
        let g = geometry(0, 3, space_h, &spaces);
        // Press at the top edge of source row 1 (offset 0): the press sits
        // 32 pt below the list top, so hovering the second space row's
        // center (51 pt above the list top) is dy = -(32 + 0 + 51) = -83.
        let (target, valid) = resolve_target(1, -83.0, 0.0, &g);
        assert_eq!(target, DragTarget::Space { space: SpaceId(2) });
        assert!(valid);
        // Press at the bottom edge (offset 32): dy = -(32 + 32 + 51) = -115.
        let (target, valid) = resolve_target(1, -115.0, 32.0, &g);
        assert_eq!(target, DragTarget::Space { space: SpaceId(2) });
        assert!(valid);
        // The tab/space zone boundary is exact for the recorded offset too:
        // with a top-edge press the pointer reaches the list top at dy = -32
        // (still the tab zone, row-unit slot), and one point above enters the
        // surface's New Space foot — not droppable.
        let (target, valid) = resolve_target(1, -32.0, 0.0, &g);
        assert_eq!(target, DragTarget::Tab { slot: 0 });
        assert!(valid);
        let (target, valid) = resolve_target(1, -33.0, 0.0, &g);
        assert_eq!(target, DragTarget::Tab { slot: 1 });
        assert!(!valid, "New Space foot is not a drop target");
    }

    #[test]
    fn pointer_over_the_favorites_band_targets_the_grid() {
        // The favorites band occupies the region directly above the tab
        // list; a drop there toggles the dragged tab in/out of the grid.
        // Source row 1's center is 48 pt below the list top; the band is
        // 60 pt tall (a wrapped two-row grid at N density).
        let (spaces, space_h) = spans_for(&[(1, 0), (2, 0)]);
        let g = geometry_with_favorites(0, 3, 60.0, space_h, &spaces);
        // 30 pt above the list top → inside the favorites band.
        let (target, valid) = resolve_target(1, -(48.0 + 30.0), center_press(), &g);
        assert_eq!(target, DragTarget::Favorites);
        assert!(valid);
        // The band's top edge boundary still targets favorites.
        let (target, _valid) = resolve_target(1, -(48.0 + 60.0), center_press(), &g);
        assert_eq!(target, DragTarget::Favorites);
        // One point above the band enters the space surface: the second
        // space row (s=1) sits `space_h - (1*34 + 17)` = 102 - 51 = 51 pt
        // above the band top → dy = -(48 + 60 + 51) = -159.
        let (target, valid) = resolve_target(1, -159.0, center_press(), &g);
        assert_eq!(target, DragTarget::Space { space: SpaceId(2) });
        assert!(valid);
        // Without a rendered favorites region there is no favorites target.
        let g = geometry(0, 3, space_h, &spaces);
        let (target, _valid) = resolve_target(1, -99.0, center_press(), &g);
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
