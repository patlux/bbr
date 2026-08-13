//! §10.9 drag insertion tick (DESIGN.md §10.9/§11.6): the 2 pt rule at the
//! target insertion slot, rendered by the tab surface while a tab drag is
//! in flight.
//!
//! The tick contributes **zero outer height** (the containing block is
//! 0-tall), so a tick never shifts row steps and the dense scroll extent
//! stays exactly `item_count × pitch` even while a drag is in flight
//! (§11.6 exact scroll math). The rule is an absolutely-positioned child
//! of the 0-tall block, so it never joins the flow and no negative margin
//! can paint outside the owning block: at the top of a block (`first` —
//! slot 0 of the pinned block or of the scroll window at the top) the rule
//! anchors just inside the block's top edge; any other slot anchors in the
//! 4 pt gap above the target row (the lower half of the gap).

use gpui::{div, prelude::*, px, Div, Pixels, Stateful, Styled};

use crate::theme::Theme;

/// The §15 stable id of the drag insertion tick (`bbr-drag-tick-<slot>`).
pub const DRAG_TICK_ID: &str = "bbr-drag-tick";

/// The §10.9 tick rule's `inset.bottom` anchor. `false` (a mid-list
/// slot) anchors the rule in the lower half of the 4 pt gap above the
/// target row (rule spans `[row_top − 2, row_top]`); `true` (slot 0 at
/// the top of its block) anchors the rule just inside the block's top
/// edge (rule spans `[block_top, block_top + 2]`). Both spans lie inside
/// the owning block's bounds — no negative margin can paint outside it —
/// and the 0-tall containing block keeps the tick's outer height at zero
/// (dense scroll extent stays exactly `item_count × pitch` during drags).
pub fn drag_tick_bottom(first: bool) -> Pixels {
    if first {
        px(-2.0)
    } else {
        px(0.0)
    }
}

/// §10.9 drag insertion tick: the 2 pt rule at the target slot. The tick
/// contributes **zero outer height** (the containing block is 0-tall), so
/// a tick never shifts row steps and the dense scroll extent stays exactly
/// `item_count × pitch` even while a drag is in flight (§11.6 exact scroll
/// math). The rule is an absolutely-positioned child of the 0-tall block,
/// so it never joins the flow and no negative margin can paint outside the
/// owning block: at the top of a block (`first` — slot 0 of the pinned
/// block or of the scroll window at the top) the rule anchors just inside
/// the block's top edge; any other slot anchors in the 4 pt gap above the
/// target row (the lower half of the gap).
///
/// `invalid` renders the muted hairline variant so an illegal boundary
/// drop is clearly signaled without asserting a committed target. The
/// stable §15 id lets E2E assert the tick's presence and state.
pub fn drag_tick(theme: &Theme, index: usize, invalid: bool, first: bool) -> Stateful<Div> {
    div()
        .id((DRAG_TICK_ID, index))
        .debug_selector(move || format!("{DRAG_TICK_ID}-{index}"))
        .h(px(0.0))
        .w_full()
        .relative()
        .child(
            div()
                .absolute()
                .bottom(drag_tick_bottom(first))
                .h(px(2.0))
                .w_full()
                .bg(if invalid {
                    theme.palette.ink_faint
                } else {
                    theme.palette.accent
                }),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_id_is_the_stable_section_15_id() {
        // DESIGN.md §10.9 assert id — unchangeable contract.
        assert_eq!(DRAG_TICK_ID, "bbr-drag-tick");
    }

    #[test]
    fn drag_tick_anchor_keeps_the_rule_inside_the_owning_block() {
        // §10.9 containment: the tick's rule never paints outside its
        // owning block and contributes zero outer height, so a drag never
        // shifts row steps or the dense scroll extent. A mid-list slot
        // anchors the rule in the lower half of the 4 pt gap above the
        // target row (bottom 0 → rule spans `[row_top − 2, row_top]`);
        // slot 0 at the top of a block anchors just inside the block's top
        // edge (bottom −2 → rule spans `[block_top, block_top + 2]`). Both
        // spans lie inside the owning block's bounds.
        assert_eq!(drag_tick_bottom(false), px(0.0), "gap anchor");
        assert_eq!(drag_tick_bottom(true), px(-2.0), "top-of-block anchor");
    }
}
