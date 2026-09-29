//! Blueprint D23: split-group commands over the active space. Each runs the
//! domain operation, then brings the sessions in line (every pane of the
//! shown group is live and framed) and closes a preview whose covered
//! layout changed.

use bbr_domain::tiling::{Edge, SplitGroup};
use bbr_domain::TabId;

use super::BrowserController;
use crate::outcome::ControllerEvent;
use crate::ControllerError;

impl BrowserController {
    fn after_split_change(&mut self) -> ControllerEvent {
        if self.preview.is_some() {
            let _ = self.close_preview();
        }
        self.sync_views();
        self.check_invariants();
        ControllerEvent::SplitChanged
    }

    /// Place the tab at `new_index` at `edge` of the tab at `beside_index`
    /// (forming or growing a group) and focus it.
    pub fn split_insert(
        &mut self,
        beside_index: usize,
        new_index: usize,
        edge: Edge,
    ) -> Result<ControllerEvent, ControllerError> {
        let beside = self.tab_at(beside_index)?;
        let new = self.tab_at(new_index)?;
        self.browser
            .split_insert(self.active_space, beside, new, edge)?;
        Ok(self.after_split_change())
    }

    /// Show the tab at `new_index` in the pane of the tab at `old_index`.
    pub fn split_replace(
        &mut self,
        old_index: usize,
        new_index: usize,
    ) -> Result<ControllerEvent, ControllerError> {
        let old = self.tab_at(old_index)?;
        let new = self.tab_at(new_index)?;
        self.browser.split_replace(self.active_space, old, new)?;
        Ok(self.after_split_change())
    }

    /// Take the tab at `index` out of its group (it stays open).
    pub fn split_remove(&mut self, index: usize) -> Result<ControllerEvent, ControllerError> {
        let tab = self.tab_at(index)?;
        self.browser.split_remove(self.active_space, tab)?;
        Ok(self.after_split_change())
    }

    /// Set a divider of the shown group.
    pub fn split_set_divider(
        &mut self,
        path: &[usize],
        divider: usize,
        leading: u32,
    ) -> Result<ControllerEvent, ControllerError> {
        let tab = self.focused_group_tab()?;
        self.browser
            .split_set_divider(self.active_space, tab, path, divider, leading)?;
        Ok(self.after_split_change())
    }

    /// Even out a row or column of the shown group.
    pub fn split_equalize(&mut self, path: &[usize]) -> Result<ControllerEvent, ControllerError> {
        let tab = self.focused_group_tab()?;
        self.browser.split_equalize(self.active_space, tab, path)?;
        Ok(self.after_split_change())
    }

    /// Restore persisted split groups of `space` (after hydration).
    pub fn restore_split_groups(
        &mut self,
        space: bbr_domain::SpaceId,
        groups: Vec<SplitGroup>,
    ) -> Result<(), ControllerError> {
        self.browser.restore_splits(space, groups)?;
        self.sync_views();
        self.check_invariants();
        Ok(())
    }

    /// Every space's split groups (for persistence).
    pub fn split_groups(&self) -> Vec<(bbr_domain::SpaceId, Vec<SplitGroup>)> {
        self.browser
            .spaces()
            .filter(|space| !space.splits.is_empty())
            .map(|space| (space.id, space.splits.clone()))
            .collect()
    }

    fn focused_group_tab(&self) -> Result<TabId, ControllerError> {
        let space = self
            .browser
            .space(self.active_space)
            .ok_or(ControllerError::NoActiveTab)?;
        match (space.active_tab, space.visible_group()) {
            (Some(tab), Some(_)) => Ok(tab),
            _ => Err(ControllerError::NoActiveTab),
        }
    }
}
