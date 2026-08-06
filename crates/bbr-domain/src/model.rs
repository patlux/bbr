//! Core persistent objects: [`Profile`], [`Space`], [`Tab`], [`ClosedTab`].

use crate::split::Split;
use crate::{ProfileId, SpaceId, TabId, TabLifecycle};

/// The six DESIGN.md §12 profile hues.
///
/// Assignment is round-robin and stable for the profile's lifetime: the hue
/// is a portable identity fact, while the concrete tile/letter colors are a
/// presentation concern resolved by the shell's palette from this name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileHue {
    Cobalt,
    Violet,
    Fuchsia,
    Amber,
    Terracotta,
    Slate,
}

impl ProfileHue {
    /// All hues in round-robin assignment order (DESIGN.md §12).
    pub const ALL: [ProfileHue; 6] = [
        ProfileHue::Cobalt,
        ProfileHue::Violet,
        ProfileHue::Fuchsia,
        ProfileHue::Amber,
        ProfileHue::Terracotta,
        ProfileHue::Slate,
    ];

    /// The canonical machine-readable name used by snapshots and E2E
    /// assertions (DESIGN.md §12 allowed values).
    pub fn name(self) -> &'static str {
        match self {
            ProfileHue::Cobalt => "cobalt",
            ProfileHue::Violet => "violet",
            ProfileHue::Fuchsia => "fuchsia",
            ProfileHue::Amber => "amber",
            ProfileHue::Terracotta => "terracotta",
            ProfileHue::Slate => "slate",
        }
    }
}

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
    /// Stable round-robin identity hue (DESIGN.md §12).
    pub hue: ProfileHue,
}

impl Profile {
    pub fn new(id: ProfileId, hue: ProfileHue, title: impl Into<String>) -> Self {
        Self {
            id,
            hue,
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
    /// Monotonic recency stamp (the dense-list resource policy's "recently
    /// used" order): bumped by open/select/reopen/destination-activation, so
    /// the session-budget policy keeps recently used identities live without
    /// a wall clock. `0` for tabs built directly via [`Tab::new`] (the
    /// [`Browser`](crate::Browser) container owns the counter).
    pub last_used: u64,
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
            last_used: 0,
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
