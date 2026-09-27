//! U2 sidebar behavior (DESIGN.md §11.3/§11.4/§11.10/§11.11): the footer
//! controls, the plus context menu (keyboard selection, Escape/outside-press
//! dismissal, action dispatch through the L5 organization commands), the
//! favorites grid activation, and the folders disclosure/list behavior.
//! The permanent §11.3 space strip lives in `app::spaces`; there is no
//! revealable spaces region or toggle state.
//!
//! The render surfaces live in `app::u2_surface` (this module owns the app
//! mutations); the rail composition in `app::sidebar_view` wires the
//! listeners to the components.

use bbr_browser::{FolderId, OrganizationEvent};
use gpui::{Context, FocusHandle, KeyDownEvent, Window};

use crate::command_bar::CommandBarMode;

use super::BbrApp;

/// The default title the plus menu's New Space action creates (no naming
/// input exists in the menu; the command-bar NewSpace staging stays the
/// naming path).
pub const PLUS_NEW_SPACE_TITLE: &str = "New Space";
/// The default title the plus menu's New Folder action creates (session-local).
pub const PLUS_NEW_FOLDER_TITLE: &str = "New Folder";

/// App-local U2 presentation/focus state. Grouped here so the browser root
/// stays bounded while the domain/controller organization authority remains
/// separate and unchanged.
pub(super) struct U2State {
    pub plus_menu_handle: FocusHandle,
    pub folders_handle: FocusHandle,
    pub plus_menu: bbr_ui::components::menu::PlusMenuState,
    pub folder_presentation: bbr_ui::components::folders::FolderPresentation,
    /// The keyboard focus anchor: the focused folder's stable domain id
    /// (`None` before any row is focused — traversal starts at the first
    /// row).
    pub folder_focus: Option<FolderId>,
    pub plus_menu_press_inside: bool,
}

impl U2State {
    pub fn new(cx: &mut Context<BbrApp>) -> Self {
        Self {
            plus_menu_handle: cx.focus_handle(),
            folders_handle: cx.focus_handle(),
            plus_menu: bbr_ui::components::menu::PlusMenuState::default(),
            folder_presentation: bbr_ui::components::folders::FolderPresentation::default(),
            folder_focus: None,
            plus_menu_press_inside: false,
        }
    }
}

impl BbrApp {
    /// §11.4 footer `[Downloads]` control: toggles the §11.12 anchored
    /// downloads panel (pointer or keyboard activation; a second activation
    /// dismisses it). The panel is the Phase-8 surface; the controller and
    /// native adapter are installed by the composition root.
    #[cfg(feature = "testkit")]
    pub fn activate_downloads(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        crate::app::downloads_behavior::toggle_downloads(self, window, cx);
    }

