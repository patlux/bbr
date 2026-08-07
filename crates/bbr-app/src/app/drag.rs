//! Tab drag state machine (DESIGN.md §10.9, §11.2, interaction contract).
//!
//! The pointer drag for tab organization: pressing a row begins a drag,
//! mouse-move resolves a typed target (an insertion slot inside the source
//! region, or a visible space row), and mouse-up commits through the
//! controller — region-bounded reorder for tab slots, move-between-spaces
//! for space rows. Escape cancels. The source row is the dragged tab (the
//! "active or selected tab" of the interaction contract), and the active tab
//! itself is never mutated by grabbing a row.
//!
//! The pure target-resolution math lives in `super::drag_target`; this module
//! owns the state machine and the production pointer listeners that drive it.

use super::drag_target::{
    resolve_target, space_row_h, space_row_spans, splice_index, tab_row_h, DragGeometry,
    DragTarget, SpaceRowSpan,
};
use super::BbrApp;
use bbr_browser::SpaceId;
use gpui::{Context, Window};

/// The in-flight drag state of the root view. Owned by `BbrApp`; the
/// production listeners (`begin_tab_drag`/`on_tab_drag_move`/
/// `end_tab_drag`/`cancel_tab_drag`) drive it, and the render derives the
/// §10.9 dragging/tick/target visuals plus the E2E snapshot fields from it.
#[derive(Debug, Clone)]
pub struct TabDrag {
    active: bool,
    /// The pressed row's active-space index — the dragged tab.
    source: usize,
    /// The pointer y (window coordinates) where the drag began.
    start_y: f32,
    /// The press's y within the source row (0 at the top edge, `row_h` at
    /// the bottom). The tab-list geometry is anchored at the press, so this
    /// offset makes the space-surface hit exact; when the layout cannot
    /// expose it (GPUI 0.2.2 offers no element geometry) the caller anchors
    /// at the row center (`row_h / 2`).
    press_in_row: f32,
    /// The current typed target, if the pointer is over a droppable zone.
    target: Option<DragTarget>,
    /// Whether the current target is a legal drop. An illegal boundary
    /// crossing keeps the target (for the tick position) but reports
    /// invalid, so the render can clearly signal the rejection (§10.9).
    valid: bool,
}

impl Default for TabDrag {
    fn default() -> Self {
        Self {
            active: false,
            source: 0,
            start_y: 0.0,
            press_in_row: 0.0,
            target: None,
            valid: false,
        }
    }
}

impl TabDrag {
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// The dragged row's active-space index.
    pub fn source(&self) -> Option<usize> {
        self.active.then_some(self.source)
    }

    /// The window y the drag began at (the E2E seam anchors its moves on it).
    #[cfg(feature = "testkit")]
    pub fn start_y(&self) -> Option<f32> {
        self.active.then_some(self.start_y)
    }

    /// Replace the press anchor after `begin` (E2E seam). No resolution has
    /// run at begin time, so overriding the anchor is consistent; the seam
    /// declares the offset its simulated press sits at within the source row.
    #[cfg(feature = "testkit")]
    pub fn set_press_in_row(&mut self, press_in_row: f32) {
        self.press_in_row = press_in_row;
    }

    /// The current typed target (`None` over no droppable zone).
    pub fn target(&self) -> Option<DragTarget> {
        self.target
    }

    /// The tab insertion slot target, when the drag currently targets one.
    pub fn tab_slot(&self) -> Option<usize> {
        match self.target {
            Some(DragTarget::Tab { slot }) => Some(slot),
            _ => None,
        }
    }

    /// The space target, when the drag currently targets a space row.
    pub fn space_target(&self) -> Option<SpaceId> {
        match self.target {
            Some(DragTarget::Space { space }) => Some(space),
            _ => None,
        }
    }

    /// Whether the current target is a legal drop (region-bound reorder or a
    /// visible space row).
    pub fn valid(&self) -> bool {
        self.valid
    }

    /// Begin a drag on row `source` at the pointer y. `press_in_row` is the
    /// press's y within the source row (0 at the top edge, `row_h` at the
    /// bottom), the anchor the tab/space geometry is resolved from; callers
    /// pass the row center when the layout does not expose the true offset.
    pub fn begin(&mut self, source: usize, position_y: f32, press_in_row: f32) {
        self.active = true;
        self.source = source;
        self.start_y = position_y;
        self.press_in_row = press_in_row;
        self.target = Some(DragTarget::Tab { slot: source });
        self.valid = true;
    }

