//! Tab drag production listeners (DESIGN.md §10.9, §11.2, §11.3, §11.10,
//! interaction contract).
//!
//! The pointer drag for tab organization: pressing a row arms a drag,
//! mouse-move activates it past the 4 pt distance (committing the drag
//! offers' growth once) and resolves a typed target — an insertion slot
//! inside the source region, the §11.10 favorites grid, the §11.2 "Pin tab
//! here" ghost row, or the §11.3 space chip under the pointer — and
//! mouse-up commits through the controller: region-bounded reorder for tab
//! slots, favorite toggle for the grid, pin for the ghost row, and the
//! single tab-move authority for chips. Escape cancels. The source row is
//! the dragged tab — the concrete pressed row — and the active tab itself
//! is never mutated by grabbing a row.
//!
//! The state machine lives in `super::drag_state`, the pure target math in
//! `super::drag_target`, and the production chip geometry in
//! `super::drag_chip`.

pub use super::drag_state::TabDrag;
use super::drag_target::{splice_index, tab_pitch, DragGeometry, DragTarget, DropRect};
use super::BbrApp;
use bbr_web::WebRect;
use gpui::{Context, Window};
use std::cell::RefCell;
use std::rc::Rc;

/// The rendered window rects of the §11.2 ghost row and the §10.9 drag
/// proxy, recorded by their anchors every frame they render (`None` while
/// they do not). The resolver hit-tests the ghost row through its recorded
/// rect — never a parallel layout model — and the snapshot reports both.
#[derive(Clone, Default)]
pub(crate) struct DragOfferRects {
    pub pin_offer: Rc<RefCell<Option<WebRect>>>,
    pub proxy: Rc<RefCell<Option<WebRect>>>,
    /// Blueprint D02: every rendered §11.11 folder row's rect, keyed by the
    /// folder's domain id (refilled each frame the folders render).
    pub folders: Rc<RefCell<Vec<(bbr_domain::FolderId, WebRect)>>>,
}

impl BbrApp {
    /// The window x the active drag began at (E2E seam anchor).
    #[cfg(feature = "testkit")]
    pub fn tab_drag_start_x(&self) -> Option<f32> {
        self.tab_drag.start_x()
    }

    /// The window y the active drag began at (E2E seam anchor).
    #[cfg(feature = "testkit")]
    pub fn tab_drag_start_y(&self) -> Option<f32> {
        self.tab_drag.start_y()
    }

    /// Override the recorded press offset within the source row (E2E seam):
    /// the simulated press's y within the row, 0 at the top edge. The
    /// production listener anchors at the row center because GPUI 0.2.2
    /// exposes no element geometry; the seam declares the offset its
    /// scenario simulates so the favorites-band hit is exact for it.
    #[cfg(feature = "testkit")]
    pub fn set_tab_drag_press_in_row(&mut self, press_in_row: f32) {
        self.tab_drag.set_press_in_row(press_in_row);
    }

    /// Production pointer entry: mouse-down on a tab row **arms** a drag of
    /// that row (the dragged tab is the row under the pointer, never the
    /// active tab unless they coincide). Chrome focus moves to the tab list
    /// (the drag is a chrome interaction). The drag presentation (tick,
    /// offers, proxy, chip targets) begins only once the pointer travels
    /// the activation distance — a plain click never shifts the rail
    /// layout.
    pub fn begin_tab_drag(
        &mut self,
        event: &gpui::MouseDownEvent,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_tab_list(window, cx);
        // The press anchor lives inside the row surface: GPUI 0.2.2 exposes
        // no element geometry to listeners, so the press's true y within the
        // source row is unknowable here; anchor at the row surface center
        // (bounds the favorites-band hit error at half a row). Rows step on
        // the §8 pitch; the anchor stays a surface measurement (`row() / 2`),
        // while the slot geometry resolves in pitch units.
        let row_surface = self.theme().density.row();
        self.tab_drag.arm(
            index,
            f32::from(event.position.x),
            f32::from(event.position.y),
            f32::from(row_surface) / 2.0,
        );
        cx.notify();
    }

