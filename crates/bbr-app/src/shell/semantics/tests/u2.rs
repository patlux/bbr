use super::*;

// U2 §15 seam tests: the footer row, plus menu, favorites grid, and folders
// list node builders lock the typed roles/names/values the later rail
// composition wires into `sidebar_node` (the builders live in
// `crate::shell::semantics::u2`).
// -------------------------------------------------------------------------
// U2 §15 seam tests: the footer row, plus menu, favorites grid, and folders
// list node builders lock the typed roles/names/values the later rail
// composition wires into `sidebar_node`.
// -------------------------------------------------------------------------

#[test]
fn footer_node_exposes_the_fixed_controls_with_the_strip_between_them() {
    let footer = super::super::u2::footer_node(&input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser(),
        &[],
        &[space_chip()],
    ));
    assert_eq!(footer.id, "bbr-footer");
    assert_eq!(footer.role, SemanticRole::Footer);
    assert_eq!(footer.children.len(), 3, "Downloads + space strip + Plus");
    for (id, name) in [("bbr-downloads", "Downloads"), ("bbr-plus", "More actions")] {
        let control = footer
            .children
            .iter()
            .find(|node| node.id == id)
            .unwrap_or_else(|| panic!("missing footer control {id}"));
        assert_eq!(control.role, SemanticRole::Button);
        assert_eq!(control.name.as_deref(), Some(name));
    }
    let strip = footer
        .children
        .iter()
        .find(|node| node.id == "bbr-space-strip")
        .expect("the permanent strip sits between the fixed controls");
    assert_eq!(strip.role, SemanticRole::List);
    assert_eq!(strip.children.len(), 1);
}

#[test]
fn plus_menu_node_mirrors_open_state_and_selection() {
    assert!(
        super::super::u2::plus_menu_node(&bbr_ui::components::menu::PlusMenuState::default())
            .is_none(),
        "a closed menu renders no node"
    );
    let state = bbr_ui::components::menu::PlusMenuState {
        open: true,
        selection: 1,
    };
    let menu = super::super::u2::plus_menu_node(&state).expect("open menu node");
    assert_eq!(menu.id, "bbr-plus-menu");
    assert_eq!(menu.role, SemanticRole::Menu);
    assert_eq!(menu.children.len(), 2);
    let new_folder = menu
        .children
        .iter()
        .find(|node| node.id == "bbr-plus-new-folder")
        .expect("new folder item");
    assert_eq!(new_folder.role, SemanticRole::MenuItem);
    assert!(
        new_folder.selected,
        "selection index 1 highlights New Folder"
    );
    let new_space = menu
        .children
        .iter()
        .find(|node| node.id == "bbr-plus-new-space")
        .expect("new space item");
    assert!(!new_space.selected);
}

#[test]
fn favorites_node_mirrors_the_grid_and_hides_when_empty() {
    let browser = browser();
    assert!(super::super::u2::favorites_node(&[], &browser).is_none());
    let tiles = crate::app::ui_adapters::favorite_tiles(&[bbr_browser::FavoriteSnapshot {
        index: 0,
        tab: bbr_domain::TabId(7),
        space: bbr_browser::SpaceId(1),
        profile: bbr_web::ProfileId(1),
        title: "Example".into(),
        url: "https://example.com".into(),
    }]);
    let section = super::super::u2::favorites_node(&tiles, &browser).expect("favorites node");
    assert_eq!(section.id, "bbr-section-favorites");
    assert_eq!(section.role, SemanticRole::Section);
    let grid = section
        .children
        .iter()
        .find(|node| node.id == "bbr-favorites")
        .expect("grid under the section");
    assert_eq!(grid.role, SemanticRole::Grid);
    let tile = grid
        .children
        .iter()
        .find(|node| node.id == "bbr-fav-0")
        .expect("tile");
    assert_eq!(tile.role, SemanticRole::Tile);
    assert_eq!(
        tile.name.as_deref(),
        Some("Example"),
        "same context: plain title"
    );
    let icon = tile
        .children
        .iter()
        .find(|node| node.id == "bbr-fav-icon-0")
        .expect("tile icon");
    assert_eq!(icon.role, SemanticRole::Favicon);
    assert_eq!(icon.name.as_deref(), Some("E"));
}

