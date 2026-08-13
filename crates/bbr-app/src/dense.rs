//! Dense-list geometry and scroll state (DESIGN.md §11.6).
//!
//! The regular tab region is a real GPUI scroll container whose children are
//! a deterministic *window* of rows (bounded by [`DENSE_WINDOW_ROWS`]) plus
//! two spacers that keep the container's content height equal to the full
//! collection height, so the native scrollbar and the native wheel listener
//! behave exactly as if every row were laid out. The scroll *anchor* (first
//! regular row at the viewport top) is derived from the production scroll
//! offset — `offset.y` is negative as the user scrolls down — so the rendered
//! window always follows the real scroll position and E2E can assert a
//! deterministic `scroll_anchor` without any app-owned shadow state.
//!
//! Scroll position changes come only from production listeners: GPUI's native
//! scroll-wheel handling on the container, the keyboard traversal handlers
//! (which move selection and set the offset through the tracked
//! [`ScrollHandle`]), and the instant selection-follow after mutations. No
//! timer, no per-frame invalidation, no pixel polling.

use gpui::{px, Pixels};

/// The bounded render window: DESIGN.md §11.6 renders the visible range plus
/// overscan; at the shell's maximum viewport (~40 rows) a 64-row window is
/// the visible range plus roughly a 24-row overscan, and it keeps the row
/// layout bounded regardless of collection size, so a 500-tab list never
/// lays out 500 rows per frame.
pub const DENSE_WINDOW_ROWS: usize = 64;

/// What the last render actually built (recorded during `Render::render`):
/// the global index of the first rendered regular row, the rendered row
/// count, the full regular-region count, and the deterministic scroll anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DenseRenderState {
    /// Global (space) index of the first rendered regular row.
    pub first_rendered: usize,
    /// How many regular rows the render built (bounded by
    /// [`DENSE_WINDOW_ROWS`]).
    pub rendered_rows: usize,
    /// Regular-region tab count (the scrollable collection size).
    pub item_count: usize,
    /// Regular-region index of the first row at the viewport top, derived
    /// from the production scroll offset.
    pub scroll_anchor: usize,
}

/// The first regular row at the viewport top, derived from the production
/// scroll offset (negative as the user scrolls down). Clamped so a stale or
/// extreme offset never renders outside the collection. `step` is the §8
/// row pitch (`density:pitch` = surface + gap) — the exact unit every
/// dense/scroll math steps in (§11.6).
pub fn anchor_from_offset(offset_y: Pixels, step: Pixels, total_rows: usize) -> usize {
    let step = f32::from(step).max(1.0);
    let scrolled = (-f32::from(offset_y)).max(0.0);
    let anchor = (scrolled / step).floor() as usize;
    clamp_anchor(anchor, total_rows)
}

/// Clamp the anchor to the last position from which a full window is visible
/// (an empty collection anchors at 0).
pub fn clamp_anchor(anchor: usize, total_rows: usize) -> usize {
    total_rows.saturating_sub(DENSE_WINDOW_ROWS).min(anchor)
}

/// The rendered window for the current anchor: `[anchor, anchor + len)` where
/// `len = min(DENSE_WINDOW_ROWS, total - anchor)`. Returns an empty range for
/// an empty collection.
pub fn window_for(anchor: usize, total_rows: usize) -> std::ops::Range<usize> {
    if total_rows == 0 {
        return 0..0;
    }
    let anchor = clamp_anchor(anchor, total_rows);
    let end = (anchor + DENSE_WINDOW_ROWS).min(total_rows);
    anchor..end
}

/// The scroll offset (negative) that puts `anchor` at the viewport top.
/// `step` is the §8 row pitch (`density:pitch` = surface + gap) — the
/// exact unit every dense/scroll math steps in (§11.6).
pub fn offset_for_anchor(anchor: usize, step: Pixels) -> Pixels {
    px(-(anchor as f32 * f32::from(step)))
}

/// The absolute scroll offset for `scroll_top_px` (clamped to the collection
/// extent by the container's own layout clamp). `step` is the §8 row pitch
/// (`density:pitch` = surface + gap) — the exact unit every dense/scroll
/// math steps in (§11.6).
pub fn scroll_top_for_anchor(anchor: usize, step: Pixels) -> f32 {
    anchor as f32 * f32::from(step)
}

/// Whether `row` (a regular-region index) lies inside the rendered window.
pub fn window_contains(window: &std::ops::Range<usize>, row: usize) -> bool {
    window.contains(&row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_derives_from_the_negative_scroll_offset() {
        let step = px(40.0);
        assert_eq!(anchor_from_offset(px(0.0), step, 500), 0);
        assert_eq!(anchor_from_offset(px(-40.0), step, 500), 1);
        assert_eq!(anchor_from_offset(px(-120.0), step, 500), 3);
        // A sub-row offset floors, never rounds up.
        assert_eq!(anchor_from_offset(px(-121.0), step, 500), 3);
    }

    #[test]
    fn anchor_clamps_to_the_last_full_window() {
        assert_eq!(clamp_anchor(0, 500), 0);
        assert_eq!(clamp_anchor(499, 500), 436);
        assert_eq!(clamp_anchor(1000, 500), 436);
        // Small collections never scroll past zero.
        assert_eq!(clamp_anchor(40, 10), 0);
        assert_eq!(clamp_anchor(0, 0), 0);
    }

    #[test]
    fn window_is_bounded_and_covers_the_tail() {
        assert_eq!(window_for(0, 500), 0..64);
        assert_eq!(window_for(436, 500), 436..500);
        assert_eq!(window_for(499, 500), 436..500);
        assert_eq!(window_for(0, 10), 0..10);
        assert_eq!(window_for(0, 0), 0..0);
    }

    #[test]
    fn offsets_are_exact_and_negative() {
        assert_eq!(offset_for_anchor(3, px(40.0)), px(-120.0));
        assert_eq!(scroll_top_for_anchor(3, px(40.0)), 120.0);
    }

    #[test]
    fn window_contains_is_precise() {
        let window = window_for(10, 500);
        assert!(window_contains(&window, 10));
        assert!(window_contains(&window, 73));
        assert!(!window_contains(&window, 74));
        assert!(!window_contains(&window, 9));
    }
}
