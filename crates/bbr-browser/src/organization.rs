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
//!   space's tab at `index` into `folder` through the one domain transition
//!   ([`FolderStore::move_tab_into_folder`]): a folder is the grouped form of
//!   pinned tabs, so a regular tab is pinned in the same step (it moves to
//!   the end of the pinned region); it keeps its identity, space membership,
//!   selection, and live session. A tab of another space or an unknown
//!   folder is rejected **whole** (neither the pin nor the membership
//!   changes).
//! - [`OrganizationController::remove_tab_from_folder`] ungroups a member:
//!   it stays pinned as a loose pinned tab. Unpinning a member through any
//!   controller path ends its membership at the next reconcile.
//! - [`OrganizationController::favorite_toggle`] toggles the active space's
//!   concrete tab at `index` in the window-global favorites grid. The tile
//!   *is* the tab (DESIGN.md §11.10): a favorite is pinned (a regular tab is
//!   pinned by the same transition) and leaves its folder, so it lists in
//!   the grid only; closing it closes it in place and the tile greys out;
//!   Remove from Favorites leaves a loose pinned tab. Its owning
//!   space/profile are derived from the controller read model, and moving
//!   the tab keeps the favorite attached to the moved tab.
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
//! Fav-Grid and Folders regions (U2b/U2c) and icon glyphs land in their own
//! lanes against this API. The app persists the organization with the
//! browser structure ([`OrganizationController::persisted`]) and restores it
//! after the controller ([`OrganizationController::restored`]; task packet
//! `closed-pins-and-favorites`).

use bbr_domain::{Favorite, Folder, FolderError, FolderInvariantViolation, FolderStore};
use bbr_domain::{SpaceId, TabId};
use bbr_web::ProfileId;

use crate::{BrowserController, ControllerError, ControllerEvent};

mod read_model;

