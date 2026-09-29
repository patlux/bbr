//! Blueprint D23 (DESIGN §11.7): a space's split groups — creating,
//! growing, rearranging and dissolving them, and restoring them.
//!
//! Every group is a normalized [`SplitGroup`] tree over tabs of its space,
//! and a tab is a pane of at most one group. The visible layout is the
//! active tab's group ([`Space::visible_group`]); selecting a tab outside
//! it keeps the group. Opening or growing a group shows it with the new
//! pane focused, and every pane of a shown group is live (Active).

use crate::tiling::{Axis, Edge, Removal, SplitGroup, TilingError};
use crate::{Browser, DomainError, Space, SpaceId, Split, TabId, TabLifecycle};

/// Take `tab` out of its group, dropping the group when one pane is left.
pub(crate) fn leave_group(space: &mut Space, tab: TabId) {
    let Some(index) = space.splits.iter().position(|group| group.contains(tab)) else {
        return;
    };
    if let Ok(Removal::Dissolved(_)) = space.splits[index].remove(tab) {
        space.splits.remove(index);
    }
}

fn tiling_error(error: TilingError) -> DomainError {
    DomainError::Tiling(error)
}

impl Browser {
    fn members_of(&self, space: SpaceId, tabs: &[TabId]) -> Result<(), DomainError> {
        let sp = self.space(space).ok_or(DomainError::UnknownSpace(space))?;
        for &tab in tabs {
            if !sp.tabs.contains(&tab) {
                return Err(DomainError::TabNotInSpace { tab, space });
            }
        }
        Ok(())
    }

    /// Make `panes` live and visible after a group change focused `focus`.
    fn show_group(&mut self, space: SpaceId, focus: TabId) {
        let panes = self
            .space(space)
            .and_then(|sp| sp.group_of(focus))
            .map(SplitGroup::members)
            .unwrap_or_default();
        for pane in panes {
            if let Some(state) = self.tabs.iter_mut().find(|state| state.id == pane) {
                if state.lifecycle != TabLifecycle::Active {
                    state.lifecycle = TabLifecycle::Active;
                }
            }
        }
        if let Some(sp) = self.space_mut(space) {
            sp.active_tab = Some(focus);
        }
        self.touch(focus);
        self.sync_lifecycles(space);
    }

    /// Two tabs side by side as a new group, `primary` leading and focused
    /// (the testkit's two-pane split). Neither may be a pane already.
    pub fn set_split(
        &mut self,
        space: SpaceId,
        primary: TabId,
        secondary: TabId,
    ) -> Result<(), DomainError> {
        if primary == secondary {
            return Err(DomainError::SplitRequiresDistinctTabs);
        }
        self.members_of(space, &[primary, secondary])?;
        let sp = self.space_mut(space).expect("checked");
        if sp.group_of(primary).is_some() || sp.group_of(secondary).is_some() {
            return Err(DomainError::SplitAlreadyExists(space));
        }
        let group = SplitGroup::pair(primary, secondary, Axis::Row).map_err(tiling_error)?;
        sp.splits.push(group);
        let focus = match sp.active_tab {
            Some(active) if active == secondary => secondary,
            _ => primary,
        };
        self.show_group(space, focus);
        Ok(())
    }

    /// Place `new` at `edge` of the pane `beside` and focus it: `beside`'s
    /// group grows, or the two form a new group. `new` leaves any other
    /// group first.
    pub fn split_insert(
        &mut self,
        space: SpaceId,
        beside: TabId,
        new: TabId,
        edge: Edge,
    ) -> Result<(), DomainError> {
        if beside == new {
            return Err(DomainError::SplitRequiresDistinctTabs);
        }
        self.members_of(space, &[beside, new])?;
        let sp = self.space_mut(space).expect("checked");
        if sp
            .group_of(new)
            .is_some_and(|group| !group.contains(beside))
        {
            leave_group(sp, new);
        }
        match sp.splits.iter_mut().find(|group| group.contains(beside)) {
            Some(group) if group.contains(new) => {
                group.move_to(new, beside, edge).map_err(tiling_error)?
            }
            Some(group) => group.insert(beside, new, edge).map_err(tiling_error)?,
            None => {
                let (first, second) = if edge.after() {
                    (beside, new)
                } else {
                    (new, beside)
                };
                let group = SplitGroup::pair(first, second, edge.axis()).map_err(tiling_error)?;
                sp.splits.push(group);
            }
        }
        self.show_group(space, new);
        Ok(())
    }

