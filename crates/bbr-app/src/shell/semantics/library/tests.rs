use crate::shell::semantics::content::content_node;
use crate::shell::semantics::SemanticContent;
use bbr_testkit::protocol::{SemanticNode, SemanticRole};

use crate::app::palette::CommandSurface;
use crate::shell::semantics::{semantic_tree, SemanticCommand, SemanticFocus, SemanticInput};
use bbr_browser::ControllerSnapshot;
use bbr_ui::components::{
    library_row_spec, LibraryCollectionAction, LibrarySort, LibraryStage, LibraryWindow,
    SavedPagesLibraryModel, LIBRARY_ID, LIBRARY_SEARCH_ID, LIBRARY_SORT_ID,
};

fn model() -> SavedPagesLibraryModel {
    SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        search: String::new(),
        sort: LibrarySort::default(),
        rows: vec![library_row_spec(
            1,
            Some("Work"),
            "Captured",
            "https://example.com/",
            None,
            false,
        )],
        total: 1,
        records_total: 1,
        window: LibraryWindow {
            first: 0,
            last: 1,
            anchor: Some(1),
        },
        collections: Vec::new(),
        remove_confirm: false,
        remove_targets: Vec::new(),
        editing_row: None,
        rename_draft: String::new(),
        selected: Vec::new(),
        collection_action: None,
        collection_draft: String::new(),
        collection_create_staged: false,
        move_sheet_open: false,
        scroll_offset: 0,
        viewport_rows: 24,
    }
}

fn browser() -> ControllerSnapshot {
    ControllerSnapshot {
        active_profile: bbr_web::ProfileId(1),
        active_profile_title: Some("Default".into()),
        active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
        active_space: bbr_browser::SpaceId(1),
        tabs: vec![],
        all_tabs: vec![],
        profiles: vec![bbr_browser::ProfileSnapshot {
            index: 0,
            id: bbr_web::ProfileId(1),
            title: "Default".into(),
            hue: bbr_browser::ProfileHue::Cobalt,
        }],
        spaces: vec![bbr_browser::SpaceSnapshot {
            index: 0,
            id: bbr_browser::SpaceId(1),
            profile: bbr_web::ProfileId(1),
            title: "Default".into(),
            active_tab: None,
            active_view: None,
            active: true,
            tab_count: 0,
        }],
        active_index: None,
        nav: Default::default(),
        web_view_hosted: true,
        engine_sessions: 0,
        active_view: None,
        lifecycle_counts: bbr_browser::LifecycleCounts::default(),
    }
}

fn find<'a>(nodes: &'a [SemanticNode], id: &str) -> Option<&'a SemanticNode> {
    for node in nodes {
        if node.id == id {
            return Some(node);
        }
        if let Some(found) = find(&node.children, id) {
            return Some(found);
        }
    }
    None
}

fn library_input(
    browser: &ControllerSnapshot,
    surface: CommandSurface,
    content: SemanticContent,
) -> SemanticInput<'_> {
    SemanticInput {
        browser,
        focus: SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        command: SemanticCommand {
            mode: "inactive".into(),
            address: "example.com".into(),
            address_identity: "secure".into(),
            text: String::new(),
            hint: None,
            profile_hint: None,
            results: vec![],
        },
        sidebar_visible: false,
        sidebar_width: 280.0,
        divider_label: "bbr-sidebar-divider".into(),
        command_surface: surface,
        rows: &[],
        space_chips: &[],
        favorites: &[],
        favorites_band_targeted: false,
        folder_rows: &[],
        folder_members: &[],
        plus_menu: bbr_ui::components::menu::PlusMenuState::default(),
        content,
    }
}

/// F2: the Library node threads the real surface focus onto the search
/// field — `true` when the `library_handle` owns GPUI focus, `false`
/// otherwise (the render ring and the semantic node never disagree).
#[test]
fn library_search_field_mirrors_the_real_surface_focus() {
    let content = SemanticContent::Library {
        model: model(),
        focused: true,
    };
    let tree = content_node(&content);
    let search = find(std::slice::from_ref(&tree), LIBRARY_SEARCH_ID).expect("search field");
    assert!(
        search.focused,
        "the focused surface focuses the search field"
    );
    let content = SemanticContent::Library {
        model: model(),
        focused: false,
    };
    let tree = content_node(&content);
    let search = find(std::slice::from_ref(&tree), LIBRARY_SEARCH_ID).expect("search field");
    assert!(!search.focused, "an unfocused Library is exposed honestly");
}

