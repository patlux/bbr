//! Controller-level tab organization: the `NewSpace`, `NewFolder`,
//! `FavoriteToggle`, and `MoveTabIntoFolder` commands plus a read model for
//! the U2 sidebar's Fav-Grid and Folders regions.
//!
//! # Integration contract (lane L5 → U2)
//!
//! [`OrganizationController`] is a **companion service** next to
//! [`BrowserController`]: it owns the session-local [`FolderStore`] from
//! `bbr-domain` and never modifies the controller's public facade. The
//! composition root creates one `OrganizationController` per
//! `BrowserController` and calls the commands below from the command bar /
//! plus menu (U2d):
//!
//! - [`OrganizationController::new_space`] delegates to the controller's
//!   existing `create_space_event` (active profile) — the only command that
//!   mutates the browser.
//! - [`OrganizationController::new_folder`] creates a session-local folder
//!   in the **active space**.
//! - [`OrganizationController::move_tab_into_folder`] groups the active
//!   space's tab at `index` into `folder`; the tab keeps its space
//!   membership, ordering, selection, and live session. A tab of another
//!   space or an unknown folder is rejected **whole** (no partial mutation).
//! - [`OrganizationController::favorite_toggle`] toggles the active space's
//!   concrete tab at `index` in the window-global favorites grid. A favorite
//!   is another presentation of the same `TabId`; its owning space/profile
//!   are derived from the controller read model. Closing the tab drops the
//!   tile, while moving it keeps the favorite attached to the moved tab.
//! - [`OrganizationController::activate_favorite`] activates the favorite's
//!   owning space (and therefore its profile/container) in the same browser
//!   window, then selects that exact tab without copying or moving it.
//!
//! Because the store cannot intercept `BrowserController` close/move
//! mutations, every command and every read reconciles stale folder
//! memberships first (deterministic, id order). Callers may also invoke
//! [`OrganizationController::reconcile`] eagerly after a close/move.
//!
//! The UI reads [`OrganizationController::organization_snapshot`] (folder
//! list, member tab ids, favorite tiles in deterministic insertion order) and
//! never touches the mutable store directly.
//!
//! # Shared integration points (not owned by this lane)
//!
//! Wiring the commands into the command bar / plus menu (U2d), rendering the
//! Fav-Grid and Folders regions (U2b/U2c), icon glyphs, and persistence of
//! folders/favorites (Phase 7) land in their own lanes against this API.

use bbr_domain::{Favorite, Folder, FolderError, FolderInvariantViolation, FolderStore};
use bbr_domain::{SpaceId, TabId};
use bbr_web::ProfileId;

use crate::snapshot::ControllerSnapshot;
use crate::{BrowserController, ControllerError, ControllerEvent};

/// Typed outcomes of the organization commands.
#[derive(Debug, Clone, PartialEq)]
pub enum OrganizationEvent {
    /// `NewSpace` command outcome (created in the active profile).
    SpaceCreated { profile: ProfileId, space: SpaceId },
    /// A session-local folder was created in the active space.
    FolderCreated {
        id: bbr_domain::FolderId,
        space: SpaceId,
        title: String,
    },
    /// The tab joined `folder` (grouping only; space membership untouched).
    TabMovedIntoFolder {
        tab: TabId,
        folder: bbr_domain::FolderId,
    },
    /// The favorites grid changed: `added` tells whether the concrete tab was
    /// inserted or removed.
    FavoriteToggled {
        added: bool,
        tab: TabId,
        space: SpaceId,
        profile: ProfileId,
    },
    /// A global favorite activated its owning space/profile and exact tab.
    FavoriteActivated {
        tab: TabId,
        space: SpaceId,
        profile: ProfileId,
        index: usize,
    },
}

/// Errors returned by the organization commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrganizationError {
    /// A delegate call to the controller facade failed (currently `NewSpace`).
    Controller(ControllerError),
    /// The folder store rejected the operation (unknown folder, tab/space
    /// mismatch, invalid favorite url).
    Folder(FolderError),
    /// The active space is not in the controller's space collection.
    UnknownSpace(SpaceId),
    /// The index does not name a tab of the active space.
    IndexOutOfBounds { index: usize, len: usize },
    /// The index does not name a favorite in the global grid.
    FavoriteIndexOutOfBounds { index: usize, len: usize },
}

impl std::fmt::Display for OrganizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Controller(e) => write!(f, "controller delegate failed: {e}"),
            Self::Folder(e) => write!(f, "folder store rejected operation: {e}"),
            Self::UnknownSpace(space) => write!(f, "unknown space {space:?}"),
            Self::IndexOutOfBounds { index, len } => {
                write!(f, "index {index} out of bounds for {len} tabs")
            }
            Self::FavoriteIndexOutOfBounds { index, len } => {
                write!(f, "index {index} out of bounds for {len} favorites")
            }
        }
    }
}