use read_model::{active_space_tab, active_space_tabs, tab_facts, tab_index};
pub use read_model::{FavoriteSnapshot, FolderSnapshot, OrganizationSnapshot};

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
    /// Blueprint D05: a space was deleted with its folders and tabs.
    SpaceDeleted {
        space: SpaceId,
        fallback: SpaceId,
        closed_tabs: usize,
        removed_folders: usize,
        activated: bool,
    },
    /// Blueprint D25: a folder was unpacked; its members stay loose pinned
    /// tabs in folder order.
    FolderUnpacked {
        id: bbr_domain::FolderId,
        members: Vec<TabId>,
    },
    /// A folder took a new (trimmed, non-empty) title.
    FolderRenamed {
        id: bbr_domain::FolderId,
        title: String,
    },
    /// The tab joined `folder` (space membership untouched). A regular tab
    /// was pinned by the same transition: `pinned_index` is its new
    /// space-local index then, `None` when it was already pinned.
    TabMovedIntoFolder {
        tab: TabId,
        folder: bbr_domain::FolderId,
        pinned_index: Option<usize>,
    },
    /// The member left `folder` and stays a loose pinned tab.
    TabRemovedFromFolder {
        tab: TabId,
        folder: bbr_domain::FolderId,
    },
    /// The favorites grid changed: `added` tells whether the concrete tab was
    /// inserted or removed. Adding a regular tab pinned it by the same
    /// transition: `pinned_index` is its new space-local index then, `None`
    /// when it was already pinned (or on removal).
    FavoriteToggled {
        added: bool,
        tab: TabId,
        space: SpaceId,
        profile: ProfileId,
        pinned_index: Option<usize>,
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

/// What a space deletion would remove (blueprint D05's delete sheet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceDeleteSummary {
    pub title: String,
    pub tabs: usize,
    pub folders: usize,
    /// How many of the space's tabs are favorites (their tiles go too).
    pub favorites: usize,
    /// Whether the space may be deleted (not the last of its profile).
    pub deletable: bool,
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

    /// Restore a persisted organization against the restored controller:
    /// folders keep their ids, members and order, the favorites grid its
    /// order. Rejected whole when it breaks an organization invariant (the
    /// caller logs it and starts with an empty organization).
    pub fn restored(
        controller: &BrowserController,
        folders: Vec<Folder>,
        favorites: Vec<TabId>,
    ) -> Result<Self, FolderInvariantViolation> {
        let snapshot = controller.snapshot();
        let store = FolderStore::restored(
            folders,
            favorites,
            |space| snapshot.spaces.iter().any(|s| s.id == space),
            |tab| tab_facts(&snapshot, tab),
        )?;
        Ok(Self { store })
    }

    /// The organization as the store persists it: folders in order and the
    /// favorites grid in order, reconciled against the controller first so
    /// a stale membership (a tab closed or unpinned since the last command)
    /// never reaches the write. Read-only: the live store reconciles on its
    /// next command.
    pub fn persisted(&self, controller: &BrowserController) -> (Vec<Folder>, Vec<TabId>) {
        let snapshot = controller.snapshot();
        let mut store = self.store.clone();
        store.reconcile_with(|tab| tab_facts(&snapshot, tab));
        let folders = store.folders().cloned().collect();
        let favorites = store.favorites().map(|favorite| favorite.tab).collect();
        (folders, favorites)
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

    /// What deleting `space` would remove (the delete sheet's copy): its
    /// title, tab count, folder count and how many of its tabs are
    /// favorites. `None` for an unknown space.
    pub fn space_delete_summary(
        &mut self,
        controller: &BrowserController,
        space: SpaceId,
    ) -> Option<SpaceDeleteSummary> {
        self.reconcile(controller);
        let snapshot = controller.snapshot();
        let title = snapshot
            .spaces
            .iter()
            .find(|s| s.id == space)?
            .title
            .clone();
        let tabs: Vec<TabId> = snapshot
            .all_tabs
            .iter()
            .filter(|tab| tab.space == space)
            .map(|tab| tab.id)
            .collect();
        Some(SpaceDeleteSummary {
            title,
            tabs: tabs.len(),
            folders: self.store.folders().filter(|f| f.space == space).count(),
            favorites: self
                .store
                .favorites()
                .filter(|favorite| tabs.contains(&favorite.tab))
                .count(),
            deletable: controller.can_delete_space(space),
        })
    }

    /// `DeleteSpace` command (blueprint D05): delete `space` with its tabs
    /// and folders through the controller. Rejections (unknown space, the
    /// last space of a profile) change nothing, folders included.
    pub fn delete_space(
        &mut self,
        controller: &mut BrowserController,
        space: SpaceId,
    ) -> Result<OrganizationEvent, OrganizationError> {
        let event = controller
            .delete_space(space)
            .map_err(OrganizationError::Controller)?;
        let ControllerEvent::SpaceDeleted {
            space,
            fallback,
            closed_tabs,
            activated,
        } = event
        else {
            unreachable!("delete_space only emits SpaceDeleted, got {event:?}")
        };
        let removed_folders = self.store.remove_folders_in(space);
        self.reconcile(controller);
        self.check_invariants(controller)
            .expect("organization invariants hold after a space deletion");
        Ok(OrganizationEvent::SpaceDeleted {
            space,
            fallback,
            closed_tabs,
            removed_folders,
            activated,
        })
    }

    /// Blueprint D06: delete `profile` through the controller and drop the
    /// folders of its spaces (favorites of its tabs reconcile away).
    pub fn delete_profile(
        &mut self,
        controller: &mut BrowserController,
        profile: bbr_domain::ProfileId,
    ) -> Result<ControllerEvent, OrganizationError> {
        let spaces: Vec<SpaceId> = controller
            .snapshot()
            .spaces
            .iter()
            .filter(|space| space.profile == profile)
            .map(|space| space.id)
            .collect();
        let event = controller
            .delete_profile(profile)
            .map_err(OrganizationError::Controller)?;
        for space in spaces {
            self.store.remove_folders_in(space);
        }
        self.reconcile(controller);
        self.check_invariants(controller)
            .expect("organization invariants hold after a profile deletion");
        Ok(event)
    }

    /// `UnpackFolder` command (blueprint D25, the folder menu's "Delete
    /// Folder — keep its tabs"): the folder goes, every member stays a loose
    /// pinned tab; no tab closes and no session changes. Unknown folders are
    /// rejected whole.
    pub fn unpack_folder(
        &mut self,
        controller: &BrowserController,
        folder: bbr_domain::FolderId,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        let members = self
            .store
            .unpack_folder(folder)
            .map_err(OrganizationError::Folder)?;
        self.check_invariants(controller)
            .expect("organization invariants hold after an unpack");
        Ok(OrganizationEvent::FolderUnpacked {
            id: folder,
            members,
        })
    }

    /// `RenameFolder` command (Blueprint D20, folder menu Rename): give the
    /// session-local `folder` a new title. Empty titles and unknown folders
    /// are rejected whole; membership, pins and sessions are untouched.
    pub fn rename_folder(
        &mut self,
        folder: bbr_domain::FolderId,
        title: &str,
    ) -> Result<OrganizationEvent, OrganizationError> {
        let title = self
            .store
            .rename_folder(folder, title)
            .map_err(OrganizationError::Folder)?;
        Ok(OrganizationEvent::FolderRenamed { id: folder, title })
    }

    /// `MoveTabIntoFolder` command: group the active space's tab at `index`
    /// into `folder`, pinning a regular tab in the same transition. The tab
    /// keeps its identity, space membership, selection, and live session;
    /// membership in a previous folder is left. Unknown folders and tabs of
    /// another space are rejected whole.
    pub fn move_tab_into_folder(
        &mut self,
        controller: &mut BrowserController,
        index: usize,
        folder: bbr_domain::FolderId,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        let tab = active_space_tab(&controller.snapshot(), index)?;
        self.group(controller, tab, folder)
    }

    /// `MoveTabIntoFolder` keyed by the exact persistent [`TabId`] (the
    /// tab/favorite context-menu authority; task packet
    /// `tab-favorite-context-menus`): group the bound tab into `folder` of
    /// **its own space** — never the active space — pinning a regular tab in
    /// the same transition. Folders of another space and unknown tabs are
    /// rejected whole. The active space is never switched.
    pub fn move_tab_into_folder_id(
        &mut self,
        controller: &mut BrowserController,
        tab: TabId,
        folder: bbr_domain::FolderId,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        self.group(controller, tab, folder)
    }

    /// Ungroup the active space's tab at `index` from `folder`: the member
    /// stays pinned and becomes a loose pinned tab (the settled
    /// folder-delete semantics unpack through this transition). Removing a
    /// non-member is a no-op; an unknown folder is rejected.
    pub fn remove_tab_from_folder(
        &mut self,
        controller: &BrowserController,
        index: usize,
        folder: bbr_domain::FolderId,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        let tab = active_space_tab(&controller.snapshot(), index)?;
        self.store.remove_tab_from_folder(folder, tab)?;
        self.check_invariants(controller)
            .expect("organization invariants hold after reconcile");
        Ok(OrganizationEvent::TabRemovedFromFolder { tab, folder })
    }

    /// The one grouping transition: the domain store validates the tab and
    /// folder before any mutation, pins a regular tab in the browser, and
    /// records the membership; the controller then re-syncs its views.
    fn group(
        &mut self,
        controller: &mut BrowserController,
        tab: TabId,
        folder: bbr_domain::FolderId,
    ) -> Result<OrganizationEvent, OrganizationError> {
        let newly_pinned = self
            .store
            .move_tab_into_folder(&mut controller.browser, folder, tab)?;
        let pinned_index = if newly_pinned {
            controller.sync_views();
            controller.check_invariants();
            Some(tab_index(&controller.snapshot(), tab).expect("a grouped tab stays in its space"))
        } else {
            None
        };
        self.check_invariants(controller)
            .expect("organization invariants hold after reconcile");
        Ok(OrganizationEvent::TabMovedIntoFolder {
            tab,
            folder,
            pinned_index,
        })
    }

    /// `FavoriteToggle` command: toggle the active space's concrete tab at
    /// `index` in the window-global favorites grid. Toggling the same `TabId`
    /// removes it; re-adding appends it at the end.
    pub fn favorite_toggle(
        &mut self,
        controller: &mut BrowserController,
        index: usize,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        let snapshot = controller.snapshot();
        let tabs = active_space_tabs(&snapshot);
        let tab = *tabs.get(index).ok_or(OrganizationError::IndexOutOfBounds {
            index,
            len: tabs.len(),
        })?;
        self.toggle_favorite(controller, tab)
    }

    /// Whether `tab` is currently a favorite (the favorites-grid membership
    /// test the tab/favorite context-menu source resolution consumes; task
    /// packet `tab-favorite-context-menus`). Read-only.
    pub fn is_favorite(&self, tab: TabId) -> bool {
        self.store.favorites().any(|favorite| favorite.tab == tab)
    }

    /// Toggle a favorite by its exact persistent `TabId` from any current
    /// space (the tab/favorite context-menu authority; task packet
    /// `tab-favorite-context-menus`). The owning space/profile are derived
    /// from the controller read model — the active space is never switched,
    /// and the tab's space membership, selection and live session are
    /// untouched; only a regular tab's region changes (it is pinned).
    /// Unknown tabs reject whole.
    pub fn favorite_toggle_id(
        &mut self,
        controller: &mut BrowserController,
        tab: TabId,
    ) -> Result<OrganizationEvent, OrganizationError> {
        self.reconcile(controller);
        self.toggle_favorite(controller, tab)
    }

    /// The one favorite transition: the domain store validates the tab,
    /// pins a regular tab in the browser and leaves its folder when adding;
    /// the controller then re-syncs its views.
    fn toggle_favorite(
        &mut self,
        controller: &mut BrowserController,
        tab: TabId,
    ) -> Result<OrganizationEvent, OrganizationError> {
        let outcome = self.store.favorite_toggle(&mut controller.browser, tab)?;
        if outcome.newly_pinned {
            controller.sync_views();
            controller.check_invariants();
        }
        let snapshot = controller.snapshot();
        let global = snapshot
            .all_tabs
            .iter()
            .find(|candidate| candidate.id == tab)
            .expect("a toggled favorite keeps its tab");
        self.check_invariants(controller)
            .expect("organization invariants hold after a favorite toggle");
        Ok(OrganizationEvent::FavoriteToggled {
            added: outcome.added,
            tab,
            space: global.space,
            profile: global.profile,
            pinned_index: outcome.newly_pinned.then_some(global.index),
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
        self.store.reconcile_with(|tab| tab_facts(&snapshot, tab))
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
                    closed: tab.lifecycle == bbr_domain::TabLifecycle::Closed,
                    selected: tab.space == snapshot.active_space
                        && snapshot.active_index == Some(tab.index),
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
        let tab_facts = |tab: TabId| tab_facts(&snapshot, tab);
        self.store.check_invariants_with(space_exists, tab_facts)
    }
}
