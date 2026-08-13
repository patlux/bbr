use super::*;

#[test]
fn hairline_presence_mirrors_the_organized_area_truthfully() {
    // §11.4: the organized-regions hairline renders exactly while the
    // organized area has content (pinned rows or a folders list); with
    // both empty, no separator node is fabricated between the active-Space
    // heading and New Tab.
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

    // Both organized regions empty: the hairline is absent, New Tab stays.
    let bare = semantic_tree(&input);
    let tab_list = find(&bare, "bbr-tab-list").expect("tab list");
    assert!(
        !tab_list
            .children
            .iter()
            .any(|node| node.id == "bbr-organized-hairline"),
        "no hairline while pinned and folders are both empty"
    );
    assert!(tab_list
        .children
        .iter()
        .any(|node| node.id == "bbr-new-tab"));

    // Folders only: the hairline appears between Folders and New Tab.
    let presentation = bbr_ui::components::folders::FolderPresentation::default();
    let folder_snapshots = [bbr_browser::FolderSnapshot {
        index: 0,
        id: bbr_domain::FolderId(1),
        space: bbr_browser::SpaceId(1),
        title: "Work".into(),
        members: vec![],
    }];
    let folder_rows =
        crate::app::ui_adapters::folder_row_specs(&folder_snapshots, &presentation, None);
    input.folder_rows = &folder_rows;
    let with_folders = semantic_tree(&input);
    let tab_list = find(&with_folders, "bbr-tab-list").expect("tab list");
    let order: Vec<&str> = tab_list
        .children
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    let hairline_at = order
        .iter()
        .position(|id| *id == "bbr-organized-hairline")
        .expect("hairline present with folders");
    assert_eq!(order[hairline_at - 1], "bbr-section-folders");
    assert_eq!(order[hairline_at + 1], "bbr-new-tab");
    assert_eq!(
        tab_list.children[hairline_at].role,
        SemanticRole::Separator,
        "typed organized-regions separator role"
    );
}
