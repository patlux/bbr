use super::*;
fn rows() -> Vec<FolderRowSpec> {
    vec![
        FolderRowSpec {
            index: 0,
            id: bbr_domain::FolderId(1),
            title: "Work".into(),
            count: 2,
            expanded: false,
            reveal_count: false,
        },
        FolderRowSpec {
            index: 1,
            id: bbr_domain::FolderId(2),
            title: "Reading".into(),
            count: 0,
            expanded: false,
            reveal_count: false,
        },
    ]
}

#[test]
fn folder_ids_are_deterministic_per_index() {
    assert_eq!(folder_row_id(0).to_string(), "bbr-folder-row-0");
    assert_eq!(folder_name_id(3).to_string(), "bbr-folder-name-3");
    assert_eq!(folder_count_id(3).to_string(), "bbr-folder-count-3");
    assert_eq!(folder_toggle_id(3).to_string(), "bbr-folder-toggle-3");
    assert_eq!(folder_item_id(1, 2).to_string(), "bbr-folder-item-1-2");
    assert_eq!(
        folder_item_title_id(1, 2).to_string(),
        "bbr-folder-item-1-2-title"
    );
    assert_eq!(
        folder_item_favicon_id(1, 2).to_string(),
        "bbr-folder-item-1-2-favicon"
    );
    assert_eq!(
        folder_item_close_id(1, 2).to_string(),
        "bbr-folder-item-1-2-close"
    );
}

#[test]
fn presentation_defaults_to_collapsed_and_toggles_deterministically() {
    let mut presentation = FolderPresentation::default();
    assert!(!presentation.is_expanded(bbr_domain::FolderId(0)));
    assert!(
        !presentation.is_expanded(bbr_domain::FolderId(7)),
        "unknown reads collapsed"
    );
    presentation.toggle(bbr_domain::FolderId(2));
    assert!(presentation.is_expanded(bbr_domain::FolderId(2)));
    presentation.toggle(bbr_domain::FolderId(2));
    assert!(!presentation.is_expanded(bbr_domain::FolderId(2)));
    assert!(
        !presentation.is_expanded(bbr_domain::FolderId(1)),
        "toggling 2 leaves 1 alone"
    );
}

#[test]
fn folder_rows_carry_counts_and_expansion() {
    let mut presentation = FolderPresentation::default();
    presentation.toggle(bbr_domain::FolderId(1));
    let mut rows = rows();
    rows[0].expanded = presentation.is_expanded(bbr_domain::FolderId(1));
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].index, 0);
    assert_eq!(rows[0].title, "Work");
    assert_eq!(rows[0].count, 2);
    assert!(rows[0].expanded);
    assert_eq!(rows[1].title, "Reading");
    assert_eq!(rows[1].count, 0);
    assert!(!rows[1].expanded);
}

#[test]
fn folder_count_reveal_is_hover_or_focus_driven() {
    // §11.11: the count paints only while the pointer hovers the row or the
    // row is the keyboard focus anchor; at rest every count stays quiet
    // (the id remains in the tree either way).
    let mut rows = rows();
    assert!(!rows[0].reveal_count, "quiet at rest");
    assert!(!rows[1].reveal_count);
    rows[0].reveal_count = true;
    assert!(rows[0].reveal_count, "hovered/focused row reveals");
    assert!(!rows[1].reveal_count, "sibling stays quiet");
}

#[test]
fn folders_ids_hide_the_empty_region_and_mirror_expansion() {
    assert!(folders_ids(&[], &[]).is_empty());
    let rows = rows();
    let ids = folders_ids(&rows, &[]);
    assert!(ids.contains(&"bbr-section-folders".to_string()));
    assert!(ids.contains(&"bbr-folders".to_string()));
    assert!(ids.contains(&"bbr-folder-row-0".to_string()));
    assert!(
        !ids.iter().any(|id| id.starts_with("bbr-folder-item")),
        "collapsed folders expose no nested ids"
    );
    let members = [
        FolderMemberVisual {
            folder: bbr_domain::FolderId(1),
            folder_index: 0,
            member: 0,
            title: "Example".into(),
            letter: 'E',
            hue: 0,
            image: None,
        },
        FolderMemberVisual {
            folder: bbr_domain::FolderId(1),
            folder_index: 0,
            member: 1,
            title: "Local".into(),
            letter: 'L',
            hue: 1,
            image: None,
        },
    ];
    let ids = folders_ids(&rows, &members);
    for required in [
        "bbr-folder-item-0-0",
        "bbr-folder-item-0-0-title",
        "bbr-folder-item-0-0-favicon",
        "bbr-folder-item-0-0-close",
        "bbr-folder-item-0-1",
        "bbr-folder-item-0-1-title",
        "bbr-folder-item-0-1-favicon",
        "bbr-folder-item-0-1-close",
    ] {
        assert!(ids.contains(&required.to_string()), "missing {required}");
    }
    assert!(
        !ids.iter().any(|id| id.starts_with("bbr-folder-item-1")),
        "folder 1 exposes no nested ids"
    );
}

#[test]
fn member_visuals_accept_the_shared_resolved_image_record() {
    let image = Arc::new(gpui::Image::from_bytes(
        gpui::ImageFormat::Png,
        b"not decoded until the asset pipeline".to_vec(),
    ));
    let member = FolderMemberVisual {
        folder: bbr_domain::FolderId(1),
        folder_index: 0,
        member: 0,
        title: "Example".into(),
        letter: 'E',
        hue: 0,
        image: Some(image.clone()),
    };
    assert!(member
        .image
        .as_ref()
        .is_some_and(|stored| Arc::ptr_eq(stored, &image)));
}

#[test]
fn member_visuals_carry_folder_order_identity() {
    let members = [
        FolderMemberVisual {
            folder: bbr_domain::FolderId(1),
            folder_index: 0,
            member: 0,
            title: "Example".into(),
            letter: 'E',
            hue: 0,
            image: None,
        },
        FolderMemberVisual {
            folder: bbr_domain::FolderId(1),
            folder_index: 0,
            member: 1,
            title: "Local".into(),
            letter: 'L',
            hue: 1,
            image: None,
        },
    ];
    assert_eq!(members[0].folder, bbr_domain::FolderId(1));
    assert_eq!(members[0].folder_index, 0);
    assert_eq!(members[0].member, 0);
    assert_eq!(members[0].letter, 'E');
    assert_eq!(members[1].member, 1);
    assert_eq!(members[1].letter, 'L');
    assert!((0..6).contains(&members[0].hue));
}
