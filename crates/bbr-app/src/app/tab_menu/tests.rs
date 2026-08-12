//! Contract tests for the app-side Tab/Favorite menu authority (task packet
//! `tab-favorite-context-menus`): the deterministic row builder (groups,
//! labels, enablement, submenu destinations, B2 bookmark parity), the
//! clamped placement (card + submenu edge flip), and the shared
//! keyboard/pointer cursor semantics (skip-disabled traversal, submenu
//! open/close).

use bbr_browser::{FolderDestination, SpaceDestination, TabMenuAvailability, TabMenuSource};
use bbr_domain::{FolderId, SpaceId, TabId, TabLifecycle};
use bbr_ui::theme::{ContrastMode, Theme, ThemeMode};
use bbr_web::ProfileId;

use super::{build_rows, tab_menu_placement, TabMenuAction, TabMenuRect, TabMenuState};

/// A settled theme for the geometry tests (the default density the shell
/// renders at rest).
fn theme() -> Theme {
    Theme::for_collection(ThemeMode::Dark, ContrastMode::Standard, 2)
}

fn source() -> TabMenuSource {
    TabMenuSource {
        tab: TabId(7),
        space: SpaceId(1),
        profile: ProfileId(1),
        index: 1,
        pinned: false,
        lifecycle: TabLifecycle::Warm,
        visible: false,
        favorite: false,
        url: "https://example.com/page".into(),
        title: "Example".into(),
        web_view: None,
        reopen_legal: false,
        pinned_count: 0,
        tab_count: 3,
    }
}

fn availability() -> TabMenuAvailability {
    TabMenuAvailability {
        reload: false,
        pinned: false,
        suspend: true,
        copy_link: true,
        move_to_start: true,
        move_to_end: true,
        favorite: false,
        close: true,
        reopen: false,
        folders: vec![
            FolderDestination {
                folder: FolderId(1),
                title: "Work".into(),
            },
            FolderDestination {
                folder: FolderId(2),
                title: "Fun".into(),
            },
        ],
        spaces: vec![
            SpaceDestination {
                space: SpaceId(2),
                title: "Second".into(),
                profile_title: None,
            },
            SpaceDestination {
                space: SpaceId(3),
                title: "Private".into(),
                profile_title: Some("Private Profile".into()),
            },
        ],
    }
}

#[test]
fn build_rows_covers_the_owned_groups_in_render_order() {
    let rows = build_rows(&source(), &availability(), false, false);
    let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "bbr-tab-menu-move-folder",
            "bbr-tab-menu-move-space",
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
    // Placement group: the two submenu parents carry the deterministic
    // destination rows with the cross-profile identity.
    let folders = &rows[0].submenu.as_ref().expect("folder submenu");
    assert_eq!(
        folders
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        vec!["bbr-tab-menu-folder-0", "bbr-tab-menu-folder-1"]
    );
    assert_eq!(folders[0].label, "Work");
    let spaces = &rows[1].submenu.as_ref().expect("space submenu");
    assert_eq!(
        spaces.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
        vec!["bbr-tab-menu-space-0", "bbr-tab-menu-space-1"]
    );
    assert_eq!(spaces[0].label, "Second");
    assert_eq!(
        spaces[1].label, "Private · Private Profile",
        "cross-profile destinations carry the visible profile identity"
    );
    // Page/session: Reload disabled (no live session), Favorite toggles to
    // Add (not a favorite), the lifecycle/suspend/close rows carry their
    // enablement, Close Tab is the danger row.
    assert_eq!(rows[2].label, "Reload Tab");
    assert!(!rows[2].enabled, "a session-less source cannot reload");
    assert_eq!(rows[3].label, "Add to Favorites");
    assert_eq!(rows[4].label, "Pin Tab");
    assert!(rows[5].enabled, "a hidden warm tab can suspend");
    assert!(rows[8].enabled, "a middle row can move to the region end");
    assert!(rows[9].danger, "Close Tab is the danger row");
    assert_eq!(rows[10].label, "Reopen Closed Tab");
    assert!(!rows[10].enabled, "no recently-closed tab yet");
}

