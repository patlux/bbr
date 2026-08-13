//! DESIGN.md §11.4 plus menu: the `[+]` footer control opens a real context
//! menu (`bbr-plus-menu`) with exactly two v1 items — **New Space**
//! (`bbr-plus-new-space`) and **New Folder** (`bbr-plus-new-folder`).
//!
//! The menu is keyboard-operable (`↑`/`↓`, Enter commits, Escape dismisses
//! without action), closes on an outside pointer press (one pointer action,
//! §13), and the chosen action preserves the invoking focus domain. New
//! Space creates a real space through the controller; New Folder creates a
//! session-local folder (§11.11) — both route through the L5 organization
//! commands. The menu surface itself (overlay geometry, elevation, focus
//! wiring) lands with the plus-menu behavior step; this module owns the
//! item identity contract and the app-local menu state.
//!
//! The state model here is presentation-only: `open` + the keyboard
//! `selection` index. The action dispatch (which command the selected item
//! routes to) is the app layer's concern (`app::u2`).

use crate::icons::Icon;

/// The §15 id of the plus menu surface.
pub const PLUS_MENU_ID: &str = "bbr-plus-menu";
/// The §15 id of the "New Space" menu item.
pub const PLUS_NEW_SPACE_ID: &str = "bbr-plus-new-space";
/// The §15 id of the "New Folder" menu item.
pub const PLUS_NEW_FOLDER_ID: &str = "bbr-plus-new-folder";
/// The v1 item count: exactly two (§11.4 — no fourth control, no third
/// item).
pub const PLUS_MENU_ITEM_COUNT: usize = 2;

/// One plus-menu item's identity: the stable §15 id, the visible label, and
/// the §7 glyph (every icon context keeps its text alternative).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlusMenuItem {
    pub id: &'static str,
    pub label: &'static str,
    pub glyph: Icon,
}

/// The two v1 plus-menu items in fixed order (selection index 0 = New Space,
/// 1 = New Folder).
pub fn plus_menu_items() -> [PlusMenuItem; PLUS_MENU_ITEM_COUNT] {
    [
        PlusMenuItem {
            id: PLUS_NEW_SPACE_ID,
            label: "New Space",
            glyph: Icon::Plus,
        },
        PlusMenuItem {
            id: PLUS_NEW_FOLDER_ID,
            label: "New Folder",
            // The plus-menu item shows the closed folder glyph (a folder
            // to be created, never one carrying an open disclosure state).
            glyph: Icon::Folder { open: false },
        },
    ]
}

/// The app-local plus-menu presentation state: whether the menu is open and
/// which item the keyboard selection currently highlights. `selection` is
/// always a valid index into [`plus_menu_items`] (clamped on read), so the
/// snapshot never reports an out-of-range selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlusMenuState {
    pub open: bool,
    pub selection: usize,
}

impl PlusMenuState {
    /// The currently selected item (clamped to the v1 item range).
    pub fn selected_item(&self) -> PlusMenuItem {
        plus_menu_items()[self.selection.min(PLUS_MENU_ITEM_COUNT - 1)]
    }

    /// Move the keyboard selection one step (`↑`/`↓` wrap within the two
    /// items; no-op while closed).
    pub fn move_selection(&mut self, step: isize) {
        if !self.open {
            return;
        }
        let len = PLUS_MENU_ITEM_COUNT as isize;
        let current = self.selection.min(PLUS_MENU_ITEM_COUNT - 1) as isize;
        self.selection = ((current + step).rem_euclid(len)) as usize;
    }
}

/// The plus-menu id set for the §15 `visible_ids` derivation: the menu
/// surface plus its two items, exposed only while the menu is open.
#[allow(dead_code)] // §15 visible-ids seam
pub fn plus_menu_ids(state: &PlusMenuState) -> Vec<String> {
    if !state.open {
        return Vec::new();
    }
    let mut ids = vec![PLUS_MENU_ID.to_string()];
    ids.extend(plus_menu_items().iter().map(|item| item.id.to_string()));
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_has_exactly_the_two_v1_items() {
        let items = plus_menu_items();
        assert_eq!(items.len(), PLUS_MENU_ITEM_COUNT);
        assert_eq!(items[0].id, PLUS_NEW_SPACE_ID);
        assert_eq!(items[0].label, "New Space");
        assert_eq!(items[1].id, PLUS_NEW_FOLDER_ID);
        assert_eq!(items[1].label, "New Folder");
    }

    #[test]
    fn state_defaults_closed_and_selection_is_always_valid() {
        let state = PlusMenuState::default();
        assert!(!state.open);
        assert_eq!(state.selected_item().id, PLUS_NEW_SPACE_ID);
    }

    #[test]
    fn keyboard_selection_wraps_within_the_items() {
        let mut state = PlusMenuState {
            open: true,
            selection: 0,
        };
        state.move_selection(1);
        assert_eq!(state.selected_item().id, PLUS_NEW_FOLDER_ID);
        state.move_selection(1);
        assert_eq!(state.selected_item().id, PLUS_NEW_SPACE_ID, "wraps down");
        state.move_selection(-1);
        assert_eq!(state.selected_item().id, PLUS_NEW_FOLDER_ID, "wraps up");
        let mut closed = PlusMenuState::default();
        closed.move_selection(1);
        assert_eq!(closed.selection, 0, "closed menu never moves");
    }

    #[test]
    fn menu_ids_expose_only_while_open() {
        assert!(plus_menu_ids(&PlusMenuState::default()).is_empty());
        let open = PlusMenuState {
            open: true,
            selection: 1,
        };
        let ids = plus_menu_ids(&open);
        assert_eq!(
            ids,
            vec![
                "bbr-plus-menu".to_string(),
                "bbr-plus-new-space".to_string(),
                "bbr-plus-new-folder".to_string(),
            ]
        );
    }
}
