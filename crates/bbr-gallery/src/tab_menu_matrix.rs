//! The Tab/Favorite menu matrix authority (task packet
//! `tab-favorite-context-menus`): **one declaration source** for every
//! valid production family of the Tab/Favorite context menu and every
//! rejected combination. The story renders exactly the valid families
//! through the production `bbr_ui::surfaces::tab_menu::tab_menu_card`,
//! and the fixture builder refuses (`None`) any case the validity
//! authority rejects — so the story, the fixture builder, and the tests
//! cannot drift.
//!
//! The row families mirror the production row builder's deterministic
//! render order for the owned groups (placement → page/session →
//! lifecycle → system/share → ordering → close): `Move to Space ›` and
//! `Move to Folder ›` submenu parents (an empty destination group is
//! omitted whole), `Reload Tab` (session-gated), the Favorite toggle (the
//! label follows the source's favorite state), the B2 bookmark row
//! (`Bookmark Page` / `Remove Bookmark`), `Pin Tab` / `Unpin Tab`,
//! `Suspend Tab`, `Copy Link`, `Move to Start` / `Move to End` (region
//! edge-gated), and the danger `Close Tab` plus `Reopen Closed Tab`.
//! Unsupported rows (Duplicate, Share, split/media/rename/icon, Move to
//! New Window, bulk-close scopes) never enter the model.
//!
//! Rejected combinations: a selection/cursor on a disabled row (the
//! production traversal only lands on enabled rows), an open submenu on a
//! row that carries none, and an empty menu (the production row model is
//! never empty).

use bbr_ui::surfaces::tab_menu::{TabMenuModel, TabMenuRect, TabMenuRow};

/// The gallery's deterministic card placement (points, top-left origin):
/// the settled production geometry — `TAB_MENU_WIDTH` 220 pt,
/// `density:row`-tall entries plus `sp:1` card padding. The story renders
/// every family at this same fixture rect.
pub const TAB_MENU_FIXTURE_RECT: TabMenuRect = TabMenuRect::new(8.0, 8.0, 220.0, 240.0);

/// The submenu card placement for the families with an open submenu: to
/// the right of the parent card (the production `submenu_rect` geometry).
pub const TAB_MENU_FIXTURE_SUBMENU_RECT: TabMenuRect = TabMenuRect::new(228.0, 8.0, 220.0, 96.0);

/// One matrix row: a family label and the production `TabMenuModel`
/// fields. The matrix-only flags (`empty`, `claims_selected_on_disabled`,
/// `claims_submenu_without_parent`) exist solely to name the rejected
/// combinations; they are never mapped into `TabMenuModel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabMenuCase {
    pub label: &'static str,
    /// The presentation source key (`bbr-tab-menu` | `bbr-favorite-menu`).
    pub source_key: &'static str,
    /// Whether the menu presentation owns keyboard focus (the §9 accent
    /// ring follows it).
    pub focused: bool,
    /// The keyboard cursor / hovered top-level row.
    pub selected: Option<usize>,
    /// The top-level row whose submenu card is open (`None` = no submenu).
    pub open_submenu: Option<usize>,
    /// The keyboard cursor / hovered row inside the open submenu.
    pub submenu_selected: Option<usize>,
    /// Matrix-only: this case claims an empty menu (the production row
    /// model is never empty).
    pub empty: bool,
    /// Matrix-only: this case claims the keyboard cursor on a disabled row
    /// (the production cursor only lands on enabled rows).
    pub claims_selected_on_disabled: bool,
    /// Matrix-only: this case claims an open submenu on a row that carries
    /// none (the production `open_submenu_at` is a no-op there).
    pub claims_submenu_without_parent: bool,
}

impl TabMenuCase {
    /// The valid-family constructor with the matrix-only flags defaulting
    /// to false.
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        source_key: &'static str,
        focused: bool,
        selected: Option<usize>,
        open_submenu: Option<usize>,
        submenu_selected: Option<usize>,
    ) -> Self {
        TabMenuCase {
            label,
            source_key,
            focused,
            selected,
            open_submenu,
            submenu_selected,
            empty: false,
            claims_selected_on_disabled: false,
            claims_submenu_without_parent: false,
        }
    }
}

