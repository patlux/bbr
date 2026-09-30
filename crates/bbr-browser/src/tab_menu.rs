//! Tab/Favorite context-menu authority (task packet
//! `tab-favorite-context-menus`): the read-only [`TabMenuSource`] resolution
//! and availability matrix keyed by the exact persistent [`TabId`], plus
//! exactly-one-typed-command dispatch against that bound id.
//!
//! # Integration contract (lane Z1 → DD-TAB-MENU)
//!
//! [`TabMenuController`] is a **portable companion** next to
//! [`BrowserController`], like [`crate::organization::OrganizationController`]:
//! it owns no state, resolves a source without mutating, and routes each
//! enabled action through the existing domain/controller authority. It never
//! switches the active space and never touches selection — a menu built from
//! [`TabMenuSource::resolve`] can open, traverse, and dismiss while the
//! complete controller snapshot and session registry stay unchanged.
//!
//! - [`TabMenuController::resolve`] is read-only: the source carries the
//!   tab's current owning `SpaceId`/`ProfileId`, space-local index, pinned
//!   region, lifecycle, visibility, favorite state, live session, and the
//!   region math the ordering rows need.
//! - [`TabMenuController::availability`] is a pure function of the source
//!   (plus the controller's space read model for the `Move to Space`
//!   destinations and the caller's per-space folder rows for `Move to
//!   Folder`). Rows whose typed command authority does not exist yet
//!   (Duplicate Tab, native Share, split/media/rename/icon/bulk) are omitted
//!   entirely — never decorative dead rows.
//! - [`TabMenuController::dispatch`] re-resolves the bound [`TabId`] against
//!   the **live** controller state at commit (never a stale menu snapshot)
//!   and dispatches exactly one typed command, returning the controller's
//!   typed event. Favorite toggling routes through
//!   [`crate::organization::OrganizationController::favorite_toggle_id`]
//!   (the favorites store is organization-owned); bookmark rows route
//!   through the app-owned B2 `SavedPageCommand` and never retain the
//!   `TabId` in the Saved Page.
//!
//! The renderer sees only a presentation DTO (the app maps this matrix onto
//! `bbr-ui::surfaces::tab_menu`); this module never reaches into `bbr-ui`.

use bbr_domain::{SpaceId, TabId, TabLifecycle};
use bbr_web::{ProfileId, WebAction, WebViewId};

use crate::outcome::ControllerEvent;
use crate::{BrowserController, ControllerError};

/// One typed tab-menu action whose command authority exists today. Rows that
/// lack an authority (Duplicate Tab, native Share, split/media/rename/icon/
/// bulk) are omitted by the app's row builder — never faked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabMenuAction {
    /// Route `WebAction::Reload` to the bound tab's live session.
    Reload,
    /// Pin a regular source, unpin a pinned source (the row label follows
    /// the source's pinned state).
    TogglePin,
    /// Move a hidden warm source through the explicit resource lifecycle.
    Suspend,
    /// Copy the bound tab's current link (the engine owns the pasteboard).
    CopyLink,
    /// Move the source to the start of its pinned/regular region.
    MoveToStart,
    /// Move the source to the end of its pinned/regular region.
    MoveToEnd,
    /// Close the bound tab in its owning space (session destroyed).
    Close,
    /// Reopen the owning space's most recently closed tab (the menu row is
    /// legal only when the space has a recently-closed tab; the reopened
    /// identity is a fresh session in that same space).
    ReopenTab,
    /// Move the bound tab into `space` (same-profile keeps the live
    /// `WebViewId`; cross-profile recreates exactly once).
    MoveToSpace(SpaceId),
}

/// One `Move to Space` destination row: the destination space identity and
/// the visible profile identity for cross-profile destinations (DESIGN.md
/// §11.3 cross-context communication).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceDestination {
    pub space: SpaceId,
    pub title: String,
    /// The owning profile's title; `Some` only for cross-profile
    /// destinations (the profile identity is visible there).
    pub profile_title: Option<String>,
}

/// One `Move to Folder` destination row: the session-local folder identity
/// and its title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderDestination {
    pub folder: bbr_domain::FolderId,
    pub title: String,
}

