//! Saved Pages Library scroll math (DESIGN.md §11.16, B3 packet
//! `saved-pages-library`): the deterministic wheel-delta → row conversion
//! and the clamp the production list wheel listener applies before it
//! stores the tracked scroll offset. Pure and unit-testable without GPUI
//! (the `bbr-ui` surface, the testkit dispatch, and the E2E share exactly
//! this math, so the wheel path never drifts from what tests assert).

use gpui::ScrollDelta;

/// The fixed row height (in device pixels) the wheel listener uses to
/// convert a pixel delta into row units. The Library rows are windowed and
/// never laid out full-list; this is a deterministic presentation constant
/// for the wheel gesture, not a layout measurement.
pub const LIBRARY_ROW_HEIGHT_PX: f32 = 32.0;

/// Converts one wheel delta into row units: line deltas scroll one row per
/// line; pixel deltas divide by the fixed row height (rounded). Positive
/// deltas scroll down (later rows), matching GPUI's native scroll
/// convention (`scroll_offset += delta`).
pub fn wheel_rows(delta: &ScrollDelta) -> i64 {
    match delta {
        ScrollDelta::Lines(lines) => lines.y.round() as i64,
        ScrollDelta::Pixels(pixels) => {
            (f64::from(pixels.y) / f64::from(LIBRARY_ROW_HEIGHT_PX)).round() as i64
        }
    }
}

/// Clamps the tracked production scroll offset after a wheel delta, in row
/// units: `[0, total.saturating_sub(viewport)]` — the last full-window
/// start, exactly the position [`visible_window`](super::visible_window)
/// clamps to. The offset itself stays clamped in the model state, so a
/// wheel-past-end gesture never stores a stale past-end offset.
pub fn scroll_offset_for_wheel(
    current: usize,
    total: usize,
    viewport: usize,
    delta_rows: i64,
) -> usize {
    let max = total.saturating_sub(viewport);
    let next = current as i64 + delta_rows;
    next.clamp(0, max as i64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px};

    #[test]
    fn wheel_rows_converts_lines_and_pixels_deterministically() {
        // Line deltas scroll one row per line.
        assert_eq!(wheel_rows(&ScrollDelta::Lines(point(0.0, 1.0))), 1);
        assert_eq!(wheel_rows(&ScrollDelta::Lines(point(0.0, -3.0))), -3);
        // Pixel deltas divide by the fixed row height.
        assert_eq!(
            wheel_rows(&ScrollDelta::Pixels(point(px(0.0), px(32.0)))),
            1
        );
        assert_eq!(
            wheel_rows(&ScrollDelta::Pixels(point(px(0.0), px(64.0)))),
            2
        );
        assert_eq!(
            wheel_rows(&ScrollDelta::Pixels(point(px(0.0), px(-16.0)))),
            -1
        );
        // A zero delta scrolls nothing.
        assert_eq!(wheel_rows(&ScrollDelta::Pixels(point(px(0.0), px(0.0)))), 0);
    }

    #[test]
    fn scroll_offset_for_wheel_updates_and_clamps_in_row_units() {
        // Plain increments and decrements.
        assert_eq!(scroll_offset_for_wheel(0, 1000, 24, 5), 5);
        assert_eq!(scroll_offset_for_wheel(500, 1000, 24, -2), 498);
        // The clamp is the last full-window start (the same math the
        // bounded window clamps to).
        assert_eq!(scroll_offset_for_wheel(0, 1000, 24, 9999), 976);
        assert_eq!(scroll_offset_for_wheel(976, 1000, 24, 1), 976);
        // Never below zero.
        assert_eq!(scroll_offset_for_wheel(3, 1000, 24, -10), 0);
        // A short collection clamps to zero (nothing to scroll).
        assert_eq!(scroll_offset_for_wheel(0, 3, 24, 1), 0);
    }
}
