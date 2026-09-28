//! Space deletion and display-order moves on the [`Browser`] (blueprint
//! D05, DESIGN.md §11.3/§11.13): the space menu's Move Left/Right and
//! Delete Space… authorities.
//!
//! Spaces render grouped by profile, in creation order within the group,
//! so a move swaps a space with its nearest neighbor **of the same
//! profile** in the container's order. Deleting a space closes its tabs
//! into the fallback space's recently-closed stack — Reopen Closed Tab is
//! the only undo — and the last space of a profile cannot be deleted.

use crate::model::ClosedTab;
use crate::{Browser, DomainError, SpaceId, TabId};

/// The outcome of [`Browser::delete_space`]: the same-profile space that
/// takes over (the next one in display order, else the previous one) and
/// the tabs that closed, in the deleted space's order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedSpace {
    pub space: SpaceId,
    pub fallback: SpaceId,
    pub closed: Vec<TabId>,
}

/// Which way a space moves in its profile group's display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceMove {
    /// Toward the start of the group (the strip's leading side).
    Earlier,
    /// Toward the end of the group.
    Later,
}

impl Browser {
    /// The container positions of the spaces sharing `space`'s profile, in
    /// display order, plus `space`'s position in that list.
    fn profile_group(&self, space: SpaceId) -> Result<(Vec<usize>, usize), DomainError> {
        let profile = self
            .space(space)
            .ok_or(DomainError::UnknownSpace(space))?
            .profile;
        let group: Vec<usize> = self
            .spaces
            .iter()
            .enumerate()
            .filter(|(_, candidate)| candidate.profile == profile)
            .map(|(index, _)| index)
            .collect();
        let at = group
            .iter()
            .position(|index| self.spaces[*index].id == space)
            .expect("the space is in its own profile group");
        Ok((group, at))
    }

    /// Whether `space` can move `direction` (a same-profile neighbor exists
    /// on that side). Unknown spaces cannot move.
    pub fn can_move_space(&self, space: SpaceId, direction: SpaceMove) -> bool {
        self.profile_group(space)
            .map(|(group, at)| match direction {
                SpaceMove::Earlier => at > 0,
                SpaceMove::Later => at + 1 < group.len(),
            })
            .unwrap_or(false)
    }

    /// Swap `space` with its same-profile neighbor on the `direction` side.
    /// Tabs, selection, sessions and every other space stay untouched.
    /// Rejects unknown spaces and a move past the group's edge whole.
    pub fn move_space(&mut self, space: SpaceId, direction: SpaceMove) -> Result<(), DomainError> {
        let (group, at) = self.profile_group(space)?;
        let neighbor = match direction {
            SpaceMove::Earlier => at.checked_sub(1),
            SpaceMove::Later => (at + 1 < group.len()).then_some(at + 1),
        }
        .ok_or(DomainError::SpaceMoveOutOfRange(space))?;
        self.spaces.swap(group[at], group[neighbor]);
        Ok(())
    }

    /// Whether `space` may be deleted: it exists and its profile keeps at
    /// least one other space.
    pub fn can_delete_space(&self, space: SpaceId) -> bool {
        self.profile_group(space)
            .map(|(group, _)| group.len() > 1)
            .unwrap_or(false)
    }

    /// Delete `space`. Its tabs close in order onto the fallback space's
    /// recently-closed stack (re-owned by the fallback, stacked so Reopen
    /// Closed Tab restores them first-to-last after the fallback's tabs);
    /// the space's own recently-closed entries move below them. The last
    /// space of a profile and unknown spaces are rejected whole. The caller
    /// owns the sessions of the closed tabs and the active-space choice.
    pub fn delete_space(&mut self, space: SpaceId) -> Result<DeletedSpace, DomainError> {
        let (group, at) = self.profile_group(space)?;
        if group.len() < 2 {
            return Err(DomainError::LastSpaceOfProfile(space));
        }
        let fallback_position = if at + 1 < group.len() {
            group[at + 1]
        } else {
            group[at - 1]
        };
        let fallback = self.spaces[fallback_position].id;
        let position = group[at];
        let removed = self.spaces.remove(position);
        let mut moved: Vec<ClosedTab> = removed
            .recently_closed
            .into_iter()
            .map(|mut closed| {
                closed.tab.space = fallback;
                closed
            })
            .collect();
        // Most recent last: the deleted space's first tab is reopened first,
        // each at the end of the fallback's region in the deleted order (the
        // former position is a real, persistable index; reopen clamps it).
        let base = self
            .space(fallback)
            .map(|state| state.tabs.len())
            .expect("fallback space exists");
        for (offset, tab_id) in removed.tabs.iter().enumerate().rev() {
            let at = self
                .tabs
                .iter()
                .position(|tab| tab.id == *tab_id)
                .expect("space member exists in the global table");
            let mut tab = self.tabs.remove(at);
            tab.space = fallback;
            moved.push(ClosedTab {
                tab,
                index: base + offset,
            });
        }
        self.space_mut(fallback)
            .expect("fallback space exists")
            .recently_closed
            .extend(moved);
        Ok(DeletedSpace {
            space,
            fallback,
            closed: removed.tabs,
        })
    }
}