/// The read-only resolution of one persistent [`TabId`] — the availability
/// matrix input. Never mutates the controller; opening, traversing, and
/// dismissing a menu built from this source leave the complete snapshot and
/// session registry unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabMenuSource {
    /// The exact persistent tab the menu is bound to.
    pub tab: TabId,
    /// Current owning space of `tab` (changes when the tab moves).
    pub space: SpaceId,
    /// Derived from `space`; identifies the isolated container.
    pub profile: ProfileId,
    /// Space-local index of the source tab.
    pub index: usize,
    pub pinned: bool,
    pub lifecycle: TabLifecycle,
    /// The tab is the active single-pane selection or a split member of its
    /// owning space (the domain's visibility rule).
    pub visible: bool,
    /// The tab is currently presented in the favorites grid.
    pub favorite: bool,
    pub url: String,
    pub title: String,
    /// The live disposable session (absent when suspended/discarded).
    pub web_view: Option<WebViewId>,
    /// The owning space has at least one recently-closed tab (the Reopen
    /// legality).
    pub reopen_legal: bool,
    /// The owning space's pinned count (the ordering region math).
    pub pinned_count: usize,
    /// The owning space's tab count.
    pub tab_count: usize,
}

/// The availability matrix for one source: which rows exist and their
/// enablement, plus the deterministic submenu destinations. Rows whose
/// authority does not exist are omitted entirely by the caller's builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabMenuAvailability {
    /// `Reload Tab` — enabled when the source owns a live session.
    pub reload: bool,
    /// The source is pinned (the row reads `Unpin Tab`; false → `Pin Tab`).
    pub pinned: bool,
    /// `Suspend Tab` — enabled only for a hidden warm tab (the domain
    /// rejects a visible → Suspended transition).
    pub suspend: bool,
    /// `Copy Link` — enabled only when the bound tab has a live
    /// dispatchable session (the engine owns the pasteboard, so a
    /// suspended/discarded source cannot copy).
    pub copy_link: bool,
    /// `Move to Start` / `Move to End` within the source's region.
    pub move_to_start: bool,
    pub move_to_end: bool,
    /// The source is a favorite (the row reads `Remove from Favorites`).
    pub favorite: bool,
    /// `Close Tab` — enabled unless the source is already a closed pinned
    /// tab (DESIGN.md §10.12).
    pub close: bool,
    /// `Reopen Closed Tab` — legal only when the source's space has a
    /// recently-closed tab.
    pub reopen: bool,
    /// `Move to Folder` submenu rows (empty → the group is omitted). The
    /// caller supplies the source space's folders; membership is per-space.
    pub folders: Vec<FolderDestination>,
    /// `Move to Space` submenu rows (empty → the group is omitted). Every
    /// other space; cross-profile destinations carry the visible profile
    /// identity.
    pub spaces: Vec<SpaceDestination>,
}

/// Errors from the tab-menu command dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabMenuError {
    /// The bound [`TabId`] no longer exists in the controller read model.
    UnknownTab(TabId),
    /// The source's owning space has no recently-closed tab; the Reopen
    /// row is not legal for this source.
    ReopenNotLegal,
    /// The delegate controller/domain mutation failed.
    Controller(ControllerError),
}

impl std::fmt::Display for TabMenuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTab(tab) => write!(f, "tab menu source {tab:?} no longer exists"),
            Self::ReopenNotLegal => write!(f, "no recently-closed tab in the source space"),
            Self::Controller(e) => write!(f, "tab menu command failed: {e}"),
        }
    }
}

impl std::error::Error for TabMenuError {}

impl From<ControllerError> for TabMenuError {
    fn from(e: ControllerError) -> Self {
        Self::Controller(e)
    }
}

impl From<bbr_domain::DomainError> for TabMenuError {
    fn from(e: bbr_domain::DomainError) -> Self {
        Self::Controller(ControllerError::Domain(e))
    }
}

/// Portable companion for the tab/favorite context menu: read-only source
/// resolution + availability, and exactly-one-typed-command dispatch keyed
/// by the bound [`TabId`]. Owns no state; constructed as a unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TabMenuController;

