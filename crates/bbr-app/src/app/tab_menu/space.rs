//! Blueprint D05 (DESIGN.md §11.3/§11.13): the space menu — the Tab menu
//! family bound to a `SpaceId`. A secondary click on a space chip opens it
//! at the pointer; opening never activates the space. Rows: Edit Space…,
//! New Tab in Space, Move Left, Move Right, Delete Space… (danger; opens
//! the confirmation sheet, disabled for the last space of a profile).

use bbr_domain::{SpaceId, SpaceMove};
use gpui::{Context, MouseDownEvent, Window};

use crate::app::BbrApp;

use super::{tab_menu_placement, TabMenuAction, TabMenuRow, TabMenuState, SPACE_MENU_SOURCE};

/// The space menu's row model in render order. Move rows are disabled at
/// the edges of the space's profile group; Delete is disabled for the last
/// space of a profile.
pub fn build_space_rows(can_left: bool, can_right: bool, can_delete: bool) -> Vec<TabMenuRow> {
    let mut delete = TabMenuRow::danger(
        format!("{SPACE_MENU_SOURCE}-delete"),
        "Delete Space…",
        TabMenuAction::DeleteSpace,
    );
    delete.enabled = can_delete;
    vec![
        TabMenuRow::action(
            format!("{SPACE_MENU_SOURCE}-edit"),
            "Edit Space…",
            true,
            TabMenuAction::EditSpace,
        ),
        TabMenuRow::action(
            format!("{SPACE_MENU_SOURCE}-new-tab"),
            "New Tab in Space",
            true,
            TabMenuAction::NewTabInSpace,
        ),
        TabMenuRow::action(
            format!("{SPACE_MENU_SOURCE}-move-left"),
            "Move Left",
            can_left,
            TabMenuAction::MoveSpaceLeft,
        ),
        TabMenuRow::action(
            format!("{SPACE_MENU_SOURCE}-move-right"),
            "Move Right",
            can_right,
            TabMenuAction::MoveSpaceRight,
        ),
        delete,
    ]
}

impl BbrApp {
    /// Open the space menu bound to `space` at the pointer — the exact app
    /// method the chip's secondary-button listener calls. Exactly one menu
    /// presentation per window (an open page menu, find bar or command
    /// surface is discarded). Opening never activates the space.
    pub fn open_space_menu(
        &mut self,
        space: SpaceId,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        if self.controller.space_title(space).is_none() {
            log::debug!("space menu requested for an unknown space {space:?}; ignored");
            return;
        }
        let rows = build_space_rows(
            self.controller.can_move_space(space, SpaceMove::Earlier),
            self.controller.can_move_space(space, SpaceMove::Later),
            self.controller.can_delete_space(space),
        );
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
            bound_space: Some(space),
            source_key: SPACE_MENU_SOURCE.into(),
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

    /// Dispatch one space-menu action against the bound `SpaceId`. Moves
    /// route through the controller's space-order authority; Edit, New Tab
    /// and Delete open the surface that owns the follow-up.
    pub(super) fn space_menu_dispatch(
        &mut self,
        space: SpaceId,
        action: TabMenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            TabMenuAction::EditSpace => {
                self.open_space_config(space, window, cx);
            }
            TabMenuAction::NewTabInSpace => {
                if self.controller.snapshot().active_space != space {
                    self.activate_space(space, window, cx);
                }
                self.activate_new_tab(window, cx);
            }
            TabMenuAction::MoveSpaceLeft | TabMenuAction::MoveSpaceRight => {
                let direction = if action == TabMenuAction::MoveSpaceLeft {
                    SpaceMove::Earlier
                } else {
                    SpaceMove::Later
                };
                match self.controller.move_space(space, direction) {
                    Ok(event) => {
                        self.emit_browser(&event);
                        self.persist_structure();
                    }
                    Err(error) => log::error!("space move failed: {error}"),
                }
            }
            TabMenuAction::DeleteSpace => self.open_space_delete_sheet(space, window, cx),
            other => log::error!("tab action {other:?} on the space menu; ignored"),
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_follow_the_board_order_with_edge_and_last_space_enablement() {
        let rows = build_space_rows(false, true, false);
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "bbr-space-menu-edit",
                "bbr-space-menu-new-tab",
                "bbr-space-menu-move-left",
                "bbr-space-menu-move-right",
                "bbr-space-menu-delete",
            ]
        );
        let enabled: Vec<bool> = rows.iter().map(|row| row.enabled).collect();
        assert_eq!(enabled, [true, true, false, true, false]);
        assert!(rows[4].danger, "Delete Space… is the danger row");
        assert!(rows.iter().all(|row| row.submenu.is_none()));
    }

    #[test]
    fn edit_new_tab_and_delete_hand_focus_to_their_surface() {
        assert!(TabMenuAction::EditSpace.hands_off_focus());
        assert!(TabMenuAction::NewTabInSpace.hands_off_focus());
        assert!(TabMenuAction::DeleteSpace.hands_off_focus());
        assert!(!TabMenuAction::MoveSpaceLeft.hands_off_focus());
        assert!(!TabMenuAction::Close.hands_off_focus());
    }
}
