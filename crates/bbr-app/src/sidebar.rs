//! Sidebar geometry and resize-divider surface (DESIGN.md §5, §11.4).
//!
//! The sidebar is the physical left rail: chrome is never sampled or
//! overlaid, the content slot re-flows instantly on collapse, and the native
//! web view always keeps strictly positive geometry. This module owns the
//! pure geometry model — min/default/max widths, the narrow-window clamp,
//! collapse memory, keyboard steps, reset, and the drag state — plus the
//! divider component; the input wiring lives in `app::sidebar`.

use gpui::{
    div, prelude::*, px, App, Div, FocusHandle, KeyDownEvent, MouseButton, MouseDownEvent,
    Stateful, Styled, Window,
};

use crate::theme::Theme;

/// DESIGN §5: the minimum adjustable sidebar width (pt).
pub const SIDEBAR_MIN: f32 = 200.0;
/// DESIGN §5: the default width — the §8 normal-density `win:sidebar.N` token.
pub const SIDEBAR_DEFAULT: f32 = 280.0;
/// DESIGN §5: the maximum adjustable sidebar width (pt).
pub const SIDEBAR_MAX: f32 = 400.0;
/// Narrow-window clamp floor: the native content region is never laid out
/// narrower than this, so the WKWebView slot keeps strictly positive geometry
/// even in degenerate window sizes. 64 pt = 2 × `sp:7`, so the floor stays
/// inside the token system.
pub const CONTENT_MIN: f32 = 64.0;
/// DESIGN §5: the drag hairline is 4 pt wide.
pub const RESIZE_HANDLE_WIDTH: f32 = 4.0;
/// Keyboard resize step for the focused divider (DESIGN §9 keyboard
/// equivalence; 16 pt = `sp:4`).
pub const RESIZE_STEP: f32 = 16.0;

/// The stable semantic id of the resize divider (DESIGN §15; shared with the
/// future Accessibility slice, never hardcoded in the render or tests).
pub const SIDEBAR_DIVIDER_ID: &str = "bbr-sidebar-divider";
/// The stable semantic label of the resize divider (future Accessibility).
pub const SIDEBAR_DIVIDER_LABEL: &str = "Sidebar divider";

/// The adjustable sidebar width model (DESIGN §5/§11.4).
///
/// - `requested` is the user-controlled width, always in
///   [`SIDEBAR_MIN`]..=[`SIDEBAR_MAX`]; it survives collapse unchanged.
/// - `effective_width` clamps it to the window so the native content region
///   keeps at least [`CONTENT_MIN`] pt — never zero or negative.
/// - Collapse preserves `last_expanded`; expand restores it.
/// - Pointer drag and the keyboard equivalents mutate the same `requested`
///   value, so both input paths always agree (pointer/keyboard parity).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarGeometry {
    requested: f32,
    last_expanded: f32,
    dragging: bool,
    drag_start_x: f32,
    drag_start_width: f32,
}

impl Default for SidebarGeometry {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarGeometry {
    pub fn new() -> Self {
        Self {
            requested: SIDEBAR_DEFAULT,
            last_expanded: SIDEBAR_DEFAULT,
            dragging: false,
            drag_start_x: 0.0,
            drag_start_width: SIDEBAR_DEFAULT,
        }
    }

    /// The user-controlled width (always in the DESIGN §5 range).
    pub fn requested(&self) -> f32 {
        self.requested
    }

    /// The width an expand restores (DESIGN §11.4: "expand restores the
    /// previous width").
    pub fn last_expanded(&self) -> f32 {
        self.last_expanded
    }

    /// The width the rail actually renders at for the given window width:
    /// the requested width clamped to the adjustable range, then further
    /// clamped so the native content region keeps at least [`CONTENT_MIN`] pt.
    /// The content width `window_width - effective` is therefore always
    /// `>= CONTENT_MIN` while the window is wider than the floor, and never
    /// negative for any window width. When the window is so narrow that even
    /// [`SIDEBAR_MIN`] would starve the content, the rail yields below the
    /// minimum rather than let the native geometry go zero or negative.
    pub fn effective_width(&self, window_width: f32) -> f32 {
        Self::effective_width_for(self.requested, window_width)
    }

    /// The pure clamp: what [`Self::effective_width`] would render for an
    /// arbitrary requested width (used by the benchmark drivers to compute
    /// the expected geometry before a mutation commits).
    pub fn effective_width_for(requested: f32, window_width: f32) -> f32 {
        let requested = requested.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
        let max_for_window = (window_width - CONTENT_MIN).max(0.0);
        requested.min(max_for_window)
    }

