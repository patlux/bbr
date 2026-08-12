//! The Context-menu matrix authority (packets
//! `component-gallery-browser-components.md` Group 9 and
//! `page-background-context-menu.md`): **one declaration source** for every
//! valid BBR page-menu family and every rejected combination. The gallery
//! story renders exactly the valid families through the production
//! `bbr_ui::surfaces::menu::context_menu_card`, and the fixture builder
//! refuses (`None`) any case the validity authority rejects — so the story,
//! the fixture builder, the matrix table, and the tests cannot drift. The
//! rejected reasons also live here.
//!
//! Matrix rows come straight from the packets' Context menu row: the
//! enriched production page menu — Back/Forward enablement (from the
//! session's real history availability), the capability-gated Bookmark
//! Page / Remove Bookmark row (B2 state), Copy Page Link, Reload, the
//! keyboard cursor + pointer hover (the pointer hover path is the
//! real-pointer E2E). Unowned capabilities (Select All / Share / Save Page
//! As / View Page Source / Inspect) never render. Rejected: a menu with
//! zero items, activation/selection of a disabled row (the production
//! cursor only lands on enabled rows), any source-less Tab/Favorite/Folder
//! menu, and any Gallery-owned rendering of WebKit link/image/selection/
//! editable/media menus — the post-packet families (Z1/Z3) and WebKit
//! element menus stay owned by their own packets and never enter this
//! model.
//!
//! The fixed rows and their display titles mirror the production
//! controller's row model (Back, Forward, Reload, Bookmark Page / Remove
//! Bookmark, Copy Page Link — DESIGN.md §11.13/§15); the gallery fixture
//! builder rebuilds only the presentation content (id + title +
//! enablement) the card renders. The stable keys stay app-domain (the
//! controller's action routing and the testkit snapshot), never in the
//! presentation model.

use bbr_ui::surfaces::menu::{ContextMenuModel, ContextMenuRect, ContextMenuRow};

/// The gallery's deterministic card placement (points, top-left origin):
/// the settled production geometry — `CONTEXT_MENU_WIDTH` 220 pt, five
/// `density:row`-tall entries plus `sp:1` card padding (DESIGN §11.4
/// compact clamp). The story renders every family at this same fixture
/// rect; the E2E derives row centers from it.
pub const CONTEXT_MENU_FIXTURE_RECT: ContextMenuRect = ContextMenuRect::new(8.0, 8.0, 220.0, 148.0);

/// The bookmark row's B2 state in a matrix family (the capability gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkFixture {
    /// B2 has no active record: the enabled `Bookmark Page` row renders.
    Save,
    /// B2 has an active record: the enabled `Remove Bookmark` row renders.
    Remove,
    /// The URL is missing or non-navigable: no bookmark row renders.
    Unavailable,
}

/// One matrix row: a family label and the production `ContextMenuModel`
/// fields. The matrix-only flags (`empty`, `claims_selected_on_disabled`,
/// `claims_tab_favorite_source`, `claims_webkit_element_menu`) exist
/// solely to name the rejected combinations in the same case shape; they
/// are never mapped into `ContextMenuModel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextMenuCase {
    pub label: &'static str,
    /// Whether the menu presentation owns keyboard focus (the §9 accent
    /// ring follows it).
    pub focused: bool,
    /// The keyboard cursor / hovered row.
    pub selected: Option<usize>,
    /// The session's real Back history availability (Back enablement).
    pub can_go_back: bool,
    /// The session's real Forward history availability (Forward
    /// enablement).
    pub can_go_forward: bool,
    /// The page URL (Copy Page Link enablement; the bookmark gate).
    pub url: Option<&'static str>,
    /// The bookmark row's B2 state.
    pub bookmark: BookmarkFixture,
    /// Matrix-only: this case claims a menu with zero items (§13/§15 —
    /// the production menu is exactly the owned rows).
    pub empty: bool,
    /// Matrix-only: this case claims the keyboard cursor on a disabled row
    /// (the production `first_enabled`/`menu_move_selection` only land on
    /// enabled rows).
    pub claims_selected_on_disabled: bool,
    /// Matrix-only: this case claims a Tab/Favorite/Folder source-bound
    /// family (Z1/Z3 own those; this page-menu model carries no source).
    pub claims_tab_favorite_source: bool,
    /// Matrix-only: this case claims a Gallery-owned WebKit element menu
    /// (link/image/selection/editable/media menus stay native and are
    /// never rebuilt here).
    pub claims_webkit_element_menu: bool,
}