/// F1: opening from an active palette leaves only Library semantics —
/// the open path resets the command surface, so the tree exposes the
/// Library and no palette/overlay node. A stale palette surface (the
/// pre-reset leak) is documented as the trapped state the open reset
/// prevents.
#[test]
fn open_library_leaves_only_library_semantics_no_trapped_palette() {
    let browser = browser();
    let content = SemanticContent::Library {
        model: model(),
        focused: true,
    };
    // Post-reset (the F1 open path): only Library semantics remain.
    let tree = semantic_tree(&library_input(
        &browser,
        CommandSurface::None,
        content.clone(),
    ));
    assert!(
        find(&tree, LIBRARY_ID).is_some(),
        "the Library node is present"
    );
    assert!(
        find(&tree, "bbr-palette").is_none() && find(&tree, "bbr-command-overlay").is_none(),
        "no palette/overlay leaks onto the tree"
    );
    // The pre-reset trapped state: a stale palette surface would leak
    // the palette node on top of the Library — exactly what the F1
    // open reset (mode/return reset + portal dismissal + ranking
    // close) prevents.
    let tree = semantic_tree(&library_input(&browser, CommandSurface::Palette, content));
    assert!(
        find(&tree, "bbr-palette").is_some(),
        "a stale palette surface is the trapped state the open reset clears"
    );
}

/// F3: the staged rename input exposes deterministic state and ids —
/// the input Field carrying the draft, the Confirm/Cancel controls, and
/// no rename-button or title/result ids for the row being edited; the
/// input owns surface focus while editing, and the search field
/// yields it.
#[test]
fn staged_rename_exposes_the_input_confirm_and_cancel_deterministically() {
    let mut model = model();
    model.editing_row = Some(1);
    model.rename_draft = "Renamed".into();
    let content = SemanticContent::Library {
        model,
        focused: true,
    };
    let tree = content_node(&content);
    let input = find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-rename-input-1",
    )
    .expect("rename input");
    assert_eq!(input.role, SemanticRole::Field);
    assert_eq!(input.name.as_deref(), Some("Renamed"));
    assert!(input.focused, "the staged input owns surface focus");
    assert!(
        find(
            std::slice::from_ref(&tree),
            "bbr-saved-pages-rename-confirm-1"
        )
        .is_some(),
        "the Confirm control is exposed"
    );
    assert!(
        find(
            std::slice::from_ref(&tree),
            "bbr-saved-pages-rename-cancel-1"
        )
        .is_some(),
        "the Cancel control is exposed"
    );
    assert!(
        find(std::slice::from_ref(&tree), "bbr-saved-pages-rename-1").is_none(),
        "the rename button is not drawn while editing"
    );
    assert!(
        find(std::slice::from_ref(&tree), "bbr-saved-page-result-1").is_none(),
        "the title/result block yields to the staged input"
    );
    let search = find(std::slice::from_ref(&tree), LIBRARY_SEARCH_ID).expect("search field");
    assert!(
        !search.focused,
        "the search field yields focus while a rename is staged"
    );
    // The surface-level Library node and the sort/close controls stay
    // stable while editing.
    let library = find(std::slice::from_ref(&tree), LIBRARY_ID).expect("library surface");
    assert!(library.focused);
    assert!(find(std::slice::from_ref(&tree), LIBRARY_SORT_ID).is_some());
}

/// G4: a search that matches nothing presents the decided no-results
/// state row (`bbr-saved-pages-no-results`) with the query in its name —
/// never the empty state, which means no records at all.
#[test]
fn no_match_search_exposes_the_no_results_state_row() {
    let mut model = model();
    model.search = "zzz".into();
    model.rows = Vec::new();
    model.total = 0;
    model.records_total = 2;
    let content = SemanticContent::Library {
        model,
        focused: true,
    };
    let tree = content_node(&content);
    let no_results =
        find(std::slice::from_ref(&tree), "bbr-saved-pages-no-results").expect("no-results row");
    assert_eq!(no_results.role, SemanticRole::State);
    assert_eq!(
        no_results.name.as_deref(),
        Some("No Saved Pages match “zzz”.")
    );
    assert!(
        find(std::slice::from_ref(&tree), "bbr-saved-pages-empty").is_none(),
        "the empty state never presents while records exist"
    );
}

