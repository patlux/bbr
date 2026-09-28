use super::*;

/// The permanent §11.3 space strip always renders its chip/identity ids as
/// part of the footer; the rest of the rail (footer + tabs) stays intact
/// either way.
#[test]
fn compact_rail_always_renders_the_space_strip_ids() {
    // §11.4: the strip is the footer's flex region — its chip/identity ids
    // render unconditionally (there is no revealable spaces region or
    // toggle state).
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
        None,
        dense_all(0),
        None,
        &super::super::U2IdsInput::default(),
    );
    // One chip (and identity child) per space, ids from the persistent
    // space id — never render position or creation-order index.
    for space in &browser.spaces {
        assert!(
            ids.contains(&format!("bbr-space-chip-{}", space.id.0)),
            "missing chip id for space {}: {ids:?}",
            space.id.0
        );
        assert!(
            ids.contains(&format!("bbr-space-identity-{}", space.id.0)),
            "missing identity id for space {}: {ids:?}",
            space.id.0
        );
    }
    for required in [
        "bbr-footer",
        "bbr-downloads",
        "bbr-space-strip",
        "bbr-plus",
        "bbr-tab-list",
    ] {
        assert!(ids.contains(&required.to_string()), "missing {required}");
    }
}

/// Blueprint D20: the `bbr-folder-rename` field id renders only while an
/// inline folder rename is open.
#[test]
fn the_folder_rename_field_id_is_gated_by_the_open_rename() {
    let browser = space_snapshot();
    let ids_with = |input: &super::super::U2IdsInput<'_>| {
        visible_ids_u2(
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
            None,
            dense_all(0),
            None,
            input,
        )
    };
    let rename = bbr_ui::components::folders::FOLDER_RENAME_ID.to_string();
    assert!(!ids_with(&super::super::U2IdsInput::default()).contains(&rename));
    let open = super::super::U2IdsInput {
        folder_renaming: true,
        ..Default::default()
    };
    assert_eq!(
        ids_with(&open).iter().filter(|id| **id == rename).count(),
        1
    );
}

#[test]
fn u2_ids_compose_the_footer_favorites_folders_and_menu_contracts() {
    use bbr_browser::{FavoriteSnapshot, FolderSnapshot, SpaceId};
    use bbr_domain::{FolderId, TabId};
    use bbr_ui::components::folders::FolderPresentation;
    use bbr_ui::components::menu::PlusMenuState;

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
        preview: None,
    };

    // Empty U2 state: the footer is always part of the rail, and so is the
    // Favorites region — its zero-favorites drop band names the region and
    // the gesture (§11.10). Folders and the plus menu stay hidden, and no
    // drag offer renders at rest.
    let ids = super::super::u2::u2_visible_ids(
        &browser,
        &[],
        false,
        false,
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
            "bbr-section-favorites".to_string(),
            "bbr-favorites-empty".to_string(),
            "bbr-favorites-empty-label".to_string(),
            "bbr-favorites-empty-hint".to_string(),
        ]
    );

    // A tab drag in flight at zero favorites: the band is the Favorites
    // target (no extra cell), the ghost row offers pinning, and the proxy
    // follows the pointer.
    let ids = super::super::u2::u2_visible_ids(
        &browser,
        &[],
        true,
        true,
        &[],
        &FolderPresentation::default(),
        &PlusMenuState::default(),
    );
    for required in [
        "bbr-favorites-empty",
        "bbr-pinned-drop-row",
        "bbr-tab-drag-proxy",
    ] {
        assert!(ids.contains(&required.to_string()), "missing {required}");
    }
    assert!(!ids.contains(&"bbr-favorites-drop-cell".to_string()));

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
    expanded.toggle(FolderId(1));
    let plus_menu = PlusMenuState {
        open: true,
        selection: 1,
    };
    let ids = super::super::u2::u2_visible_ids(
        &browser, &favorites, false, false, &folders, &expanded, &plus_menu,
    );
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
    // At rest the partly filled grid shows the glyph slot; a drag swaps it
    // for the drop cell. A pinned source gets no ghost row.
    assert!(ids.contains(&"bbr-favorites-empty".to_string()));
    let dragging = super::super::u2::u2_visible_ids(
        &browser, &favorites, true, false, &folders, &expanded, &plus_menu,
    );
    assert!(dragging.contains(&"bbr-favorites-drop-cell".to_string()));
    assert!(!dragging.contains(&"bbr-favorites-empty".to_string()));
    assert!(!dragging.contains(&"bbr-pinned-drop-row".to_string()));
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
