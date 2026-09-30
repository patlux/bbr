//! Blueprint D19 (several windows, task packet `multi-window.md`): a tab
//! dragged out of one window and dropped on another window's rail.

use bbr_domain::TabId;

use super::BrowserController;
use crate::outcome::ControllerEvent;
use crate::ControllerError;

impl BrowserController {
    /// The space a tab lives in.
    pub fn space_of_tab(&self, tab: TabId) -> Option<bbr_domain::SpaceId> {
        Some(self.browser.tab(tab)?.space)
    }

    /// A tab dragged from another window lands in this (the current)
    /// window's active space and shows here. `slot` is the insertion
    /// position among the space's tabs as the rail showed them (`None`:
    /// where a move puts it); the tab stays in its pinned or regular region.
    /// From another space it moves first — across profiles its session is
    /// recreated in the destination store. Selecting it here takes it from
    /// the window that showed it.
    pub fn drop_tab_here(
        &mut self,
        tab: TabId,
        slot: Option<usize>,
    ) -> Result<ControllerEvent, ControllerError> {
        let space = self.active_space;
        let from = self.space_of_tab(tab).ok_or(ControllerError::Domain(
            bbr_domain::DomainError::UnknownTab(tab),
        ))?;
        let before = self
            .browser
            .space(space)
            .and_then(|state| state.tabs.iter().position(|candidate| *candidate == tab));
        let moved = if from == space {
            None
        } else {
            Some(self.move_tab_id(tab, space)?)
        };
        if let Some(slot) = slot {
            let state = self
                .browser
                .space(space)
                .ok_or(ControllerError::UnknownSpace(space))?;
            let index = state
                .tabs
                .iter()
                .position(|candidate| *candidate == tab)
                .expect("the tab is in the space");
            let pinned = state.pinned_count;
            let (low, high) = if index < pinned {
                (0, pinned.saturating_sub(1))
            } else {
                (pinned, state.tabs.len().saturating_sub(1))
            };
            // A slot counted among the rail's rows: within the space the
            // dragged row itself is one of them (splice), a moved-in tab was
            // not.
            let wanted = match before {
                Some(before) if slot > before => slot - 1,
                _ => slot,
            };
            let to = wanted.clamp(low, high);
            if to != index {
                self.browser.reorder_tab(space, index, to)?;
            }
        }
        self.browser.select_tab(space, tab)?;
        self.close_preview_if_stale();
        self.sync_views();
        self.check_invariants();
        let index = self.index_of(tab).expect("the tab is in the space");
        Ok(moved.unwrap_or(ControllerEvent::TabSelected { index }))
    }
}
