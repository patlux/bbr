use super::*;

/// The §11.4 compact rail hides the §11.3 space rows (and their chips)
/// until the footer `[Spaces]` control reveals them; the rest of the rail
/// (footer + tabs) stays intact either way.
#[test]
fn compact_spaces_region_hides_space_rows_until_revealed() {
    // §11.4 compact rail: the footer `[Spaces]` control reveals the
    // §11.3 space rows. With the region hidden, no space row/chip id is
    // exposed and the §9 traversal cannot reach them; the rest of the
    // rail (footer + tabs) stays intact.
    let browser = space_snapshot();
    let ids = visible_ids_u2(
        &browser,
        false,
        crate::command_bar::CommandBarMode::Inactive,
        true,
        None,
        None,
        closed(),
        crate::app::command_rank::CommandResultState::Inactive,
        &[],
        &[],
        None,
        dense_all(0),
        None,
        &super::super::U2IdsInput {
            spaces_visible: false,
            ..Default::default()
        },
    );
    assert!(
        !ids.iter().any(|id| id.starts_with("bbr-space-row")
            || id.starts_with("bbr-space-list")
            || id.starts_with("bbr-space-new")
            || id.starts_with("bbr-space-title")
            || id.starts_with("bbr-space-count")),
        "hidden region exposes no space ids: {ids:?}"
    );
    assert!(
        !ids.iter().any(|id| id.starts_with("bbr-profile-chip")),
        "hidden region exposes no chips: {ids:?}"
    );
    for required in ["bbr-footer", "bbr-downloads", "bbr-space-strip", "bbr-plus"] {
        assert!(ids.contains(&required.to_string()), "missing {required}");
    }
    assert!(ids.contains(&"bbr-tab-list".to_string()));
}

#[test]
fn u2_ids_compose_the_footer_favorites_folders_and_menu_contracts() {
    use crate::components::folders::FolderPresentation;
    use crate::components::menu::PlusMenuState;
    use bbr_browser::{FavoriteSnapshot, FolderSnapshot, SpaceId};
    use bbr_domain::{FolderId, TabId};

    // A controller snapshot whose global tab model resolves the folder
    // members (TabId 1 and 2) to the nested row ids.
    let browser = ControllerSnapshot {
        active_profile: bbr_web::ProfileId(1),
        active_profile_title: Some("Default".into()),
        active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
        active_space: SpaceId(1),
        tabs: Vec::new(),
        all_tabs: vec![
            global_tab(1, "https://example.com/a", "Example"),
            global_tab(2, "http://localhost:8080/x", "Local"),
        ],
        profiles: Vec::new(),
        spaces: Vec::new(),
        active_index: None,
        nav: Default::default(),
        web_view_hosted: false,
        engine_sessions: 0,
        active_view: None,
        lifecycle_counts: bbr_browser::LifecycleCounts::default(),
    };

    // Empty U2 state: the footer is always part of the rail; the
    // favorites/folders regions and the plus menu stay hidden.
    let ids = super::super::u2::u2_visible_ids(
        &browser,
        &[],
        &[],
        &FolderPresentation::default(),
        &PlusMenuState::default(),
    );
    assert_eq!(
        ids,
        vec![
            "bbr-footer".to_string(),
            "bbr-downloads".to_string(),
            "bbr-space-strip".to_string(),
            "bbr-plus".to_string(),
        ]
    );

    let favorites = vec![FavoriteSnapshot {
        index: 0,
        tab: TabId(7),
        space: SpaceId(1),
        profile: bbr_web::ProfileId(1),
        title: "Example".into(),
        url: "https://example.com".into(),
    }];
    let folders = vec![FolderSnapshot {
        index: 0,
        id: FolderId(1),
        space: SpaceId(1),
        title: "Work".into(),
        members: vec![TabId(1), TabId(2)],
    }];
    let mut expanded = FolderPresentation::default();
    expanded.toggle(0);
    let plus_menu = PlusMenuState {
        open: true,
        selection: 1,
    };
    let ids =
        super::super::u2::u2_visible_ids(&browser, &favorites, &folders, &expanded, &plus_menu);
    for required in [
        "bbr-footer",
        "bbr-downloads",
        "bbr-space-strip",
        "bbr-plus",
        "bbr-section-favorites",
        "bbr-favorites",
        "bbr-fav-0",
        "bbr-fav-icon-0",
        "bbr-section-folders",
        "bbr-folders",
        "bbr-folder-row-0",
        "bbr-folder-name-0",
        "bbr-folder-count-0",
        "bbr-folder-toggle-0",
        "bbr-folder-item-0-0",
        "bbr-folder-item-0-0-title",
        "bbr-folder-item-0-1-favicon",
        "bbr-folder-item-0-1-close",
        "bbr-plus-menu",
        "bbr-plus-new-space",
        "bbr-plus-new-folder",
    ] {
        assert!(ids.contains(&required.to_string()), "missing {required}");
    }
}

fn global_tab(id: u64, url: &str, title: &str) -> bbr_browser::GlobalTabSnapshot {
    bbr_browser::GlobalTabSnapshot {
        id: bbr_domain::TabId(id),
        space: bbr_browser::SpaceId(1),
        profile: bbr_web::ProfileId(1),
        index: (id - 1) as usize,
        url: url.into(),
        title: title.into(),
        pinned: false,
        web_view: None,
        lifecycle: bbr_browser::TabLifecycle::Warm,
        last_used: 0,
    }
}
