use super::*;

#[test]
fn visible_window_is_bounded_and_clamped() {
    // Empty / zero viewport never produces a range.
    assert_eq!(visible_window(0, 10, 0), (0, 0));
    assert_eq!(visible_window(1000, 0, 5), (0, 0));
    // Full window on a short collection.
    assert_eq!(visible_window(3, 10, 0), (0, 3));
    // Bounded window into 1,000 rows: never lays out the full list.
    assert_eq!(visible_window(1000, 24, 0), (0, 24));
    assert_eq!(visible_window(1000, 24, 100), (100, 124));
    // The offset clamps to the last visible position; the window still
    // renders only the viewport.
    assert_eq!(visible_window(1000, 24, 5000), (976, 1000));
    assert_eq!(visible_window(1000, 24, 999), (976, 1000));
}

#[test]
fn display_title_ladder_is_label_then_captured_then_host() {
    assert_eq!(
        library_display_title(Some("Work"), "Captured", "https://example.com/a"),
        "Work"
    );
    assert_eq!(
        library_display_title(None, "Captured", "https://example.com/a"),
        "Captured"
    );
    assert_eq!(
        library_display_title(None, "  ", "https://example.com/a"),
        "example.com"
    );
    assert_eq!(library_display_title(None, "", "about:blank"), "about");
}

#[test]
fn row_spec_uses_mid_ellipsis_and_the_monogram() {
    let long = "https://verylongsubdomain.example.com/a/very/long/path?with=a+query";
    let row = library_row_spec(7, None, "Title", long, Some("Work"), true);
    assert_eq!(row.bookmark, 7);
    assert_eq!(row.title, "Title");
    assert!(row.url.contains('…'));
    assert!(row.url.chars().count() < long.chars().count());
    assert!(row.url.ends_with("query"), "the tail survives: {}", row.url);
    assert_eq!(row.collection.as_deref(), Some("Work"));
    assert!(row.selected);
    assert_eq!(row.favicon.letter, 'V');
}

#[test]
fn sort_orders_cycle_deterministically_and_compare_titles() {
    let mut sort = LibrarySort::default();
    let order: Vec<&str> = (0..4)
        .map(|_| {
            let label = sort.label();
            sort = sort.next();
            label
        })
        .collect();
    assert_eq!(
        order,
        vec!["Recently updated", "Oldest first", "Title A–Z", "Title Z–A",]
    );
    assert_eq!(sort, LibrarySort::default(), "the cycle closes");
    assert_eq!(
        LibrarySort::TitleAscending.title_cmp("beta", "alpha"),
        std::cmp::Ordering::Greater
    );
    assert_eq!(
        LibrarySort::TitleDescending.title_cmp("beta", "alpha"),
        std::cmp::Ordering::Less
    );
    assert_eq!(
        LibrarySort::UpdatedNewest.title_cmp("beta", "alpha"),
        std::cmp::Ordering::Equal,
        "updated orders use the provider presentation, never a title key"
    );
}

#[test]
fn decided_states_are_derived_from_the_model() {
    let base = SavedPagesLibraryModel {
        stage: LibraryStage::Loading,
        records_total: 0,
        search: String::new(),
        sort: LibrarySort::default(),
        rows: Vec::new(),
        total: 0,
        window: LibraryWindow {
            first: 0,
            last: 0,
            anchor: None,
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
    };
    let loading = base.clone();
    assert!(loading.is_loading());
    assert!(!loading.is_empty());

    let empty = SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        ..base.clone()
    };
    assert!(empty.is_empty());
    assert!(!empty.has_no_results());

    let no_results = SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        total: 1000,
        records_total: 1000,
        rows: Vec::new(),
        ..base.clone()
    };
    assert!(!no_results.is_empty());
    assert!(no_results.has_no_results());
    // A search that matches nothing keeps `records_total` (records
    // exist) while `total` drops to 0 — the decided no-results state
    // still presents.
    let searched_no_match = SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        total: 0,
        records_total: 1000,
        rows: Vec::new(),
        ..base.clone()
    };
    assert!(!searched_no_match.is_empty());
    assert!(searched_no_match.has_no_results());

    let populated = SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        total: 1000,
        records_total: 1000,
        rows: vec![library_row_spec(
            1,
            None,
            "A",
            "https://a.test/",
            None,
            false,
        )],
        ..base
    };
    assert!(!populated.is_empty());
    assert!(!populated.has_no_results());
}

#[test]
fn collection_action_cycle_is_deterministic_and_input_staging_is_derived() {
    // The pointer cycle is rename → remove → none and closes.
    let mut action = LibraryCollectionAction::Rename;
    assert_eq!(action.next(), Some(LibraryCollectionAction::Remove));
    action = LibraryCollectionAction::Remove;
    assert_eq!(action.next(), None);
    // A staged rename or a staged create owns the collection draft
    // (the keyboard authority), a staged remove never does.
    let base = SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        records_total: 1,
        search: String::new(),
        sort: LibrarySort::default(),
        rows: Vec::new(),
        total: 1,
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
        collection_action: Some((7, LibraryCollectionAction::Rename)),
        collection_draft: String::new(),
        collection_create_staged: false,
        move_sheet_open: false,
        scroll_offset: 0,
        viewport_rows: 24,
    };
    assert!(base.collection_input_staged());
    let create = SavedPagesLibraryModel {
        collection_action: None,
        collection_create_staged: true,
        ..base.clone()
    };
    assert!(create.collection_input_staged());
    let remove = SavedPagesLibraryModel {
        collection_action: Some((7, LibraryCollectionAction::Remove)),
        ..base
    };
    assert!(!remove.collection_input_staged(), "remove has no input");
}
