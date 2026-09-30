//! Blueprint D25 (DESIGN §11.11/§11.13): the folder menu — the Tab menu
//! family bound to a `FolderId`. A secondary click on a folder row opens it
//! at the pointer; opening never toggles the folder. Rows: Rename… (the
//! D20 inline rename), New Tab in Folder (⌘T whose committed tab joins the
//! folder), Delete Folder — keep its tabs (the settled Unpack). The board's
//! "Open All Tabs" has no settled BBR semantics and the danger row "Close
//! Folder and Its Tabs…" contradicts the 2026-09-27 settlement (DESIGN §19
//! question 13), so both stay omitted.

use bbr_domain::FolderId;
use gpui::{Context, MouseDownEvent, Window};

use crate::app::BbrApp;

use super::{tab_menu_placement, TabMenuAction, TabMenuRow, TabMenuState, FOLDER_MENU_SOURCE};

/// The folder menu's row model in render order.
pub fn build_folder_rows() -> Vec<TabMenuRow> {
    vec![
        TabMenuRow::action(
            format!("{FOLDER_MENU_SOURCE}-rename"),
            "Rename…",
            true,
            TabMenuAction::RenameFolder,
        ),
        TabMenuRow::action(
            format!("{FOLDER_MENU_SOURCE}-new-tab"),
            "New Tab in Folder",
            true,
            TabMenuAction::NewTabInFolder,
        ),
        TabMenuRow::action(
            format!("{FOLDER_MENU_SOURCE}-unpack"),
            "Delete Folder — keep its tabs",
            true,
            TabMenuAction::UnpackFolder,
        ),
    ]
}

impl BbrApp {
    /// Open the folder menu bound to `folder` at the pointer — the exact app
    /// method the folder row's secondary-button listener calls.
    pub fn open_folder_menu(
        &mut self,
        folder: FolderId,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        let known = self
            .organization
            .get()
            .organization_snapshot(&self.controller.read())
            .folders
            .iter()
            .any(|f| f.id == folder);
        if !known {
            return;
        }
        let rows = build_folder_rows();
        let point = [f32::from(event.position.x), f32::from(event.position.y)];
        let window_size = [
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        ];
        let theme = self.theme();
        let (anchor, _) = tab_menu_placement(point, &rows, None, window_size, &theme);
        self.menu_discard(cx);
        self.find_discard(cx);
        self.dismiss_command_portal(cx);
        let mut state = TabMenuState {
            open: true,
            bound_folder: Some(folder),
            source_key: FOLDER_MENU_SOURCE.into(),
            rows,
            anchor,
            origin_chrome_focused: self.chrome_focused(window),
            ..TabMenuState::default()
        };
        state.selected = state.first_enabled();
        self.tab_menu.state = state;
        self.tab_menu.state.focused = true;
        window.focus(&self.menu_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        self.emit_focus(window);
        cx.notify();
    }

    /// Dispatch one folder-menu action against the bound `FolderId`.
    pub(super) fn folder_menu_dispatch(
        &mut self,
        folder: FolderId,
        action: TabMenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            TabMenuAction::RenameFolder => self.begin_folder_rename(folder, window, cx),
            TabMenuAction::NewTabInFolder => {
                let space = self
                    .organization
                    .get()
                    .organization_snapshot(&self.controller.read())
                    .folders
                    .iter()
                    .find(|f| f.id == folder)
                    .map(|f| f.space);
                let Some(space) = space else {
                    return;
                };
                if self.controller.read().snapshot().active_space != space {
                    self.activate_space(space, window, cx);
                }
                self.activate_new_tab(window, cx);
                // After the palette opened (its open clears a stale target).
                self.u2.new_tab_folder = Some(folder);
            }
            TabMenuAction::UnpackFolder => {
                let unpacked = self
                    .organization
                    .get()
                    .unpack_folder(&self.controller.read(), folder);
                match unpacked {
                    Ok(event) => log::debug!("folder unpacked: {event:?}"),
                    Err(error) => log::error!("could not unpack folder: {error}"),
                }
                self.persist_structure();
            }
            other => log::error!("action {other:?} on the folder menu; ignored"),
        }
        cx.notify();
    }

    /// The ⌘T commit of New Tab in Folder: the freshly opened active tab
    /// joins the pending folder (pinned by the folder invariant).
    pub(crate) fn join_pending_new_tab_folder(&mut self, cx: &mut Context<Self>) {
        let Some(folder) = self.u2.new_tab_folder.take() else {
            return;
        };
        let snapshot = self.controller.read().snapshot();
        let Some(tab) = snapshot
            .all_tabs
            .iter()
            .find(|tab| {
                tab.space == snapshot.active_space && Some(tab.index) == snapshot.active_index
            })
            .map(|tab| tab.id)
        else {
            return;
        };
        let moved = self.organization.get().move_tab_into_folder_id(
            &mut self.controller.get(),
            tab,
            folder,
        );
        match moved {
            Ok(event) => self.apply_grouping(&event, cx),
            Err(error) => log::error!("new tab could not join its folder: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_follow_the_board_without_the_unsettled_rows() {
        let rows = build_folder_rows();
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "bbr-folder-menu-rename",
                "bbr-folder-menu-new-tab",
                "bbr-folder-menu-unpack",
            ]
        );
        assert!(rows.iter().all(|row| row.enabled && !row.danger));
        assert!(TabMenuAction::RenameFolder.hands_off_focus());
        assert!(TabMenuAction::NewTabInFolder.hands_off_focus());
        assert!(!TabMenuAction::UnpackFolder.hands_off_focus());
    }
}
