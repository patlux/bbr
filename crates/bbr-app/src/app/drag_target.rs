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
//!
//! Blueprint D02: every region target is hit in two dimensions — the
//! Favorites band, the ghost row and the folder rows through the rects
//! their anchors recorded, the list only inside the rail's x-span — so the
//! same resolver serves the docked and the floating rail, and a pointer
//! over the page beside a row never drops onto it.

use bbr_browser::SpaceId;
use bbr_domain::FolderId;

/// A rendered drop region in window points (top-left origin).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DropRect {
    /// Whether `(x, y)` lies inside the rect (half-open on the far edges).
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

impl From<bbr_web::WebRect> for DropRect {
    fn from(rect: bbr_web::WebRect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

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
    /// A §11.11 folder row: move the dragged tab into that folder (a
    /// regular tab is pinned by the same transition).
    Folder { folder: FolderId },
    /// The regular region, for a pinned row or a Favorite dragged back out
    /// of the pinned area (task packet `drag-back-out-of-pins`): unpin the
    /// dragged tab and insert it before the regular row at `slot` (tab
    /// index; `slot == len` is the region's end).
    Unpin { slot: usize },
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
    /// The §11.10 favorites region's rendered drop band in window
    /// coordinates (recorded by the production band anchor, never a
    /// parallel layout model). `None` when the region does not render (no
    /// tiles and no in-flight drag — no droppable band).
    pub favorites_band: Option<DropRect>,
    /// The §11.2 ghost row's rendered rect (recorded by its anchor). `None`
    /// while no ghost row renders or it does not offer pinning to this
    /// source.
    pub pin_offer: Option<DropRect>,
    /// The §11.11 folder rows' rendered rects (recorded by their anchors).
    pub folders: &'a [(FolderId, DropRect)],
    /// The rail's horizontal span `(x_left, x_right)` in window points: a
    /// pointer outside it has no target. `None` skips the check (pure
    /// list-math tests).
    pub rail: Option<(f32, f32)>,
    /// The §11.3 strip chips' visual spans, in canonical display order
    /// (empty when the strip layout has not run yet — no chip targets).
    pub chips: &'a [ChipSpan],
    /// §11.11: the pinned region's rendered (loose) rows as tab indices,
    /// when grouped pinned tabs render inside their folders instead. `None`
    /// means the pinned rows are exactly `0..pinned`.
    pub loose: Option<&'a [usize]>,
    /// The rendered tab rows' rects keyed by tab index (loose pinned rows
    /// and the regular window, recorded by their anchors). A drag back out
    /// of the pinned area resolves its slot against them.
    pub rows: &'a [(usize, DropRect)],
    /// The window y where the regular region begins (the New Tab row's
    /// bottom edge). `None` while unknown: a pinned row then stays inside
    /// its region (pure list-math tests).
    pub regular_top: Option<f32>,
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
/// - The pointer over a §11.11 folder row's recorded rect targets that
///   folder (a drop moves the dragged tab into it).
/// - A pointer outside the rail's x-span has no target.
/// - A pinned source over the regular region (below the New Tab row)
///   targets [`DragTarget::Unpin`] at the regular slot under the pointer:
///   dragging a pinned row back among the regular tabs unpins it there.
/// - The pointer over the tab list resolves to an insertion slot inside the
///   source region. A regular row dragged above its region (into the
///   pinned rows) is illegal: the target clamps to the region edge so the
///   tick renders there, but `valid` is false — the ghost row is the way
///   to pin.
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
    if let Some(hit) = region_hit(pointer_x, pointer_y, geometry, true) {
        return hit;
    }
    let DragGeometry {
        pinned,
        len,
        pitch,
        loose,
        ..
    } = *geometry;
    // Task packet `drag-back-out-of-pins`: a pinned row dragged below the
    // New Tab row is dropped among the regular tabs (it is unpinned there).
    if source < pinned {
        if let Some(slot) = regular_slot(pointer_y, geometry) {
            return (Some(DragTarget::Unpin { slot }), true);
        }
    }
    // §11.11: a pinned source resolves among the rendered (loose) pinned
    // rows — grouped tabs render inside their folders — and the slot maps
    // back to its tab index (the row after the last loose row is the
    // region end, `pinned`).
    if source < pinned {
        if let Some(visual) = loose.and_then(|rows| rows.iter().position(|row| *row == source)) {
            let rows = loose.expect("checked above");
            let above_list = -(dy) - (visual as f32 * pitch + press_in_row);
            if above_list > 0.0 {
                return (None, false);
            }
            let raw = visual as isize + (dy / pitch).round() as isize;
            let hi = rows.len() as isize;
            let slot = rows
                .get(raw.clamp(0, hi) as usize)
                .copied()
                .unwrap_or(pinned);
            let valid = raw <= hi || (pinned == len && raw > hi);
            return (Some(DragTarget::Tab { slot }), valid);
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

/// The hits every drag shares, in precedence order: outside the rail (no
/// target), a space chip, the footer strip band (no target), the Favorites
/// band, the §11.2 ghost row and a §11.11 folder row. `favorites` is false
/// for a Favorite's own drag: its grid is then no target. `None` when the
/// pointer is over none of them (the tab list decides).
fn region_hit(
    pointer_x: f32,
    pointer_y: f32,
    geometry: &DragGeometry,
    favorites: bool,
) -> Option<(Option<DragTarget>, bool)> {
    let DragGeometry {
        favorites_band,
        pin_offer,
        folders,
        rail,
        chips,
        ..
    } = *geometry;
    // Blueprint D02: the rail is the drop surface; the page beside it is
    // not (a pointer over the web view at a row's height drops nothing).
    if let Some((left, right)) = rail {
        if pointer_x < left || pointer_x >= right {
            return Some((None, false));
        }
    }
    // §11.3 drag-to-chip: the strip sits below the tab list in the footer;
    // a pointer inside a rendered chip's span targets that exact persistent
    // chip. The chip hit is absolute (window position), independent of the
    // press anchor, so it is exact for the pointer's real position over the
    // rendered strip.
    for chip in chips {
        if chip.contains(pointer_x, pointer_y) {
            return Some((Some(DragTarget::Chip { space: chip.space }), true));
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
            return Some((None, false));
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
    if favorites_band.is_some_and(|band| band.contains(pointer_x, pointer_y)) {
        // A Favorite dragged over its own grid has nowhere to go there.
        return Some(if favorites {
            (Some(DragTarget::Favorites), true)
        } else {
            (None, false)
        });
    }
    // §11.2 drag-to-pin: the ghost row the organized area appends while an
    // unpinned tab is dragged, hit by its recorded absolute span. It is a
    // region-level offer: a drop pins the tab at the end of the pinned
    // block.
    if pin_offer.is_some_and(|row| row.contains(pointer_x, pointer_y)) {
        return Some((Some(DragTarget::PinOffer), true));
    }
    // §11.11 drag-to-folder: a pointer over a rendered folder row targets
    // that folder; a drop moves the dragged tab into it.
    if let Some((folder, _)) = folders
        .iter()
        .find(|(_, row)| row.contains(pointer_x, pointer_y))
    {
        return Some((Some(DragTarget::Folder { folder: *folder }), true));
    }
    None
}

/// Task packet `drag-back-out-of-pins`: the regular-region slot under the
/// pointer for a pinned row or a Favorite dragged back out of the pinned
/// area — `None` above the region (the New Tab row and everything over
/// it). The slot is the first rendered regular row whose center lies below
/// the pointer, else the end after the last rendered one (`len` for an
/// empty region).
fn regular_slot(pointer_y: f32, geometry: &DragGeometry) -> Option<usize> {
    if pointer_y < geometry.regular_top? {
        return None;
    }
    let mut rows: Vec<_> = geometry
        .rows
        .iter()
        .filter(|(index, _)| *index >= geometry.pinned)
        .collect();
    rows.sort_by_key(|(index, _)| *index);
    for (index, rect) in &rows {
        if pointer_y < rect.y + rect.height / 2.0 {
            return Some(*index);
        }
    }
    Some(rows.last().map_or(geometry.len, |(index, _)| index + 1))
}

/// Task packet `drag-back-out-of-pins`: the target of a Favorite tile's
/// drag. The tile is not a row, so there is no press anchor in the list:
/// every target is hit at the pointer's absolute position — the shared
/// region hits (its own grid excluded), the regular region (unpin), and
/// the loose pinned rows (stay pinned, leave Favorites). The ghost row is
/// offered to a Favorite too, so an empty organized area still takes it.
pub fn resolve_favorite_target(
    pointer_x: f32,
    pointer_y: f32,
    geometry: &DragGeometry,
) -> (Option<DragTarget>, bool) {
    if let Some(hit) = region_hit(pointer_x, pointer_y, geometry, false) {
        return hit;
    }
    if let Some(slot) = regular_slot(pointer_y, geometry) {
        return (Some(DragTarget::Unpin { slot }), true);
    }
    let mut rows: Vec<_> = geometry
        .rows
        .iter()
        .filter(|(index, _)| *index < geometry.pinned)
        .collect();
    rows.sort_by_key(|(index, _)| *index);
    let (Some((_, first)), Some((_, last))) = (rows.first(), rows.last()) else {
        return (None, false);
    };
    // Half a pitch above the first and below the last loose row still
    // addresses the block's edges.
    let reach = geometry.pitch / 2.0;
    if pointer_y < first.y - reach || pointer_y >= last.y + last.height + reach {
        return (None, false);
    }
    let slot = rows
        .iter()
        .find(|(_, rect)| pointer_y < rect.y + rect.height / 2.0)
        .map_or(geometry.pinned, |(index, _)| *index);
    (Some(DragTarget::Tab { slot }), true)
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

#[cfg(test)]
#[path = "drag_target_back_tests.rs"]
mod back_tests;