impl std::error::Error for OrganizationError {}

impl From<FolderError> for OrganizationError {
    fn from(e: FolderError) -> Self {
        Self::Folder(e)
    }
}

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
}

/// Read-only snapshot of the session-local organization state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationSnapshot {
    /// Folders in creation order.
    pub folders: Vec<FolderSnapshot>,
    /// Favorite tiles in deterministic insertion order.
    pub favorites: Vec<FavoriteSnapshot>,
}

/// Portable companion service for controller-level tab organization.
///
/// See the module docs for the integration contract. Constructed once per
/// `BrowserController`; session-local state lives here (persistence is
/// Phase 7).
#[derive(Debug, Clone, PartialEq)]
pub struct OrganizationController {
    store: FolderStore,
}

impl Default for OrganizationController {
    fn default() -> Self {
        Self::new()
    }
}

impl OrganizationController {
    pub fn new() -> Self {
        Self {
            store: FolderStore::new(),
        }
    }

    // ---- Commands ---------------------------------------------------------

    /// `NewSpace` command: create a space in the active profile. Delegates to
    /// the controller facade's typed space creation and does not switch the
    /// active space (staging stays in the command bar).
    pub fn new_space(
        &mut self,
        controller: &mut BrowserController,
        title: impl Into<String>,
    ) -> Result<OrganizationEvent, OrganizationError> {
        let profile = controller.active_profile();
        match controller
            .create_space_event(profile, title)
            .map_err(OrganizationError::Controller)?
        {
            ControllerEvent::SpaceCreated { profile, space } => {
                Ok(OrganizationEvent::SpaceCreated { profile, space })
            }
            other => unreachable!("create_space_event only emits SpaceCreated, got {other:?}"),
        }
    }

    /// `NewFolder` command: create a session-local folder in the active
    /// space. The folder starts empty; tabs join it via
    /// [`OrganizationController::move_tab_into_folder`].
    pub fn new_folder(
        &mut self,
        controller: &BrowserController,
        title: impl Into<String>,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        let snapshot = controller.snapshot();
        let space = snapshot.active_space;
        if !snapshot.spaces.iter().any(|s| s.id == space) {
            return Err(OrganizationError::UnknownSpace(space));
        }
        let id = self.store.create_folder_in(space, title);
        let title = self
            .store
            .folder(id)
            .expect("folder just created")
            .title
            .clone();
        self.check_invariants(controller)
            .expect("organization invariants hold after reconcile");
        Ok(OrganizationEvent::FolderCreated { id, space, title })
    }

    /// `MoveTabIntoFolder` command: group the active space's tab at `index`
    /// into `folder`. The tab keeps its space membership, ordering,
    /// selection, and live session; membership in a previous folder is left.
    /// Unknown folders and tabs of another space are rejected whole.
    pub fn move_tab_into_folder(
        &mut self,
        controller: &BrowserController,
        index: usize,
        folder: bbr_domain::FolderId,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        let snapshot = controller.snapshot();
        let tabs = active_space_tabs(&snapshot);
        let tab = *tabs.get(index).ok_or(OrganizationError::IndexOutOfBounds {
            index,
            len: tabs.len(),
        })?;
        let tab_space_id = tab_space(&snapshot, tab)
            .ok_or(OrganizationError::Folder(FolderError::UnknownTab(tab)))?;
        self.store.move_membership(folder, tab, tab_space_id)?;
        self.check_invariants(controller)
            .expect("organization invariants hold after reconcile");
        Ok(OrganizationEvent::TabMovedIntoFolder { tab, folder })
    }

    /// `FavoriteToggle` command: toggle the active space's concrete tab at
    /// `index` in the window-global favorites grid. Toggling the same `TabId`
    /// removes it; re-adding appends it at the end.
    pub fn favorite_toggle(
        &mut self,
        controller: &BrowserController,
        index: usize,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        let snapshot = controller.snapshot();
        let tabs = active_space_tabs(&snapshot);
        let tab = *tabs.get(index).ok_or(OrganizationError::IndexOutOfBounds {
            index,
            len: tabs.len(),
        })?;
        let global = snapshot
            .all_tabs
            .iter()
            .find(|candidate| candidate.id == tab)
            .expect("active-space tab exists in global snapshot");
        let outcome = self.store.favorite_toggle_tab(tab);
        self.check_invariants(controller)
            .expect("organization invariants hold after reconcile");
        Ok(OrganizationEvent::FavoriteToggled {
            added: outcome.added,
            tab,
            space: global.space,
            profile: global.profile,
        })
    }