impl ContextMenuCase {
    /// The valid-family constructor: the production fields with the
    /// matrix-only flags defaulting to false. `#[allow]` keeps the dense
    /// const table readable (the fields are the documented matrix row).
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        focused: bool,
        selected: Option<usize>,
        can_go_back: bool,
        can_go_forward: bool,
        url: Option<&'static str>,
        bookmark: BookmarkFixture,
    ) -> Self {
        ContextMenuCase {
            label,
            focused,
            selected,
            can_go_back,
            can_go_forward,
            url,
            bookmark,
            empty: false,
            claims_selected_on_disabled: false,
            claims_tab_favorite_source: false,
            claims_webkit_element_menu: false,
        }
    }
}

/// The enriched page-menu rows in render order, with the enablement the
/// production controller derives at open (Back/Forward from the session's
/// real history availability; the bookmark row from B2 state — omitted for
/// a missing/non-navigable URL; Copy Page Link from a resolvable page URL;
/// Reload always). The gallery fixture builder is the plain presentation
/// counterpart of the app's row model — same ids, same titles, same
/// enablement rules, no controller.
pub fn menu_rows(
    can_go_back: bool,
    can_go_forward: bool,
    url: Option<&str>,
    bookmark: BookmarkFixture,
) -> Vec<ContextMenuRow> {
    let link_enabled = url.is_some_and(|u| !u.is_empty());
    let mut rows = vec![
        ContextMenuRow {
            id: "bbr-page-menu-back".into(),
            title: "Back".into(),
            enabled: can_go_back,
            disabled_reason: (!can_go_back).then(|| "No back history".into()),
        },
        ContextMenuRow {
            id: "bbr-page-menu-forward".into(),
            title: "Forward".into(),
            enabled: can_go_forward,
            disabled_reason: (!can_go_forward).then(|| "No forward history".into()),
        },
        ContextMenuRow {
            id: "bbr-page-menu-reload".into(),
            title: "Reload".into(),
            enabled: true,
            disabled_reason: None,
        },
    ];
    match bookmark {
        BookmarkFixture::Save => rows.push(ContextMenuRow {
            id: "bbr-page-menu-bookmark".into(),
            title: "Bookmark Page".into(),
            enabled: true,
            disabled_reason: None,
        }),
        BookmarkFixture::Remove => rows.push(ContextMenuRow {
            id: "bbr-page-menu-bookmark".into(),
            title: "Remove Bookmark".into(),
            enabled: true,
            disabled_reason: None,
        }),
        BookmarkFixture::Unavailable => {}
    }
    rows.push(ContextMenuRow {
        id: "bbr-page-menu-copy-link".into(),
        title: "Copy Page Link".into(),
        enabled: link_enabled,
        disabled_reason: (!link_enabled).then(|| "No page URL to copy".into()),
    });
    rows
}