#[test]
fn build_rows_flips_favorite_label_and_omits_empty_groups() {
    let mut s = source();
    s.favorite = true;
    let mut a = availability();
    a.favorite = true;
    a.folders.clear();
    a.spaces.clear();
    let rows = build_rows(&s, &a, false, false);
    let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
    assert!(
        !ids.contains(&"bbr-tab-menu-move-folder"),
        "an empty folder group is omitted whole"
    );
    assert!(
        !ids.contains(&"bbr-tab-menu-move-space"),
        "an empty space group is omitted whole"
    );
    let favorite = rows
        .iter()
        .find(|row| row.id == "bbr-tab-menu-favorite")
        .expect("row");
    assert_eq!(favorite.label, "Remove from Favorites");
}

#[test]
fn build_rows_bookmark_parity_follows_b2_state() {
    // B2 state: save enabled (no record) → `Bookmark Page`; remove enabled
    // (record exists) → `Remove Bookmark`; neither (non-navigable URL) →
    // the row is omitted entirely.
    let rows_save = build_rows(&source(), &availability(), true, false);
    let save = rows_save
        .iter()
        .find(|row| row.id == "bbr-tab-menu-bookmark")
        .expect("bookmark row");
    assert_eq!(save.label, "Bookmark Page");
    assert!(save.enabled);
    let rows_remove = build_rows(&source(), &availability(), false, true);
    let remove = rows_remove
        .iter()
        .find(|row| row.id == "bbr-tab-menu-bookmark")
        .expect("bookmark row");
    assert_eq!(remove.label, "Remove Bookmark");
    let rows_omit = build_rows(&source(), &availability(), false, false);
    assert!(
        rows_omit
            .iter()
            .all(|row| row.id != "bbr-tab-menu-bookmark"),
        "a non-navigable URL omits the bookmark row"
    );
}

#[test]
fn placement_clamps_the_card_and_flips_the_submenu_at_the_right_edge() {
    let rows = build_rows(&source(), &availability(), false, false);
    // Pointer near the bottom-right corner of a 1080×700 window: the card
    // clamps fully on screen and the submenu flips to the left of the card.
    let (card, submenu) =
        tab_menu_placement([1040.0, 680.0], &rows, Some(0), [1080.0, 700.0], &theme());
    assert!(
        card.x + card.width <= 1080.0 - 8.0,
        "card right edge clamped"
    );
    assert!(card.y + card.height <= 700.0 - 8.0, "card bottom clamped");
    let submenu = submenu.expect("open submenu");
    assert_eq!(
        submenu.x + submenu.width,
        card.x,
        "flipped to the card's left"
    );
    assert!(submenu.x >= 8.0, "submenu left edge clamped");
    assert!(
        submenu.y + submenu.height <= 700.0 - 8.0,
        "submenu bottom clamped"
    );
}

#[test]
fn placement_keeps_the_submenu_right_when_there_is_room() {
    let rows = build_rows(&source(), &availability(), false, false);
    let (card, submenu) =
        tab_menu_placement([100.0, 100.0], &rows, Some(0), [1080.0, 700.0], &theme());
    let submenu = submenu.expect("open submenu");
    assert_eq!(submenu.x, card.x + card.width, "submenu draws to the right");
    assert_eq!(submenu.y, card.y, "submenu aligns with the card top");
}

#[test]
fn cursor_moves_skip_disabled_rows_and_wrap() {
    let rows = build_rows(&source(), &availability(), false, false);
    let mut state = TabMenuState {
        rows: rows.clone(),
        selected: Some(1), // Move to Space (enabled)
        ..TabMenuState::default()
    };
    // Down skips the disabled Reload row (index 2) and lands on the
    // Favorite row.
    state.move_selection(1);
    assert_eq!(state.selected, Some(3), "skips the disabled Reload row");
    // Up returns to the previous enabled row.
    state.move_selection(-1);
    assert_eq!(state.selected, Some(1));
    // Down from the last enabled row wraps to the first enabled row
    // (Move to Folder).
    state.selected = Some(9); // Close Tab (the last enabled row)
    state.move_selection(1);
    assert_eq!(state.selected, Some(0), "wraps to the first enabled row");
    // The disabled rows are never landed on.
    state.selected = Some(4);
    state.move_selection(1);
    assert_eq!(state.selected, Some(5));
}