    /// Resolve the drag target for a new pointer y and update the state.
    ///
    /// `pinned` is the pinned/regular boundary (§11.2), `len` the tab count,
    /// `row_h` the §8 row height, `favorites_h` the rendered §11.10
    /// favorites region's height (0 when the grid is empty), and `spaces`
    /// the rendered space rows (top-down, with their surface-relative y
    /// offsets) plus the space surface's total height. Pure geometry: the
    /// pointer position itself is never stored.
    #[allow(clippy::too_many_arguments)] // mirrors `DragGeometry` field-for-field
    pub fn move_to(
        &mut self,
        position_y: f32,
        pinned: usize,
        len: usize,
        row_h: f32,
        favorites_h: f32,
        space_h: f32,
        spaces: &[SpaceRowSpan],
    ) {
        let dy = position_y - self.start_y;
        let geometry = DragGeometry {
            pinned,
            len,
            row_h,
            favorites_h,
            space_h,
            spaces,
        };
        let (target, valid) = resolve_target(self.source, dy, self.press_in_row, &geometry);
        self.target = Some(target);
        self.valid = valid;
    }

    /// Clear the drag state (commit already happened or was cancelled).
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl BbrApp {
    /// The window y the active drag began at (E2E seam anchor).
    #[cfg(feature = "testkit")]
    pub fn tab_drag_start_y(&self) -> Option<f32> {
        self.tab_drag.start_y()
    }

    /// Override the recorded press offset within the source row (E2E seam):
    /// the simulated press's y within the row, 0 at the top edge. The
    /// production listener anchors at the row center because GPUI 0.2.2
    /// exposes no element geometry; the seam declares the offset its
    /// scenario simulates so the space-surface hit is exact for it.
    #[cfg(feature = "testkit")]
    pub fn set_tab_drag_press_in_row(&mut self, press_in_row: f32) {
        self.tab_drag.set_press_in_row(press_in_row);
    }

    /// Production pointer entry: mouse-down on a tab row starts a drag of
    /// that row (the dragged tab is the row under the pointer, never the
    /// active tab unless they coincide). Chrome focus moves to the tab list
    /// (the drag is a chrome interaction); the target starts at the source
    /// slot and follows the pointer on move.
    pub fn begin_tab_drag(
        &mut self,
        event: &gpui::MouseDownEvent,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_tab_list(window, cx);
        let row_h = tab_row_h(&self.theme());
        // GPUI 0.2.2 exposes no element geometry to listeners, so the press's
        // true y within the source row is unknowable here; anchor at the row
        // center (bounds the space-surface hit error at half a row). The E2E
        // seam and the resolution unit tests pass the true offset where the
        // geometry is known.
        self.tab_drag
            .begin(index, f32::from(event.position.y), row_h / 2.0);
        cx.notify();
    }

    /// Production drag-move: while a drag is active, resolve the typed
    /// target from the pointer's vertical displacement. Runs from the root's
    /// mouse-move listener so the drag keeps working anywhere over the
    /// window; a move without the left button held (released outside the
    /// window) also ends the drag without committing.
    pub fn on_tab_drag_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_drag.is_active() {
            return;
        }
        if event.pressed_button != Some(gpui::MouseButton::Left) {
            self.tab_drag.clear();
            cx.notify();
            return;
        }
        self.resolve_tab_drag(f32::from(event.position.y));
        cx.notify();
    }

    /// Recompute the drag target for a pointer y. Shared by the production
    /// mouse-move listener and the E2E seam, so both paths resolve targets
    /// through the exact same geometry. The favorites band (when the grid is
    /// rendered) and the space surface (only while the §11.4 spaces region
    /// is revealed) sit above the tab list; a hidden region contributes no
    /// droppable band.
    pub(crate) fn resolve_tab_drag(&mut self, position_y: f32) {
        let browser = self.controller.snapshot();
        let theme = self.theme();
        if !self.tab_drag.is_active() {
            return;
        }
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let organization = self.organization.organization_snapshot(&self.controller);
        // The favorites band is rendered whenever tiles exist, in the same
        // column position the render uses (above the tab list).
        let favorites_h = crate::components::favorites::favorites_region_height(
            &theme,
            organization.favorites.len(),
        );
        let (spaces, space_h) = if self.u2.spaces_visible {
            self.space_row_spans()
        } else {
            (Vec::new(), 0.0)
        };
        self.tab_drag.move_to(
            position_y,
            pinned,
            browser.tabs.len(),
            tab_row_h(&theme),
            favorites_h,
            space_h,
            &spaces,
        );
    }