#[test]
fn favorite_tile_name_includes_the_owner_context_when_it_differs() {
    // A favorite owned by another space/profile renders its owner context in
    // the semantic name (DESIGN.md §11.10): the tile stays distinguishable
    // from every current space/profile without relying on color.
    let mut cross = browser(); // active context: space 1, profile 1
    cross.spaces.push(bbr_browser::SpaceSnapshot {
        index: 1,
        id: bbr_browser::SpaceId(2),
        profile: bbr_web::ProfileId(2),
        title: "Work".into(),
        active_tab: Some(0),
        active_view: None,
        active: false,
        tab_count: 1,
    });
    cross.profiles.push(bbr_browser::ProfileSnapshot {
        index: 1,
        id: bbr_web::ProfileId(2),
        title: "Personal".into(),
        hue: bbr_browser::ProfileHue::Fuchsia,
    });
    let tiles = crate::app::ui_adapters::favorite_tiles(&[bbr_browser::FavoriteSnapshot {
        index: 0,
        tab: bbr_domain::TabId(7),
        space: bbr_browser::SpaceId(2),
        profile: bbr_web::ProfileId(2),
        title: "Workmail".into(),
        url: "https://mail.example".into(),
    }]);
    let section = super::super::u2::favorites_node(&tiles, &cross).expect("favorites node");
    let grid = section
        .children
        .iter()
        .find(|node| node.id == "bbr-favorites")
        .expect("grid");
    let tile = grid
        .children
        .iter()
        .find(|node| node.id == "bbr-fav-0")
        .expect("tile");
    assert_eq!(
        tile.name.as_deref(),
        Some("Workmail · in Space · Work · in Profile · Personal"),
        "the tile's name names its owning space/profile when they differ"
    );
}

#[test]
fn folders_node_mirrors_expansion_values_and_nested_items() {
    assert!(super::super::u2::folders_node(&[], &[]).is_none());
    let mut presentation = bbr_ui::components::folders::FolderPresentation::default();
    presentation.toggle(bbr_domain::FolderId(1));
    let rows = crate::app::ui_adapters::folder_row_specs(
        &[bbr_browser::FolderSnapshot {
            index: 0,
            id: bbr_domain::FolderId(1),
            space: bbr_browser::SpaceId(1),
            title: "Work".into(),
            members: vec![bbr_domain::TabId(1), bbr_domain::TabId(2)],
        }],
        &presentation,
    );
    let section = super::super::u2::folders_node(&rows, &[]).expect("folders node");
    assert_eq!(section.id, "bbr-section-folders");
    let list = section
        .children
        .iter()
        .find(|node| node.id == "bbr-folders")
        .expect("folders list");
    assert_eq!(list.role, SemanticRole::List);
    let folder = list
        .children
        .iter()
        .find(|node| node.id == "bbr-folder-row-0")
        .expect("folder row");
    assert_eq!(folder.role, SemanticRole::Folder);
    assert_eq!(folder.name.as_deref(), Some("Work"));
    assert_eq!(folder.value.as_deref(), Some("2"));
    let toggle = folder
        .children
        .iter()
        .find(|node| node.id == "bbr-folder-toggle-0")
        .expect("folder toggle");
    assert_eq!(toggle.role, SemanticRole::Disclosure);
    assert_eq!(toggle.value.as_deref(), Some("expanded"));
    let count = folder
        .children
        .iter()
        .find(|node| node.id == "bbr-folder-count-0")
        .expect("folder count");
    assert_eq!(count.role, SemanticRole::Count);
    assert_eq!(count.value.as_deref(), Some("2"));

    // The collapsed folder reports the collapsed value.
    let collapsed = crate::app::ui_adapters::folder_row_specs(
        &[bbr_browser::FolderSnapshot {
            index: 1,
            id: bbr_domain::FolderId(2),
            space: bbr_browser::SpaceId(1),
            title: "Reading".into(),
            members: vec![],
        }],
        &bbr_ui::components::folders::FolderPresentation::default(),
    );
    let section = super::super::u2::folders_node(&collapsed, &[]).expect("folders node");
    let list = section
        .children
        .iter()
        .find(|node| node.id == "bbr-folders")
        .expect("folders list");
    let row = list
        .children
        .iter()
        .find(|node| node.id == "bbr-folder-row-1")
        .expect("folder row");
    let toggle = row
        .children
        .iter()
        .find(|node| node.id == "bbr-folder-toggle-1")
        .expect("folder toggle");
    assert_eq!(toggle.value.as_deref(), Some("collapsed"));
}

#[test]
fn folder_item_node_exposes_the_nested_row_anatomy() {
    let item = super::super::u2::folder_item_node(0, 1, "Example", 'E');
    assert_eq!(item.id, "bbr-folder-item-0-1");
    assert_eq!(item.role, SemanticRole::Row);
    assert_eq!(item.name.as_deref(), Some("Example"));
    assert_eq!(
        item.children
            .iter()
            .find(|node| node.id == "bbr-folder-item-0-1-favicon")
            .map(|node| node.name.clone()),
        Some(Some("E".to_string()))
    );
    assert!(item
        .children
        .iter()
        .any(|node| node.id == "bbr-folder-item-0-1-title"));
    assert_eq!(
        item.children
            .iter()
            .find(|node| node.id == "bbr-folder-item-0-1-close")
            .map(|node| node.name.clone()),
        Some(Some("Close tab".to_string()))
    );
}

