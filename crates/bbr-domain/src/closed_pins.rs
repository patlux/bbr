//! Closed pinned tabs on the [`Browser`] (DESIGN.md §10.12, task packet
//! `closed-pins-and-favorites`): a pinned tab is a kept place. Closing it
//! keeps the identity at its index as [`TabLifecycle::Closed`] at its
//! pinned page, and an *in-place* record on the space's recently-closed
//! stack remembers the page it showed. ⇧⌘T (or History › Recently closed)
//! brings that page back in the same row; selecting the row reopens it at
//! its pinned page and makes the record moot.
//!
//! Invariant: an in-place record always names a closed pinned member of its
//! space, at most once. Every path that opens, unpins or moves a closed row
//! drops its record ([`Browser::forget_reopened_in_place`]).

use crate::model::{ClosedTab, PinnedPage};
use crate::{Browser, DomainError, SpaceId, TabId, TabLifecycle};

impl Browser {
    /// Close the pinned `tab` of `space` in place. It resets to its pinned
    /// page, owns no session, leaves its split group and hands the
    /// selection to `focus_after` when it was active. Closing a closed tab
    /// is a no-op (no second record).
    pub(crate) fn close_pinned_in_place(
        &mut self,
        space: SpaceId,
        tab: TabId,
        focus_after: Option<TabId>,
        was_active: bool,
    ) -> Result<(), DomainError> {
        let index = self.tab_index_in(space, tab)?;
        let state = self
            .tabs
            .iter_mut()
            .find(|state| state.id == tab)
            .expect("ordered tab must exist in the global table");
        if state.lifecycle == TabLifecycle::Closed {
            return Ok(());
        }
        let last_page = state.clone();
        if let Some(page) = state.pinned_page.clone() {
            state.url = page.url;
            state.title = page.title;
        }
        state.lifecycle = TabLifecycle::Closed;
        let sp = self.space_mut(space).expect("checked above");
        crate::split_groups::leave_group(sp, tab);
        if was_active {
            sp.active_tab = focus_after;
        }
        sp.recently_closed.push(ClosedTab {
            tab: last_page,
            index,
            in_place: true,
        });
        self.closed_tabs += 1;
        self.sync_lifecycles(space);
        Ok(())
    }

    /// Reopen an in-place record (already removed from the stack): the
    /// closed row shows the page it closed on again and takes the
    /// selection.
    pub(crate) fn reopen_in_place(&mut self, space: SpaceId, record: ClosedTab) -> TabId {
        let id = record.tab.id;
        let state = self
            .tabs
            .iter_mut()
            .find(|state| state.id == id)
            .expect("an in-place record names a closed member");
        state.url = record.tab.url;
        state.title = record.tab.title;
        self.space_mut(space)
            .expect("the record's space exists")
            .active_tab = Some(id);
        self.touch(id);
        self.sync_lifecycles(space);
        id
    }

    /// Drop the in-place records of `space` whose row is no longer a closed
    /// member (it was reopened by a selection, unpinned or moved away):
    /// reopening it again would have nothing to bring back.
    pub(crate) fn forget_reopened_in_place(&mut self, space: SpaceId) {
        let Some(sp) = self.space(space) else {
            return;
        };
        if !sp.recently_closed.iter().any(|closed| closed.in_place) {
            return;
        }
        let waiting: Vec<TabId> = sp
            .tabs
            .iter()
            .copied()
            .filter(|id| {
                self.tab(*id)
                    .is_some_and(|tab| tab.lifecycle == TabLifecycle::Closed)
            })
            .collect();
        if let Some(sp) = self.space_mut(space) {
            sp.recently_closed
                .retain(|closed| !closed.in_place || waiting.contains(&closed.tab.id));
        }
    }

    // ---- Id-preserving restore -------------------------------------------

    /// Restore a pinned tab's kept place after [`Browser::restore_tab`]
    /// imported it as pinned: the page it was pinned at and whether it
    /// waits closed. Rejects an unknown or unpinned tab. The caller keeps a
    /// closed tab out of the selection and the split (persisted truth never
    /// shows a closed row).
    pub fn restore_kept_place(
        &mut self,
        tab: TabId,
        page: PinnedPage,
        closed: bool,
    ) -> Result<(), DomainError> {
        let state = self
            .tabs
            .iter_mut()
            .find(|state| state.id == tab)
            .ok_or(DomainError::UnknownTab(tab))?;
        if !state.pinned {
            return Err(DomainError::InvalidKeptPlace(tab));
        }
        state.pinned_page = Some(page);
        if closed {
            state.lifecycle = TabLifecycle::Closed;
        }
        Ok(())
    }

    /// Restore one in-place recently-closed record: `tab` carries the page
    /// the closed row showed when it closed. It names a restored closed
    /// pinned member of `space` (its id is that row's, so no identity is
    /// reserved) and at most one record per row.
    pub fn restore_in_place_closed(
        &mut self,
        space: SpaceId,
        tab: crate::Tab,
        former_position: usize,
    ) -> Result<(), DomainError> {
        let id = tab.id;
        let sp = self.space(space).ok_or(DomainError::UnknownSpace(space))?;
        let waiting = sp.tabs.contains(&id)
            && self
                .tab(id)
                .is_some_and(|state| state.pinned && state.lifecycle == TabLifecycle::Closed);
        let recorded = sp.recently_closed.iter().any(|closed| closed.tab.id == id);
        if !waiting || recorded {
            return Err(DomainError::InvalidKeptPlace(id));
        }
        let mut last_page = tab;
        last_page.space = space;
        last_page.pinned = true;
        last_page.lifecycle = TabLifecycle::Closed;
        self.space_mut(space)
            .expect("validated above")
            .recently_closed
            .push(ClosedTab {
                tab: last_page,
                index: former_position,
                in_place: true,
            });
        Ok(())
    }
}