    /// Show `new` in the pane of `old` (which leaves the group) and focus
    /// it.
    pub fn split_replace(
        &mut self,
        space: SpaceId,
        old: TabId,
        new: TabId,
    ) -> Result<(), DomainError> {
        self.members_of(space, &[old, new])?;
        let sp = self.space_mut(space).expect("checked");
        if sp.group_of(new).is_some() {
            return Err(DomainError::Tiling(TilingError::AlreadyMember(new)));
        }
        let group = sp
            .splits
            .iter_mut()
            .find(|group| group.contains(old))
            .ok_or(DomainError::Tiling(TilingError::UnknownPane(old)))?;
        group.replace(old, new).map_err(tiling_error)?;
        self.show_group(space, new);
        Ok(())
    }

    /// Take `tab` out of its group; the tab stays open. The focus stays.
    pub fn split_remove(&mut self, space: SpaceId, tab: TabId) -> Result<(), DomainError> {
        self.members_of(space, &[tab])?;
        let sp = self.space_mut(space).expect("checked");
        leave_group(sp, tab);
        self.sync_lifecycles(space);
        Ok(())
    }

    /// Dissolve the group `tab` belongs to; every tab stays open and the
    /// focus stays. Returns the dissolved group.
    pub fn split_dissolve(
        &mut self,
        space: SpaceId,
        tab: TabId,
    ) -> Result<Option<SplitGroup>, DomainError> {
        let sp = self
            .space_mut(space)
            .ok_or(DomainError::UnknownSpace(space))?;
        let group = sp
            .splits
            .iter()
            .position(|group| group.contains(tab))
            .map(|index| sp.splits.remove(index));
        self.sync_lifecycles(space);
        Ok(group)
    }

    /// Dissolve the visible group of `space` (the active tab's).
    pub fn close_split(&mut self, space: SpaceId) -> Result<Option<SplitGroup>, DomainError> {
        let active = self
            .space(space)
            .ok_or(DomainError::UnknownSpace(space))?
            .active_tab;
        match active {
            Some(tab) => self.split_dissolve(space, tab),
            None => Ok(None),
        }
    }

    /// Set a divider of the group `tab` belongs to (see
    /// [`SplitGroup::set_divider`]).
    pub fn split_set_divider(
        &mut self,
        space: SpaceId,
        tab: TabId,
        path: &[usize],
        divider: usize,
        leading: u32,
    ) -> Result<(), DomainError> {
        self.group_mut(space, tab)?
            .set_divider(path, divider, leading)
            .map_err(tiling_error)
    }

    /// Even out a row or column of the group `tab` belongs to.
    pub fn split_equalize(
        &mut self,
        space: SpaceId,
        tab: TabId,
        path: &[usize],
    ) -> Result<(), DomainError> {
        self.group_mut(space, tab)?
            .equalize(path)
            .map_err(tiling_error)
    }

    fn group_mut(&mut self, space: SpaceId, tab: TabId) -> Result<&mut SplitGroup, DomainError> {
        self.space_mut(space)
            .ok_or(DomainError::UnknownSpace(space))?
            .splits
            .iter_mut()
            .find(|group| group.contains(tab))
            .ok_or(DomainError::Tiling(TilingError::UnknownPane(tab)))
    }

    /// Restore a space's groups verbatim (persisted layouts): every pane a
    /// member of the space, no tab in two groups. No selection change;
    /// visibility follows the active tab.
    pub fn restore_splits(
        &mut self,
        space: SpaceId,
        groups: Vec<SplitGroup>,
    ) -> Result<(), DomainError> {
        let mut seen = Vec::new();
        for group in &groups {
            for tab in group.members() {
                if seen.contains(&tab) {
                    return Err(DomainError::SplitAlreadyExists(space));
                }
                seen.push(tab);
            }
        }
        self.members_of(space, &seen)?;
        self.space_mut(space).expect("checked").splits = groups;
        self.sync_lifecycles(space);
        Ok(())
    }

    /// Restore (or clear) a space's legacy two-pane split: the pair becomes
    /// the space's only group and shows (an active tab outside it moves to
    /// the primary pane, as the two-pane contract required). No other
    /// lifecycle change; visibility follows [`Browser::is_visible`].
    pub fn restore_split(
        &mut self,
        space: SpaceId,
        split: Option<Split>,
    ) -> Result<(), DomainError> {
        let Some(split) = split else {
            self.space_mut(space)
                .ok_or(DomainError::UnknownSpace(space))?
                .splits
                .clear();
            return Ok(());
        };
        if split.primary == split.secondary {
            return Err(DomainError::SplitRequiresDistinctTabs);
        }
        self.members_of(space, &[split.primary, split.secondary])?;
        let sp = self.space_mut(space).expect("checked");
        if !sp.splits.is_empty() {
            return Err(DomainError::SplitAlreadyExists(space));
        }
        sp.splits.push(
            SplitGroup::pair(split.primary, split.secondary, Axis::Row).map_err(tiling_error)?,
        );
        if !matches!(sp.active_tab, Some(active) if split.contains(active)) {
            sp.active_tab = Some(split.primary);
        }
        Ok(())
    }
}
