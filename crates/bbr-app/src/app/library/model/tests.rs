use super::*;

fn record(id: u64, url: &str, title: &str, updated: u64) -> SavedPageRecord {
    SavedPageRecord {
        id,
        canonical_url: url.into(),
        captured_title: title.into(),
        user_label: None,
        created_at: 1,
        updated_at: updated,
        collection_id: None,
    }
}

fn view(search: &str, sort: LibrarySort, viewport: usize) -> LibraryStateView<'_> {
    LibraryStateView {
        search,
        sort,
        selected: &[],
        remove_confirm: false,
        remove_targets: &[],
        scroll_offset: 0,
        viewport_rows: viewport,
        editing_row: None,
        rename_draft: "",
        collection_action: None,
        collection_draft: "",
        collection_create_staged: false,
        move_sheet_open: false,
    }
}

#[test]
fn search_matches_title_label_and_url_case_insensitively() {
    let mut page = record(1, "https://Example.com/Work", "Captured", 1);
    assert!(record_matches(&page, "captured"));
    assert!(record_matches(&page, "CAPTURED"));
    assert!(record_matches(&page, "work"));
    assert!(record_matches(&page, "example.com"));
    assert!(!record_matches(&page, "missing"));
    page.user_label = Some("Buchhaltung".into());
    assert!(record_matches(&page, "buchhaltung"));
    assert!(record_matches(&page, "BUCHHALTUNG"));
    assert!(!record_matches(&page, "not-there"));
    // The empty query matches everything (the unfiltered state).
    assert!(record_matches(&page, ""));
}

#[test]
fn build_filters_sorts_and_windows_deterministically() {
    // The provider order (most recently updated first) is the
    // `UpdatedNewest` input contract; the builder never re-sorts it.
    let pages = vec![
        record(2, "https://beta.test/", "Beta", 30),
        record(3, "https://gamma.test/", "Gamma", 20),
        record(1, "https://alpha.test/", "Alpha", 10),
    ];
    // UpdatedNewest keeps the provider presentation order.
    let model = build_library_model(&pages, &[], &view("", LibrarySort::default(), 24));
    assert_eq!(model.total, 3);
    assert_eq!(
        model.rows.iter().map(|r| r.bookmark).collect::<Vec<_>>(),
        vec![2, 3, 1]
    );
    // Search filters; the window is bounded over the matches.
    let model = build_library_model(&pages, &[], &view("beta", LibrarySort::default(), 24));
    assert_eq!(model.total, 1);
    assert_eq!(model.rows[0].bookmark, 2);
    assert_eq!(
        model.window,
        LibraryWindow {
            first: 0,
            last: 1,
            anchor: Some(2)
        }
    );
    // No results: total > 0 is false here (0 matches); the no-results
    // state derives from a non-empty total with an empty window.
    let model = build_library_model(&pages, &[], &view("zzz", LibrarySort::default(), 24));
    assert_eq!(model.total, 0);
    assert!(model.rows.is_empty());
}

#[test]
fn sort_orders_apply_the_deterministic_key() {
    let pages = vec![
        record(1, "https://beta.test/", "Beta", 30),
        record(2, "https://alpha.test/", "Alpha", 10),
        record(3, "https://gamma.test/", "Gamma", 20),
    ];
    let model = build_library_model(&pages, &[], &view("", LibrarySort::UpdatedOldest, 24));
    assert_eq!(
        model.rows.iter().map(|r| r.bookmark).collect::<Vec<_>>(),
        vec![2, 3, 1],
        "least recently updated first (the B1 key ascending)"
    );
    let model = build_library_model(&pages, &[], &view("", LibrarySort::TitleAscending, 24));
    assert_eq!(
        model.rows.iter().map(|r| r.bookmark).collect::<Vec<_>>(),
        vec![2, 1, 3],
        "display title ascending"
    );
    let model = build_library_model(&pages, &[], &view("", LibrarySort::TitleDescending, 24));
    assert_eq!(
        model.rows.iter().map(|r| r.bookmark).collect::<Vec<_>>(),
        vec![3, 1, 2],
        "display title descending"
    );
}

#[test]
fn selection_and_collection_membership_flow_into_the_model() {
    let pages = vec![
        record(1, "https://a.test/", "A", 1),
        record(2, "https://b.test/", "B", 2),
    ];
    let collections = vec![CollectionRecord {
        id: 7,
        title: "Work".into(),
    }];
    // Move page 2 into collection 7 (simulating the app mutation).
    let mut pages = pages;
    pages[1].collection_id = Some(7);
    let state = LibraryStateView {
        search: "",
        sort: LibrarySort::default(),
        selected: &[1, 2],
        remove_confirm: true,
        remove_targets: &[2],
        scroll_offset: 0,
        viewport_rows: 24,
        editing_row: None,
        rename_draft: "",
        collection_action: Some((7, LibraryCollectionAction::Rename)),
        collection_draft: "Work",
        collection_create_staged: false,
        move_sheet_open: true,
    };
    let model = build_library_model(&pages, &collections, &state);
    assert!(model.rows.iter().all(|r| r.selected));
    assert_eq!(model.selected, vec![1, 2]);
    assert_eq!(model.collections[0].member_count, 1);
    assert_eq!(model.collections[0].title, "Work");
    assert_eq!(model.rows[0].collection, None);
    assert_eq!(model.rows[1].collection.as_deref(), Some("Work"));
    assert!(model.remove_confirm);
    assert_eq!(model.remove_targets, vec![2]);
    assert_eq!(
        model.collection_action,
        Some((7, LibraryCollectionAction::Rename))
    );
    assert_eq!(model.collection_draft, "Work");
    assert!(model.move_sheet_open);
}