/// Every valid page-menu family — the single declaration the story
/// renders. The enablement dimensions (Back/Forward history, URL presence,
/// the B2 bookmark gate), the §9 focus ring (focused vs hairline), and the
/// keyboard cursor position (selection) are covered; the pointer-hover
/// path is the real-pointer E2E.
#[rustfmt::skip]
pub const CONTEXT_MENU_VALID: &[ContextMenuCase] = &[
    ContextMenuCase::new("fresh presentation — no history, no URL", false, Some(2), false, false, None, BookmarkFixture::Unavailable),
    ContextMenuCase::new("focused — Back available", true, Some(0), true, false, None, BookmarkFixture::Unavailable),
    ContextMenuCase::new("focused — Back + Forward available", true, Some(0), true, true, None, BookmarkFixture::Unavailable),
    ContextMenuCase::new("focused — Bookmark Page + Copy Page Link", true, Some(2), false, false, Some("https://example.com/"), BookmarkFixture::Save),
    ContextMenuCase::new("focused — Remove Bookmark + Copy Page Link", true, Some(3), false, false, Some("https://example.com/"), BookmarkFixture::Remove),
    ContextMenuCase::new("focused — fully enabled page menu", true, Some(0), true, true, Some("https://example.com/"), BookmarkFixture::Save),
    ContextMenuCase::new("unfocused with URL — hairline card", false, Some(2), false, false, Some("https://example.com/"), BookmarkFixture::Save),
    ContextMenuCase::new("keyboard cursor on Copy Page Link", true, Some(4), false, false, Some("https://example.com/"), BookmarkFixture::Save),
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `ContextMenuCase` shape as the valid rows, so
/// one validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedContextMenu {
    pub label: &'static str,
    pub case: ContextMenuCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`context_menu_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const CONTEXT_MENU_REJECTED: &[RejectedContextMenu] = &[
    RejectedContextMenu {
        label: "a menu with zero items",
        case: ContextMenuCase {
            label: "a menu with zero items",
            focused: true,
            selected: None,
            can_go_back: false,
            can_go_forward: false,
            url: None,
            bookmark: BookmarkFixture::Unavailable,
            empty: true,
            claims_selected_on_disabled: false,
            claims_tab_favorite_source: false,
            claims_webkit_element_menu: false,
        },
        reason: "§13/§15: the production page menu always renders its owned rows (Back, Forward, Reload, the B2-gated bookmark row, Copy Page Link); a zero-item menu never renders",
    },
    RejectedContextMenu {
        label: "selection on a disabled row",
        case: ContextMenuCase {
            label: "selection on a disabled row",
            focused: true,
            selected: Some(0),
            can_go_back: false,
            can_go_forward: false,
            url: None,
            bookmark: BookmarkFixture::Unavailable,
            empty: false,
            claims_selected_on_disabled: true,
            claims_tab_favorite_source: false,
            claims_webkit_element_menu: false,
        },
        reason: "the production keyboard cursor only lands on enabled rows (first_enabled/menu_move_selection skip disabled entries); a disabled row never carries selection or activation",
    },
    RejectedContextMenu {
        label: "a source-less Tab/Favorite/Folder menu",
        case: ContextMenuCase {
            label: "a source-less Tab/Favorite/Folder menu",
            focused: true,
            selected: Some(0),
            can_go_back: true,
            can_go_forward: false,
            url: Some("https://example.com/"),
            bookmark: BookmarkFixture::Save,
            empty: false,
            claims_selected_on_disabled: false,
            claims_tab_favorite_source: true,
            claims_webkit_element_menu: false,
        },
        reason: "§11.13: Tab/Favorite/Folder menus are bounded Z1/Z3 packet families bound to an exact source id; the page-menu model carries no source and never invents one",
    },
    RejectedContextMenu {
        label: "a Gallery-owned WebKit element menu",
        case: ContextMenuCase {
            label: "a Gallery-owned WebKit element menu",
            focused: true,
            selected: Some(0),
            can_go_back: true,
            can_go_forward: false,
            url: Some("https://example.com/"),
            bookmark: BookmarkFixture::Save,
            empty: false,
            claims_selected_on_disabled: false,
            claims_tab_favorite_source: false,
            claims_webkit_element_menu: true,
        },
        reason: "§11.13: link/image/selection/editable/media element menus stay WebKit-native and are ownership metadata at most — the Gallery never rebuilds them",
    },
];

/// The one page-menu validity authority: rejects the impossible
/// combinations the matrix forbids. The fixture builder refuses (`None`)
/// any case this rejects, so the story can never render a rejected menu.
pub fn context_menu_case_valid(case: &ContextMenuCase) -> bool {
    // §13/§15: the production menu always renders its owned rows.
    if case.empty {
        return false;
    }
    // §11.13: the cursor only lands on enabled rows (the production
    // first_enabled/menu_move_selection contract).
    if let Some(selected) = case.selected {
        let items = menu_rows(
            case.can_go_back,
            case.can_go_forward,
            case.url,
            case.bookmark,
        );
        if selected >= items.len() || !items[selected].enabled {
            return false;
        }
    }
    // §11.13: source-bound menu families are Z1/Z3-owned; WebKit element
    // menus are never rebuilt by the Gallery.
    if case.claims_tab_favorite_source || case.claims_webkit_element_menu {
        return false;
    }
    true
}

/// Build the production `ContextMenuModel` for a valid matrix family;
/// `None` for any rejected combination. The rows come from the shared
/// `menu_rows` declaration (the same rows the production controller
/// builds), so the story and the production render can never drift.
pub fn build_context_menu_fixture(case: &ContextMenuCase) -> Option<ContextMenuModel> {
    if !context_menu_case_valid(case) {
        return None;
    }
    Some(ContextMenuModel {
        focused: case.focused,
        selected: case.selected,
        items: menu_rows(
            case.can_go_back,
            case.can_go_forward,
            case.url,
            case.bookmark,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        for case in CONTEXT_MENU_VALID {
            assert!(
                context_menu_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let fixture = build_context_menu_fixture(case)
                .unwrap_or_else(|| panic!("valid family builds: {}", case.label));
            assert_eq!(fixture.focused, case.focused, "{}", case.label);
            assert_eq!(fixture.selected, case.selected, "{}", case.label);
            let rows = menu_rows(
                case.can_go_back,
                case.can_go_forward,
                case.url,
                case.bookmark,
            );
            assert_eq!(fixture.items.len(), rows.len(), "{}", case.label);
        }
        for rejected in CONTEXT_MENU_REJECTED {
            assert!(
                !context_menu_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_context_menu_fixture(&rejected.case).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        // The story renders exactly these families in this order; the E2E
        // pins the same stable ids. Reordering or removing a family
        // fails here — the no-drift contract.
        let labels: Vec<&str> = CONTEXT_MENU_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "fresh presentation — no history, no URL",
                "focused — Back available",
                "focused — Back + Forward available",
                "focused — Bookmark Page + Copy Page Link",
                "focused — Remove Bookmark + Copy Page Link",
                "focused — fully enabled page menu",
                "unfocused with URL — hairline card",
                "keyboard cursor on Copy Page Link",
            ]
        );
        // Labels are unique (ids derive from position).
        let mut seen = BTreeSet::new();
        for case in CONTEXT_MENU_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    #[test]
    fn page_menu_is_exactly_the_owned_rows() {
        // §15: the owned rows in render order with the production
        // enablement rules (Back/Forward from real history availability,
        // Reload always, the bookmark row from B2 state, Copy Page Link
        // from a resolvable URL). Unowned capabilities never render.
        let rows = menu_rows(false, false, None, BookmarkFixture::Unavailable);
        let titles: Vec<&str> = rows.iter().map(|r| r.title.as_ref()).collect();
        assert_eq!(titles, vec!["Back", "Forward", "Reload", "Copy Page Link"]);
        assert!(!rows[0].enabled, "Back disabled with no history");
        assert_eq!(rows[0].disabled_reason.as_deref(), Some("No back history"));
        assert!(!rows[1].enabled, "Forward disabled with no history");
        assert_eq!(
            rows[1].disabled_reason.as_deref(),
            Some("No forward history")
        );
        assert!(rows[2].enabled, "Reload always enabled");
        assert!(!rows[3].enabled, "Copy Page Link disabled without a URL");
        assert_eq!(
            rows[3].disabled_reason.as_deref(),
            Some("No page URL to copy")
        );
        // The per-action §15 ids follow the packet's family contract.
        assert_eq!(rows[0].id, "bbr-page-menu-back");
        assert_eq!(rows[1].id, "bbr-page-menu-forward");
        assert_eq!(rows[2].id, "bbr-page-menu-reload");
        assert_eq!(rows[3].id, "bbr-page-menu-copy-link");
        // Real history availability + a resolvable URL enable the rows.
        let full = menu_rows(
            true,
            true,
            Some("https://example.com/"),
            BookmarkFixture::Save,
        );
        assert!(full[0].enabled && full[1].enabled);
        assert_eq!(full[3].title, "Bookmark Page");
        assert!(full[3].enabled);
        assert!(full[4].enabled);
        // An empty URL never enables Copy Page Link.
        let empty_url = menu_rows(true, true, Some(""), BookmarkFixture::Unavailable);
        assert!(!empty_url[3].enabled);
    }

    #[test]
    fn bookmark_row_is_capability_gated_by_b2_state() {
        // Save / Remove / Unavailable map to the production row shape.
        let save = menu_rows(
            false,
            false,
            Some("https://example.com/"),
            BookmarkFixture::Save,
        );
        assert_eq!(save[3].title, "Bookmark Page");
        assert!(save[3].enabled);
        let remove = menu_rows(
            false,
            false,
            Some("https://example.com/"),
            BookmarkFixture::Remove,
        );
        assert_eq!(remove[3].title, "Remove Bookmark");
        assert!(remove[3].enabled);
        let unavailable = menu_rows(
            false,
            false,
            Some("https://example.com/"),
            BookmarkFixture::Unavailable,
        );
        assert!(
            !unavailable.iter().any(|r| r.id == "bbr-page-menu-bookmark"),
            "the bookmark row is omitted when B2 cannot operate"
        );
    }

    #[test]
    fn selection_dimensions_cover_the_keyboard_cursor_contract() {
        // The valid families cover cursor landings (first enabled: Reload
        // at 2 / Back at 0, and moved cursors on Bookmark Page at 3 and
        // Copy Page Link at 4); the rejected column proves a disabled-row
        // selection is impossible.
        let landings: BTreeSet<usize> = CONTEXT_MENU_VALID
            .iter()
            .filter_map(|c| c.selected)
            .collect();
        assert_eq!(landings, BTreeSet::from([0, 2, 3, 4]));
        for case in CONTEXT_MENU_VALID {
            let fixture = build_context_menu_fixture(case).unwrap();
            if let Some(selected) = fixture.selected {
                assert!(
                    fixture.items[selected].enabled,
                    "the cursor only lands on enabled rows: {}",
                    case.label
                );
            }
        }
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = CONTEXT_MENU_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec![
                "a menu with zero items",
                "selection on a disabled row",
                "a source-less Tab/Favorite/Folder menu",
                "a Gallery-owned WebKit element menu",
            ]
        );
        for rejected in CONTEXT_MENU_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }
}
