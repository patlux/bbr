//! The [`Browser`] container: owns profiles/spaces/tabs and enforces the
//! domain invariants through every mutation.

use crate::model::{ClosedTab, Profile, Space, Tab};
use crate::split::Split;
use crate::{DomainError, ProfileId, SpaceId, TabId, TabLifecycle};

/// Owning container for all persistent browser objects.
///
/// Identity is stable and sequential: every created object gets a fresh id
/// from a single monotonically increasing counter, so ids never collide
/// within this container (UUID generation is deferred to persistence).
///
/// State is kept in insertion order (`Vec`) so the UI can present creation
/// order without re-sorting. Lookups are linear because the real collections
/// stay small; the contiguous id counter means a tab id can never collide
/// with a profile or space id. Runtime engine state (live WKWebViews) is
/// deliberately kept elsewhere; the domain only carries page identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Browser {
    next_id: u64,
    pub(crate) profiles: Vec<Profile>,
    pub(crate) spaces: Vec<Space>,
    pub(crate) tabs: Vec<Tab>,
}

impl Default for Browser {
    fn default() -> Self {
        Self::new()
    }
}

impl Browser {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            profiles: Vec::new(),
            spaces: Vec::new(),
            tabs: Vec::new(),
        }
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    // ---- Lookups ---------------------------------------------------------

    pub fn profile(&self, id: ProfileId) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    pub fn profiles(&self) -> impl Iterator<Item = &Profile> {
        self.profiles.iter()
    }

    pub fn space(&self, id: SpaceId) -> Option<&Space> {
        self.spaces.iter().find(|s| s.id == id)
    }

    pub fn spaces(&self) -> impl Iterator<Item = &Space> {
        self.spaces.iter()
    }

    pub fn tab(&self, id: TabId) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn tabs(&self) -> impl Iterator<Item = &Tab> {
        self.tabs.iter()
    }

    pub(crate) fn space_mut(&mut self, id: SpaceId) -> Option<&mut Space> {
        self.spaces.iter_mut().find(|s| s.id == id)
    }

    /// The split currently set on `space`, if any.
    pub fn split_of(&self, id: SpaceId) -> Option<&Split> {
        self.space(id).and_then(|s| s.split.as_ref())
    }

    // ---- Lookup helpers --------------------------------------------------

    /// The index of `tab` inside `space`, validating both the space and that
    /// the tab belongs to it.
    fn tab_index_in(&self, space: SpaceId, tab: TabId) -> Result<usize, DomainError> {
        let sp = self.space(space).ok_or(DomainError::UnknownSpace(space))?;
        sp.tabs
            .iter()
            .position(|t| *t == tab)
            .ok_or(DomainError::TabNotInSpace { tab, space })
    }

    /// Whether `tab` is currently visible (selected single pane, or either
    /// split pane). Public because the orchestration layer derives engine
    /// visibility directly from domain state.
    pub fn is_visible(&self, tab: TabId) -> bool {
        let Some(tab_state) = self.tab(tab) else {
            return false;
        };
        let Some(space) = self.space(tab_state.space) else {
            return false;
        };
        space.split.is_some_and(|split| split.contains(tab))
            || (space.split.is_none() && space.active_tab == Some(tab))
    }

    /// Synchronize resource policy with the tabs currently visible in a
    /// space. Newly visible tabs reactivate; tabs that stop being visible stay
    /// live but become warm. Suspended/discarded background tabs remain so.
    fn sync_lifecycles(&mut self, space: SpaceId) {
        let visible: Vec<TabId> = self
            .space(space)
            .map(|state| {
                state
                    .split
                    .map(|split| vec![split.primary, split.secondary])
                    .unwrap_or_else(|| state.active_tab.into_iter().collect())
            })
            .unwrap_or_default();
        let members = self
            .space(space)
            .map(|state| state.tabs.clone())
            .unwrap_or_default();
        for id in members {
            if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                if visible.contains(&id) {
                    // Discarded (crashed) visible tabs stay recoverable until
                    // explicitly re-activated; everything else visible becomes
                    // Active.
                    if tab.lifecycle != TabLifecycle::Discarded {
                        tab.lifecycle = TabLifecycle::Active;
                    }
                } else if tab.lifecycle == TabLifecycle::Active {
                    tab.lifecycle = TabLifecycle::Warm;
                }
            }
        }
    }

    // ---- Mutations -------------------------------------------------------

    pub fn create_profile(&mut self, title: impl Into<String>) -> ProfileId {
        let id = ProfileId(self.alloc_id());
        self.profiles.push(Profile::new(id, title));
        id
    }

    pub fn create_space(
        &mut self,
        profile: ProfileId,
        title: impl Into<String>,
    ) -> Result<SpaceId, DomainError> {
        if !self.profiles.iter().any(|p| p.id == profile) {
            return Err(DomainError::UnknownProfile(profile));
        }
        let id = SpaceId(self.alloc_id());
        self.spaces.push(Space::new(id, profile, title));
        Ok(id)
    }

    /// Open a new (unpinned) tab in `space`, append it, and activate it.
    pub fn open_tab(
        &mut self,
        space: SpaceId,
        title: impl Into<String>,
        url: impl Into<String>,
    ) -> Result<TabId, DomainError> {
        if !self.spaces.iter().any(|s| s.id == space) {
            return Err(DomainError::UnknownSpace(space));
        }
        let id = TabId(self.alloc_id());
        self.tabs.push(Tab::new(id, space, title, url));
        let sp = self.space_mut(space).expect("checked above");
        sp.tabs.push(id);
        sp.active_tab = Some(id);
        sp.split = None;
        self.sync_lifecycles(space);
        Ok(id)
    }

    /// Close `tab`, remembering it for [`Browser::reopen_tab`]. The active
    /// tab moves to the closed tab's next sibling (else previous, else
    /// `None`); closing the last tab leaves the space empty. If the tab was
    /// part of a split, the split is dissolved (both panes die, matching the
    /// two-pane v1 teardown rule).
    pub fn close_tab(&mut self, space: SpaceId, tab: TabId) -> Result<(), DomainError> {
        // Validate and compute focus before any mutation (the neighborhood
        // must be read against the pre-removal ordering).
        let index = self.tab_index_in(space, tab)?;
        let focus_after = self
            .space(space)
            .and_then(|s| s.neighbor_after_removal(index));
        let was_active = self.space(space).is_some_and(|s| s.active_tab == Some(tab));

        // Drop the tab from the global table (looked up by id, not by the
        // space-local index).
        let pos = self
            .tabs
            .iter()
            .position(|t| t.id == tab)
            .expect("ordered tab must exist in the global table");
        let closed_tab = self.tabs.remove(pos);

        let sp = self.space_mut(space).expect("checked above");
        sp.tabs.remove(index);
        if index < sp.pinned_count {
            sp.pinned_count -= 1;
        }
        sp.recently_closed.push(ClosedTab {
            tab: closed_tab,
            index,
        });
        sp.split = sp.split.filter(|s| !s.contains(tab));
        if was_active {
            sp.active_tab = focus_after;
        }
        self.sync_lifecycles(space);
        Ok(())
    }

    /// Restore the most recently closed tab of `space` at its former index
    /// (clamped to the current length, honoring the pinned region) and
    /// activate it.
    pub fn reopen_tab(&mut self, space: SpaceId) -> Result<TabId, DomainError> {
        let ClosedTab { tab, index } = self
            .space_mut(space)
            .ok_or(DomainError::UnknownSpace(space))?
            .recently_closed
            .pop()
            .ok_or(DomainError::NothingToReopen(space))?;
        let (len, pinned_count) = {
            let s = self.space(space).expect("validated above");
            (s.tabs.len(), s.pinned_count)
        };
        let insert_at = restore_index(index, len, tab.pinned, pinned_count);
        let id = tab.id;
        let pinned = tab.pinned;
        self.tabs.push(tab);
        let sp = self.space_mut(space).expect("validated above");
        sp.tabs.insert(insert_at, id);
        if pinned {
            sp.pinned_count += 1;
        }
        sp.active_tab = Some(id);
        sp.split = None;
        self.sync_lifecycles(space);
        Ok(id)
    }

    pub fn select_tab(&mut self, space: SpaceId, tab: TabId) -> Result<(), DomainError> {
        let sp = self
            .space_mut(space)
            .ok_or(DomainError::UnknownSpace(space))?;
        if !sp.tabs.contains(&tab) {
            return Err(DomainError::TabNotInSpace { tab, space });
        }
        sp.active_tab = Some(tab);
        if sp.split.is_some_and(|split| !split.contains(tab)) {
            sp.split = None;
        }
        // Selection reactivates a suspended/discarded tab: the orchestration
        // layer recreates its session (lifecycle drives session policy).
        if let Some(state) = self.tabs.iter_mut().find(|state| state.id == tab) {
            if state.lifecycle != TabLifecycle::Active {
                state.lifecycle = TabLifecycle::Active;
            }
        }
        self.sync_lifecycles(space);
        Ok(())
    }

    /// Move a hidden tab through the explicit resource lifecycle. Visibility
    /// changes themselves are driven by selection/split operations and cannot
    /// be bypassed through this method.
    pub fn transition_tab_lifecycle(
        &mut self,
        tab: TabId,
        next: TabLifecycle,
    ) -> Result<(), DomainError> {
        let visible = self.is_visible(tab);
        let current = self
            .tab(tab)
            .map(|tab| tab.lifecycle)
            .ok_or(DomainError::UnknownTab(tab))?;
        if (visible && next != TabLifecycle::Active) || (!visible && next == TabLifecycle::Active) {
            return Err(DomainError::LifecycleVisibilityConflict {
                tab,
                requested: next,
            });
        }
        if !current.can_transition_to(next) {
            return Err(DomainError::IllegalLifecycleTransition {
                tab,
                from: current,
                to: next,
            });
        }
        self.tabs
            .iter_mut()
            .find(|state| state.id == tab)
            .expect("tab validated above")
            .lifecycle = next;
        Ok(())
    }

    /// Mark a tab discarded after its WebKit process terminated (or explicit
    /// resource discard). Unlike [`Browser::transition_tab_lifecycle`], this
    /// bypasses the visibility guard: a crash is an external event and can
    /// hit a visible/warm tab, which must become recoverable (session
    /// destroyed) rather than be forced back to `Active`. Reactivation is a
    /// normal selection. The orchestration layer destroys the session.
    pub fn discard_tab(&mut self, tab: TabId) -> Result<(), DomainError> {
        if !self.tabs.iter().any(|t| t.id == tab) {
            return Err(DomainError::UnknownTab(tab));
        }
        self.tabs
            .iter_mut()
            .find(|state| state.id == tab)
            .expect("tab validated above")
            .lifecycle = TabLifecycle::Discarded;
        Ok(())
    }

    /// Persist a navigation result onto the tab's persistent identity.
    ///
    /// The engine reports the live URL/title asynchronously; without this the
    /// domain would keep the tab's original URL and a reopen would restore a
    /// stale address. Only non-empty values are written so a transient empty
    /// title cannot clobber a known one.
    pub fn apply_navigation(
        &mut self,
        tab: TabId,
        url: &str,
        title: &str,
    ) -> Result<(), DomainError> {
        let state = self
            .tabs
            .iter_mut()
            .find(|state| state.id == tab)
            .ok_or(DomainError::UnknownTab(tab))?;
        if !url.is_empty() {
            state.url = url.to_string();
        }
        if !title.is_empty() {
            state.title = title.to_string();
        }
        Ok(())
    }

    /// Reorder within `space` by current index. Reordering across the
    /// pinned/unpinned boundary is rejected so the pinned-first invariant
    /// holds by construction.
    pub fn reorder_tab(
        &mut self,
        space: SpaceId,
        from: usize,
        to: usize,
    ) -> Result<(), DomainError> {
        let sp = self
            .space_mut(space)
            .ok_or(DomainError::UnknownSpace(space))?;
        let len = sp.tabs.len();
        if from >= len {
            return Err(DomainError::IndexOutOfBounds { index: from, len });
        }
        if to >= len {
            return Err(DomainError::IndexOutOfBounds { index: to, len });
        }
        if (from < sp.pinned_count) != (to < sp.pinned_count) {
            return Err(DomainError::ReorderAcrossPinnedBoundary);
        }
        let tab = sp.tabs.remove(from);
        sp.tabs.insert(to, tab);
        Ok(())
    }

    /// Pin `tab`: move it to the end of the pinned region.
    pub fn pin_tab(&mut self, space: SpaceId, tab: TabId) -> Result<(), DomainError> {
        let sp = self
            .space_mut(space)
            .ok_or(DomainError::UnknownSpace(space))?;
        let idx = sp
            .tabs
            .iter()
            .position(|t| *t == tab)
            .ok_or(DomainError::TabNotInSpace { tab, space })?;
        if idx < sp.pinned_count {
            return Ok(()); // already pinned
        }
        sp.tabs.remove(idx);
        sp.tabs.insert(sp.pinned_count, tab);
        sp.pinned_count += 1;
        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == tab) {
            t.pinned = true;
        }
        Ok(())
    }

    /// Unpin `tab`: move it to the end of the unpinned region.
    pub fn unpin_tab(&mut self, space: SpaceId, tab: TabId) -> Result<(), DomainError> {
        let sp = self
            .space_mut(space)
            .ok_or(DomainError::UnknownSpace(space))?;
        let idx = sp
            .tabs
            .iter()
            .position(|t| *t == tab)
            .ok_or(DomainError::TabNotInSpace { tab, space })?;
        if idx >= sp.pinned_count {
            return Ok(()); // already unpinned
        }
        sp.tabs.remove(idx);
        sp.pinned_count -= 1;
        sp.tabs.push(tab);
        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == tab) {
            t.pinned = false;
        }
        Ok(())
    }

    /// Set the two-pane split of `space`. Both tabs must belong to the space
    /// and must differ from each other.
    pub fn set_split(
        &mut self,
        space: SpaceId,
        primary: TabId,
        secondary: TabId,
    ) -> Result<(), DomainError> {
        let sp = self
            .space_mut(space)
            .ok_or(DomainError::UnknownSpace(space))?;
        if primary == secondary {
            return Err(DomainError::SplitRequiresDistinctTabs);
        }
        if sp.split.is_some() {
            return Err(DomainError::SplitAlreadyExists(space));
        }
        let member = |t: TabId| sp.tabs.contains(&t);
        if !member(primary) || !member(secondary) {
            return Err(DomainError::TabNotInSpace {
                tab: if !member(primary) { primary } else { secondary },
                space,
            });
        }
        sp.split = Some(Split::new(primary, secondary));
        if !matches!(sp.active_tab, Some(active) if active == primary || active == secondary) {
            sp.active_tab = Some(primary);
        }
        // Both panes must own live sessions: reactivate discarded/suspended
        // members when the split is opened.
        for pane in [primary, secondary] {
            if let Some(state) = self.tabs.iter_mut().find(|state| state.id == pane) {
                if state.lifecycle != TabLifecycle::Active {
                    state.lifecycle = TabLifecycle::Active;
                }
            }
        }
        self.sync_lifecycles(space);
        Ok(())
    }

    /// Remove the split of `space`; both tabs stay open and pinned state is
    /// untouched.
    pub fn close_split(&mut self, space: SpaceId) -> Result<Option<Split>, DomainError> {
        let split = self
            .space_mut(space)
            .ok_or(DomainError::UnknownSpace(space))?
            .split
            .take();
        self.sync_lifecycles(space);
        Ok(split)
    }

    /// Move `tab` into `to_space`. The tab keeps its pinned state; the source
    /// space's active tab and split are repaired, and `to_space` activates
    /// the moved tab. Profiles may differ — the domain allows the move and
    /// relies on the engine layer to recreate the web view when the profiles
    /// differ (PLAN.md §4).
    pub fn move_tab(&mut self, tab: TabId, to_space: SpaceId) -> Result<(), DomainError> {
        if !self.spaces.iter().any(|s| s.id == to_space) {
            return Err(DomainError::UnknownSpace(to_space));
        }
        let from_space = self
            .tab(tab)
            .map(|t| t.space)
            .ok_or(DomainError::UnknownTab(tab))?;
        if from_space == to_space {
            return Ok(());
        }

        let index = self.tab_index_in(from_space, tab)?;
        let pinned = self
            .space(from_space)
            .is_some_and(|s| index < s.pinned_count);
        let focus_after = self
            .space(from_space)
            .and_then(|s| s.neighbor_after_removal(index));

        // Source bookkeeping.
        let sp_from = self.space_mut(from_space).expect("checked above");
        sp_from.tabs.remove(index);
        if pinned {
            sp_from.pinned_count -= 1;
        }
        if sp_from.active_tab == Some(tab) {
            sp_from.active_tab = focus_after;
        }
        sp_from.split = sp_from.split.filter(|s| !s.contains(tab));

        // Destination: honor pinned ordering, activate the moved tab.
        let sp_to = self.space_mut(to_space).expect("checked above");
        let insert_at = if pinned {
            sp_to.pinned_count
        } else {
            sp_to.tabs.len()
        };
        sp_to.tabs.insert(insert_at, tab);
        if pinned {
            sp_to.pinned_count += 1;
        }
        sp_to.active_tab = Some(tab);
        sp_to.split = None;

        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == tab) {
            t.space = to_space;
            // Destination activation: the moved tab becomes visible there, so
            // its session is recreated if it was suspended/discarded.
            if t.lifecycle != TabLifecycle::Active {
                t.lifecycle = TabLifecycle::Active;
            }
        }
        self.sync_lifecycles(from_space);
        self.sync_lifecycles(to_space);
        Ok(())
    }
}

/// Former index clamped to the current list while respecting the pinned
/// region: a pinned tab reopens inside the pinned region, an unpinned tab
/// outside it.
fn restore_index(index: usize, len: usize, pinned: bool, pinned_count: usize) -> usize {
    if pinned {
        index.min(pinned_count).min(len)
    } else {
        index
            .saturating_sub(pinned_count)
            .min(len.saturating_sub(pinned_count))
            + pinned_count
    }
}
