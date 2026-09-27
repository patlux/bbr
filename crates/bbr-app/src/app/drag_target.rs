//! Pure tab-drag target resolution (DESIGN.md §10.9, §11.2, §11.3).
//!
//! The geometry that turns a pointer position into a typed drop target:
//! region-bounded insertion slots for the tab list, the §11.10 favorites
//! grid band above it, and the §11.3 permanent strip's rendered space chips
//! below it. Kept separate from the drag state machine in `super::drag` so
//! the resolution math and its unit tests stay small and layout-independent
//! — the only layout constants are the §8 density pitch (surface + gap),
//! derived from the same theme tokens the render uses, and the chip spans
//! the app derives from the production strip layout (never invented render
//! indices).
//!
//! Target resolution is anchored at the press: the pointer's distance above
//! the tab list top is `-(dy) - (source * pitch + press_in_row)`, where
//! `press_in_row` is the press's y within the source row surface (0 at the
//! top edge, `row` at the bottom; the row center `row / 2` is the default
//! anchor). Tab-slot resolution is exact in pitch units regardless of the
//! press offset (pitch-unit rounding from the source row); the favorites
//! band is addressed through the same anchor. The strip chips live below
//! the tab list in the footer, so they are hit-tested by the pointer's
//! absolute window position against the rendered chip spans — a pointer
//! entering a rendered chip targets that exact persistent chip, never a
//! render index. Outside the chips, the tab list, and the favorites grid
//! there is no drop target (an invalid no-target state).

use bbr_browser::SpaceId;

/// §8 normal-density row pitch the drag resolution uses. Derived from the
/// same theme token the shell renders with, so a density change is picked
/// up by construction. The pitch (`density:pitch` = surface + gap) is the
/// step rows render on, so drag resolution and rendered geometry stay in
/// the same units; the press anchor stays within the row surface
/// (`density.row()`), see [`DragGeometry`].
pub fn tab_pitch(theme: &bbr_ui::theme::Theme) -> f32 {
    f32::from(theme.density.pitch())
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
    /// The §11.2 "Pin tab here" ghost row: pin the dragged (unpinned) tab.
    PinOffer,
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
/// boundary (§11.2), the tab count, the §8 row pitch, the §11.10
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
    /// The §8 row pitch (surface + gap) — the step the rendered rows sit
    /// on, so pointer-to-slot resolution lands on the exact slot the render
    /// shows (§11.6).
    pub pitch: f32,
    /// The §11.10 favorites region's rendered drop band in absolute window
    /// coordinates `(y_top, y_bottom)` (recorded by the production band
    /// anchor, never a parallel layout model). `None` when the region does
    /// not render (no tiles and no in-flight drag — no droppable band).
    pub favorites_band: Option<(f32, f32)>,
    /// The §11.2 ghost row's rendered span `(y_top, y_bottom)` in window
    /// coordinates (recorded by its anchor). `None` while no ghost row
    /// renders or it does not offer pinning to this source.
    pub pin_offer: Option<(f32, f32)>,
    /// The §11.3 strip chips' visual spans, in canonical display order
    /// (empty when the strip layout has not run yet — no chip targets).
    pub chips: &'a [ChipSpan],
}

/// Resolve the drag target for a source row and a pointer position.
///
/// Pure and unit-testable: `pointer_x`/`pointer_y` are the pointer's
/// window position, `dy` its vertical displacement from the press,
/// `press_in_row` the press's y within the source row surface (0 at the
/// surface's top edge, `row` at its bottom; the caller passes the true
/// offset when the layout exposes it and the row center `row / 2`
/// otherwise), and `geometry` the tab-list/strip layout (see
/// [`DragGeometry`]). Rows sit on the §8 pitch (surface + gap), so the
/// source anchor and the slot step both use the pitch units the render
/// lays out in.
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
/// - The pointer over the §11.2 ghost row's recorded span targets the pin
///   offer (a drop pins the dragged tab).
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
        pitch,
        favorites_band,
        pin_offer,
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
    // §11.2 drag-to-pin: the ghost row the organized area appends while an
    // unpinned tab is dragged, hit by its recorded absolute span. It is a
    // region-level offer: a drop pins the tab at the end of the pinned
    // block.
    if let Some((row_top, row_bottom)) = pin_offer {
        if pointer_y >= row_top && pointer_y < row_bottom {
            return (Some(DragTarget::PinOffer), true);
        }
    }
    // Distance from the pointer up to the tab list's top edge. The press
    // sits `source * pitch + press_in_row` below it (rows step on the §8
    // pitch); `dy` is the pointer's displacement from the press.
    let above_list = -(dy) - (source as f32 * pitch + press_in_row);
    if above_list <= 0.0 {
        // Pointer over (or below) the tab list: resolve the insertion slot.
        // A pointer in the gap between the list bottom and the footer strip
        // still drops at the region end (the pre-strip list geometry); the
        // strip band itself is handled above.
        let raw = source as isize + (dy / pitch).round() as isize;
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
#[path = "drag_target_tests.rs"]
mod tests;
