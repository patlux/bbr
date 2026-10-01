//! Unpinning to a position (task packet `drag-back-out-of-pins`): a pinned
//! row or a Favorite dragged back into the regular region lands exactly
//! where it was dropped, in one transition — never first at the region's
//! end and then reordered.

use crate::{Browser, DomainError, SpaceId, TabId};

impl Browser {
    /// Unpin `tab` and place it at `position` inside the regular region:
    /// `0` is before the first regular tab, the current regular count is
    /// after the last one. Everything [`Browser::unpin_tab`] does applies
    /// (the pinned page is forgotten, a closed tab becomes an unloaded
    /// regular tab, an in-place reopen record is dropped). A tab that is not
    /// pinned or a position past the region's end is rejected before any
    /// mutation.
    pub fn unpin_tab_to(
        &mut self,
        space: SpaceId,
        tab: TabId,
        position: usize,
    ) -> Result<(), DomainError> {
        let sp = self.space(space).ok_or(DomainError::UnknownSpace(space))?;
        let index = sp
            .tabs
            .iter()
            .position(|candidate| *candidate == tab)
            .ok_or(DomainError::TabNotInSpace { tab, space })?;
        if index >= sp.pinned_count {
            return Err(DomainError::NotPinned(tab));
        }
        // The region the tab joins: the regular tabs plus the tab itself.
        let region = sp.tabs.len() - sp.pinned_count + 1;
        if position >= region {
            return Err(DomainError::IndexOutOfBounds {
                index: position,
                len: region,
            });
        }
        self.unpin_tab(space, tab)?;
        let sp = self.space_mut(space).expect("checked above");
        let moved = sp.tabs.pop().expect("unpinning appends the tab");
        let to = sp.pinned_count + position;
        sp.tabs.insert(to, moved);
        Ok(())
    }
}
