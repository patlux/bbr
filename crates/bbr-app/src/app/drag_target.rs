//! Pure tab-drag target resolution (DESIGN.md §10.9, §11.2, §11.3).
//!
//! The geometry that turns a pointer position into a typed drop target:
//! region-bounded insertion slots for the tab list, the §11.10 favorites
//! grid band above it, and the §11.3 permanent strip's rendered space chips
//! below it. Kept separate from the drag state machine in `super::drag` so
//! the resolution math and its unit tests stay small and layout-independent
//! — the only layout constants are the §8 density row heights, derived from
//! the same theme tokens the render uses, and the chip spans the app derives
//! from the production strip layout (never invented render indices).
//!
//! Target resolution is anchored at the press: the pointer's distance above
//! the tab list top is `-(dy) - (source * row_h + press_in_row)`, where
//! `press_in_row` is the press's y within the source row (0 at the top edge,
//! `row_h` at the bottom). Tab-slot resolution is exact in row units
//! regardless of the press offset (row-unit rounding from the source row);
//! the favorites band is addressed through the same anchor. The strip chips
//! live below the tab list in the footer, so they are hit-tested by the
//! pointer's absolute window position against the rendered chip spans — a
//! pointer entering a rendered chip targets that exact persistent chip,
//! never a render index. Outside the chips, the tab list, and the favorites
//! grid there is no drop target (an invalid no-target state).

use bbr_browser::SpaceId;

/// §8 normal-density row height the drag resolution uses. Derived from the
/// same theme token the shell renders with, so a density change is picked
/// up by construction.
pub fn tab_row_h(theme: &bbr_ui::theme::Theme) -> f32 {
    f32::from(theme.density.row())
}

/// A drag's current typed target (DESIGN.md §10.9): an insertion slot in
/// the tab list, the §11.10 favorites grid, or the §11.3 space chip under
/// the pointer. `Tab { slot }` means "insert before the row at `slot`";
/// `slot` equal to the source region's end renders at the end of that
/// region. `Chip { space }` moves the dragged tab into that space (the
/// persistent [`SpaceId`] — never a render position). The favorites target
/// toggles the dragged tab in/out of the window-global favorites grid
/// (add/remove — never a copy or a move).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragTarget {
    /// Insertion slot in the tab list, within the source tab's region.
    Tab { slot: usize },
    /// The §11.10 favorites grid: toggle the dragged tab as a favorite.
    Favorites,
    /// A §11.3 identity chip in the permanent footer strip: move the
    /// dragged tab into that exact persistent space.
    Chip { space: SpaceId },
}

/// One rendered space chip's visual span inside the strip (DESIGN.md
/// §11.3/§10.9), in window coordinates, clipped to the strip viewport. The
/// app derives these from the production `space_strip_scroll` handle's
/// laid-out chip bounds plus the signed scroll offset — the exact geometry
/// the pointer enters — never from a render index or an invented layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChipSpan {
    /// The persistent space this chip presents (the chip id identity).
    pub space: SpaceId,
    pub x_left: f32,
    pub x_right: f32,
    pub y_top: f32,
    pub y_bottom: f32,
}

impl ChipSpan {
    /// The chip's horizontal center (the deterministic pointer entry point
    /// the E2E seam and the benchmark driver use).
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn center_x(&self) -> f32 {
        (self.x_left + self.x_right) / 2.0
    }

    /// The chip's vertical center.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn center_y(&self) -> f32 {
        (self.y_top + self.y_bottom) / 2.0
    }

    /// Whether the pointer at `(x, y)` lies inside this chip's visible
    /// span (the production chip hit test).
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x_left && x < self.x_right && y >= self.y_top && y < self.y_bottom
    }
}