impl TabMenuController {
    /// Resolve the exact persistent `tab` to its current presentation
    /// context. `is_favorite` reports favorites-grid membership (the
    /// session-local organization store owns it; the app passes a closure
    /// over its [`crate::organization::OrganizationController`]). `None`
    /// when the tab no longer exists.
    pub fn resolve(
        controller: &BrowserController,
        is_favorite: impl Fn(TabId) -> bool,
        tab: TabId,
    ) -> Option<TabMenuSource> {
        let state = controller.browser.tab(tab)?;
        let space = controller.browser.space(state.space)?;
        let index = space.tabs.iter().position(|t| *t == tab)?;
        let profile = controller.browser.profile(space.profile)?.id;
        let web_view = controller.registry.view_for_tab(tab);
        Some(TabMenuSource {
            tab,
            space: space.id,
            profile,
            index,
            pinned: state.pinned,
            lifecycle: state.lifecycle,
            visible: controller.browser.is_visible(tab),
            favorite: is_favorite(tab),
            url: state.url.clone(),
            title: state.title.clone(),
            web_view,
            reopen_legal: !space.recently_closed.is_empty(),
            pinned_count: space.pinned_count,
            tab_count: space.tabs.len(),
        })
    }

    /// The availability matrix for one resolved source — a pure read over
    /// the source plus the controller's space collection (for the `Move to
    /// Space` destinations). `folders` are the source space's session-local
    /// folder rows the caller derived from the organization read model.
    /// Never mutates. The caller's row builder adds the B2 bookmark row
    /// separately (the Saved Page authority is app-owned).
    pub fn availability(
        controller: &BrowserController,
        source: &TabMenuSource,
        folders: &[FolderDestination],
    ) -> TabMenuAvailability {
        let reload = source.web_view.is_some();
        // Copy Link routes `WebAction::CopyLink` to the bound tab's live
        // session; a session-less (suspended/discarded) source cannot
        // dispatch, so the row is disabled there.
        let copy_link = source.web_view.is_some();
        // Region math: the pinned region is `[0, pinned_count)`; the
        // regular region `[pinned_count, tab_count)`. Move to Start/End are
        // no-ops at the region edges and are disabled there.
        let region_end = if source.pinned {
            source.pinned_count.saturating_sub(1)
        } else {
            source.tab_count.saturating_sub(1)
        };
        let move_to_start = source.index != 0;
        let move_to_end = source.index != region_end;
        let suspend = !source.visible && matches!(source.lifecycle, TabLifecycle::Warm);
        let spaces = other_spaces(controller, source);
        TabMenuAvailability {
            reload,
            pinned: source.pinned,
            suspend,
            copy_link,
            move_to_start,
            move_to_end,
            favorite: source.favorite,
            close: source.lifecycle != TabLifecycle::Closed,
            reopen: source.reopen_legal,
            folders: folders.to_vec(),
            spaces,
        }
    }