#[test]
fn submenu_open_selects_first_enabled_and_left_closes() {
    let rows = build_rows(&source(), &availability(), false, false);
    let mut state = TabMenuState {
        rows,
        selected: Some(0),
        ..TabMenuState::default()
    };
    state.open_submenu_at(0);
    assert_eq!(state.open_submenu, Some(0));
    assert_eq!(state.submenu_selected, Some(0), "first enabled submenu row");
    // Left closes the submenu and returns to the parent cursor.
    state.close_submenu();
    assert_eq!(state.open_submenu, None);
    assert_eq!(state.submenu_selected, None);
}

#[test]
fn submenu_cursor_moves_within_the_open_submenu() {
    let rows = build_rows(&source(), &availability(), false, false);
    let mut state = TabMenuState {
        rows,
        open_submenu: Some(0),
        submenu_selected: Some(0),
        ..TabMenuState::default()
    };
    assert!(state.move_submenu_selection(1));
    assert_eq!(state.submenu_selected, Some(1));
    // Moving the top-level cursor does not disturb the submenu.
    state.move_selection(1);
    assert_eq!(state.open_submenu, Some(0));
    assert_eq!(state.submenu_selected, Some(1));
}

#[test]
fn submenu_rows_carry_typed_actions_and_parents_never_dispatch() {
    let rows = build_rows(&source(), &availability(), false, false);
    // Submenu parent rows carry **no** action — activating them opens the
    // submenu card; they must never dispatch a placeholder command.
    assert_eq!(rows[0].action, None, "Move to Folder never dispatches");
    assert_eq!(rows[1].action, None, "Move to Space never dispatches");
    assert!(rows[0].submenu.is_some());
    assert!(rows[1].submenu.is_some());
    let folder = &rows[0].submenu.as_ref().expect("folder submenu")[1];
    assert_eq!(
        folder.action,
        Some(TabMenuAction::MoveToFolder(FolderId(2)))
    );
    let space = &rows[1].submenu.as_ref().expect("space submenu")[1];
    assert_eq!(space.action, Some(TabMenuAction::MoveToSpace(SpaceId(3))));
}

#[test]
fn action_rows_carry_their_typed_action() {
    let rows = build_rows(&source(), &availability(), false, false);
    assert_eq!(rows[2].action, Some(TabMenuAction::Reload));
    assert_eq!(rows[5].action, Some(TabMenuAction::Suspend));
    assert_eq!(rows[9].action, Some(TabMenuAction::Close));
}

#[test]
fn opening_a_submenu_assigns_its_resolved_geometry_and_closing_clears_it() {
    let rows = build_rows(&source(), &availability(), false, false);
    let mut state = TabMenuState {
        rows,
        selected: Some(0),
        anchor: TabMenuRect::new(100.0, 100.0, 220.0, 148.0),
        ..TabMenuState::default()
    };
    state.open_submenu_at(0);
    state.recompute_submenu_rect([1080.0, 700.0], &theme());
    let submenu = state.submenu_rect.expect("the open submenu is placed");
    assert_eq!(
        submenu.x,
        state.anchor.x + state.anchor.width,
        "the destination card draws to the right of the parent"
    );
    assert_eq!(submenu.y, state.anchor.y, "top-aligned with the parent");
    assert!(submenu.width > 0.0 && submenu.height > 0.0);
    // Edge flip: a card near the window's right edge flips the submenu to
    // its left, and the submenu stays fully inside the window.
    let mut edge = TabMenuState {
        rows: build_rows(&source(), &availability(), false, false),
        selected: Some(0),
        anchor: TabMenuRect::new(852.0, 100.0, 220.0, 148.0),
        ..TabMenuState::default()
    };
    edge.open_submenu_at(0);
    edge.recompute_submenu_rect([1080.0, 700.0], &theme());
    let flipped = edge.submenu_rect.expect("the flipped submenu is placed");
    assert_eq!(
        flipped.x + flipped.width,
        edge.anchor.x,
        "the destination card flips to the parent's left at the right edge"
    );
    assert!(flipped.x >= 8.0, "the flipped card stays clamped on screen");
    // Closing the submenu clears the resolved geometry.
    state.close_submenu();
    state.recompute_submenu_rect([1080.0, 700.0], &theme());
    assert_eq!(state.submenu_rect, None, "no submenu, no placement");
}

#[test]
fn rect_round_trip_is_exact() {
    let rect = TabMenuRect::new(10.0, 20.0, 220.0, 148.0);
    assert_eq!(rect, TabMenuRect::new(10.0, 20.0, 220.0, 148.0));
}