/// The tab-list geometry a drag resolves against: the pinned/regular
/// boundary (§11.2), the tab count, the §8 row height, the §11.10
/// favorites region's rendered drop-band span (its absolute window-y
/// bounds from the production band anchor; `None` when no band renders —
/// no droppable band), and the rendered strip chips below it. Grouped so
/// the pure resolver stays readable; the drag state machine builds it per
/// move from the same theme tokens, the recorded band rect, and the
/// production strip layout.
#[derive(Debug, Clone, Copy)]
pub struct DragGeometry<'a> {
    pub pinned: usize,
    pub len: usize,
    pub row_h: f32,
    /// The §11.10 favorites region's rendered drop band in absolute window
    /// coordinates `(y_top, y_bottom)` (recorded by the production band
    /// anchor, never a parallel layout model). `None` when the region does
    /// not render (no tiles and no in-flight drag — no droppable band).
    pub favorites_band: Option<(f32, f32)>,
    /// The §11.3 strip chips' visual spans, in canonical display order
    /// (empty when the strip layout has not run yet — no chip targets).
    pub chips: &'a [ChipSpan],
}

/// Resolve the drag target for a source row and a pointer position.
///
/// Pure and unit-testable: `pointer_x`/`pointer_y` are the pointer's
/// window position, `dy` its vertical displacement from the press,
/// `press_in_row` the press's y within the source row (0 at the row's top
/// edge, `row_h` at its bottom; the caller passes the true offset when the
/// layout exposes it and the row center `row_h / 2` otherwise), and
/// `geometry` the tab-list/strip layout (see [`DragGeometry`]).
///
/// Semantics (interaction contract):
/// - A pointer inside a rendered chip's span targets that exact persistent
///   chip (move). Chips win over the surrounding footer chrome, so a drop
///   on the `[Downloads]`/`[+]` controls or the gap between chips never
///   resolves to a slot.
/// - The pointer over the favorites region's rendered drop band (its
///   absolute window-y span, recorded from the production band anchor)
///   targets the §11.10 grid (a drop toggles the dragged tab in/out of the
///   window-global favorites). The band is addressed by the pointer's
///   absolute position, independent of the press anchor — exactly like the
///   strip chips — so it never drifts from the rendered region.
/// - The pointer over the tab list resolves to an insertion slot inside the
///   source region. A slot outside the region (the pinned/regular boundary
///   crossing) is illegal: the target clamps to the region edge so the tick
///   renders there, but `valid` is false.
/// - A pointer above the tab list that is not over the favorites band (the
///   fixed chrome — command field, brand row, section labels, gaps) is
///   **no target**: the drop is rejected whole.
/// - A pointer below the tab list that is not over a chip (the footer's
///   fixed controls, the gaps) is **no target**: the drop is rejected whole.
pub fn resolve_target(
    source: usize,
    pointer_x: f32,
    pointer_y: f32,
    dy: f32,
    press_in_row: f32,
    geometry: &DragGeometry,
) -> (Option<DragTarget>, bool) {
    if geometry.len == 0 {
        return (None, false);
    }
    let DragGeometry {
        pinned,
        len,
        row_h,
        favorites_band,
        chips,
    } = *geometry;
    // §11.3 drag-to-chip: the strip sits below the tab list in the footer;
    // a pointer inside a rendered chip's span targets that exact persistent
    // chip. The chip hit is absolute (window position), independent of the
    // press anchor, so it is exact for the pointer's real position over the
    // rendered strip.
    for chip in chips {
        if chip.contains(pointer_x, pointer_y) {
            return (Some(DragTarget::Chip { space: chip.space }), true);
        }
    }
    // The footer band (the strip's y-range) without a chip hit is **no
    // target**: the fixed `[Downloads]`/`[+]` controls and the gaps between
    // chips reject the drop whole. The strip band comes from the rendered
    // chip spans (all chips share the strip row band); with no laid-out
    // chip yet there is no footer evidence and the legacy list geometry
    // applies.
    if let Some(strip) = chips.first() {
        if pointer_y >= strip.y_top && pointer_y < strip.y_bottom {
            return (None, false);
        }
    }
    // §11.10 drag-to-favorites: the favorites region's rendered drop band
    // in absolute window-y coordinates (recorded by the production band
    // anchor — the exact geometry the pointer enters). A pointer inside
    // the band targets the grid; a drop toggles the dragged tab in/out of
    // the window-global favorites. The band is hit by the pointer's
    // absolute position, independent of the press anchor, so it lines up
    // exactly with the rendered region (labels/gaps above the list never
    // mis-target it).
    if let Some((band_top, band_bottom)) = favorites_band {
        if pointer_y >= band_top && pointer_y < band_bottom {
            return (Some(DragTarget::Favorites), true);
        }
    }
    // Distance from the pointer up to the tab list's top edge. The press
    // sits `source * row_h + press_in_row` below it; `dy` is the pointer's
    // displacement from the press.
    let above_list = -(dy) - (source as f32 * row_h + press_in_row);
    if above_list <= 0.0 {
        // Pointer over (or below) the tab list: resolve the insertion slot.
        // A pointer in the gap between the list bottom and the footer strip
        // still drops at the region end (the pre-strip list geometry); the
        // strip band itself is handled above.
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
        (Some(DragTarget::Tab { slot }), valid)
    } else {
        // Pointer above the tab list, not over the favorites band: the
        // fixed chrome (command field / brand row / section labels / the
        // gaps between the band and the list) is not droppable.
        (None, false)
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

    /// A 32 pt row geometry over the given tab list (no chips rendered).
    fn geometry<'a>(pinned: usize, len: usize, chips: &'a [ChipSpan]) -> DragGeometry<'a> {
        DragGeometry {
            pinned,
            len,
            row_h: 32.0,
            favorites_band: None,
            chips,
        }
    }

    /// The same geometry with a rendered favorites band above the list
    /// (its absolute window-y span `(y_top, y_bottom)` from the production
    /// band anchor).
    fn geometry_with_favorites<'a>(
        pinned: usize,
        len: usize,
        favorites_band: (f32, f32),
        chips: &'a [ChipSpan],
    ) -> DragGeometry<'a> {
        DragGeometry {
            pinned,
            len,
            row_h: 32.0,
            favorites_band: Some(favorites_band),
            chips,
        }
    }

    /// A chip span at the given x/y (the pure resolver only consumes the
    /// rect; the production derivation lives in `app::drag_chip`).
    fn chip(space: u64, x_left: f32, x_right: f32, y_top: f32, y_bottom: f32) -> ChipSpan {
        ChipSpan {
            space: SpaceId(space),
            x_left,
            x_right,
            y_top,
            y_bottom,
        }
    }

    #[test]
    fn reorder_targets_stay_inside_the_source_region() {
        // Pinned region [0, 2), regular [2, 5).
        let chips = [];
        let g = geometry(2, 5, &chips);
        // Regular tab at 3 dragged one row down: slot 4 (valid).
        let (target, valid) = resolve_target(3, 10.0, 232.0, 32.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Tab { slot: 4 }));
        assert!(valid);
        // …and one row up: slot 2 (still regular — the first regular slot).
        let (target, valid) = resolve_target(3, 10.0, 168.0, -32.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Tab { slot: 2 }));
        assert!(valid);
        // Pinned tab at 0 dragged one row down stays pinned.
        let (target, valid) = resolve_target(0, 10.0, 232.0, 32.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Tab { slot: 1 }));
        assert!(valid);
    }

    #[test]
    fn boundary_crossing_is_illegal_and_clamps_to_the_region_edge() {
        let chips = [];
        let g = geometry(2, 5, &chips);
        // Regular tab at 2 dragged one row up (still inside the list): the
        // target clamps to the region start (slot 2, the first regular row)
        // but is invalid — the pinned/regular boundary crossing.
        let (target, valid) = resolve_target(2, 10.0, 368.0, -32.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Tab { slot: 2 }));
        assert!(!valid, "boundary crossing is rejected whole");
        // Pinned tab dragged into the regular region (still inside the
        // list): clamps to the region end and is invalid.
        let (target, valid) = resolve_target(0, 10.0, 296.0, 96.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Tab { slot: 2 }));
        assert!(!valid);
    }

    #[test]
    fn dragging_to_the_region_end_is_valid_at_the_list_bottom_edge() {
        let chips = [];
        let g = geometry(2, 5, &chips);
        // Regular tab at 4 (last) dragged to the list's bottom edge
        // (dy = list_bottom_dy = 5*32 - (4*32 + 16) = 16): slot 5 = end of
        // the regular region, valid.
        let (target, valid) = resolve_target(4, 10.0, 216.0, 16.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Tab { slot: 5 }));
        assert!(valid);
        // Pinned last tab at 1 dragged down to the boundary edge: slot 2
        // (end of the pinned region) is valid.
        let (target, valid) = resolve_target(1, 10.0, 232.0, 32.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Tab { slot: 2 }));
        assert!(valid);
        // …and one more row is the boundary crossing: invalid.
        let (target, valid) = resolve_target(1, 10.0, 264.0, 64.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Tab { slot: 2 }));
        assert!(!valid);
    }

    #[test]
    fn below_the_list_outside_chips_is_no_target() {
        // A rendered strip (the footer band) below the list: a pointer in
        // the strip's y-range but outside every chip — the fixed
        // `[Downloads]`/`[+]` controls or the gaps between chips — is **no
        // target**: the drop is rejected whole.
        let chips = [
            chip(2, 60.0, 150.0, 640.0, 676.0),
            chip(4, 170.0, 240.0, 640.0, 676.0),
        ];
        let g = geometry(2, 5, &chips);
        // The gap between the two chips (y in the strip band, x between
        // them): no chip hit, no target.
        let (target, valid) = resolve_target(4, 160.0, 658.0, 200.0, center_press(), &g);
        assert_eq!(target, None, "the chip gap is not a drop target");
        assert!(!valid);
        // The fixed `[Downloads]` control left of the strip (still in the
        // strip band, outside the chips): no target.
        let (target, valid) = resolve_target(4, 20.0, 658.0, 200.0, center_press(), &g);
        assert_eq!(target, None, "fixed footer chrome is not a drop target");
        assert!(!valid);
        // A pointer still inside the gap between the list bottom and the
        // strip band keeps the legacy list geometry: the drop lands at the
        // region end.
        let (target, valid) = resolve_target(4, 10.0, 600.0, 200.0, center_press(), &g);
        assert_eq!(
            target,
            Some(DragTarget::Tab { slot: 5 }),
            "end-of-region drop"
        );
        assert!(valid);
    }

    #[test]
    fn pointer_over_the_favorites_band_targets_the_grid() {
        // The favorites region's rendered drop band is addressed by the
        // pointer's absolute window-y against its recorded span (independent
        // of the press anchor, exactly like the strip chips). A pointer
        // inside the band toggles the dragged tab in/out of the grid; a
        // pointer above the band is the fixed chrome; with no rendered band
        // there is no favorites target.
        let chips = [];
        let g = geometry_with_favorites(0, 3, (100.0, 160.0), &chips);
        // A pointer at y 130 (inside the band) targets the grid. The dy is
        // set so the pointer sits above the list (the press at y 200, row
        // center 16 of source row 1).
        let (target, valid) = resolve_target(1, 10.0, 130.0, -70.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Favorites));
        assert!(valid);
        // One point above the band is the fixed chrome: not droppable.
        let (target, valid) = resolve_target(1, 10.0, 99.0, -101.0, center_press(), &g);
        assert_eq!(target, None);
        assert!(
            !valid,
            "fixed chrome above the favorites band is not a drop target"
        );
        // Without a rendered favorites region there is no favorites target.
        let g = geometry(0, 3, &chips);
        let (target, _valid) = resolve_target(1, 10.0, 130.0, -70.0, center_press(), &g);
        assert_ne!(target, Some(DragTarget::Favorites));
    }

    #[test]
    fn the_favorites_band_is_hit_by_absolute_position_not_the_press_anchor() {
        // The band resolution is independent of the press anchor: any press
        // row resolves the band the same way once the pointer is inside its
        // rendered span (the band is addressed by absolute window-y, like
        // the chips — the anchor only feeds the tab-slot math). The band
        // span may overlap the anchor's list-top zone (the anchor is a
        // half-row approximation; the recorded rect is the real geometry),
        // so the band check runs first and wins inside the span.
        let chips = [];
        let g = geometry_with_favorites(2, 5, (120.0, 180.0), &chips);
        for source in [0, 2, 4] {
            let (target, valid) = resolve_target(source, 10.0, 150.0, -90.0, center_press(), &g);
            assert_eq!(target, Some(DragTarget::Favorites), "source {source}");
            assert!(valid, "source {source}");
        }
        // A pointer well below the band (over the list) still resolves the
        // source-region slot, not favorites.
        let (target, valid) = resolve_target(2, 10.0, 300.0, 60.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Tab { slot: 4 }));
        assert!(valid);
    }

    #[test]
    fn pointer_entering_a_rendered_chip_targets_that_exact_persistent_chip() {
        // The strip chips sit below the tab list in the footer. The chip
        // hit is absolute window position, so a pointer entering a chip's
        // span targets that exact persistent chip regardless of the press
        // anchor or the row displacement.
        let chips = [
            chip(7, 60.0, 150.0, 640.0, 676.0),
            chip(4, 160.0, 240.0, 640.0, 676.0),
        ];
        let g = geometry(0, 3, &chips);
        // Pointer over the second chip (persistent id 4): exact chip target,
        // valid, even though the y is deep in the footer band.
        let (target, valid) = resolve_target(1, 200.0, 658.0, 400.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Chip { space: SpaceId(4) }));
        assert!(valid);
        // The gap between chips is no target (the drop is rejected whole).
        let (target, valid) = resolve_target(1, 155.0, 658.0, 400.0, center_press(), &g);
        assert_eq!(target, None, "the gap between chips is not a target");
        assert!(!valid);
        // The footer chrome left of the strip is no target.
        let (target, valid) = resolve_target(1, 20.0, 658.0, 400.0, center_press(), &g);
        assert_eq!(
            target, None,
            "footer chrome outside the chips is not a target"
        );
        assert!(!valid);
        // A pointer inside a chip's y band but outside every chip's x span
        // is still no target.
        let (target, _valid) = resolve_target(1, 300.0, 658.0, 400.0, center_press(), &g);
        assert_eq!(target, None);
    }

    #[test]
    fn chip_hit_wins_over_the_tab_slot_zone_and_is_exact_for_the_press() {
        // A pointer in the footer band with a chip under it targets the
        // chip even when the y-anchored slot math would otherwise clamp
        // into the list.
        let chips = [chip(9, 60.0, 150.0, 640.0, 676.0)];
        let g = geometry(0, 3, &chips);
        let (target, valid) = resolve_target(2, 100.0, 650.0, 500.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Chip { space: SpaceId(9) }));
        assert!(valid);
        // Chip spans clip: a pointer at the chip's bottom edge is inside;
        // one point below is not.
        let (target, _valid) = resolve_target(2, 100.0, 675.5, 500.0, center_press(), &g);
        assert_eq!(target, Some(DragTarget::Chip { space: SpaceId(9) }));
        let (target, _valid) = resolve_target(2, 100.0, 677.0, 500.0, center_press(), &g);
        assert_ne!(target, Some(DragTarget::Chip { space: SpaceId(9) }));
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