    /// Set the width from any input path (pointer drag, keyboard step, or
    /// reset). Clamped to the adjustable range; while the sidebar is visible
    /// the new width is also remembered as the last expanded width.
    pub fn set_width(&mut self, width: f32, visible: bool) {
        self.requested = width.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
        if visible {
            self.last_expanded = self.requested;
        }
    }

    /// Restore the persisted sidebar presentation state (Phase 7 warm
    /// restore): the requested width and the collapse memory, both clamped
    /// to the adjustable range exactly like every other input path. The
    /// collapsed flag is the app's `sidebar_visible` decision, not geometry
    /// state; an active drag is abandoned.
    pub fn restore(&mut self, width: f32, last_expanded: f32) {
        self.requested = width.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
        self.last_expanded = last_expanded.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
        self.dragging = false;
    }

    /// Keyboard decrease (`←` on the focused divider).
    pub fn decrease(&mut self, visible: bool) {
        self.set_width(self.requested - RESIZE_STEP, visible);
    }

    /// Keyboard increase (`→` on the focused divider).
    pub fn increase(&mut self, visible: bool) {
        self.set_width(self.requested + RESIZE_STEP, visible);
    }

    /// Keyboard reset (`0` on the focused divider): back to the §8 default.
    pub fn reset(&mut self, visible: bool) {
        self.set_width(SIDEBAR_DEFAULT, visible);
    }

    /// Collapse: the current width stays remembered as the last expanded
    /// width; an active drag is abandoned.
    pub fn collapse(&mut self) {
        self.dragging = false;
    }

    /// Expand: restore the last expanded width (DESIGN §11.4).
    pub fn expand(&mut self) {
        self.requested = self.last_expanded.clamp(SIDEBAR_MIN, SIDEBAR_MAX);
    }

    /// Pointer drag began on the divider: remember where and at which width,
    /// so the drag is a relative delta that clamps to the same range as the
    /// keyboard path.
    pub fn begin_drag(&mut self, start_x: f32, window_width: f32) {
        self.dragging = true;
        self.drag_start_x = start_x;
        self.drag_start_width = self.effective_width(window_width);
    }

    /// Pointer drag moved to `x`: the width follows the pointer delta,
    /// clamped exactly like every other mutation.
    pub fn drag_to(&mut self, x: f32, visible: bool) {
        if !self.dragging {
            return;
        }
        let delta = x - self.drag_start_x;
        self.set_width(self.drag_start_width + delta, visible);
    }