#[test]
fn one_thousand_records_stay_bounded_with_a_stable_anchor() {
    let pages: Vec<SavedPageRecord> = (0..1000)
        .map(|i| {
            record(
                i,
                &format!("https://page-{i}.test/"),
                &format!("Page {i}"),
                i,
            )
        })
        .collect();
    // The window never renders the full collection.
    let model = build_library_model(&pages, &[], &view("", LibrarySort::default(), 24));
    assert_eq!(model.total, 1000);
    assert_eq!(model.rows.len(), 24);
    assert_eq!(
        model.window,
        LibraryWindow {
            first: 0,
            last: 24,
            anchor: Some(0)
        }
    );
    // A scrolled offset windows around the anchor and clamps at the end.
    let state = LibraryStateView {
        search: "",
        sort: LibrarySort::default(),
        selected: &[],
        remove_confirm: false,
        remove_targets: &[],
        scroll_offset: 500,
        viewport_rows: 24,
        editing_row: None,
        rename_draft: "",
        collection_action: None,
        collection_draft: "",
        collection_create_staged: false,
        move_sheet_open: false,
    };
    let model = build_library_model(&pages, &[], &state);
    assert_eq!(model.window.first, 500);
    assert_eq!(model.window.anchor, Some(500));
    assert_eq!(model.rows.first().unwrap().bookmark, 500);
    assert_eq!(model.scroll_offset, 500);
    let state = LibraryStateView {
        scroll_offset: 9999,
        ..LibraryStateView {
            search: "",
            sort: LibrarySort::default(),
            selected: &[],
            remove_confirm: false,
            remove_targets: &[],
            scroll_offset: 0,
            viewport_rows: 24,
            editing_row: None,
            rename_draft: "",
            collection_action: None,
            collection_draft: "",
            collection_create_staged: false,
            move_sheet_open: false,
        }
    };
    let model = build_library_model(&pages, &[], &state);
    assert_eq!(
        model.window,
        LibraryWindow {
            first: 976,
            last: 1000,
            anchor: Some(976)
        }
    );
    assert_eq!(model.viewport_rows, 24);
}

#[test]
fn no_match_search_presents_no_results_never_empty() {
    let pages = vec![
        record(1, "https://alpha.test/", "Alpha", 1),
        record(2, "https://beta.test/", "Beta", 2),
    ];
    // A search that matches nothing keeps the unfiltered record count:
    // the decided no-results state presents (records exist, none
    // match), never the empty state (which means no records at all).
    let model = build_library_model(&pages, &[], &view("zzz", LibrarySort::default(), 24));
    assert_eq!(model.total, 0);
    assert_eq!(model.records_total, 2);
    assert!(model.rows.is_empty());
    assert!(model.has_no_results());
    assert!(!model.is_empty());
    // An empty provider with a query stays the empty state.
    let model = build_library_model(&[], &[], &view("zzz", LibrarySort::default(), 24));
    assert!(model.is_empty());
    assert!(!model.has_no_results());
}

#[test]
fn staged_rename_flows_into_the_model_and_the_visible_ids() {
    let pages = vec![
        record(1, "https://a.test/", "Alpha", 1),
        record(2, "https://b.test/", "Beta", 2),
    ];
    let state = LibraryStateView {
        search: "",
        sort: LibrarySort::default(),
        selected: &[],
        remove_confirm: false,
        remove_targets: &[],
        scroll_offset: 0,
        viewport_rows: 24,
        editing_row: Some(1),
        rename_draft: "Work",
        collection_action: None,
        collection_draft: "",
        collection_create_staged: false,
        move_sheet_open: false,
    };
    let model = build_library_model(&pages, &[], &state);
    assert_eq!(model.editing_row, Some(1));
    assert_eq!(model.rename_draft, "Work");
    // The visible-id mirror advertises the staged input/confirm/cancel
    // ids for the editing row and the normal row ids for every other
    // row — never the rename button of the row being edited.
    let ids = library_visible_ids(&model);
    let editing_row_ids = bbr_ui::components::library_row_ids(1, true);
    for id in editing_row_ids {
        assert!(ids.contains(&id.to_string()), "editing row advertises {id}");
    }
    let plain_row_ids = bbr_ui::components::library_row_ids(2, false);
    for id in plain_row_ids {
        assert!(ids.contains(&id.to_string()), "plain row advertises {id}");
    }
    assert!(
        !ids.contains(&bbr_ui::components::saved_page_row_rename_id(1).to_string()),
        "the editing row never advertises the rename button id"
    );
    assert!(ids.contains(&bbr_ui::components::saved_page_row_rename_id(2).to_string()));
}