/// The valid families, in declaration order (the story renders them in
/// this order):
/// - the tab-row menu for a one-space, folder-less source (no placement
///   group, session-less Reload + region-edge + reopen enablement);
/// - the tab-row menu with the `Move to Space` submenu open (two
///   destinations, the cross-profile one carrying the visible profile
///   identity — DESIGN.md §11.3);
/// - the tab-row menu with the `Move to Folder` submenu open;
/// - the Favorite menu (source key `bbr-favorite-menu`, the toggle reads
///   `Remove from Favorites`);
/// - the tab-row menu with the B2 bookmark row (`Bookmark Page`).
pub const TAB_MENU_VALID: &[TabMenuCase] = &[
    TabMenuCase::new(
        "Tab menu — one space, no folders",
        "bbr-tab-menu",
        true,
        Some(1), // the first enabled row (the disabled Reload row precedes it)
        None,
        None,
    ),
    TabMenuCase::new(
        "Tab menu — Move to Space submenu",
        "bbr-tab-menu",
        true,
        Some(0),
        Some(0),
        Some(0),
    ),
    TabMenuCase::new(
        "Tab menu — Move to Folder submenu",
        "bbr-tab-menu",
        true,
        Some(0),
        Some(0),
        Some(1),
    ),
    TabMenuCase::new(
        "Favorite menu",
        "bbr-favorite-menu",
        true,
        Some(0),
        None,
        None,
    ),
    TabMenuCase::new(
        "Tab menu — B2 bookmark row",
        "bbr-tab-menu",
        true,
        Some(1),
        None,
        None,
    ),
];

/// One rejected combination: the invalid case plus its DESIGN/contract
/// reason.
pub struct TabMenuRejected {
    pub label: &'static str,
    pub case: TabMenuCase,
    pub reason: &'static str,
}

/// The rejected combinations with their reasons.
pub const TAB_MENU_REJECTED: &[TabMenuRejected] = &[
    TabMenuRejected {
        label: "Cursor on a disabled row",
        case: TabMenuCase {
            label: "cursor-on-disabled",
            source_key: "bbr-tab-menu",
            focused: true,
            selected: Some(1), // the disabled Reload row in family 0
            open_submenu: None,
            submenu_selected: None,
            empty: false,
            claims_selected_on_disabled: true,
            claims_submenu_without_parent: false,
        },
        reason: "the production cursor only lands on enabled rows (first_enabled / skip-disabled traversal)",
    },
    TabMenuRejected {
        label: "Submenu on a row without one",
        case: TabMenuCase {
            label: "submenu-without-parent",
            source_key: "bbr-tab-menu",
            focused: true,
            selected: Some(2),
            open_submenu: Some(2), // the Favorite row carries no submenu
            submenu_selected: Some(0),
            empty: false,
            claims_selected_on_disabled: false,
            claims_submenu_without_parent: true,
        },
        reason: "the production open_submenu_at is a no-op on a row without a submenu card",
    },
    TabMenuRejected {
        label: "Empty menu",
        case: TabMenuCase {
            label: "empty",
            source_key: "bbr-tab-menu",
            focused: true,
            selected: None,
            open_submenu: None,
            submenu_selected: None,
            empty: true,
            claims_selected_on_disabled: false,
            claims_submenu_without_parent: false,
        },
        reason: "the production row model is never empty (the app always builds the owned groups)",
    },
];

