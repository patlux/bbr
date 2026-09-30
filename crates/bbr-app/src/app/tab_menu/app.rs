//! App-side integration of the Tab/Favorite context menu (task packet
//! `tab-favorite-context-menus`).
//!
//! The `BbrApp` surface of the menu controller: the exact production methods
//! the sidebar row/favorite right-click listeners, the menu card's row
//! element listeners, the root key-down listener, the outside-press
//! dismissal, and the testkit seams call. The deterministic row model, the
//! placement geometry, and the presentation state live in the parent module
//! (`super::TabMenuState`, `build_rows`, `tab_menu_placement`); this module
//! owns only the wiring between the app and that domain.
//!
//! Opening, traversing, hovering, and dismissing never select, activate,
//! move, reload, close, recreate, or change the profile/space/session of the
//! source tab. Every enabled action dispatches exactly one typed command
//! against the bound `TabId` through the shared portable authority (see the
//! parent module doc); disabled and omitted actions cannot dispatch.

use bbr_browser::TabMenuAction as BrowserTabMenuAction;
use bbr_domain::TabId;
use gpui::{Context, Window};

use crate::app::BbrApp;

#[cfg(feature = "testkit")]
use super::TabMenuRow;
use super::{
    build_rows, source_folders, tab_menu_placement, TabMenuAction, TabMenuState,
    FAVORITE_MENU_SOURCE, TAB_MENU_SOURCE,
};

impl BbrApp {
    /// Open the Tab/Favorite menu bound to the tab row at `index` of the
    /// active space — the exact app method the row's secondary-button
    /// mouse-down listener calls (§11.13). The pointer position anchors the
    /// menu; opening never selects or mutates the source. A right-click on
    /// another row while the menu is open re-binds it to that row.
    pub fn open_tab_menu_from_row(
        &mut self,
        index: usize,
        event: &gpui::MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        let Some(tab) = self.active_space_tab_id(index) else {
            return;
        };
        self.open_tab_menu(tab, TAB_MENU_SOURCE, event, window, cx);
    }

    /// Open the Tab/Favorite menu bound to the Favorite tile at `index` —
    /// the exact app method the tile's secondary-button mouse-down listener
    /// calls. A Favorite is another presentation of the same concrete tab,
    /// so this consumes the same authority with the Favorite source key;
    /// opening never activates the tile.
    pub fn open_tab_menu_from_favorite(
        &mut self,
        index: usize,
        event: &gpui::MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        let Some(tab) = self
            .organization
            .bookkeep()
            .organization_snapshot(&self.controller.read())
            .favorites
            .get(index)
            .map(|favorite| favorite.tab)
        else {
            return;
        };
        self.open_tab_menu(tab, FAVORITE_MENU_SOURCE, event, window, cx);
    }