    /// Dispatch exactly one typed command against the bound `tab`, resolved
    /// against the **live** controller state at commit (never a stale menu
    /// snapshot), so the source's current location always wins. The active
    /// space and selection are never changed: every mutation operates in the
    /// source's owning space (same-profile moves keep the live `WebViewId`;
    /// cross-profile moves recreate exactly once in the destination store,
    /// per the shared move authority).
    ///
    /// Favorite toggling and bookmark rows are app/organization-owned and
    /// are not dispatched here.
    pub fn dispatch(
        controller: &mut BrowserController,
        tab: TabId,
        action: TabMenuAction,
    ) -> Result<ControllerEvent, TabMenuError> {
        let source =
            Self::resolve(controller, |_| false, tab).ok_or(TabMenuError::UnknownTab(tab))?;
        match action {
            TabMenuAction::Reload => {
                let view = controller
                    .registry
                    .view_for_tab(tab)
                    .ok_or(ControllerError::NoSession(tab))?;
                controller
                    .engine
                    .borrow_mut()
                    .action(view, WebAction::Reload);
                controller.check_invariants();
                Ok(ControllerEvent::NavigationRouted { view })
            }
            TabMenuAction::CopyLink => {
                let view = controller
                    .registry
                    .view_for_tab(tab)
                    .ok_or(ControllerError::NoSession(tab))?;
                controller
                    .engine
                    .borrow_mut()
                    .action(view, WebAction::CopyLink);
                controller.check_invariants();
                Ok(ControllerEvent::NavigationRouted { view })
            }
            TabMenuAction::TogglePin => {
                let space = source.space;
                if source.pinned {
                    controller.browser.unpin_tab(space, tab)?;
                } else {
                    controller.browser.pin_tab(space, tab)?;
                }
                controller.sync_views();
                controller.check_invariants();
                let index = current_index(controller, tab);
                Ok(if source.pinned {
                    ControllerEvent::TabUnpinned { index }
                } else {
                    ControllerEvent::TabPinned { index }
                })
            }
            TabMenuAction::Suspend => {
                controller
                    .browser
                    .transition_tab_lifecycle(tab, TabLifecycle::Suspended)?;
                controller.sync_views();
                controller.check_invariants();
                Ok(ControllerEvent::TabSuspended {
                    index: current_index(controller, tab),
                })
            }
            TabMenuAction::MoveToStart | TabMenuAction::MoveToEnd => {
                let region_end = if source.pinned {
                    source.pinned_count.saturating_sub(1)
                } else {
                    source.tab_count.saturating_sub(1)
                };
                let to = match action {
                    TabMenuAction::MoveToStart => 0,
                    _ => region_end,
                };
                if source.index != to {
                    controller
                        .browser
                        .reorder_tab(source.space, source.index, to)?;
                    controller.sync_views();
                    controller.check_invariants();
                }
                Ok(ControllerEvent::TabReordered {
                    from: source.index,
                    to,
                })
            }
            TabMenuAction::Close => {
                controller.destroy_session_of(tab);
                controller.provenance.invalidate_for(tab);
                controller.browser.close_tab(source.space, tab)?;
                controller.sync_views();
                controller.check_invariants();
                Ok(ControllerEvent::TabClosed {
                    index: source.index,
                })
            }
            TabMenuAction::ReopenTab => {
                if !source.reopen_legal {
                    return Err(TabMenuError::ReopenNotLegal);
                }
                let reopened = controller.browser.reopen_tab(source.space)?;
                controller.close_preview_if_stale();
                controller.sync_views();
                controller.check_invariants();
                let index = controller
                    .browser
                    .space(source.space)
                    .and_then(|space| space.tabs.iter().position(|t| *t == reopened))
                    .unwrap_or(0);
                let url = controller
                    .browser
                    .tab(reopened)
                    .map(|t| t.url.clone())
                    .unwrap_or_default();
                Ok(ControllerEvent::TabReopened { index, url })
            }
            TabMenuAction::MoveToSpace(to_space) => {
                let from_profile = source.profile;
                let to_profile = controller
                    .browser
                    .space(to_space)
                    .ok_or(ControllerError::UnknownSpace(to_space))?
                    .profile;
                controller.browser.move_tab(tab, to_space)?;
                controller.sync_views();
                controller.check_invariants();
                Ok(ControllerEvent::TabMoved {
                    tab,
                    to_space,
                    recreated: from_profile != to_profile,
                })
            }
        }
    }
}

/// The current space-local index of `tab` in its owning space (0 when gone).
fn current_index(controller: &BrowserController, tab: TabId) -> usize {
    controller
        .browser
        .tab(tab)
        .and_then(|state| controller.browser.space(state.space))
        .and_then(|space| space.tabs.iter().position(|t| *t == tab))
        .unwrap_or(0)
}

/// The `Move to Space` destinations: every space except the source's own, in
/// canonical creation order. Cross-profile destinations carry the visible
/// profile identity (DESIGN.md §11.3).
fn other_spaces(controller: &BrowserController, source: &TabMenuSource) -> Vec<SpaceDestination> {
    controller
        .browser
        .spaces()
        .filter(|space| space.id != source.space)
        .map(|space| SpaceDestination {
            space: space.id,
            title: space.title.clone(),
            profile_title: (space.profile != source.profile)
                .then(|| {
                    controller
                        .browser
                        .profile(space.profile)
                        .map(|p| p.title.clone())
                })
                .flatten(),
        })
        .collect()
}

#[cfg(test)]
mod tests;
