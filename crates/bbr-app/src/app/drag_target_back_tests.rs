//! Dragging back out of the pinned area (task packet
//! `drag-back-out-of-pins`): a pinned row dropped below the New Tab row
//! unpins at the regular slot under the pointer; a Favorite tile resolves
//! every target at the pointer's absolute position.

use super::*;
use bbr_domain::FolderId;

/// A rail-wide rect spanning window y `top..top + height`.
fn rect(top: f32, height: f32) -> DropRect {
    DropRect {
        x: 0.0,
        y: top,
        width: 300.0,
        height,
    }
}

/// The rendered layout these tests share (window y, 40 pt pitch, 36 pt
/// rows): Favorites 40..100; loose pinned rows 0 and 1 at 120 and 160
/// (pinned tab 2 is a Favorite, not a row); a folder row at 200; the ghost
/// row at 240; New Tab 280..316; regular rows 3 and 4 at 320 and 360;
/// chips at 600..636.
const ROWS: [(usize, DropRect); 4] = [
    (
        0,
        DropRect {
            x: 0.0,
            y: 120.0,
            width: 300.0,
            height: 36.0,
        },
    ),
    (
        1,
        DropRect {
            x: 0.0,
            y: 160.0,
            width: 300.0,
            height: 36.0,
        },
    ),
    (
        3,
        DropRect {
            x: 0.0,
            y: 320.0,
            width: 300.0,
            height: 36.0,
        },
    ),
    (
        4,
        DropRect {
            x: 0.0,
            y: 360.0,
            width: 300.0,
            height: 36.0,
        },
    ),
];

fn layout<'a>(
    folders: &'a [(FolderId, DropRect)],
    chips: &'a [ChipSpan],
    rows: &'a [(usize, DropRect)],
) -> DragGeometry<'a> {
    DragGeometry {
        pinned: 3,
        len: 5,
        pitch: 40.0,
        favorites_band: Some(rect(40.0, 60.0)),
        pin_offer: Some(rect(240.0, 36.0)),
        folders,
        rail: Some((0.0, 300.0)),
        chips,
        loose: Some(&[0, 1]),
        rows,
        regular_top: Some(316.0),
    }
}

fn chips() -> [ChipSpan; 1] {
    [ChipSpan {
        space: SpaceId(9),
        x_left: 60.0,
        x_right: 150.0,
        y_top: 600.0,
        y_bottom: 636.0,
    }]
}

/// Pinned row 1 pressed at its center (y 178) and moved to `y`.
fn drag_pinned_row(y: f32, geometry: &DragGeometry) -> (Option<DragTarget>, bool) {
    resolve_target(1, 10.0, y, y - 178.0, 18.0, geometry)
}

#[test]
fn a_pinned_row_below_new_tab_unpins_at_the_slot_under_the_pointer() {
    let folders = [(FolderId(7), rect(200.0, 36.0))];
    let chips = chips();
    let g = layout(&folders, &chips, &ROWS);
    // Upper half of regular row 3 → before it; lower half → before row 4.
    assert_eq!(
        drag_pinned_row(325.0, &g),
        (Some(DragTarget::Unpin { slot: 3 }), true)
    );
    assert_eq!(
        drag_pinned_row(345.0, &g),
        (Some(DragTarget::Unpin { slot: 4 }), true)
    );
    // Below the last regular row: the region's end.
    assert_eq!(
        drag_pinned_row(450.0, &g),
        (Some(DragTarget::Unpin { slot: 5 }), true)
    );
    // The gap between New Tab and the first row belongs to the region.
    assert_eq!(
        drag_pinned_row(317.0, &g),
        (Some(DragTarget::Unpin { slot: 3 }), true)
    );
    // Over the New Tab row the drag is still a pinned drag: it clamps to
    // the pinned region's end and is not a drop.
    assert_eq!(
        drag_pinned_row(300.0, &g),
        (Some(DragTarget::Tab { slot: 3 }), false)
    );
    // Within the loose rows it reorders as before.
    assert_eq!(
        drag_pinned_row(138.0, &g),
        (Some(DragTarget::Tab { slot: 0 }), true)
    );
    // The strip and the folder still win.
    assert_eq!(
        drag_pinned_row(618.0, &g).0,
        None,
        "the strip band outside a chip is no target"
    );
    assert_eq!(
        drag_pinned_row(210.0, &g),
        (
            Some(DragTarget::Folder {
                folder: FolderId(7)
            }),
            true
        )
    );
}