/// The selection toolbar + move sheet mirror the multi-select surface: the
/// toolbar with its Clear/Remove/Move controls while any record is
/// selected, the move sheet options while open, and neither while the set
/// is empty.
#[test]
fn selection_toolbar_and_move_sheet_expose_deterministic_ids() {
    let mut model = model();
    model.selected = vec![1];
    let content = SemanticContent::Library {
        model: model.clone(),
        focused: true,
    };
    let tree = content_node(&content);
    assert!(find(std::slice::from_ref(&tree), "bbr-saved-pages-selection").is_some());
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-selection-clear"
    )
    .is_some());
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-selection-remove"
    )
    .is_some());
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-selection-move"
    )
    .is_some());
    assert!(
        find(std::slice::from_ref(&tree), "bbr-saved-pages-move-none").is_none(),
        "the sheet is closed at rest"
    );
    // The move sheet opens: per-collection options + the no-collection
    // option appear.
    model.move_sheet_open = true;
    model
        .collections
        .push(bbr_ui::components::LibraryCollectionSpec {
            id: 7,
            title: "Work".into(),
            member_count: 1,
        });
    let content = SemanticContent::Library {
        model: model.clone(),
        focused: true,
    };
    let tree = content_node(&content);
    assert!(find(std::slice::from_ref(&tree), "bbr-saved-pages-move-7").is_some());
    assert!(find(std::slice::from_ref(&tree), "bbr-saved-pages-move-none").is_some());
    // An empty selection renders no toolbar at all.
    model.selected.clear();
    let content = SemanticContent::Library {
        model,
        focused: true,
    };
    let tree = content_node(&content);
    assert!(find(std::slice::from_ref(&tree), "bbr-saved-pages-selection").is_none());
    assert!(find(std::slice::from_ref(&tree), "bbr-saved-pages-move-none").is_none());
}

/// The collection action surface mirrors the staged pointer cycle: the
/// create entry always presents, the staged rename input + Confirm/Cancel
/// present while rename is staged, the staged remove confirm + ok/cancel
/// present while remove is staged, and the staged create form replaces the
/// create entry while staging a new collection.
#[test]
fn collection_action_surface_exposes_stable_ids_per_stage() {
    // The create entry is always present at rest.
    let base = model();
    let content = SemanticContent::Library {
        model: base.clone(),
        focused: true,
    };
    let tree = content_node(&content);
    assert!(
        find(
            std::slice::from_ref(&tree),
            "bbr-saved-pages-collection-create"
        )
        .is_some(),
        "the create entry is always present"
    );
    assert!(
        find(
            std::slice::from_ref(&tree),
            "bbr-saved-pages-collection-create-input"
        )
        .is_none(),
        "the create form is not staged at rest"
    );

    // Staged rename: the input (seeded draft) + Confirm/Cancel.
    let rename = SavedPagesLibraryModel {
        collection_action: Some((7, LibraryCollectionAction::Rename)),
        collection_draft: "Work".into(),
        ..base.clone()
    };
    let content = SemanticContent::Library {
        model: rename,
        focused: true,
    };
    let tree = content_node(&content);
    let input = find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-rename-input-7",
    )
    .expect("rename input");
    assert_eq!(input.role, SemanticRole::Field);
    assert_eq!(input.name.as_deref(), Some("Work"));
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-rename-confirm-7"
    )
    .is_some());
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-rename-cancel-7"
    )
    .is_some());

    // Staged remove: the destructive confirm + ok/cancel.
    let remove = SavedPagesLibraryModel {
        collection_action: Some((7, LibraryCollectionAction::Remove)),
        ..base.clone()
    };
    let content = SemanticContent::Library {
        model: remove,
        focused: true,
    };
    let tree = content_node(&content);
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-remove-confirm-7"
    )
    .is_some());
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-remove-ok-7"
    )
    .is_some());
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-remove-cancel-7"
    )
    .is_some());

    // Staged create: the create form (input + confirm/cancel) replaces the
    // create entry; the search field yields focus to the staged input.
    let create = SavedPagesLibraryModel {
        collection_create_staged: true,
        collection_draft: "Inbox".into(),
        ..base
    };
    let content = SemanticContent::Library {
        model: create,
        focused: true,
    };
    let tree = content_node(&content);
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-create-input"
    )
    .is_some());
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-create-confirm"
    )
    .is_some());
    assert!(find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-create-cancel"
    )
    .is_some());
    assert!(
        find(
            std::slice::from_ref(&tree),
            "bbr-saved-pages-collection-create"
        )
        .is_none(),
        "the staged create form replaces the entry"
    );
}

/// The search field mirrors the real surface focus at rest only: a staged
/// collection input owns the focus ring instead.
#[test]
fn collection_input_owns_surface_focus_like_the_rename_input() {
    let mut model = model();
    model.collection_action = Some((7, LibraryCollectionAction::Rename));
    let content = SemanticContent::Library {
        model,
        focused: true,
    };
    let tree = content_node(&content);
    let search = find(std::slice::from_ref(&tree), LIBRARY_SEARCH_ID).expect("search field");
    assert!(
        !search.focused,
        "the search field yields focus while a collection input is staged"
    );
    let rename_input = find(
        std::slice::from_ref(&tree),
        "bbr-saved-pages-collection-rename-input-7",
    )
    .expect("collection rename input");
    assert!(rename_input.focused);
}