    /// The shared open path: resolve the exact persistent `TabId` to its
    /// current source, compute the availability matrix (with the source
    /// space's folder rows and the app-owned B2 bookmark state), build the
    /// deterministic row model, resolve the clamped placement at the pointer,
    /// focus the menu, and snap the open state. Exactly one menu
    /// presentation per window: an open page context menu, find bar, or
    /// command surface is discarded.
    fn open_tab_menu(
        &mut self,
        tab: TabId,
        source_key: &str,
        event: &gpui::MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let point = [f32::from(event.position.x), f32::from(event.position.y)];
        let Some(source) = bbr_browser::TabMenuController::resolve(
            &self.controller.read(),
            |candidate| self.organization.read().is_favorite(candidate),
            tab,
        ) else {
            log::debug!("tab menu requested for an unknown tab {tab:?}; ignored");
            return;
        };
        let folders = source_folders(
            &mut self.organization.get(),
            &self.controller.read(),
            source.space,
        );
        let availability = bbr_browser::TabMenuController::availability(
            &self.controller.read(),
            &source,
            &folders,
        );
        let bookmark_save = {
            let command = self.saved_pages;
            command.can_save(self, &source.url)
        };
        let bookmark_remove = {
            let command = self.saved_pages;
            command.can_remove(self, &source.url)
        };
        let rows = build_rows(&source, &availability, bookmark_save, bookmark_remove);
        let window_size = [
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        ];
        let theme = self.theme();
        let (anchor, _) = tab_menu_placement(point, &rows, None, window_size, &theme);
        // Exactly one menu presentation per window (the settled mutual
        // exclusion of the page menu / find / command surfaces).
        self.menu_discard(cx);
        self.find_discard(cx);
        self.dismiss_command_portal(cx);
        let mut state = TabMenuState {
            open: true,
            focused: false,
            bound_tab: Some(tab),
            bound_space: None,
            bound_folder: None,
            source_key: source_key.into(),
            selected: None,
            open_submenu: None,
            submenu_selected: None,
            rows,
            anchor,
            submenu_rect: None,
            origin_chrome_focused: self.chrome_focused(window),
            press_inside: false,
            pending_submenu_close: false,
        };
        state.selected = state.first_enabled();
        self.tab_menu.state = state;
        self.tab_menu_focus(window, cx);
        cx.notify();
    }

    /// Focus the menu (main-window GPUI focus; the menu is ordinary chrome —
    /// never a portal surface).
    fn tab_menu_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tab_menu.state.focused = true;
        window.focus(&self.menu_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Pointer-hover path into the open menu: the exact app method the
    /// top-level row's `on_hover` element listener calls (the keyboard
    /// cursor and the pointer hover share one selection). Entering a
    /// submenu-bearing row opens its submenu; entering any other row closes
    /// an open submenu. Leaving the open submenu's parent row defers the
    /// close until another row is entered, so the pointer crossing into the
    /// submenu card never flickers it. The open submenu's resolved
    /// placement is recomputed from the live window geometry whenever it
    /// opens or the relevant selection changes. No-op when the menu is
    /// closed.
    pub fn tab_menu_select(
        &mut self,
        index: usize,
        hovered: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_menu.state.open {
            return;
        }
        if hovered {
            if self
                .tab_menu
                .state
                .row_at(index)
                .is_some_and(|row| !row.enabled)
            {
                // A disabled row is visible but never becomes the shared
                // keyboard/pointer cursor (the cursor always lands on an
                // enabled row).
                cx.notify();
                return;
            }
            if self.tab_menu.state.open_submenu.is_some()
                && self.tab_menu.state.open_submenu != Some(index)
            {
                self.tab_menu.state.close_submenu();
            }
            self.tab_menu.state.pending_submenu_close = false;
            self.tab_menu.state.selected = Some(index);
            if self
                .tab_menu
                .state
                .row_at(index)
                .is_some_and(|row| row.submenu.is_some())
            {
                self.tab_menu.state.open_submenu_at(index);
            }
            self.tab_menu_refresh_submenu_rect(window);
        } else if self.tab_menu.state.open_submenu == Some(index) {
            self.tab_menu.state.pending_submenu_close = true;
        }
        cx.notify();
    }

    /// Pointer-hover path into the open submenu: the exact app method a
    /// submenu row's `on_hover` listener calls. Entering a submenu row
    /// confirms the open submenu (cancelling the deferred parent close) and
    /// moves the submenu cursor.
    pub fn tab_menu_submenu_select(
        &mut self,
        index: usize,
        hovered: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_menu.state.open {
            return;
        }
        if hovered {
            self.tab_menu.state.pending_submenu_close = false;
            self.tab_menu.state.submenu_selected = Some(index);
        } else if self.tab_menu.state.submenu_selected == Some(index) {
            self.tab_menu.state.submenu_selected = None;
        }
        cx.notify();
    }

    /// Pointer-press path into the open menu: the exact app method the
    /// top-level row's mouse-down listener calls — select the row, activate
    /// it, and dismiss the menu with the origin focus restored. A press on a
    /// disabled row never activates; like a native menu it dismisses the
    /// menu (the row does not stop propagation, so the root's
    /// one-pointer-action closes it).
    pub fn tab_menu_activate(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.tab_menu.state.open {
            return;
        }
        let enabled = self
            .tab_menu
            .state
            .row_at(index)
            .is_some_and(|row| row.enabled);
        if !enabled {
            self.tab_menu_close(window, cx);
            return;
        }
        self.tab_menu.state.selected = Some(index);
        self.tab_menu_activate_selected(window, cx);
    }

    /// Pointer-press path into the open submenu: commits the submenu row
    /// through the same typed-command authority, then dismisses the menu.
    pub fn tab_menu_submenu_activate(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_menu.state.open {
            return;
        }
        let enabled = self
            .tab_menu
            .state
            .submenu_row_at(index)
            .is_some_and(|row| row.enabled);
        if !enabled {
            self.tab_menu_close(window, cx);
            return;
        }
        self.tab_menu.state.submenu_selected = Some(index);
        self.tab_menu_activate_selected(window, cx);
    }

    /// The §13 inside-press marker for the card's row listeners: a press
    /// inside the open card marks it and stops propagation so the root
    /// outside-press handler never dismisses it, then activates the row.
    pub fn on_tab_menu_row_press(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_menu.state.open {
            return;
        }
        self.tab_menu.state.press_inside = true;
        cx.stop_propagation();
        self.tab_menu_activate(index, window, cx);
    }

    /// The §13 inside-press marker for the submenu card's row listeners.
    pub fn on_tab_menu_submenu_press(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_menu.state.open {
            return;
        }
        self.tab_menu.state.press_inside = true;
        cx.stop_propagation();
        self.tab_menu_submenu_activate(index, window, cx);
    }

    /// Escape / outside-press / close-seam: dismiss the menu and restore the
    /// focus domain captured at open. No-op when no menu is open.
    pub fn tab_menu_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.tab_menu.state.open {
            return;
        }
        let origin_chrome_focused = self.tab_menu.state.origin_chrome_focused;
        self.tab_menu_discard(cx);
        if origin_chrome_focused {
            self.focus_command_bar(window, cx);
        } else {
            self.focus_web(window, cx);
        }
    }