#[test]
fn an_empty_regular_region_takes_the_drop_at_its_end() {
    let chips = chips();
    let rows = &ROWS[..2];
    let g = DragGeometry {
        len: 3,
        ..layout(&[], &chips, rows)
    };
    assert_eq!(
        drag_pinned_row(330.0, &g),
        (Some(DragTarget::Unpin { slot: 3 }), true)
    );
}

#[test]
fn without_a_recorded_new_tab_row_a_pinned_row_stays_in_its_region() {
    let chips = chips();
    let g = DragGeometry {
        regular_top: None,
        ..layout(&[], &chips, &ROWS)
    };
    assert_eq!(
        drag_pinned_row(345.0, &g),
        (Some(DragTarget::Tab { slot: 3 }), false)
    );
}

#[test]
fn a_regular_row_never_unpins_and_is_never_offered_its_own_region_as_unpin() {
    let chips = chips();
    let g = layout(&[], &chips, &ROWS);
    // Regular row 3 pressed at its center (338) and moved down one pitch.
    let (target, valid) = resolve_target(3, 10.0, 378.0, 40.0, 18.0, &g);
    assert_eq!(target, Some(DragTarget::Tab { slot: 4 }));
    assert!(valid);
}

#[test]
fn a_favorite_resolves_every_target_at_the_pointer() {
    let folders = [(FolderId(7), rect(200.0, 36.0))];
    let chips = chips();
    let g = layout(&folders, &chips, &ROWS);
    let at = |y: f32| resolve_favorite_target(10.0, y, &g);
    // Its own grid is no target.
    assert_eq!(at(70.0), (None, false));
    // Among the loose pinned rows: before row 0, before row 1, at the end.
    assert_eq!(at(125.0), (Some(DragTarget::Tab { slot: 0 }), true));
    assert_eq!(at(170.0), (Some(DragTarget::Tab { slot: 1 }), true));
    assert_eq!(at(190.0), (Some(DragTarget::Tab { slot: 3 }), true));
    // Half a pitch above the first loose row still addresses it.
    assert_eq!(at(105.0), (Some(DragTarget::Tab { slot: 0 }), true));
    // The folder row, the ghost row, the regular region, a chip.
    assert_eq!(
        at(210.0),
        (
            Some(DragTarget::Folder {
                folder: FolderId(7)
            }),
            true
        )
    );
    assert_eq!(at(250.0), (Some(DragTarget::PinOffer), true));
    assert_eq!(at(365.0), (Some(DragTarget::Unpin { slot: 4 }), true));
    assert_eq!(
        resolve_favorite_target(100.0, 618.0, &g),
        (Some(DragTarget::Chip { space: SpaceId(9) }), true)
    );
    // The New Tab row and the page beside the rail are no target.
    assert_eq!(at(300.0), (None, false));
    assert_eq!(resolve_favorite_target(320.0, 330.0, &g), (None, false));
}

#[test]
fn a_favorite_without_loose_rows_takes_the_ghost_row() {
    let chips = chips();
    let rows = &ROWS[2..];
    let g = DragGeometry {
        loose: Some(&[]),
        ..layout(&[], &chips, rows)
    };
    assert_eq!(
        resolve_favorite_target(10.0, 250.0, &g),
        (Some(DragTarget::PinOffer), true)
    );
    assert_eq!(resolve_favorite_target(10.0, 150.0, &g), (None, false));
}
