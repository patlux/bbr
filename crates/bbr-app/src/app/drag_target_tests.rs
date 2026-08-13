use super::*;

/// Row-surface-center press offset for the §8 normal row (36 pt
/// surface; the press anchor stays within the surface, `row() / 2`).
fn center_press() -> f32 {
    18.0
}

/// A 40 pt pitch geometry over the given tab list (rows step on
/// surface + gap; no chips rendered).
fn geometry<'a>(pinned: usize, len: usize, chips: &'a [ChipSpan]) -> DragGeometry<'a> {
    DragGeometry {
        pinned,
        len,
        pitch: 40.0,
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
        pitch: 40.0,
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
fn the_list_top_edge_targets_slot_zero_but_fixed_chrome_does_not() {
    let chips = [];
    let g = geometry(2, 5, &chips);
    // The first row is pressed at its 18 pt surface center. Moving exactly
    // to the list's top edge stays over the list and targets its first slot.
    let (target, valid) = resolve_target(0, 10.0, 182.0, -18.0, center_press(), &g);
    assert_eq!(target, Some(DragTarget::Tab { slot: 0 }));
    assert!(valid);
    // A full 40 pt pitch from that center is 22 pt above the list: fixed
    // chrome, not an insertion target. This protects the product contract
    // while the renderer independently keeps the slot-0 tick contained.
    let (target, valid) = resolve_target(0, 10.0, 160.0, -40.0, center_press(), &g);
    assert_eq!(target, None);
    assert!(!valid);
}

#[test]
fn reorder_targets_stay_inside_the_source_region() {
    // Pinned region [0, 2), regular [2, 5).
    let chips = [];
    let g = geometry(2, 5, &chips);
    // Regular tab at 3 dragged one pitch (40 pt) down: slot 4 (valid).
    let (target, valid) = resolve_target(3, 10.0, 232.0, 40.0, center_press(), &g);
    assert_eq!(target, Some(DragTarget::Tab { slot: 4 }));
    assert!(valid);
    // …and one pitch up: slot 2 (still regular — the first regular slot).
    let (target, valid) = resolve_target(3, 10.0, 152.0, -40.0, center_press(), &g);
    assert_eq!(target, Some(DragTarget::Tab { slot: 2 }));
    assert!(valid);
    // Pinned tab at 0 dragged one pitch down stays pinned.
    let (target, valid) = resolve_target(0, 10.0, 232.0, 40.0, center_press(), &g);
    assert_eq!(target, Some(DragTarget::Tab { slot: 1 }));
    assert!(valid);
}

#[test]
fn boundary_crossing_is_illegal_and_clamps_to_the_region_edge() {
    let chips = [];
    let g = geometry(2, 5, &chips);
    // Regular tab at 2 dragged one pitch up (still inside the list): the
    // target clamps to the region start (slot 2, the first regular row)
    // but is invalid — the pinned/regular boundary crossing.
    let (target, valid) = resolve_target(2, 10.0, 368.0, -40.0, center_press(), &g);
    assert_eq!(target, Some(DragTarget::Tab { slot: 2 }));
    assert!(!valid, "boundary crossing is rejected whole");
    // Pinned tab dragged three pitches into the regular region (still
    // inside the list): clamps to the region end and is invalid.
    let (target, valid) = resolve_target(0, 10.0, 296.0, 120.0, center_press(), &g);
    assert_eq!(target, Some(DragTarget::Tab { slot: 2 }));
    assert!(!valid);
}

#[test]
fn dragging_to_the_region_end_is_valid_at_the_list_bottom_edge() {
    let chips = [];
    let g = geometry(2, 5, &chips);
    // Regular tab at 4 (last) dragged to the list's bottom edge
    // (dy = list_bottom_dy = 5*40 - (4*40 + 18) = 22): slot 5 = end of
    // the regular region, valid.
    let (target, valid) = resolve_target(4, 10.0, 216.0, 22.0, center_press(), &g);
    assert_eq!(target, Some(DragTarget::Tab { slot: 5 }));
    assert!(valid);
    // Pinned last tab at 1 dragged down to the boundary edge: slot 2
    // (end of the pinned region) is valid.
    let (target, valid) = resolve_target(1, 10.0, 232.0, 40.0, center_press(), &g);
    assert_eq!(target, Some(DragTarget::Tab { slot: 2 }));
    assert!(valid);
    // …and one more pitch is the boundary crossing: invalid.
    let (target, valid) = resolve_target(1, 10.0, 264.0, 80.0, center_press(), &g);
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
    // center 18 of source row 1).
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