    /// Close the menu WITHOUT touching focus (the invoking mutation — a
    /// command-surface entry, a find open, or a page-menu open — owns the
    /// focus choreography afterwards).
    pub(crate) fn tab_menu_discard(&mut self, cx: &mut Context<Self>) {
        if !self.tab_menu.state.open {
            return;
        }
        self.tab_menu.state = TabMenuState::default();
        cx.notify();
    }

    /// §13 one-pointer-action: a pointer press outside the open menu card
    /// dismisses it. The card's row listeners mark presses inside (their
    /// handlers run first in the bubble phase) and stop propagation; any
    /// other press — a tab row, a space chip, the divider, the web content
    /// boundary — closes the menu. The flag is consumed here so the next
    /// press starts clean.
    pub fn dismiss_tab_menu_on_outside_press(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inside = self.tab_menu.state.press_inside;
        self.tab_menu.state.press_inside = false;
        if self.tab_menu.state.open && !inside {
            self.tab_menu_close(window, cx);
        }
    }

    /// Handle menu-mode keystrokes from the root key-down listener:
    /// Up/Down move the cursor (top-level, or inside the open submenu),
    /// Right opens a submenu, Left closes it, Enter commits the selection,
    /// Escape dismisses.
    pub(crate) fn on_tab_menu_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_menu.state.open {
            return;
        }
        let ks = &event.keystroke;
        match ks.key.as_str() {
            "escape" => {
                cx.stop_propagation();
                self.tab_menu_close(window, cx);
            }
            "enter" => {
                cx.stop_propagation();
                self.tab_menu_activate_selected(window, cx);
            }
            "up" | "down" => {
                cx.stop_propagation();
                let delta = if ks.key == "up" { -1 } else { 1 };
                let moved = if self.tab_menu.state.open_submenu.is_some() {
                    self.tab_menu.state.move_submenu_selection(delta)
                } else {
                    self.tab_menu.state.move_selection(delta)
                };
                if moved {
                    cx.notify();
                }
            }
            "right" => {
                cx.stop_propagation();
                if self.tab_menu.state.open_submenu.is_none() {
                    if let Some(index) = self.tab_menu.state.selected {
                        self.tab_menu.state.open_submenu_at(index);
                        self.tab_menu_refresh_submenu_rect(window);
                        cx.notify();
                    }
                }
            }
            "left" => {
                cx.stop_propagation();
                if self.tab_menu.state.open_submenu.is_some() {
                    self.tab_menu.state.close_submenu();
                    self.tab_menu_refresh_submenu_rect(window);
                    cx.notify();
                }
            }
            _ => {}
        }
        cx.notify();
    }

    /// Activate the selected row through its typed-command authority and
    /// dismiss the menu with the origin focus restored. Enter or a pointer
    /// press on a **closed** submenu parent opens its submenu card instead
    /// — it never dispatches a command (the parent row carries no action).
    /// A disabled selection is a no-op (the traversal never lands there;
    /// the pointer path checks first).
    fn tab_menu_activate_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // A closed submenu parent activates by opening its submenu card.
        if self.tab_menu.state.open_submenu.is_none() {
            if let Some(index) = self.tab_menu.state.selected {
                if self
                    .tab_menu
                    .state
                    .row_at(index)
                    .is_some_and(|row| row.submenu.is_some())
                {
                    self.tab_menu.state.open_submenu_at(index);
                    self.tab_menu_refresh_submenu_rect(window);
                    cx.notify();
                    return;
                }
            }
        }
        let Some(action) = self.tab_menu_selected_action() else {
            return;
        };
        if action.hands_off_focus() {
            // The committed surface (space configuration, ⌘T palette,
            // delete sheet) takes focus: close first, never restore.
            let space = self.tab_menu.state.bound_space;
            let folder = self.tab_menu.state.bound_folder;
            self.tab_menu_discard(cx);
            if let Some(space) = space {
                self.space_menu_dispatch(space, action, window, cx);
            } else if let Some(folder) = folder {
                self.folder_menu_dispatch(folder, action, window, cx);
            }
            return;
        }
        self.tab_menu_dispatch(action, window, cx);
        self.tab_menu_close(window, cx);
    }

    /// The typed action of the current selection (the open submenu's row
    /// when one is open; the top-level selection otherwise). `None` when no
    /// action is bound — a submenu parent row never dispatches.
    fn tab_menu_selected_action(&self) -> Option<TabMenuAction> {
        if self.tab_menu.state.open_submenu.is_some() {
            self.tab_menu
                .state
                .submenu_selected
                .and_then(|index| self.tab_menu.state.submenu_row_at(index))
        } else {
            self.tab_menu
                .state
                .selected
                .and_then(|index| self.tab_menu.state.row_at(index))
        }
        .and_then(|row| row.action)
    }

    /// Recompute the open submenu's resolved placement from the live window
    /// geometry and theme (edge flip included). Called whenever the submenu
    /// opens, closes, or the relevant selection changes so the Folder/Space
    /// destination card visibly renders. No-op when no submenu is open.
    fn tab_menu_refresh_submenu_rect(&mut self, window: &mut Window) {
        let window_size = [
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        ];
        let theme = self.theme();
        self.tab_menu
            .state
            .recompute_submenu_rect(window_size, &theme);
    }

    /// Dispatch one typed action against the bound `TabId` through the
    /// owning authority: browser rows via the portable
    /// `bbr_browser::TabMenuController` (which re-resolves the **live**
    /// controller state at commit), organization rows via the id-keyed
    /// `OrganizationController` commands, B2 rows via the one app-owned
    /// `SavedPageCommand` (the Saved Page never retains the `TabId`).
    fn tab_menu_dispatch(
        &mut self,
        action: TabMenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(space) = self.tab_menu.state.bound_space {
            self.space_menu_dispatch(space, action, window, cx);
            return;
        }
        if let Some(folder) = self.tab_menu.state.bound_folder {
            self.folder_menu_dispatch(folder, action, window, cx);
            return;
        }
        let Some(tab) = self.tab_menu.state.bound_tab else {
            return;
        };
        match action {
            TabMenuAction::Reload => self.dispatch_browser(tab, BrowserTabMenuAction::Reload),
            TabMenuAction::TogglePin => self.dispatch_browser(tab, BrowserTabMenuAction::TogglePin),
            TabMenuAction::Suspend => self.dispatch_browser(tab, BrowserTabMenuAction::Suspend),
            TabMenuAction::CopyLink => self.dispatch_browser(tab, BrowserTabMenuAction::CopyLink),
            TabMenuAction::MoveToStart => {
                self.dispatch_browser(tab, BrowserTabMenuAction::MoveToStart)
            }
            TabMenuAction::MoveToEnd => self.dispatch_browser(tab, BrowserTabMenuAction::MoveToEnd),
            TabMenuAction::Close => self.dispatch_browser(tab, BrowserTabMenuAction::Close),
            TabMenuAction::ReopenTab => self.dispatch_browser(tab, BrowserTabMenuAction::ReopenTab),
            TabMenuAction::MoveToSpace(space) => {
                self.dispatch_browser(tab, BrowserTabMenuAction::MoveToSpace(space))
            }
            TabMenuAction::MoveToFolder(folder) => {
                let moved = self.organization.get().move_tab_into_folder_id(
                    &mut self.controller.get(),
                    tab,
                    folder,
                );
                match moved {
                    Ok(event) => self.apply_grouping(&event, cx),
                    Err(error) => log::error!("tab menu folder move failed: {error}"),
                }
            }
            TabMenuAction::ToggleFavorite => {
                let toggled = self
                    .organization
                    .get()
                    .favorite_toggle_id(&mut self.controller.get(), tab);
                match toggled {
                    Ok(event) => self.apply_favorite_toggle(&event, cx),
                    Err(error) => log::error!("tab menu favorite toggle failed: {error}"),
                }
            }
            TabMenuAction::BookmarkSave => {
                let Some(source) = self.live_tab_menu_source() else {
                    return;
                };
                let command = self.saved_pages;
                let _ = command.save(self, &source.url, &source.title);
            }
            TabMenuAction::BookmarkRemove => {
                let Some(source) = self.live_tab_menu_source() else {
                    return;
                };
                if let Ok(canonical) = bbr_domain::canonical_url(&source.url) {
                    let saved = self
                        .store
                        .read()
                        .saved_page_by_url(&canonical)
                        .map(|record| record.id);
                    if let Some(id) = saved {
                        let command = self.saved_pages;
                        command.remove(self, bbr_command::id::BookmarkId(id));
                    }
                }
            }
            TabMenuAction::EditSpace
            | TabMenuAction::NewTabInSpace
            | TabMenuAction::MoveSpaceLeft
            | TabMenuAction::MoveSpaceRight
            | TabMenuAction::DeleteSpace
            | TabMenuAction::RenameFolder
            | TabMenuAction::NewTabInFolder
            | TabMenuAction::UnpackFolder => {
                log::error!("space/folder menu action {action:?} without its source; ignored")
            }
        }
        window.refresh();
        cx.notify();
    }

    /// Route one browser-owned action through the portable tab-menu
    /// controller and broadcast its typed event (the E2E change stream and
    /// the UI share one sink).
    fn dispatch_browser(&mut self, tab: TabId, action: BrowserTabMenuAction) {
        let dispatched =
            bbr_browser::TabMenuController::dispatch(&mut self.controller.get(), tab, action);
        match dispatched {
            Ok(event) => self.emit_browser(&event),
            Err(error) => log::error!("tab menu command failed: {error}"),
        }
    }

    /// The live source of the bound tab at commit (never the stale menu
    /// snapshot).
    fn live_tab_menu_source(&self) -> Option<bbr_browser::TabMenuSource> {
        let tab = self.tab_menu.state.bound_tab?;
        bbr_browser::TabMenuController::resolve(&self.controller.read(), |_| false, tab)
    }

    /// The exact `TabId` of the active space's tab at `index` (the tab row
    /// the right-click opened on).
    fn active_space_tab_id(&self, index: usize) -> Option<TabId> {
        let snapshot = self.controller.read().snapshot();
        snapshot
            .all_tabs
            .iter()
            .find(|tab| tab.space == snapshot.active_space && tab.index == index)
            .map(|tab| tab.id)
    }

    /// The typed E2E snapshot state of the menu (wire mirror; the `TabId`
    /// crosses the wire only as a stable integer for identity evidence).
    #[cfg(feature = "testkit")]
    pub fn tab_menu_snapshot(&self) -> bbr_testkit::protocol::TabMenuSnapshot {
        let state = &self.tab_menu.state;
        bbr_testkit::protocol::TabMenuSnapshot {
            open: state.open,
            focused: state.focused,
            bound_tab: state.bound_tab.map(|tab| tab.0),
            bound_space: state.bound_space.map(|space| space.0),
            bound_folder: state.bound_folder.map(|folder| folder.0),
            source_key: state.source_key.clone(),
            selected: state.selected,
            open_submenu: state.open_submenu,
            submenu_selected: state.submenu_selected,
            rows: state.rows.iter().map(row_protocol).collect(),
            anchor: [
                state.anchor.x,
                state.anchor.y,
                state.anchor.width,
                state.anchor.height,
            ],
            submenu_rect: state
                .submenu_rect
                .map(|rect| [rect.x, rect.y, rect.width, rect.height]),
        }
    }
    /// The §15 ids the open Tab/Favorite menu renders (DESIGN.md §15): the
    /// card id (the presentation source key), the top-level row ids, and —
    /// while a submenu is open — the submenu card id plus its row ids.
    /// Mirrors the render's gating exactly so the E2E id assertion cannot
    /// drift from the shell. Empty while the menu is closed.
    #[cfg(feature = "testkit")]
    pub fn tab_menu_visible_ids(&self) -> Vec<String> {
        let state = &self.tab_menu.state;
        if !state.open {
            return Vec::new();
        }
        let mut ids = vec![state.source_key.clone()];
        for row in &state.rows {
            ids.push(row.id.clone());
        }
        if state.open_submenu.is_some() {
            ids.push(format!(
                "{}-submenu",
                bbr_ui::surfaces::tab_menu::TAB_MENU_ID
            ));
            if let Some(submenu) = state
                .open_submenu
                .and_then(|parent| state.rows.get(parent))
                .and_then(|row| row.submenu.as_ref())
            {
                for row in submenu {
                    ids.push(row.id.clone());
                }
            }
        }
        ids
    }
}

/// Map one app-side row onto the wire snapshot (recursively for submenu
/// rows).
#[cfg(feature = "testkit")]
fn row_protocol(row: &TabMenuRow) -> bbr_testkit::protocol::TabMenuRowSnapshot {
    bbr_testkit::protocol::TabMenuRowSnapshot {
        id: row.id.clone(),
        label: row.label.clone(),
        enabled: row.enabled,
        danger: row.danger,
        submenu: row
            .submenu
            .as_ref()
            .map(|rows| rows.iter().map(row_protocol).collect())
            .unwrap_or_default(),
    }
}
