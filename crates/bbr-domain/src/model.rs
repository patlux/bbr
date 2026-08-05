//! Core persistent objects: [`Profile`], [`Space`], [`Tab`], [`ClosedTab`].

use crate::split::Split;
use crate::{ProfileId, SpaceId, TabId, TabLifecycle};

/// A browsing identity with an isolated website data store.
///
/// Profile identity reuses [`ProfileId`] from `bbr-web`, the same type the
/// macOS adapter maps to a persistent `WKWebsiteDataStore`. The domain holds
/// nothing platform-specific: isolation is a contract of the engine layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub id: ProfileId,
    /// User-visible name (e.g. "Work", "Personal").
    pub title: String,
}

impl Profile {
    pub fn new(id: ProfileId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
        }
    }
}

/// An ordered workspace attached to exactly one profile.
///
/// Tabs are kept in explicit insertion order; the first `pinned_count`
/// entries are pinned and keep their relative order. This mirrors the
/// "vertical tabs, pinned first" chrome without duplicating render state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Space {
    pub id: SpaceId,
    pub profile: ProfileId,
    pub title: String,
    /// Tab order inside the space. All first `pinned_count` tabs are pinned.
    pub tabs: Vec<TabId>,
    /// Number of leading pinned tabs in [`Space::tabs`].
    pub pinned_count: usize,
    /// The selected tab, or `None` when the space has no tabs yet.
    pub active_tab: Option<TabId>,
    /// Recently closed tabs, most recent last ([`Space::reopen_tab`] pops).
    pub recently_closed: Vec<ClosedTab>,
    /// The two-pane split of this space. v1 supports at most one split.
    pub split: Option<Split>,
}

impl Space {
    pub fn new(id: SpaceId, profile: ProfileId, title: impl Into<String>) -> Self {
        Self {
            id,
            profile,
            title: title.into(),
            tabs: Vec::new(),
            pinned_count: 0,
            active_tab: None,
            recently_closed: Vec::new(),
            split: None,
        }
    }

    /// The tab that would take focus after the tab at `index` is removed:
    /// its next sibling, else its previous sibling, else `None`.
    ///
    /// Must be called against the **pre-removal** ordering (the tab is still
    /// at `index`), which is the invariant [`crate::Browser`] relies on.
    pub(crate) fn neighbor_after_removal(&self, index: usize) -> Option<TabId> {
        self.tabs
            .get(index + 1)
            .copied()
            .or_else(|| index.checked_sub(1).and_then(|i| self.tabs.get(i)).copied())
    }
}

/// A persistent page identity attached to exactly one space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub id: TabId,
    /// Owning space. The owning profile is derived via the space.
    pub space: SpaceId,
    pub title: String,
    pub url: String,
    /// Pinned tabs always sort before unpinned ones in their space.
    pub pinned: bool,
    /// Policy state for the disposable native web session.
    pub lifecycle: TabLifecycle,
}

impl Tab {
    pub fn new(
        id: TabId,
        space: SpaceId,
        title: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            id,
            space,
            title: title.into(),
            url: url.into(),
            pinned: false,
            lifecycle: TabLifecycle::Active,
        }
    }
}

/// Snapshot of a closed tab kept for undo-close.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedTab {
    pub tab: Tab,
    /// Former index in [`Space::tabs`], used to restore position on reopen.
    pub index: usize,
}