    /// Production drag-end: mouse-up anywhere in the window (or, via the
    /// out-listener, released outside it) commits a legal target through the
    /// controller and clears the drag. An illegal boundary crossing or an
    /// empty target commits nothing (the drop is rejected whole).
    pub fn end_tab_drag(
        &mut self,
        _event: &gpui::MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_drag.is_active() {
            return;
        }
        let target = self.tab_drag.target();
        let valid = self.tab_drag.valid();
        let source = self.tab_drag.source().expect("active drag has a source");
        self.tab_drag.clear();
        if valid {
            if let Some(target) = target {
                self.commit_tab_drop(source, target, window, cx);
            }
        }
        cx.notify();
    }

    /// Escape during a drag cancels it: the target is cleared, nothing is
    /// committed, and chrome focus stays in the tab list (the drag is a
    /// chrome interaction).
    pub fn cancel_tab_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.tab_drag.is_active() {
            self.tab_drag.clear();
            self.focus_tab_list(window, cx);
            cx.notify();
        }
    }

    /// Commit a legal drop: a tab slot reorders within the source region
    /// (splice semantics), a space row moves the tab between spaces through
    /// the controller's profile-boundary recreation policy, and the
    /// favorites grid toggles the dragged tab in/out of the window-global
    /// favorites (add/remove — the tab keeps its space, order, selection,
    /// and live session). A drop back on the source slot (or into the tab's
    /// own space) is a no-op: the drag never emits a spurious reorder/move
    /// event.
    fn commit_tab_drop(
        &mut self,
        source: usize,
        target: DragTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match target {
            DragTarget::Tab { slot } => {
                if slot == source {
                    return;
                }
                let to = splice_index(source, slot);
                if self.reorder_tab_at(source, to, cx).is_some() {
                    self.focus_tab_list(window, cx);
                }
            }
            DragTarget::Space { space } => {
                if self.controller.snapshot().active_space == space {
                    return;
                }
                if self
                    .move_tab_to_space_at(source, space, window, cx)
                    .is_some()
                {
                    self.focus_tab_list(window, cx);
                }
            }
            DragTarget::Favorites => {
                self.favorite_toggle_tab(source, window, cx);
            }
        }
    }

    /// The space surface's rendered row spans (top-down, §11.3 grouping with
    /// hairlines, New Space foot), for drag-target resolution.
    fn space_row_spans(&self) -> (Vec<SpaceRowSpan>, f32) {
        let browser = self.controller.snapshot();
        let rows: Vec<(bbr_browser::SpaceId, usize)> = crate::shell::space_groups(
            &browser,
            crate::shell::space_title_budget_chars(&self.theme()),
        )
        .into_iter()
        .flat_map(|group| group.rows.into_iter())
        .map(|row| {
            let space = browser
                .spaces
                .iter()
                .find(|s| s.index == row.index)
                .map(|s| s.id)
                .unwrap_or_else(|| panic!("space row derives from the snapshot"));
            (space, row.profile_index)
        })
        .collect();
        space_row_spans(&rows, space_row_h(&self.theme()))
    }
}

#[cfg(test)]
mod tests {
    use super::TabDrag;
    use crate::app::drag_target::{space_row_spans, DragTarget};
    use bbr_browser::SpaceId;

    #[test]
    fn move_to_accumulates_dy_from_the_press_position() {
        let rows = [(SpaceId(1), 0)];
        let (spaces, space_h) = space_row_spans(&rows, 34.0);
        let mut drag = TabDrag::default();
        // Row-center press anchor (offset 16 in a 32 pt row).
        drag.begin(1, 200.0, 16.0);
        // 32 pt down from the press → slot 2 (valid regular slot).
        drag.move_to(232.0, 0, 3, 32.0, 0.0, space_h, &spaces);
        assert_eq!(drag.tab_slot(), Some(2));
        assert!(drag.valid());
        // Back to the press position → the source slot.
        drag.move_to(200.0, 0, 3, 32.0, 0.0, space_h, &spaces);
        assert_eq!(drag.tab_slot(), Some(1));
        assert!(drag.valid());
        // A pointer over the rendered favorites band targets the grid.
        drag.move_to(200.0 - 48.0 - 30.0, 0, 3, 32.0, 60.0, space_h, &spaces);
        assert_eq!(drag.target(), Some(DragTarget::Favorites));
        assert!(drag.valid());
    }
}
