//! Fixture builders for the Saved Pages Library stories (B3 packet
//! `saved-pages-library`, Gallery G2/G3): plain `bbr-ui` DTOs with
//! deterministic, stable values — the §11.16 row-anatomy variants, the
//! flat collection rows, and the full composition state matrix
//! (empty/loading/populated/no results/selection/multi/destructive/long/
//! missing favicon/large 1k/rename). Every fixture is valid for its
//! `bbr-ui` component (proven by the fixture-builder tests below); the
//! gallery owns fixture construction only — every visual decision stays in
//! `bbr-ui`/DESIGN, and no repository/browser/controller/network authority
//! or persistence claim appears here.

use bbr_ui::components::{
    library_row_spec, visible_window, LibraryCollectionSpec, LibraryRowSpec, LibrarySort,
    LibraryStage, LibraryWindow, SavedPagesLibraryModel,
};

/// The deterministic fixture bookmark ids the Library stories render.
pub const LIBRARY_BOOKMARKS: [u64; 4] = [1, 2, 3, 4];

/// The deterministic fixture collection ids.
pub const LIBRARY_COLLECTION_IDS: [u64; 2] = [10, 11];

/// The G3 Library composition state matrix (DESIGN §11.16 + B3 packet):
/// every decided state the surface must present.
pub const LIBRARY_STATES: [&str; 11] = [
    "empty",
    "loading",
    "populated",
    "no results",
    "selection",
    "multi",
    "destructive",
    "long",
    "missing favicon",
    "large 1k",
    "rename",
];

/// The G2 "Library row" story states: the row-level states the row anatomy
/// renders (at rest, selected, staged rename).
pub const LIBRARY_ROW_STATES: [&str; 3] = ["default", "selected", "editing"];

/// The G2 "Library search" story states.
pub const LIBRARY_SEARCH_STATES: [&str; 3] = ["empty", "typed", "focused"];

/// The G2 "Library remove confirm" story states.
pub const LIBRARY_REMOVE_STATES: [&str; 2] = ["single", "multi"];

/// One deterministic §11.16 row spec (the production `library_row_spec`
/// ladder: user label → captured title → hostname/URL fallback, §4
/// mid-ellipsis, §7.2 monogram).
fn row(
    bookmark: u64,
    label: Option<&str>,
    title: &str,
    url: &str,
    collection: Option<&str>,
    selected: bool,
) -> LibraryRowSpec {
    library_row_spec(bookmark, label, title, url, collection, selected)
}

/// The §11.16 row-anatomy fixture set for the G2 "Library row" story: the
/// default row, the collection-labeled row, the long title/URL row, the
/// missing-title (hostname fallback) row, and the missing-favicon
/// (monogram) row — plus the staged-rename row per state. Returns the
/// rows, the row whose rename is staged (`editing`), and the staged draft.
pub fn library_row_fixture(state: &str) -> (Vec<LibraryRowSpec>, Option<u64>, String) {
    let mut rows = vec![
        row(
            LIBRARY_BOOKMARKS[0],
            Some("Work"),
            "Captured title",
            "https://example.com/work",
            None,
            false,
        ),
        row(
            LIBRARY_BOOKMARKS[1],
            None,
            "Recipes",
            "https://cookbook.example.com/",
            Some("Home"),
            false,
        ),
        row(
            LIBRARY_BOOKMARKS[2],
            None,
            "",
            "https://verylongsubdomain.example.com/a/very/long/path?with=a+query",
            None,
            false,
        ),
        row(
            LIBRARY_BOOKMARKS[3],
            None,
            "Missing favicon",
            "https://localhost:8080/",
            None,
            false,
        ),
    ];
    let (editing, draft) = match state {
        "selected" => {
            rows[0].selected = true;
            (None, String::new())
        }
        "editing" => (Some(LIBRARY_BOOKMARKS[0]), "Work".to_string()),
        _ => (None, String::new()),
    };
    (rows, editing, draft)
}