#[test]
fn sidebar_tree_composes_the_u2_footer_favorites_folders_and_plus_menu() {
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];
    let mut input = input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    // §11.4 U2 surfaces: favorites tiles, an expanded folder with a member,
    // and an open plus menu selection. The derived specs are bound to locals
    // so the input's borrowed slices outlive the tree construction.
    let favorites = crate::app::ui_adapters::favorite_tiles(&[bbr_browser::FavoriteSnapshot {
        index: 0,
        tab: bbr_domain::TabId(7),
        space: bbr_browser::SpaceId(1),
        profile: bbr_web::ProfileId(1),
        title: "Example".into(),
        url: "https://example.com".into(),
    }]);
    input.favorites = &favorites;
    let mut presentation = bbr_ui::components::folders::FolderPresentation::default();
    presentation.toggle(bbr_domain::FolderId(1));
    let folder_snapshots = [bbr_browser::FolderSnapshot {
        index: 0,
        id: bbr_domain::FolderId(1),
        space: bbr_browser::SpaceId(1),
        title: "Work".into(),
        members: vec![bbr_domain::TabId(1)],
    }];
    let folder_rows = crate::app::ui_adapters::folder_row_specs(&folder_snapshots, &presentation);
    input.folder_rows = &folder_rows;
    let folder_members = [bbr_ui::components::FolderMemberVisual {
        folder: bbr_domain::FolderId(1),
        folder_index: 0,
        member: 0,
        title: "Example".into(),
        letter: 'E',
        hue: 0,
    }];
    input.folder_members = &folder_members;
    input.plus_menu = bbr_ui::components::menu::PlusMenuState {
        open: true,
        selection: 0,
    };
    let tree = semantic_tree(&input);
    // The footer row with the two fixed controls plus the permanent space
    // strip between them (one chip node per space) sits under the sidebar.
    let sidebar = find(&tree, "bbr-sidebar").expect("sidebar");
    let footer = sidebar
        .children
        .iter()
        .find(|node| node.id == "bbr-footer")
        .expect("footer under the sidebar");
    assert_eq!(footer.role, SemanticRole::Footer);
    assert_eq!(footer.children.len(), 3, "Downloads + strip + Plus");
    let strip = footer
        .children
        .iter()
        .find(|node| node.id == "bbr-space-strip")
        .expect("strip between the fixed controls");
    assert_eq!(strip.role, SemanticRole::List);
    assert_eq!(strip.children.len(), 1, "one chip per space");
    assert_eq!(strip.children[0].id, "bbr-space-chip-1");
    // The favorites section sits above the tab list.
    let favorites = sidebar
        .children
        .iter()
        .find(|node| node.id == "bbr-section-favorites")
        .expect("favorites section");
    assert!(favorites
        .children
        .iter()
        .any(|node| node.id == "bbr-favorites"));
    // The folders list sits inside the tab list between Pinned and Tabs.
    let tab_list = find(&tree, "bbr-tab-list").expect("tab list");
    let folders = tab_list
        .children
        .iter()
        .find(|node| node.id == "bbr-section-folders")
        .expect("folders section inside the tab list");
    assert!(folders.children.iter().any(|node| node.id == "bbr-folders"));
    // The expanded folder's nested row exposes the member anatomy.
    let item = find(&tree, "bbr-folder-item-0-0").expect("nested member row");
    assert_eq!(item.role, SemanticRole::Row);
    assert_eq!(item.name.as_deref(), Some("Example"));
    assert!(find(&tree, "bbr-folder-item-0-0-close").is_some());
    // The open plus menu sits above the footer with its two items.
    let menu = sidebar
        .children
        .iter()
        .find(|node| node.id == "bbr-plus-menu")
        .expect("plus menu");
    assert_eq!(menu.role, SemanticRole::Menu);
    assert_eq!(menu.children.len(), 2);
    let new_space = menu
        .children
        .iter()
        .find(|node| node.id == "bbr-plus-new-space")
        .expect("new space item");
    assert!(new_space.selected);
}

#[test]
fn the_permanent_strip_always_renders_chip_nodes() {
    // The strip is the footer's flex region — it renders one chip per space
    // unconditionally, selected exactly on the active space.
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];
    let single = input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    let tree = semantic_tree(&single);
    let strip = find(&tree, "bbr-space-strip").expect("strip always renders");
    assert_eq!(strip.role, SemanticRole::List);
    assert_eq!(strip.name.as_deref(), Some("Spaces"));
    let chip = find(&tree, "bbr-space-chip-1").expect("chip always renders");
    assert!(chip.selected, "the active space's chip renders selected");
    assert!(find(&tree, "bbr-space-identity-1").is_some());
    // An unselected chip (a second, inactive space) stays quiet.
    let quiet = SpaceChipVisuals {
        space: bbr_browser::SpaceId(2),
        title: "Reading".into(),
        identity: "book".into(),
        theme: bbr_browser::SpaceTheme::Pine,
        selected: false,
        drag_target: false,
        companion: None,
    };
    let chips = [space_chip(), quiet];
    let two = input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &chips,
    );
    let tree = semantic_tree(&two);
    let chip = find(&tree, "bbr-space-chip-1").expect("active chip");
    assert!(chip.selected);
    let quiet = find(&tree, "bbr-space-chip-2").expect("inactive chip");
    assert!(!quiet.selected);
    assert_eq!(
        quiet.children[0].id, "bbr-space-identity-2",
        "identity id derives from the persistent space id"
    );
}
