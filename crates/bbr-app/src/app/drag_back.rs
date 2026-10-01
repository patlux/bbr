//! Dragging back out of the pinned area (task packet
//! `drag-back-out-of-pins`, DESIGN.md §10.9, §11.2, §11.10).
//!
//! A pinned row dragged below the New Tab row is unpinned at the regular
//! slot it is dropped on. A Favorite tile is draggable too: its press
//! activates the tile (as before) and arms a drag of that tab, which can be
//! dropped among the loose pinned rows or on the ghost row (it stays pinned
//! and leaves Favorites), among the regular tabs (it is unpinned, which
//! ends the favorite), on a folder row (grouping ends the favorite) or on a
//! space chip (it moves and stays a Favorite). Every drop goes through the
//! same one-transition authorities as the menus and keyboard.

use super::drag_target::splice_index;
use super::BbrApp;
use gpui::{Context, MouseDownEvent, Window};

impl BbrApp {
    /// The §11.2 ghost row is offered: to an unpinned row, and to a
    /// Favorite tile (dropped there it becomes a loose pinned tab). Armed
    /// presses count, so the activation growth includes it.
    pub(crate) fn drag_offers_pin_row(&self) -> bool {
        self.drag_source_unpinned() || self.tab_drag.grabbed_favorite()
    }

    /// The §11.10 drop cell is offered to an active row drag; a Favorite's
    /// own grid is never a target for it.
    pub(crate) fn drag_offers_favorites(&self) -> bool {
        self.tab_drag.is_active() && !self.tab_drag.grabbed_favorite()
    }

    /// The press on Favorite tile `index`: activate it (switch to its space
    /// and select its tab, reopening a closed one) and arm a drag of that
    /// tab. Below the activation distance the press stays a plain
    /// activation.
    pub fn press_favorite_tile(
        &mut self,
        index: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab = self
            .organization
            .bookkeep()
            .organization_snapshot(&self.controller.read())
            .favorites
            .get(index)
            .map(|favorite| favorite.tab);
        self.activate_favorite(index, window, cx);
        let Some(row) = tab.and_then(|tab| self.index_of_tab(tab)) else {
            return;
        };
        let row_surface = self.theme().density.row();
        self.tab_drag.arm_favorite(
            row,
            f32::from(event.position.x),
            f32::from(event.position.y),
            f32::from(row_surface) / 2.0,
        );
        cx.notify();
    }

    /// Commit a drop into the regular region: unpin the tab at `source`
    /// and insert it before the regular row at `slot` (tab index) in one
    /// transition. A Favorite ends with the pin. Returns whether it
    /// committed.
    pub(crate) fn commit_unpin_drop(
        &mut self,
        source: usize,
        slot: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let pinned = crate::shell::pinned_count(&self.controller.read().snapshot().tabs);
        let Some(position) = slot.checked_sub(pinned) else {
            return false;
        };
        let result = self
            .controller
            .get()
            .unpin_tab_to(source, position)
            .map_err(|error| {
                log::error!("could not unpin tab {source} to {position}: {error}");
                error
            })
            .ok();
        let Some(outcome) = result else {
            return false;
        };
        self.clear_command_profile_hint();
        self.sync_command_bar();
        self.emit_browser(&outcome);
        // The pin, the order and an ended Favorite must survive a force
        // quit.
        self.persist_structure();
        self.focus_tab_list(window, cx);
        cx.notify();
        true
    }

    /// Commit a Favorite dropped in the pinned area: move it before the
    /// pinned row at `slot` (tab index; the region's end is the pinned
    /// count) and take it out of Favorites, so it renders as a loose pinned
    /// row there. Returns whether it committed.
    pub(crate) fn commit_favorite_to_pinned(
        &mut self,
        source: usize,
        slot: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let pinned = crate::shell::pinned_count(&self.controller.read().snapshot().tabs);
        let favorite = self
            .tab_id_at(source)
            .is_some_and(|tab| self.organization.read().is_favorite(tab));
        if !favorite || source >= pinned || slot > pinned {
            return false;
        }
        let to = splice_index(source, slot);
        if to != source && self.reorder_tab_at(source, to, cx).is_none() {
            return false;
        }
        // Remove from Favorites (the toggle of a Favorite removes it).
        self.favorite_toggle_tab(to, window, cx);
        true
    }
}
