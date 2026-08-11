//! The Plus-menu matrix authority (packet
//! `component-gallery-browser-components.md`, Group 4): **one declaration
//! source** for every valid §11.4 plus-menu state family and every
//! rejected combination. The gallery story renders exactly the valid
//! families through the production `bbr_ui::components::menu` symbols
//! (`plus_menu_items`, `PlusMenuState`, `plus_menu_ids`), and the fixture
//! builder refuses (`None`) any case the validity authority rejects — so
//! the story, the fixture builder, the matrix table, and the tests cannot
//! drift. The rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Plus-menu row: closed /
//! open; selection; the keyboard `↑`/`↓`/Enter/Escape state machine (the
//! production `PlusMenuState::move_selection` wrap); and the two v1 items
//! (**New Space**, **New Folder**). Rejected: a third item (exactly two in
//! v1, §11.4) and an item while the menu is closed (the ids expose only
//! while the menu is open).
//!
//! The overlay surface (geometry, elevation, focus wiring) is app-owned
//! until the plus-menu behavior step (`bbr-ui::components::menu` owns the
//! item identity contract and the app-local state; the app's
//! `plus_menu_surface` owns the surface). The story therefore proves the
//! production item identities, the state matrix, and the recorder wiring —
//! never a gallery-local copy of the app's overlay.

use bbr_ui::components::menu::PLUS_MENU_ITEM_COUNT;
use bbr_ui::components::PlusMenuState;

/// One matrix row: a family label, the production `PlusMenuState` fields,
/// and the matrix-only claims that name the rejected combinations in the
/// same case shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlusMenuCase {
    pub label: &'static str,
    /// Whether the §11.4 menu surface is open (ids expose only while
    /// open).
    pub open: bool,
    /// The keyboard selection index (0 = New Space, 1 = New Folder).
    pub selection: usize,
    /// Matrix-only: this case claims a third menu item (§11.4 rejects it —
    /// exactly two v1 items).
    pub claims_third_item: bool,
    /// Matrix-only: this case claims an item while the menu is closed
    /// (§11.4 — the ids expose only while the menu is open).
    pub claims_item_while_closed: bool,
}

