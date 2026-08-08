//! Production drag-to-chip geometry (DESIGN.md §11.3/§10.9).
//!
//! The §11.3 footer strip's rendered chips are the drag targets for
//! moving a tab into a space. This module derives each chip's visual span
//! from the **production strip layout** — the tracked `space_strip_scroll`
//! [`gpui::ScrollHandle`] the render owns, whose laid-out per-child bounds
//! plus the signed scroll offset are exactly the geometry the pointer
//! enters — so a drag hits the exact persistent chip under the pointer
//! without inventing render-index identity or a parallel layout model. The
//! canonical chip order is the same [`BbrApp::space_chip_visuals`] vector
//! the render, the snapshot, the semantic tree, and the scroll seam all
//! consume.
//!
//! The pure hit resolution lives in `super::drag_target`; this module only
//! maps the laid-out strip onto typed [`ChipSpan`]s and back.

use bbr_browser::SpaceId;

use super::drag_target::ChipSpan;
use super::BbrApp;

impl BbrApp {
    /// Each rendered strip chip's visible span in window coordinates, in
    /// canonical display order (the same vector `space_chip_visuals`
    /// produces). A chip's raw layout bounds come from the production
    /// scroll handle (`bounds_for_item`, the laid-out child rect the render
    /// paints); the signed scroll offset places it in the viewport, and the
    /// viewport clip removes the scrolled-out parts — a chip the pointer
    /// cannot see cannot be hit. Chips the strip has not laid out yet (or
    /// that are fully scrolled out) contribute no span. Read-only: never
    /// mutates the handle, the drag state, or the model.
    pub(crate) fn chip_spans(&self) -> Vec<ChipSpan> {
        let scroll = &self.space_strip_scroll;
        let viewport = scroll.bounds();
        let offset = scroll.offset();
        let px = f32::from;
        let vp = [
            px(viewport.left()),
            px(viewport.top()),
            px(viewport.right()),
            px(viewport.bottom()),
        ];
        let ox = px(offset.x);
        let oy = px(offset.y);
        self.space_chip_visuals()
            .iter()
            .enumerate()
            .filter_map(|(index, visual)| {
                let raw = scroll.bounds_for_item(index)?;
                let left = (px(raw.left()) + ox).max(vp[0]);
                let right = (px(raw.right()) + ox).min(vp[2]);
                let top = (px(raw.top()) + oy).max(vp[1]);
                let bottom = (px(raw.bottom()) + oy).min(vp[3]);
                if right <= left || bottom <= top {
                    return None;
                }
                Some(ChipSpan {
                    space: visual.space,
                    x_left: left,
                    x_right: right,
                    y_top: top,
                    y_bottom: bottom,
                })
            })
            .collect()
    }

    /// The rendered span of the exact persistent chip `space`, when the
    /// strip currently lays it out and it is not scrolled fully out of
    /// view. The deterministic pointer entry the E2E seam and the benchmark
    /// driver use to enter that chip through the production path.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub(crate) fn chip_span_for(&self, space: SpaceId) -> Option<ChipSpan> {
        self.chip_spans()
            .into_iter()
            .find(|span| span.space == space)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_span_contains_and_centers_are_consistent() {
        let span = ChipSpan {
            space: SpaceId(7),
            x_left: 60.0,
            x_right: 150.0,
            y_top: 640.0,
            y_bottom: 676.0,
        };
        assert!(span.contains(105.0, 658.0), "chip center is inside");
        assert!(span.contains(60.0, 640.0), "the top-left edge is inside");
        assert!(!span.contains(59.0, 658.0), "left of the chip is outside");
        assert!(!span.contains(151.0, 658.0), "right of the chip is outside");
        assert!(!span.contains(105.0, 639.0), "above the chip is outside");
        assert_eq!(span.center_x(), 105.0);
        assert_eq!(span.center_y(), 658.0);
    }

    #[test]
    fn chip_spans_keep_the_persistent_space_identity() {
        // The span's identity is the persistent SpaceId, never a render
        // position: two spans in display order keep their own ids.
        let spans = [
            ChipSpan {
                space: SpaceId(4),
                x_left: 20.0,
                x_right: 80.0,
                y_top: 0.0,
                y_bottom: 36.0,
            },
            ChipSpan {
                space: SpaceId(9),
                x_left: 90.0,
                x_right: 150.0,
                y_top: 0.0,
                y_bottom: 36.0,
            },
        ];
        assert_eq!(spans[0].space, SpaceId(4));
        assert_eq!(spans[1].space, SpaceId(9));
        assert!(spans[1].contains(120.0, 18.0));
        assert!(!spans[0].contains(120.0, 18.0));
    }
}