    /// Production drag-move: while a drag is active, resolve the typed
    /// target from the pointer's position. Runs from the root's mouse-move
    /// listener so the drag keeps working anywhere over the window; a move
    /// without the left button held (released outside the window) also ends
    /// the drag without committing.
    pub fn on_tab_drag_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_drag.is_active() && !self.tab_drag.is_armed() {
            return;
        }
        if event.pressed_button != Some(gpui::MouseButton::Left) {
            self.tab_drag.clear();
            cx.notify();
            return;
        }
        let x = f32::from(event.position.x);
        let y = f32::from(event.position.y);
        if self.tab_drag.is_armed() {
            // Below the activation distance the press stays a click: no
            // presentation, no layout shift.
            if !self.tab_drag.activation_exceeded(x, y) {
                return;
            }
            // The activation event commits the offers' growth and keeps the
            // no-op source slot: the offers render in the next frame, so the
            // recorded region rects are still pre-growth and a target
            // resolved now could commit a drop the user never saw.
            let growth = self.drag_activation_growth();
            self.tab_drag.activate(growth, x, y);
            cx.notify();
            return;
        }
        self.resolve_tab_drag(x, y, window);
        // Blueprint D23: over the card the drag previews a split drop.
        if let Some(source) = self.tab_drag.source() {
            let had = self.split_view.drop.is_some();
            self.update_split_drop(source, x, y);
            if had != self.split_view.drop.is_some() {
                self.sync_portal_overlays(cx);
            }
        }
        cx.notify();
    }

    /// Whether the pressed row (armed or active) is an unpinned tab — the
    /// only kind the §11.2 ghost row offers pinning to.
    pub(crate) fn drag_source_unpinned(&self) -> bool {
        let browser = self.controller.read().snapshot();
        self.tab_drag
            .pressed_row()
            .and_then(|row| browser.tabs.get(row))
            .is_some_and(|tab| !tab.pinned)
    }

    /// The rail growth the drag offers commit above the pressed row when
    /// the drag activates: the shared `bbr-ui` geometry function the render
    /// lays out with (Favorites drop-cell row when the last row is full;
    /// the ghost row above an unpinned source).
    fn drag_activation_growth(&mut self) -> f32 {
        let favorites = self
            .organization
            .bookkeep()
            .organization_snapshot(&self.controller.read())
            .favorites
            .len();
        bbr_ui::components::drag_offer::activation_growth(
            &self.theme(),
            favorites,
            self.drag_source_unpinned(),
        )
    }

    /// Recompute the drag target for a pointer position. Shared by the
    /// production mouse-move listener and the E2E seam, so both paths
    /// resolve targets through the exact same geometry. The favorites
    /// region and the §11.2 ghost row are hit through the window rects
    /// their anchors recorded in the last render; the fixed chrome above
    /// the list is not droppable, and the §11.3 strip chips below it are
    /// hit through their rendered spans (the same canonical chip visuals
    /// the render uses).
    pub(crate) fn resolve_tab_drag(&mut self, position_x: f32, position_y: f32, window: &Window) {
        let browser = self.controller.read().snapshot();
        let theme = self.theme();
        if !self.tab_drag.is_active() {
            return;
        }
        let pinned = crate::shell::pinned_count(&browser.tabs);
        // §11.10: the favorites drop band's absolute window-y span, recorded
        // by the production band anchor during the last render (the exact
        // geometry the pointer enters — never a parallel layout model).
        // `None` while no band renders (no tiles and no in-flight drag).
        let favorites_band = self.favorites_band_rect.borrow().map(DropRect::from);
        // §11.3: the rendered strip chips' visual spans, derived from the
        // production strip layout (the same `space_chip_visuals` order the
        // render, the snapshot, and the semantic tree consume).
        let chips = self.chip_spans();
        // §11.2: the ghost row's recorded span, only while it offers pinning
        // to this drag's (unpinned) source.
        let pin_offer = if self.drag_source_unpinned() {
            self.drag_offer_rects.pin_offer.borrow().map(DropRect::from)
        } else {
            None
        };
        // §11.11: the pinned region's rendered rows exclude grouped tabs.
        let organization = self
            .organization
            .bookkeep()
            .organization_snapshot(&self.controller.read());
        let grouped = crate::shell::organized::grouped_indices(&organization.folders, &browser);
        let loose = crate::shell::organized::loose_pinned(pinned, &grouped);
        // §11.11: the folder rows' recorded rects (blueprint D02 2-D
        // targets), and the rail's x-span — the page beside it is no target.
        let folders: Vec<_> = self
            .drag_offer_rects
            .folders
            .borrow()
            .iter()
            .map(|(folder, rect)| (*folder, DropRect::from(*rect)))
            .collect();
        let rail = Some(self.rail_span(window));
        let geometry = DragGeometry {
            loose: Some(&loose),
            pinned,
            len: browser.tabs.len(),
            pitch: tab_pitch(&theme),
            favorites_band,
            pin_offer,
            folders: &folders,
            rail,
            chips: &chips,
        };
        self.tab_drag.move_to(position_x, position_y, &geometry);
    }

    /// The rail's horizontal span `(x_left, x_right)` in window points —
    /// the tab drag's drop surface (blueprint D02). The docked rail spans
    /// the window's leading edge to its effective width.
    pub(crate) fn rail_span(&self, window: &Window) -> (f32, f32) {
        if !self.sidebar_visible {
            return (0.0, 0.0);
        }
        let width = f32::from(window.bounds().size.width);
        (0.0, self.sidebar.effective_width(width))
    }

    /// Production drag-end: mouse-up anywhere in the window (or, via the
    /// out-listener, released outside it) commits a legal target through the
    /// controller and clears the drag. An illegal boundary crossing, an
    /// empty target, or a no-target zone commits nothing (the drop is
    /// rejected whole).
    pub fn end_tab_drag(
        &mut self,
        _event: &gpui::MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.finish_tab_drag(window, cx);
        cx.notify();
    }

    /// The production drag-finish core: clear the drag state and commit a
    /// legal target. Shared by the mouse-up listener (`end_tab_drag`) and
    /// the benchmark seam so the drop path can never fork. Returns whether
    /// a mutation was committed (a no-op drop — own space, own slot,
    /// invalid zone, or no target — reports `false`).
    pub(crate) fn finish_tab_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.tab_drag.is_armed() {
            // The press never exceeded the activation distance: a click,
            // not a drag — nothing to commit.
            self.tab_drag.clear();
            return false;
        }
        if !self.tab_drag.is_active() {
            return false;
        }
        let target = self.tab_drag.target();
        let valid = self.tab_drag.valid();
        let source = self.tab_drag.source().expect("active drag has a source");
        self.tab_drag.clear();
        // Blueprint D23: a drop on the card splits.
        if let Some(drop) = self.split_view.drop.take() {
            self.sync_portal_overlays(cx);
            return self.commit_split_drop(source, drop, cx);
        }
        if valid {
            if let Some(target) = target {
                return self.commit_tab_drop(source, target, window, cx);
            }
        }
        false
    }

    /// Escape during a drag cancels it: the target is cleared, nothing is
    /// committed, and chrome focus stays in the tab list (the drag is a
    /// chrome interaction).
    pub fn cancel_tab_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.tab_drag.is_active() || self.tab_drag.is_armed() {
            self.tab_drag.clear();
            if self.split_view.drop.take().is_some() {
                self.sync_portal_overlays(cx);
            }
            self.focus_tab_list(window, cx);
            cx.notify();
        }
    }

    /// Commit a legal drop: a tab slot reorders within the source region
    /// (splice semantics), the favorites grid toggles the dragged tab in/out
    /// of the window-global favorites (add/remove — the tab keeps its space,
    /// order, selection, and live session), the §11.2 ghost row pins the
    /// dragged tab, and a §11.3 chip moves the
    /// dragged tab into that space through the existing single tab-move
    /// authority (`move_tab_to_space_at` — same-profile keeps the live
    /// session; cross-profile recreates it exactly once in the destination
    /// store). A drop back on the source slot, into the dragged tab's own
    /// space, or onto an invalid zone is a no-op: the drag never emits a
    /// spurious reorder/move event. Returns whether a mutation was
    /// committed.
    fn commit_tab_drop(
        &mut self,
        source: usize,
        target: DragTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match target {
            DragTarget::Tab { slot } => {
                if slot == source {
                    return false;
                }
                let to = splice_index(source, slot);
                if self.reorder_tab_at(source, to, cx).is_some() {
                    self.focus_tab_list(window, cx);
                    true
                } else {
                    false
                }
            }
            DragTarget::Favorites => {
                self.favorite_toggle_tab(source, window, cx);
                true
            }
            DragTarget::PinOffer => {
                // §11.2: the ghost row pins the dragged tab through the one
                // pin authority (appended to the pinned block; the live
                // session and the selection are untouched). A pinned source
                // is never offered the row, so this never unpins.
                let pinned = self
                    .controller
                    .read()
                    .snapshot()
                    .tabs
                    .get(source)
                    .is_some_and(|tab| tab.pinned);
                if pinned || self.toggle_pin(source, cx).is_none() {
                    return false;
                }
                self.focus_tab_list(window, cx);
                true
            }
            DragTarget::Folder { folder } => {
                // §11.11: the one move-into-folder authority (a regular tab
                // is pinned by the same transition; identity, space,
                // selection and the live session are untouched).
                self.move_tab_into_folder(source, folder, window, cx)
            }
            DragTarget::Chip { space } => {
                // Dropping on the dragged tab's own space is a no-op: the
                // dragged row lives in the active space, so the destination
                // equals the source exactly when it is the active space.
                if self.controller.read().snapshot().active_space == space {
                    return false;
                }
                if self
                    .move_tab_to_space_at(source, space, window, cx)
                    .is_some()
                {
                    self.focus_tab_list(window, cx);
                    true
                } else {
                    false
                }
            }
        }
    }
}