/// Every valid plus-menu family — the single declaration the story
/// renders.
#[rustfmt::skip]
pub const PLUS_MENU_VALID: &[PlusMenuCase] = &[
    PlusMenuCase {
        label: "closed",
        open: false,
        selection: 0,
        claims_third_item: false,
        claims_item_while_closed: false,
    },
    PlusMenuCase {
        label: "open — New Space selected",
        open: true,
        selection: 0,
        claims_third_item: false,
        claims_item_while_closed: false,
    },
    PlusMenuCase {
        label: "open — New Folder selected",
        open: true,
        selection: 1,
        claims_third_item: false,
        claims_item_while_closed: false,
    },
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `PlusMenuCase` shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedPlusMenu {
    pub label: &'static str,
    pub case: PlusMenuCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`plus_menu_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const PLUS_MENU_REJECTED: &[RejectedPlusMenu] = &[
    RejectedPlusMenu {
        label: "a third item",
        case: PlusMenuCase {
            label: "a third item",
            open: true,
            selection: 0,
            claims_third_item: true,
            claims_item_while_closed: false,
        },
        reason: "§11.4: the v1 plus menu has exactly two items — New Space (bbr-plus-new-space) and New Folder (bbr-plus-new-folder)",
    },
    RejectedPlusMenu {
        label: "an item while the menu is closed",
        case: PlusMenuCase {
            label: "an item while the menu is closed",
            open: false,
            selection: 0,
            claims_third_item: false,
            claims_item_while_closed: true,
        },
        reason: "§11.4: the menu ids expose only while the menu is open — a closed menu renders no item (plus_menu_ids returns nothing)",
    },
];

/// The one plus-menu validity authority: rejects the impossible
/// combinations the matrix forbids. The fixture builder refuses (`None`)
/// any case this rejects, so the story can never render a rejected menu.
pub fn plus_menu_case_valid(case: &PlusMenuCase) -> bool {
    // §11.4: exactly two v1 items — no third item exists.
    if case.claims_third_item {
        return false;
    }
    // §11.4: the ids expose only while the menu is open.
    if case.claims_item_while_closed {
        return false;
    }
    // The production state clamps the selection to the v1 item range; a
    // declared selection past the item list is an impossible family.
    if case.open && case.selection >= PLUS_MENU_ITEM_COUNT {
        return false;
    }
    true
}

/// Build the production `PlusMenuState` for a valid matrix family; `None`
/// for any rejected combination.
pub fn build_plus_menu_fixture(case: &PlusMenuCase) -> Option<PlusMenuState> {
    if !plus_menu_case_valid(case) {
        return None;
    }
    Some(PlusMenuState {
        open: case.open,
        selection: case.selection,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_ui::components::menu::{plus_menu_ids, plus_menu_items, PLUS_MENU_ID};
    use std::collections::BTreeSet;

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        for case in PLUS_MENU_VALID {
            assert!(
                plus_menu_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let state = build_plus_menu_fixture(case)
                .unwrap_or_else(|| panic!("valid family builds: {}", case.label));
            assert_eq!(state.open, case.open, "{}", case.label);
            assert_eq!(
                state.selected_item().id,
                plus_menu_items()[case.selection.min(PLUS_MENU_ITEM_COUNT - 1)].id,
                "{}",
                case.label
            );
        }
        for rejected in PLUS_MENU_REJECTED {
            assert!(
                !plus_menu_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_plus_menu_fixture(&rejected.case).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        let labels: Vec<&str> = PLUS_MENU_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "closed",
                "open — New Space selected",
                "open — New Folder selected",
            ]
        );
        let mut seen = BTreeSet::new();
        for case in PLUS_MENU_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    #[test]
    fn the_two_v1_items_are_the_production_contract() {
        // §11.4: exactly two items — New Space and New Folder — through
        // the production `plus_menu_items` declaration.
        let items = plus_menu_items();
        assert_eq!(items.len(), PLUS_MENU_ITEM_COUNT);
        assert_eq!(items[0].id, "bbr-plus-new-space");
        assert_eq!(items[0].label, "New Space");
        assert_eq!(items[1].id, "bbr-plus-new-folder");
        assert_eq!(items[1].label, "New Folder");
        // The surface id is the production §15 id.
        assert_eq!(PLUS_MENU_ID, "bbr-plus-menu");
    }

    #[test]
    fn ids_expose_only_while_open_and_selection_wraps() {
        // §11.4: `plus_menu_ids` returns the surface + its two items only
        // while the menu is open.
        let closed = build_plus_menu_fixture(&PLUS_MENU_VALID[0]).unwrap();
        assert!(plus_menu_ids(&closed).is_empty());
        let open = build_plus_menu_fixture(&PLUS_MENU_VALID[1]).unwrap();
        assert_eq!(
            plus_menu_ids(&open),
            vec![
                "bbr-plus-menu".to_string(),
                "bbr-plus-new-space".to_string(),
                "bbr-plus-new-folder".to_string(),
            ]
        );
        // §11.4 keyboard state machine: ↑/↓ wrap within the two items
        // through the production `move_selection`.
        let mut state = PlusMenuState {
            open: true,
            selection: 0,
        };
        state.move_selection(1);
        assert_eq!(state.selected_item().id, "bbr-plus-new-folder");
        state.move_selection(1);
        assert_eq!(state.selected_item().id, "bbr-plus-new-space", "wraps down");
        state.move_selection(-1);
        assert_eq!(state.selected_item().id, "bbr-plus-new-folder", "wraps up");
        let mut closed = PlusMenuState::default();
        closed.move_selection(1);
        assert_eq!(closed.selection, 0, "closed menu never moves");
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = PLUS_MENU_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec!["a third item", "an item while the menu is closed"]
        );
        for rejected in PLUS_MENU_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }
}
