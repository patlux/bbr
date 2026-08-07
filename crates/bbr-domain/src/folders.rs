//! Session-local tab organization: [`Folder`] entities, the favorites grid,
//! and the [`FolderStore`] companion that enforces their invariants.
//!
//! This is the portable foundation for the U2 sidebar's Fav-Grid and Folders
//! regions (docs/delivery/2026-08-07-browser-mvp-parallel-plan.md, lane L5).
//! It is **session-local**: folders and favorites live in memory for the
//! browser session and are not persisted — durability is Phase 7.
//!
//! # Why a companion store?
//!
//! [`Browser`] is the canonical owner of profiles/spaces/tabs and must not be
//! edited by a feature lane. `FolderStore` therefore holds the organization
//! state **beside** the `Browser` (the orchestration layer owns both) and
//! validates against the `Browser`'s public read API at mutation time. This
//! preserves every existing ownership invariant:
//!
//! - A [`Profile`] owns spaces; a [`Space`] owns tabs; a [`Tab`] belongs to
//!   exactly one space (its profile is derived, never stored).
//! - A [`Folder`] belongs to exactly one space, and every member tab belongs
//!   to that same space. Moving a tab into a folder never changes the tab's
//!   space membership or ordering — the folder is a grouping layer, not a
//!   replacement for space ownership.
//! - Favorites are **page identities** (a title + url snapshot), not tab
//!   references, so closing a tab never invalidates a favorite tile.
//!
//! # Invariants enforced by [`FolderStore`]
//!
//! 1. Folder ids are unique and stable for the store's lifetime.
//! 2. Every folder references an existing space.
//! 3. Every folder member tab exists and belongs to the folder's space.
//! 4. A tab belongs to at most one folder.
//! 5. A folder lists a tab at most once.
//! 6. Favorites carry a non-empty url and urls are unique (a page is present
//!    at most once in the grid).
//!
//! Mutations reject invalid input **whole** (before any mutation); the
//! invariant checker is public and unit-tested so tampering is detectable.
//! Because `FolderStore` cannot intercept `Browser` mutations (close/move),
//! memberships can go stale; [`FolderStore::reconcile`] deterministically
//! drops memberships for tabs that no longer exist or changed space, and
//! every orchestration-layer command reconciles before observing state.
//!
//! # Deterministic favorites ordering
//!
//! Favorites keep **insertion order**: the grid renders in the order pages
//! were added. Re-adding a page after removal appends it at the end, and
//! toggling a page that is already present removes it (its identity is the
//! url). Identical operation sequences therefore always produce identical
//! orderings.

use std::collections::HashSet;

use crate::{Browser, SpaceId, TabId};

/// Stable identifier of a session-local folder (a distinct id space from
/// profile/space/tab ids; numeric equality with another kind is harmless
/// because the types never mix).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FolderId(pub u64);

/// A session-local folder grouping tabs of exactly one space.
///
/// Member tabs are kept in explicit insertion order. Membership is a
/// grouping over the space's existing tab list — a member keeps its space
/// membership, selection, and live session untouched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub id: FolderId,
    /// The space this folder belongs to. Every member tab belongs to it.
    pub space: SpaceId,
    pub title: String,
    /// Member tabs in deterministic insertion order.
    pub tabs: Vec<TabId>,
}

/// A favorite page identity in the session-local favorites grid.
///
/// Favorites are page shortcuts, not tab references: the grid keeps its
/// tiles even after the tab that was favorited is closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorite {
    pub title: String,
    pub url: String,
}

/// Outcome of one favorites toggle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoriteToggle {
    /// `true` when the page was added to the grid, `false` when removed.
    pub added: bool,
    pub title: String,
    pub url: String,
}

/// Errors returned by folder/favorites operations.
///
/// Invalid input is rejected before any mutation, so a rejected operation
/// never partially changes the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderError {
    UnknownFolder(FolderId),
    UnknownSpace(SpaceId),
    UnknownTab(TabId),
    /// The tab exists but belongs to a different space than the folder.
    TabNotInFolderSpace {
        tab: TabId,
        folder: FolderId,
    },
    /// Favorites need a non-empty url identity (a tab with no url cannot be
    /// pinned into the grid).
    InvalidFavoriteUrl,
}

impl std::fmt::Display for FolderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFolder(id) => write!(f, "unknown folder {id:?}"),
            Self::UnknownSpace(id) => write!(f, "unknown space {id:?}"),
            Self::UnknownTab(id) => write!(f, "unknown tab {id:?}"),
            Self::TabNotInFolderSpace { tab, folder } => write!(
                f,
                "tab {tab:?} does not belong to the space of folder {folder:?}"
            ),
            Self::InvalidFavoriteUrl => write!(f, "a favorite needs a non-empty url"),
        }
    }
}

impl std::error::Error for FolderError {}

/// A structural invariant violation found by
/// [`FolderStore::check_invariants`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderInvariantViolation(pub String);

impl std::fmt::Display for FolderInvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "folder invariant violated: {}", self.0)
    }
}

