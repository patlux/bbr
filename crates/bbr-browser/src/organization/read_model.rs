//! The organization read model: the folder/favorite snapshot types the
//! UI/E2E consume and the controller read-model helpers the commands use to
//! resolve tabs and their canonical membership facts.

use bbr_domain::{SpaceId, TabFacts, TabId};
use bbr_web::ProfileId;

use super::OrganizationError;
use crate::snapshot::ControllerSnapshot;

/// One folder as presented to the UI/E2E.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderSnapshot {
    /// Position in creation order — presentation/order metadata only (the
    /// position-derived `bbr-folder-<n>` id source). `id` is the stable
    /// identity; state and commands key on `id`, never on this position.
    pub index: usize,
    pub id: bbr_domain::FolderId,
    pub space: SpaceId,
    pub title: String,
    /// Member tab ids in folder order. Resolve to space-local indices via
    /// `ControllerSnapshot::all_tabs` (which carries both).
    pub members: Vec<TabId>,
}

/// One global favorite tile as presented to the UI/E2E.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoriteSnapshot {
    /// Position in deterministic insertion order — the fav-grid tile order.
    pub index: usize,
    /// The exact tab this tile additionally presents.
    pub tab: TabId,
    /// Current owner of `tab`; changes if that same tab is moved.
    pub space: SpaceId,
    /// Derived from `space`; identifies the isolated container to activate.
    pub profile: ProfileId,
    /// Current live tab metadata, never a copied destination identity.
    pub title: String,
    pub url: String,
    /// The tab is closed in place (DESIGN.md §10.12): the tile greys out
    /// and activating it reopens the pinned page.
    pub closed: bool,
    /// The tab is the window's selected tab (its space is the active space
    /// and it is that space's active tab): the tile is the only place the
    /// selection shows, because a Favorite leaves the tab list.
    pub selected: bool,
}

/// Read-only snapshot of the session-local organization state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationSnapshot {
    /// Folders in creation order.
    pub folders: Vec<FolderSnapshot>,
    /// Favorite tiles in deterministic insertion order.
    pub favorites: Vec<FavoriteSnapshot>,
}

/// The active space's tabs in space order (resolved from the global read
/// model, which carries both tab ids and space-local indices).
pub(super) fn active_space_tabs(snapshot: &ControllerSnapshot) -> Vec<TabId> {
    let mut tabs: Vec<_> = snapshot
        .all_tabs
        .iter()
        .filter(|tab| tab.space == snapshot.active_space)
        .collect();
    tabs.sort_by_key(|tab| tab.index);
    tabs.into_iter().map(|tab| tab.id).collect()
}

/// The active space's tab at space-local `index`.
pub(super) fn active_space_tab(
    snapshot: &ControllerSnapshot,
    index: usize,
) -> Result<TabId, OrganizationError> {
    let tabs = active_space_tabs(snapshot);
    tabs.get(index)
        .copied()
        .ok_or(OrganizationError::IndexOutOfBounds {
            index,
            len: tabs.len(),
        })
}

/// A tab's current space-local index per the controller's read model.
pub(super) fn tab_index(snapshot: &ControllerSnapshot, tab: TabId) -> Option<usize> {
    snapshot
        .all_tabs
        .iter()
        .find(|t| t.id == tab)
        .map(|t| t.index)
}

/// The canonical membership facts (space, pinned) of a tab per the
/// controller's read model.
pub(super) fn tab_facts(snapshot: &ControllerSnapshot, tab: TabId) -> Option<TabFacts> {
    snapshot
        .all_tabs
        .iter()
        .find(|t| t.id == tab)
        .map(|t| TabFacts {
            space: t.space,
            pinned: t.pinned,
        })
}