    /// Activate a favorite from any current space in the same browser window.
    /// The destination space switch implicitly selects its profile/container;
    /// then the exact favorited tab is selected by its current space-local
    /// index. The tab is neither copied nor moved.
    pub fn activate_favorite(
        &mut self,
        controller: &mut BrowserController,
        favorite_index: usize,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        let favorite = self
            .store
            .favorites()
            .nth(favorite_index)
            .cloned()
            .ok_or_else(|| OrganizationError::FavoriteIndexOutOfBounds {
                index: favorite_index,
                len: self.store.favorites().count(),
            })?;
        let snapshot = controller.snapshot();
        let global = snapshot
            .all_tabs
            .iter()
            .find(|candidate| candidate.id == favorite.tab)
            .ok_or(OrganizationError::Folder(FolderError::UnknownTab(
                favorite.tab,
            )))?;
        let (tab, space, profile, index) = (global.id, global.space, global.profile, global.index);
        controller
            .activate_space(space)
            .map_err(OrganizationError::Controller)?;
        controller
            .select_tab(index)
            .map_err(OrganizationError::Controller)?;
        Ok(OrganizationEvent::FavoriteActivated {
            tab,
            space,
            profile,
            index,
        })
    }

    // ---- Read model -------------------------------------------------------

    /// Deterministically drop folder memberships for tabs that no longer
    /// exist in the controller or no longer belong to their folder's space.
    /// Returns the number of memberships removed. Every command and every
    /// snapshot reconciles first, so this only needs explicit calls for eager
    /// cleanup right after a close/move.
    pub fn reconcile(&mut self, controller: &BrowserController) -> usize {
        let snapshot = controller.snapshot();
        self.store.reconcile_with(|tab| tab_space(&snapshot, tab))
    }

    /// Folders in creation order (reconciled against the controller).
    pub fn folders(&mut self, controller: &BrowserController) -> Vec<Folder> {
        self.reconcile(controller);
        self.store.folders().cloned().collect()
    }

    /// Favorite tiles in deterministic insertion order.
    pub fn favorites(&self) -> impl Iterator<Item = &Favorite> {
        self.store.favorites()
    }

    /// Read-only snapshot for the UI/E2E: folders in creation order and
    /// favorite tiles in deterministic insertion order. Reconciles first so
    /// the read model never renders a closed or moved tab.
    pub fn organization_snapshot(
        &mut self,
        controller: &BrowserController,
    ) -> OrganizationSnapshot {
        self.reconcile(controller);
        let snapshot = controller.snapshot();
        let folders = self
            .store
            .folders()
            .enumerate()
            .map(|(index, folder)| FolderSnapshot {
                index,
                id: folder.id,
                space: folder.space,
                title: folder.title.clone(),
                members: folder.tabs.clone(),
            })
            .collect();
        let favorites = self
            .store
            .favorites()
            .enumerate()
            .map(|(index, favorite)| {
                let tab = snapshot
                    .all_tabs
                    .iter()
                    .find(|candidate| candidate.id == favorite.tab)
                    .expect("reconcile removed missing favorite tabs");
                FavoriteSnapshot {
                    index,
                    tab: tab.id,
                    space: tab.space,
                    profile: tab.profile,
                    title: tab.title.clone(),
                    url: tab.url.clone(),
                }
            })
            .collect();
        OrganizationSnapshot { folders, favorites }
    }

    /// Structural agreement between the store and the controller's public
    /// read model. Detector, not repair tool: call after a reconcile (every
    /// command and snapshot already does).
    pub fn check_invariants(
        &self,
        controller: &BrowserController,
    ) -> Result<(), FolderInvariantViolation> {
        let snapshot = controller.snapshot();
        let space_exists = |space: SpaceId| snapshot.spaces.iter().any(|s| s.id == space);
        let tab_space = |tab: TabId| tab_space(&snapshot, tab);
        self.store.check_invariants_with(space_exists, tab_space)
    }
}

/// The active space's tabs in space order (resolved from the global read
/// model, which carries both tab ids and space-local indices).
fn active_space_tabs(snapshot: &ControllerSnapshot) -> Vec<TabId> {
    let mut tabs: Vec<_> = snapshot
        .all_tabs
        .iter()
        .filter(|tab| tab.space == snapshot.active_space)
        .collect();
    tabs.sort_by_key(|tab| tab.index);
    tabs.into_iter().map(|tab| tab.id).collect()
}

/// The space a tab belongs to per the controller's read model.
fn tab_space(snapshot: &ControllerSnapshot, tab: TabId) -> Option<SpaceId> {
    snapshot
        .all_tabs
        .iter()
        .find(|t| t.id == tab)
        .map(|t| t.space)
}