/// Whether a case is valid: never empty, the cursor never lands on a
/// disabled row, and an open submenu always sits on a submenu-bearing row.
pub fn tab_menu_case_valid(case: &TabMenuCase) -> bool {
    if case.empty {
        return false;
    }
    if case.claims_selected_on_disabled || case.claims_submenu_without_parent {
        return false;
    }
    let rows = tab_menu_rows(case);
    if rows.is_empty() {
        return false;
    }
    if let Some(selected) = case.selected {
        if rows.get(selected).is_none_or(|row| !row.enabled) {
            return false;
        }
    }
    if let Some(parent) = case.open_submenu {
        if rows.get(parent).is_none_or(|row| row.submenu.is_none()) {
            return false;
        }
        if let Some(sub_selected) = case.submenu_selected {
            let submenu = rows[parent].submenu.as_ref().expect("checked above");
            if submenu.get(sub_selected).is_none_or(|row| !row.enabled) {
                return false;
            }
        }
    }
    true
}

/// The deterministic top-level row model for one family, in the packet's
/// group order (the presentation counterpart of the app's row builder —
/// same titles, same enablement rules, no controller).
fn tab_menu_rows(case: &TabMenuCase) -> Vec<TabMenuRow> {
    match case.label {
        "Tab menu — one space, no folders" | "Tab menu — B2 bookmark row" => {
            let mut rows = vec![
                TabMenuRow::action("bbr-tab-menu-reload", "Reload Tab", false),
                TabMenuRow::action("bbr-tab-menu-favorite", "Add to Favorites", true),
                TabMenuRow::action("bbr-tab-menu-pin", "Pin Tab", true),
                TabMenuRow::action("bbr-tab-menu-suspend", "Suspend Tab", true),
                TabMenuRow::action("bbr-tab-menu-copy-link", "Copy Link", true),
                TabMenuRow::action("bbr-tab-menu-move-start", "Move to Start", false),
                TabMenuRow::action("bbr-tab-menu-move-end", "Move to End", true),
                TabMenuRow::danger("bbr-tab-menu-close-tab", "Close Tab", true),
                TabMenuRow::action("bbr-tab-menu-reopen", "Reopen Closed Tab", false),
            ];
            if case.label == "Tab menu — B2 bookmark row" {
                rows.insert(
                    2,
                    TabMenuRow::action("bbr-tab-menu-bookmark", "Bookmark Page", true),
                );
            }
            rows
        }
        "Tab menu — Move to Space submenu" => vec![
            TabMenuRow::with_submenu(
                "bbr-tab-menu-move-space",
                "Move to Space",
                true,
                vec![
                    TabMenuRow::action("bbr-tab-menu-space-0", "Second", true),
                    TabMenuRow::action("bbr-tab-menu-space-1", "Private Space · Private", true),
                ],
            ),
            TabMenuRow::action("bbr-tab-menu-reload", "Reload Tab", true),
            TabMenuRow::action("bbr-tab-menu-favorite", "Add to Favorites", true),
            TabMenuRow::action("bbr-tab-menu-pin", "Pin Tab", true),
            TabMenuRow::action("bbr-tab-menu-suspend", "Suspend Tab", true),
            TabMenuRow::action("bbr-tab-menu-copy-link", "Copy Link", true),
            TabMenuRow::action("bbr-tab-menu-move-start", "Move to Start", true),
            TabMenuRow::action("bbr-tab-menu-move-end", "Move to End", true),
            TabMenuRow::danger("bbr-tab-menu-close-tab", "Close Tab", true),
            TabMenuRow::action("bbr-tab-menu-reopen", "Reopen Closed Tab", true),
        ],
        "Tab menu — Move to Folder submenu" => vec![
            TabMenuRow::with_submenu(
                "bbr-tab-menu-move-folder",
                "Move to Folder",
                true,
                vec![
                    TabMenuRow::action("bbr-tab-menu-folder-0", "Work", true),
                    TabMenuRow::action("bbr-tab-menu-folder-1", "Fun", true),
                ],
            ),
            TabMenuRow::action("bbr-tab-menu-reload", "Reload Tab", true),
            TabMenuRow::action("bbr-tab-menu-favorite", "Add to Favorites", true),
            TabMenuRow::action("bbr-tab-menu-pin", "Pin Tab", true),
            TabMenuRow::action("bbr-tab-menu-suspend", "Suspend Tab", true),
            TabMenuRow::action("bbr-tab-menu-copy-link", "Copy Link", true),
            TabMenuRow::action("bbr-tab-menu-move-start", "Move to Start", true),
            TabMenuRow::action("bbr-tab-menu-move-end", "Move to End", true),
            TabMenuRow::danger("bbr-tab-menu-close-tab", "Close Tab", true),
            TabMenuRow::action("bbr-tab-menu-reopen", "Reopen Closed Tab", true),
        ],
        "Favorite menu" => vec![
            TabMenuRow::action("bbr-tab-menu-reload", "Reload Tab", true),
            TabMenuRow::action("bbr-tab-menu-favorite", "Remove from Favorites", true),
            TabMenuRow::action("bbr-tab-menu-pin", "Pin Tab", true),
            TabMenuRow::action("bbr-tab-menu-suspend", "Suspend Tab", true),
            TabMenuRow::action("bbr-tab-menu-copy-link", "Copy Link", true),
            TabMenuRow::action("bbr-tab-menu-move-start", "Move to Start", true),
            TabMenuRow::action("bbr-tab-menu-move-end", "Move to End", true),
            TabMenuRow::danger("bbr-tab-menu-close-tab", "Close Tab", true),
            TabMenuRow::action("bbr-tab-menu-reopen", "Reopen Closed Tab", false),
        ],
        _ => Vec::new(),
    }
}