impl std::error::Error for FolderInvariantViolation {}

/// Session-local owner of folders and the favorites grid, kept beside the
/// [`Browser`] (see module docs for the ownership rationale).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderStore {
    /// Folder ids are sequential from one counter (like the `Browser` id
    /// policy), so they never collide within this store.
    next_id: u64,
    /// Folders in creation order. `pub(crate)` like `Browser`'s tables so
    /// the invariant checker's tamper-detection tests can forge violations
    /// (mutations themselves forbid them).
    pub(crate) folders: Vec<Folder>,
    /// Favorite pages in deterministic insertion order. `pub(crate)` for the
    /// same tamper-detection reason.
    pub(crate) favorites: Vec<Favorite>,
}

impl Default for FolderStore {
    /// [`FolderStore::new`] keeps the id counter consistent with any
    /// `Default` construction.
    fn default() -> Self {
        Self::new()
    }
}

impl FolderStore {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            folders: Vec::new(),
            favorites: Vec::new(),
        }
    }

    // ---- Folders ----------------------------------------------------------

    /// Create a session-local folder in `space`, validated against the live
    /// browser. The folder starts empty.
    pub fn create_folder(
        &mut self,
        browser: &Browser,
        space: SpaceId,
        title: impl Into<String>,
    ) -> Result<FolderId, FolderError> {
        if browser.space(space).is_none() {
            return Err(FolderError::UnknownSpace(space));
        }
        Ok(self.create_folder_in(space, title))
    }

    /// Validated-facts primitive: create a folder in `space` without
    /// consulting the browser. The caller must already have established that
    /// `space` exists (the orchestration layer validates against its public
    /// read model). Prefer [`FolderStore::create_folder`] where the browser
    /// is available.
    pub fn create_folder_in(&mut self, space: SpaceId, title: impl Into<String>) -> FolderId {
        let id = FolderId(self.next_id);
        self.next_id += 1;
        self.folders.push(Folder {
            id,
            space,
            title: title.into(),
            tabs: Vec::new(),
        });
        id
    }

    /// Move `tab` into `folder`, validating the tab against the live browser.
    ///
    /// The tab keeps its space membership and ordering; membership is a
    /// grouping layer. Moving a tab into a folder it already belongs to is a
    /// no-op; moving it from another folder removes it there first. A tab of
    /// a different space is rejected whole (no partial mutation).
    pub fn move_tab_into_folder(
        &mut self,
        browser: &Browser,
        folder: FolderId,
        tab: TabId,
    ) -> Result<(), FolderError> {
        let tab_space = browser
            .tab(tab)
            .map(|state| state.space)
            .ok_or(FolderError::UnknownTab(tab))?;
        self.move_membership(folder, tab, tab_space)
    }

    /// Validated-facts primitive: move `tab` (which the caller established
    /// exists and lives in `tab_space`) into `folder`. Prefer
    /// [`FolderStore::move_tab_into_folder`] where the browser is available.
    pub fn move_membership(
        &mut self,
        folder: FolderId,
        tab: TabId,
        tab_space: SpaceId,
    ) -> Result<(), FolderError> {
        let target = self
            .folders
            .iter()
            .find(|f| f.id == folder)
            .ok_or(FolderError::UnknownFolder(folder))?;
        if target.space != tab_space {
            return Err(FolderError::TabNotInFolderSpace { tab, folder });
        }
        if target.tabs.contains(&tab) {
            return Ok(()); // already a member: no-op, like pinning a pinned tab
        }
        // Leave any other folder (deterministic: folder creation order).
        for f in &mut self.folders {
            if f.id != folder {
                f.tabs.retain(|t| *t != tab);
            }
        }
        let target = self
            .folders
            .iter_mut()
            .find(|f| f.id == folder)
            .expect("folder validated above");
        target.tabs.push(tab);
        Ok(())
    }

    /// Remove `tab` from `folder`. Removing a non-member is a no-op (like
    /// unpinning an unpinned tab); the folder itself must exist.
    pub fn remove_tab_from_folder(
        &mut self,
        folder: FolderId,
        tab: TabId,
    ) -> Result<(), FolderError> {
        let target = self
            .folders
            .iter_mut()
            .find(|f| f.id == folder)
            .ok_or(FolderError::UnknownFolder(folder))?;
        target.tabs.retain(|t| *t != tab);
        Ok(())
    }

    // ---- Favorites --------------------------------------------------------

    /// Toggle `tab`'s page identity in the favorites grid, reading the
    /// current title/url from the live browser. Adding a page that is already
    /// present removes it; removal of a page re-added later appends it at the
    /// end of the grid.
    pub fn favorite_toggle(
        &mut self,
        browser: &Browser,
        tab: TabId,
    ) -> Result<FavoriteToggle, FolderError> {
        let state = browser.tab(tab).ok_or(FolderError::UnknownTab(tab))?;
        self.favorite_toggle_page(state.title.clone(), state.url.clone())
    }

    /// Validated-facts primitive: toggle a page identity (title + url) in
    /// the grid. Prefer [`FolderStore::favorite_toggle`] where the browser
    /// is available.
    pub fn favorite_toggle_page(
        &mut self,
        title: impl Into<String>,
        url: impl Into<String>,
    ) -> Result<FavoriteToggle, FolderError> {
        let title = title.into();
        let url = url.into();
        if url.is_empty() {
            return Err(FolderError::InvalidFavoriteUrl);
        }
        if let Some(pos) = self.favorites.iter().position(|f| f.url == url) {
            self.favorites.remove(pos);
            return Ok(FavoriteToggle {
                added: false,
                title,
                url,
            });
        }
        self.favorites.push(Favorite {
            title: title.clone(),
            url: url.clone(),
        });
        Ok(FavoriteToggle {
            added: true,
            title,
            url,
        })
    }

    // ---- Reconciliation ---------------------------------------------------

    /// Deterministically drop memberships for tabs that no longer exist in
    /// the browser or no longer belong to their folder's space (the browser
    /// owns close/move and cannot notify this store). Returns the number of
    /// memberships removed.
    pub fn reconcile(&mut self, browser: &Browser) -> usize {
        self.reconcile_with(|tab| browser.tab(tab).map(|state| state.space))
    }

    /// Validated-facts variant of [`FolderStore::reconcile`] for callers
    /// that resolve tab→space facts without the browser (the orchestration
    /// layer uses its public read model). `tab_space` returns `None` for
    /// tabs that no longer exist. Iteration stays in folder creation order,
    /// then member order, so reconciliation is fully deterministic.
    pub fn reconcile_with(&mut self, tab_space: impl Fn(TabId) -> Option<SpaceId>) -> usize {
        let mut removed = 0;
        for folder in &mut self.folders {
            let before = folder.tabs.len();
            folder
                .tabs
                .retain(|tab| tab_space(*tab) == Some(folder.space));
            removed += before - folder.tabs.len();
        }
        removed
    }

    // ---- Lookups ----------------------------------------------------------

    pub fn folders(&self) -> impl Iterator<Item = &Folder> {
        self.folders.iter()
    }

    pub fn folder(&self, id: FolderId) -> Option<&Folder> {
        self.folders.iter().find(|f| f.id == id)
    }

    pub fn favorites(&self) -> impl Iterator<Item = &Favorite> {
        self.favorites.iter()
    }

    // ---- Invariants -------------------------------------------------------

    /// Validate every invariant (see module docs) against the live browser.
    pub fn check_invariants(&self, browser: &Browser) -> Result<(), FolderInvariantViolation> {
        self.check_invariants_with(
            |space| browser.space(space).is_some(),
            |tab| browser.tab(tab).map(|state| state.space),
        )
    }

    /// Validated-facts variant of [`FolderStore::check_invariants`] for
    /// callers that resolve space/tab facts without the browser. Requires a
    /// prior reconcile (or a guarantee that no browser mutation has gone
    /// stale): it is a detector, not a repair tool.
    pub fn check_invariants_with(
        &self,
        space_exists: impl Fn(SpaceId) -> bool,
        tab_space: impl Fn(TabId) -> Option<SpaceId>,
    ) -> Result<(), FolderInvariantViolation> {
        let fail = |msg: String| FolderInvariantViolation(msg);

        let mut seen_folders: HashSet<FolderId> = HashSet::new();
        let mut members: HashSet<TabId> = HashSet::new();
        for folder in &self.folders {
            if !seen_folders.insert(folder.id) {
                return Err(fail(format!("duplicate folder id {:?}", folder.id)));
            }
            if !space_exists(folder.space) {
                return Err(fail(format!(
                    "folder {:?} references unknown space {:?}",
                    folder.id, folder.space
                )));
            }
            let mut seen_tabs: HashSet<TabId> = HashSet::new();
            for (i, tab) in folder.tabs.iter().enumerate() {
                if !seen_tabs.insert(*tab) {
                    return Err(fail(format!(
                        "folder {:?} lists tab {:?} more than once at index {i}",
                        folder.id, tab
                    )));
                }
                match tab_space(*tab) {
                    Some(space) if space == folder.space => {}
                    Some(space) => {
                        return Err(fail(format!(
                            "folder {:?} member tab {:?} belongs to space {:?}, not {:?}",
                            folder.id, tab, space, folder.space
                        )))
                    }
                    None => {
                        return Err(fail(format!(
                            "folder {:?} member tab {:?} does not exist",
                            folder.id, tab
                        )))
                    }
                }
            }
            for tab in &folder.tabs {
                if !members.insert(*tab) {
                    return Err(fail(format!(
                        "tab {:?} belongs to more than one folder",
                        tab
                    )));
                }
            }
        }

        let mut urls: HashSet<&str> = HashSet::new();
        for favorite in &self.favorites {
            if favorite.url.is_empty() {
                return Err(fail("a favorite needs a non-empty url identity".into()));
            }
            if !urls.insert(favorite.url.as_str()) {
                return Err(fail(format!("duplicate favorite url {:?}", favorite.url)));
            }
        }
        Ok(())
    }
}