    /// Pointer drag ended (mouse up or the button left the window).
    pub fn end_drag(&mut self) {
        self.dragging = false;
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }
}

/// DESIGN §5/§11.4 resize hairline: a 4 pt full-height strip on the sidebar's
/// right edge with the `ew-resize` cursor, focusable for the keyboard
/// equivalents (`←`/`→` resize, `0` reset). The caller wires the production
/// pointer begin-drag listener and the focused keyboard handler; the id and
/// label are the stable §15 surface identity shared with the snapshot and
/// the future Accessibility slice.
pub fn sidebar_divider(
    theme: &Theme,
    divider_handle: FocusHandle,
    focused: bool,
    on_mouse_down: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_key_down: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id(SIDEBAR_DIVIDER_ID)
        .track_focus(&divider_handle)
        .w(px(RESIZE_HANDLE_WIDTH))
        .h_full()
        .flex_shrink_0()
        .bg(if focused {
            theme.palette.accent
        } else {
            theme.palette.hairline
        })
        .cursor_ew_resize()
        .on_mouse_down(MouseButton::Left, on_mouse_down)
        .on_key_down(on_key_down)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wide() -> f32 {
        1080.0
    }

    #[test]
    fn defaults_follow_the_design_contract() {
        let g = SidebarGeometry::new();
        assert_eq!(g.requested(), SIDEBAR_DEFAULT);
        assert_eq!(g.last_expanded(), SIDEBAR_DEFAULT);
        assert_eq!(g.effective_width(wide()), SIDEBAR_DEFAULT);
        assert!(!g.is_dragging());
    }

    #[test]
    fn set_width_clamps_to_the_adjustable_range() {
        let mut g = SidebarGeometry::new();
        g.set_width(10.0, true);
        assert_eq!(g.requested(), SIDEBAR_MIN);
        g.set_width(9999.0, true);
        assert_eq!(g.requested(), SIDEBAR_MAX);
        g.set_width(300.0, true);
        assert_eq!(g.requested(), 300.0);
        assert_eq!(
            g.last_expanded(),
            300.0,
            "visible resize remembers the width"
        );
    }

    #[test]
    fn keyboard_steps_and_reset_use_the_same_range() {
        let mut g = SidebarGeometry::new();
        g.decrease(true);
        assert_eq!(g.requested(), SIDEBAR_DEFAULT - RESIZE_STEP);
        g.increase(true);
        assert_eq!(g.requested(), SIDEBAR_DEFAULT);
        // Stepping past the min/max clamps like the pointer path.
        for _ in 0..100 {
            g.decrease(true);
        }
        assert_eq!(g.requested(), SIDEBAR_MIN);
        for _ in 0..100 {
            g.increase(true);
        }
        assert_eq!(g.requested(), SIDEBAR_MAX);
        g.reset(true);
        assert_eq!(g.requested(), SIDEBAR_DEFAULT);
        assert_eq!(g.last_expanded(), SIDEBAR_DEFAULT);
    }

    #[test]
    fn collapse_preserves_and_expand_restores_the_last_expanded_width() {
        let mut g = SidebarGeometry::new();
        g.set_width(300.0, true);
        g.collapse();
        assert_eq!(g.last_expanded(), 300.0, "collapse keeps the width");
        // A width mutation while hidden must not clobber the memory.
        g.set_width(200.0, false);
        assert_eq!(g.last_expanded(), 300.0);
        g.expand();
        assert_eq!(
            g.requested(),
            300.0,
            "expand restores the last expanded width"
        );
        assert_eq!(g.effective_width(wide()), 300.0);
    }

    #[test]
    fn narrow_window_clamp_keeps_content_positive_and_never_negative() {
        let mut g = SidebarGeometry::new();
        g.set_width(SIDEBAR_MAX, true);
        // A normal compact window keeps the full adjustable range.
        assert_eq!(g.effective_width(700.0), SIDEBAR_MAX);
        // Once the window cannot host the requested width plus the content
        // floor, the rail yields so the content keeps exactly CONTENT_MIN.
        let narrow = SIDEBAR_MIN + CONTENT_MIN; // 264
        assert_eq!(g.effective_width(narrow), SIDEBAR_MIN);
        assert_eq!(narrow - g.effective_width(narrow), CONTENT_MIN);
        let tight = 220.0;
        assert!(
            g.effective_width(tight) < SIDEBAR_MIN,
            "rail yields below min"
        );
        assert_eq!(tight - g.effective_width(tight), CONTENT_MIN);
        // Degenerate windows still never produce negative content.
        for w in [120.0, 64.0, 10.0, 0.0] {
            let effective = g.effective_width(w);
            assert!(effective >= 0.0, "effective width never negative");
            assert!(w - effective >= 0.0, "content width never negative");
            if w > CONTENT_MIN {
                assert_eq!(w - effective, CONTENT_MIN);
            } else {
                assert_eq!(w - effective, w, "content takes everything left");
            }
        }
    }

    #[test]
    fn drag_is_relative_delta_clamped_like_every_other_path() {
        let mut g = SidebarGeometry::new();
        g.begin_drag(
            g.effective_width(wide()) - RESIZE_HANDLE_WIDTH / 2.0,
            wide(),
        );
        assert!(g.is_dragging());
        // Dragging +40 pt to the right grows the width by 40.
        g.drag_to(g.drag_start_x + 40.0, true);
        assert_eq!(g.requested(), SIDEBAR_DEFAULT + 40.0);
        // Dragging far past the max clamps to the adjustable range.
        g.drag_to(g.drag_start_x + 4000.0, true);
        assert_eq!(g.requested(), SIDEBAR_MAX);
        g.drag_to(g.drag_start_x - 4000.0, true);
        assert_eq!(g.requested(), SIDEBAR_MIN);
        g.end_drag();
        assert!(!g.is_dragging());
        // Moves after the drag ended are ignored.
        let before = g.requested();
        g.drag_to(9999.0, true);
        assert_eq!(g.requested(), before);
    }

    #[test]
    fn drag_math_matches_pointer_parity() {
        // The same target width reached by a pointer drag and by keyboard
        // steps lands on the same requested value (parity by construction).
        let delta = 2.0 * RESIZE_STEP;
        let mut pointer = SidebarGeometry::new();
        pointer.begin_drag(258.0, wide());
        pointer.drag_to(258.0 + delta, true);
        pointer.end_drag();

        let mut keyboard = SidebarGeometry::new();
        for _ in 0..(delta / RESIZE_STEP) as usize {
            keyboard.increase(true);
        }
        assert_eq!(pointer.requested(), keyboard.requested());
    }
}