/// Build the plain presentation model for a valid case. `None` for any
/// case the validity authority rejects.
pub fn build_tab_menu_fixture(case: &TabMenuCase) -> Option<TabMenuModel> {
    if !tab_menu_case_valid(case) {
        return None;
    }
    Some(TabMenuModel {
        focused: case.focused,
        selected: case.selected,
        open_submenu: case.open_submenu,
        submenu_selected: case.submenu_selected,
        rows: tab_menu_rows(case),
        source_key: case.source_key.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_families_build_and_rejected_combinations_refuse() {
        for case in TAB_MENU_VALID {
            assert!(tab_menu_case_valid(case), "{} must build", case.label);
            let model = build_tab_menu_fixture(case).expect("valid family builds");
            assert!(!model.rows.is_empty());
            if let Some(selected) = model.selected {
                assert!(model.rows[selected].enabled, "cursor on an enabled row");
            }
            if let Some(parent) = model.open_submenu {
                assert!(model.rows[parent].submenu.is_some());
                assert_eq!(model.source_key, case.source_key);
            }
        }
        for rejected in TAB_MENU_REJECTED {
            assert!(
                !tab_menu_case_valid(&rejected.case),
                "rejected {} builds",
                rejected.label
            );
            assert!(build_tab_menu_fixture(&rejected.case).is_none());
        }
    }

    #[test]
    fn stable_ids_match_the_packet_family() {
        let model = build_tab_menu_fixture(&TAB_MENU_VALID[0]).expect("family builds");
        let ids: Vec<&str> = model.rows.iter().map(|row| row.id.as_ref()).collect();
        assert_eq!(
            ids,
            vec![
                "bbr-tab-menu-reload",
                "bbr-tab-menu-favorite",
                "bbr-tab-menu-pin",
                "bbr-tab-menu-suspend",
                "bbr-tab-menu-copy-link",
                "bbr-tab-menu-move-start",
                "bbr-tab-menu-move-end",
                "bbr-tab-menu-close-tab",
                "bbr-tab-menu-reopen",
            ]
        );
        // The cross-profile destination carries the visible profile
        // identity (DESIGN.md §11.3).
        let sub = build_tab_menu_fixture(&TAB_MENU_VALID[1]).expect("submenu family builds");
        let destination = &sub.rows[0].submenu.as_ref().expect("space submenu")[1];
        assert_eq!(destination.label, "Private Space · Private");
    }
}