    /// §11.4 footer `[+]` control: open the plus menu, or close it on a
    /// second activation. The pointer path keeps the chrome focus domain;
    /// the menu owns keyboard focus while open.
    pub fn toggle_plus_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.u2.plus_menu.open {
            self.close_plus_menu(window, cx);
        } else {
            self.open_plus_menu(window, cx);
        }
    }

    /// Open the plus menu: reset the keyboard selection, focus the menu
    /// surface, and snap the open state. The Tab/Favorite menu (task packet
    /// `tab-favorite-context-menus`) is discarded — exactly one sidebar
    /// chrome menu may own the keyboard.
    pub fn open_plus_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tab_menu_discard(cx);
        self.u2.plus_menu.open = true;
        self.u2.plus_menu.selection = 0;
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        self.command_ranking.close();
        // At most one sidebar popover at a time: opening the plus menu
        // dismisses the §11.3 configuration surface and the §11.12
        // downloads panel (the config draft is not applied).
        if self.space_config.open {
            self.space_config.open = false;
            self.space_config.space = None;
            self.space_config.press_inside = false;
        }
        crate::app::downloads_behavior::downloads_close(self, window, cx);
        window.focus(&self.u2.plus_menu_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Close the plus menu without dispatching an action: the invoking
    /// focus domain (chrome) is restored per §11.4.
    pub fn close_plus_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.u2.plus_menu.open {
            return;
        }
        self.u2.plus_menu.open = false;
        self.focus_chrome(window, cx);
    }

    /// §11.4 plus-menu keyboard operation: `↑`/`↓` move the selection
    /// (wrapping within the two v1 items), Enter commits the selected item,
    /// Escape dismisses without action. The menu stops propagation so the
    /// app-wide Escape binding (FocusWeb) never fires while the menu is open.
    pub(super) fn on_plus_menu_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.u2.plus_menu.open {
            return;
        }
        match event.keystroke.key.as_str() {
            "up" => {
                self.u2.plus_menu.move_selection(-1);
                cx.stop_propagation();
            }
            "down" => {
                self.u2.plus_menu.move_selection(1);
                cx.stop_propagation();
            }
            "enter" => {
                cx.stop_propagation();
                self.commit_plus_menu(window, cx);
            }
            "escape" => {
                cx.stop_propagation();
                self.close_plus_menu(window, cx);
            }
            _ => {}
        }
        cx.notify();
    }

    /// §11.4: a pointer press outside the menu surface dismisses it (one
    /// pointer action). The menu surface marks presses inside via
    /// `plus_menu_press_inside` (its handler runs first in the bubble
    /// phase); any other press — a footer control, a tab row, the divider,
    /// the web content — closes the menu. The flag is consumed here so the
    /// next press starts clean.
    pub fn dismiss_plus_menu_on_outside_press(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inside = self.u2.plus_menu_press_inside;
        self.u2.plus_menu_press_inside = false;
        if self.u2.plus_menu.open && !inside {
            self.close_plus_menu(window, cx);
        }
    }

    /// §11.4 plus-menu item commit at `index` (the pointer path): select the
    /// item (clamped to the v1 range) and dispatch it. The E2E gateway and
    /// the render item listener share this dispatch seam.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // testkit gateway seam
    pub fn commit_plus_menu_item(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.u2.plus_menu.open {
            return;
        }
        self.u2.plus_menu.selection = index;
        self.commit_plus_menu(window, cx);
    }

    /// Commit the selected plus-menu item: New Space creates a real space in
    /// the active profile through the L5 command, New Folder creates a
    /// session-local folder in the active space. The menu closes and the
    /// chrome focus domain is restored.
    pub fn commit_plus_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let selected = self.u2.plus_menu.selected_item();
        match selected.id {
            bbr_ui::components::menu::PLUS_NEW_SPACE_ID => {
                match self
                    .organization
                    .new_space(&mut self.controller, PLUS_NEW_SPACE_TITLE)
                {
                    Ok(OrganizationEvent::SpaceCreated { profile, space }) => {
                        // The creation is a semantic browser mutation and
                        // must be observable like every other path (the
                        // command-bar NewSpace emits SpaceCreated too).
                        self.emit_browser(&bbr_browser::ControllerEvent::SpaceCreated {
                            profile,
                            space,
                        });
                        self.sync_command_bar();
                        self.close_plus_menu(window, cx);
                        // §11.3 create→activate→configure: the new Space
                        // becomes active immediately (the standing
                        // daily-driver decision) and its optional name/icon
                        // configuration surface opens on top of the usable
                        // Space; Escape/cancel keeps it usable.
                        self.activate_space(space, window, cx);
                        self.open_space_config(space, window, cx);
                        self.persist_structure();
                        cx.notify();
                    }
                    Ok(_) => unreachable!("new_space only emits SpaceCreated"),
                    Err(error) => log::error!("could not create space: {error}"),
                }
            }
            bbr_ui::components::menu::PLUS_NEW_FOLDER_ID => {
                match self
                    .organization
                    .new_folder(&self.controller, PLUS_NEW_FOLDER_TITLE)
                {
                    Ok(OrganizationEvent::FolderCreated { .. }) => cx.notify(),
                    Ok(_) => unreachable!("new_folder only emits FolderCreated"),
                    Err(error) => log::error!("could not create folder: {error}"),
                }
            }
            other => log::error!("unknown plus-menu item: {other}"),
        }
        self.close_plus_menu(window, cx);
    }

    /// §11.10 favorites tile activation: a favorite is an additional
    /// presentation of one concrete `TabId` (DESIGN.md §11.10 data source),
    /// never a URL shortcut. Activation routes through the corrected L5
    /// command: the owning space — and therefore its profile/container — is
    /// activated inside the same browser window, then that exact tab is
    /// selected by its current space-local index. The tab is neither copied,
    /// moved, nor recreated; its warm session stays warm.
    pub fn activate_favorite(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let result = self
            .organization
            .activate_favorite(&mut self.controller, index);
        match result {
            Ok(OrganizationEvent::FavoriteActivated {
                tab,
                space,
                profile,
                index: tab_index,
            }) => {
                // The controller already switched the active space and
                // selected the exact tab. Mirror both mutations through the
                // same observation path every other space/tab mutation uses
                // (the testkit browser sink), then refresh the app
                // presentation: command-bar text, the §11.3 hint (cleared —
                // a favorite activation is a tab mutation, and the tile's
                // own semantic name carries the owner context), the visible
                // selection, and chrome focus in the tab list.
                self.emit_browser(&bbr_browser::ControllerEvent::SpaceActivated { space });
                self.emit_browser(&bbr_browser::ControllerEvent::TabSelected { index: tab_index });
                self.clear_command_profile_hint();
                self.sync_command_bar();
                self.ensure_active_visible();
                self.focus_tab_list(window, cx);
                log::debug!(
                    "favorite {index} activated: tab {tab:?} in space {space:?} (profile {profile:?}) at index {tab_index}"
                );
                cx.notify();
            }
            Ok(other) => {
                unreachable!("activate_favorite only emits FavoriteActivated, got {other:?}")
            }
            Err(error) => log::error!("could not activate favorite {index}: {error}"),
        }
    }

    /// §11.10 favorites add/remove path: toggle the active space's concrete
    /// tab at `index` in the window-global favorites grid through the L5
    /// command. Toggling never changes the tab's space membership, order,
    /// selection, or live session (interaction contract). The drag-into-
    /// grid drop commits through this path.
    pub fn favorite_toggle_tab(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let result = self.organization.favorite_toggle(&self.controller, index);
        match result {
            Ok(OrganizationEvent::FavoriteToggled {
                added,
                tab,
                space,
                profile,
            }) => {
                log::debug!(
                    "favorite {}: tab {tab:?} of space {space:?} (profile {profile:?})",
                    if added { "added" } else { "removed" }
                );
                self.focus_tab_list(window, cx);
                cx.notify();
            }
            Ok(other) => unreachable!("favorite_toggle only emits FavoriteToggled, got {other:?}"),
            Err(error) => log::error!("could not toggle favorite at {index}: {error}"),
        }
    }

    /// §11.11 folder disclosure: expand/collapse the folder with domain id
    /// `id`. Toggling snaps (`dur:instant`); the presentation state is
    /// app-local (the domain model is L5-owned).
    pub fn toggle_folder(&mut self, id: FolderId, cx: &mut Context<Self>) {
        self.u2.folder_presentation.toggle(id);
        cx.notify();
    }

    /// Pointer entry into a folder row: focus the folders region with that
    /// row's folder id as the keyboard anchor (§11.11 pointer or keyboard
    /// Enter/Space toggling).
    pub fn focus_folder_row(&mut self, id: FolderId, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        self.command_ranking.close();
        self.u2.folder_focus = Some(id);
        window.focus(&self.u2.folders_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// §11.11 folders-region keyboard operation: `↑`/`↓` move the focused
    /// folder row (wrapping), Enter/Space toggle it. Runs only while the
    /// folders region owns chrome focus and no command mode is active. The
    /// anchor is a `FolderId`; traversal moves through the snapshot's
    /// folder list by position and resolves back to the id (a stale anchor
    /// for a deleted folder falls back to the first row).
    pub(super) fn on_folders_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.command_bar_mode.is_active() {
            return;
        }
        let organization = self.organization.organization_snapshot(&self.controller);
        let count = organization.folders.len();
        if count == 0 {
            return;
        }
        let focused_position = self
            .u2
            .folder_focus
            .and_then(|id| organization.folders.iter().position(|f| f.id == id))
            .unwrap_or(0);
        match event.keystroke.key.as_str() {
            "up" | "down" => {
                let delta = if event.keystroke.key == "up" { -1 } else { 1 };
                let next = (focused_position as isize + delta).rem_euclid(count as isize) as usize;
                self.u2.folder_focus = Some(organization.folders[next].id);
                cx.stop_propagation();
            }
            "enter" | "space" => {
                let id = organization.folders[focused_position.min(count - 1)].id;
                self.u2.folder_presentation.toggle(id);
                cx.stop_propagation();
            }
            _ => {}
        }
        let _ = window;
        cx.notify();
    }

    /// §11.11 nested row activation: select the member tab (the folder's
    /// members are active-space tabs; the member's space-local index is the
    /// production selection path). The folder is resolved by its stable
    /// domain id.
    pub fn select_folder_member(
        &mut self,
        folder: FolderId,
        member: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.folder_member_tab_index(folder, member) else {
            return;
        };
        if self.select_tab(index, cx).is_some() {
            cx.notify();
        }
        let _ = window;
    }

    /// §11.11 nested row close: close the member tab (the folder keeps its
    /// row; the L5 reconcile drops the stale membership). The folder is
    /// resolved by its stable domain id.
    pub fn close_folder_member(
        &mut self,
        folder: FolderId,
        member: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.folder_member_tab_index(folder, member) else {
            return;
        };
        if self.close_tab(index, cx).is_some() {
            cx.notify();
        }
        let _ = window;
    }

    /// §11.11 move-into-folder command path: route the active space's tab at
    /// `index` into the session-local folder with domain id `folder` through
    /// the L5 `MoveIntoFolder` command. A folder is the grouped form of
    /// pinned tabs, so a regular tab is pinned by the same transition; its
    /// identity, space membership, selection, and live session are
    /// untouched.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // testkit gateway seam
    pub fn move_tab_into_folder(
        &mut self,
        index: usize,
        folder: FolderId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match self
            .organization
            .move_tab_into_folder(&mut self.controller, index, folder)
        {
            Ok(event) => {
                self.apply_grouping(&event, cx);
                self.focus_tab_list(window, cx);
            }
            Err(error) => log::error!("could not move tab into folder: {error}"),
        }
    }

    /// Ungroup the active space's tab at `index` from `folder`: the member
    /// stays pinned as a loose pinned tab (the transition the settled
    /// Unpack/delete semantics consume). No user entry exists yet; the
    /// folder menu packet adds it.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // testkit gateway seam
    pub fn remove_tab_from_folder(
        &mut self,
        index: usize,
        folder: FolderId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match self
            .organization
            .remove_tab_from_folder(&self.controller, index, folder)
        {
            Ok(event) => {
                log::debug!("{event:?}");
                self.focus_tab_list(window, cx);
                cx.notify();
            }
            Err(error) => log::error!("could not remove tab from folder: {error}"),
        }
    }

    /// Mirror a grouping outcome. A regular tab pinned by the grouping
    /// transition is a structural browser mutation: it is observable like
    /// every other pin (`TabPinned`) and must survive a force quit.
    pub(super) fn apply_grouping(&mut self, event: &OrganizationEvent, cx: &mut Context<Self>) {
        match event {
            OrganizationEvent::TabMovedIntoFolder {
                tab,
                folder,
                pinned_index,
            } => {
                log::debug!("tab {tab:?} moved into folder {folder:?}");
                if let Some(index) = pinned_index {
                    self.clear_command_profile_hint();
                    self.sync_command_bar();
                    self.emit_browser(&bbr_browser::ControllerEvent::TabPinned { index: *index });
                    self.persist_structure();
                }
                cx.notify();
            }
            other => unreachable!("grouping only emits TabMovedIntoFolder, got {other:?}"),
        }
    }

    /// Resolve a folder's member position to the active-space tab index via
    /// the L5 read model (membership reconciles on read, so the member
    /// always exists). The folder is resolved by its stable domain id.
    fn folder_member_tab_index(&mut self, folder: FolderId, member: usize) -> Option<usize> {
        let organization = self.organization.organization_snapshot(&self.controller);
        let folder_snapshot = organization.folders.iter().find(|f| f.id == folder)?;
        let tab_id = *folder_snapshot.members.get(member)?;
        self.controller
            .snapshot()
            .all_tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .map(|tab| tab.index)
    }
}