/// The flat collection rows (§11.16, one level only).
pub fn library_collections() -> Vec<LibraryCollectionSpec> {
    vec![
        LibraryCollectionSpec {
            id: LIBRARY_COLLECTION_IDS[0],
            title: "Work".into(),
            member_count: 3,
        },
        LibraryCollectionSpec {
            id: LIBRARY_COLLECTION_IDS[1],
            title: "Recipes".into(),
            member_count: 1,
        },
    ]
}

/// A count-line model: the §11.16 windowed total presentation.
pub fn library_count_model(total: usize) -> SavedPagesLibraryModel {
    SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        search: String::new(),
        sort: LibrarySort::default(),
        rows: Vec::new(),
        total,
        records_total: total,
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
    }
}

/// A populated §11.16 model: ready, the given rows, the flat collections,
/// and the full window (the surface renders exactly these rows).
fn populated(
    rows: Vec<LibraryRowSpec>,
    collections: Vec<LibraryCollectionSpec>,
) -> SavedPagesLibraryModel {
    SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        search: String::new(),
        sort: LibrarySort::default(),
        total: rows.len(),
        records_total: rows.len(),
        window: LibraryWindow {
            first: 0,
            last: rows.len(),
            anchor: rows.first().map(|r| r.bookmark),
        },
        rows,
        collections,
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

/// The long-title/URL rows (§11.16 long data: truncation, never wrap).
fn long_rows() -> Vec<LibraryRowSpec> {
    vec![
        row(
            LIBRARY_BOOKMARKS[0],
            Some("A very long user label that keeps going and going and going and going"),
            "Captured title",
            "https://example.com/work",
            None,
            false,
        ),
        row(
            LIBRARY_BOOKMARKS[1],
            None,
            "A captured title that is extremely long and should be truncated by the row anatomy",
            "https://verylongsubdomain.example.com/a/very/long/path?with=a+query&and=another+parameter",
            Some("Work"),
            false,
        ),
    ]
}

/// The 1k bounded model: 1,000 deterministic records behind the §11.16
/// bounded window — the surface carries exactly the window rows, never the
/// full list (the production `visible_window` math, exercised here).
fn large_1k_model() -> SavedPagesLibraryModel {
    let all: Vec<LibraryRowSpec> = (1..=1000)
        .map(|i| {
            row(
                i,
                None,
                &format!("Saved Page {i}"),
                &format!("https://example.com/page/{i}"),
                None,
                false,
            )
        })
        .collect();
    let (first, last) = visible_window(1000, 24, 0);
    SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        search: String::new(),
        sort: LibrarySort::default(),
        rows: all[first..last].to_vec(),
        total: 1000,
        records_total: 1000,
        window: LibraryWindow {
            first,
            last,
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

/// The full §11.16 composition model for one decided state (the G3 state
/// matrix). The state selector drives the fixture; the surface renders the
/// exact production `bbr-ui` composition.
pub fn library_model(state: &str) -> SavedPagesLibraryModel {
    let base = SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        search: String::new(),
        sort: LibrarySort::default(),
        rows: Vec::new(),
        total: 0,
        records_total: 0,
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
    match state {
        "empty" => base,
        "loading" => SavedPagesLibraryModel {
            stage: LibraryStage::Loading,
            ..base
        },
        "populated" => populated(library_row_fixture("default").0, library_collections()),
        "no results" => SavedPagesLibraryModel {
            search: "zzz".into(),
            records_total: 4,
            ..base
        },
        "selection" => populated(library_row_fixture("selected").0, library_collections()),
        "multi" => {
            let mut rows = library_row_fixture("default").0;
            rows[0].selected = true;
            rows[1].selected = true;
            populated(rows, library_collections())
        }
        "destructive" => SavedPagesLibraryModel {
            remove_confirm: true,
            remove_targets: vec![LIBRARY_BOOKMARKS[0], LIBRARY_BOOKMARKS[1]],
            ..populated(library_row_fixture("default").0, library_collections())
        },
        "long" => populated(long_rows(), library_collections()),
        // Every Library row presents the §7.2 monogram (no favicon blob is
        // ever stored in a Saved Page record); this state makes the
        // missing-favicon presentation explicit.
        "missing favicon" => populated(library_row_fixture("default").0, library_collections()),
        "large 1k" => large_1k_model(),
        "rename" => SavedPagesLibraryModel {
            editing_row: Some(LIBRARY_BOOKMARKS[0]),
            rename_draft: "Work".into(),
            ..populated(library_row_fixture("default").0, library_collections())
        },
        _ => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_fixtures_are_valid_and_stable() {
        for state in LIBRARY_ROW_STATES {
            let (rows, editing, draft) = library_row_fixture(state);
            assert!(!rows.is_empty(), "rows non-empty for {state}");
            for row in &rows {
                assert!(!row.title.is_empty(), "title ladder never empty");
                assert!(!row.url.is_empty(), "url never empty");
                assert!((0..6).contains(&row.favicon.hue), "monogram hue in range");
                assert!(
                    row.favicon.letter.is_ascii_uppercase(),
                    "monogram letter uppercase"
                );
            }
            // The editing state stages exactly row 1 with a draft.
            if state == "editing" {
                assert_eq!(editing, Some(1));
                assert_eq!(draft, "Work");
            } else {
                assert_eq!(editing, None);
                assert!(draft.is_empty());
            }
            // Stable: rebuilding yields the identical fixture.
            assert_eq!(
                (rows, editing, draft),
                library_row_fixture(state),
                "stable per state"
            );
        }
    }

    #[test]
    fn collection_fixtures_are_valid_and_stable() {
        let collections = library_collections();
        assert_eq!(collections.len(), 2);
        assert_eq!(collections[0].id, LIBRARY_COLLECTION_IDS[0]);
        assert_eq!(collections[0].title, "Work");
        assert_eq!(collections[0].member_count, 3);
        assert_eq!(collections, library_collections(), "stable");
    }

    #[test]
    fn composition_states_derive_the_decided_states() {
        // The state matrix covers every decided state the surface must
        // present (DESIGN §11.16 + B3 packet).
        assert_eq!(LIBRARY_STATES.len(), 11);
        for state in LIBRARY_STATES {
            let model = library_model(state);
            match state {
                "empty" => {
                    assert!(model.is_empty());
                    assert!(!model.is_loading());
                    assert!(!model.has_no_results());
                }
                "loading" => {
                    assert!(model.is_loading());
                    assert!(!model.is_empty());
                }
                "no results" => {
                    assert!(model.has_no_results());
                    assert!(!model.is_empty());
                    assert_eq!(model.search, "zzz");
                }
                "destructive" => {
                    assert!(model.remove_confirm);
                    assert_eq!(
                        model.remove_targets,
                        vec![LIBRARY_BOOKMARKS[0], LIBRARY_BOOKMARKS[1]]
                    );
                }
                "rename" => {
                    assert_eq!(model.editing_row, Some(LIBRARY_BOOKMARKS[0]));
                    assert_eq!(model.rename_draft, "Work");
                }
                "large 1k" => {
                    assert_eq!(model.total, 1000);
                    assert_eq!(model.records_total, 1000);
                    // The bounded window: the surface carries exactly the
                    // window rows, never the full 1,000-row list.
                    assert_eq!(model.rows.len(), 24);
                    assert_eq!(model.window.first, 0);
                    assert_eq!(model.window.last, 24);
                    assert_eq!(model.window.anchor, Some(LIBRARY_BOOKMARKS[0]));
                }
                _ => {
                    assert!(!model.is_empty());
                    assert!(!model.is_loading());
                    assert!(!model.has_no_results());
                    assert!(!model.rows.is_empty(), "{state} renders rows");
                }
            }
        }
    }

    #[test]
    fn large_1k_uses_the_production_visible_window_math() {
        let model = library_model("large 1k");
        let (first, last) = visible_window(1000, 24, 0);
        assert_eq!((model.window.first, model.window.last), (first, last));
        // The window rows are the deterministic prefix of the full order.
        assert_eq!(model.rows[0].bookmark, 1);
        assert_eq!(model.rows[23].bookmark, 24);
        // The count line presents the windowed total.
        assert_eq!(model.total, 1000);
    }

    #[test]
    fn count_models_are_valid() {
        for total in [1, 24, 1000] {
            let model = library_count_model(total);
            assert_eq!(model.total, total);
            assert_eq!(model.records_total, total);
            assert!(model.rows.is_empty());
        }
    }
}
