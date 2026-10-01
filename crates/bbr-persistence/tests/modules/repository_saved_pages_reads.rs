//! Read and recovery contracts of the global Saved Pages store (schema v3,
//! B1 packet `saved-pages-portable-persistence`): the checked-in current-
//! schema fixture loads through the focused row methods with zero writes,
//! process-independent reopen preserves every field, browser-state writes
//! leave the Saved Pages tables untouched, and the single-row reads reject
//! externally corrupt rows exactly like the list/restore load (review F3
//! regressions).

use bbr_persistence::{CorruptionIssue, PersistenceError, Repository, SavedPageRecord, StoreLoad};

use super::support::{inventory, saved_page_record, TempDb};

#[test]
fn current_schema_fixture_loads_through_the_row_methods_without_writes() {
    // The checked-in current-schema fixture loads through the focused row
    // methods too — read-only, no writes, every field preserved.
    let db = TempDb::from_fixture("schema_v5");
    let repository = db.repository();
    let before = inventory(&db.0);
    let pages = repository.load_saved_pages().unwrap();
    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0].canonical_url, "https://example.test/pinned");
    assert_eq!(pages[1].user_label.as_deref(), Some("My label"));
    assert_eq!(pages[1].collection_id, Some(1));
    assert_eq!(pages[2].collection_id, None);
    assert_eq!(repository.load_collections().unwrap().len(), 2);
    assert_eq!(
        repository
            .saved_page_by_url("https://archive.test/a")
            .unwrap()
            .unwrap()
            .id,
        1002
    );
    assert_eq!(
        inventory(&db.0),
        before,
        "matching valid reads perform no writes"
    );
}

#[test]
fn reopen_preserves_saved_pages_across_process_independent_close_and_reopen() {
    // A process-independent close/reopen: the TempDb file survives, a brand
    // new Repository (a new connection — no in-memory state) reads it back.
    let db = TempDb::new("reopen");
    {
        let repository = db.repository();
        repository.initialize_and_restore().unwrap();
        let record = saved_page_record(
            5000,
            "https://persist.test/page",
            "Persisted",
            Some("label"),
            100,
            200,
            None,
        );
        repository.store_saved_page(&record).unwrap();
        repository.remove_saved_page(5001).unwrap(); // no-op remove
    }
    // New Repository instance over the same file: the Saved Page survives
    // with every field, no tab/session object involved anywhere.
    let reopened = Repository::new(&db.0);
    let outcome = reopened.restore_read_only().unwrap();
    assert_eq!(outcome.store, StoreLoad::Loaded);
    let record = outcome
        .snapshot
        .saved_pages
        .iter()
        .find(|page| page.id == 5000)
        .expect("the saved page survives reopen");
    assert_eq!(record.canonical_url, "https://persist.test/page");
    assert_eq!(record.captured_title, "Persisted");
    assert_eq!(record.user_label.as_deref(), Some("label"));
    assert_eq!(record.created_at, 100);
    assert_eq!(record.updated_at, 200);
}

#[test]
fn browser_state_write_leaves_saved_pages_untouched() {
    // The browser structural write owns only the browser tables; the global
    // Saved Pages rows survive it byte-for-byte (the isolation invariant).
    let db = TempDb::new("isolation");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let mut snapshot = super::support::sample_snapshot(3);
    snapshot.collections = vec![super::support::collection_record(1, "Work")];
    snapshot.saved_pages = vec![saved_page_record(
        1000,
        "https://example.test/page",
        "Title",
        None,
        100,
        100,
        Some(1),
    )];
    repository.replace_snapshot(&snapshot).unwrap();
    let state = support_browser_state();
    repository.store_browser_state(&state).unwrap();
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(restored.snapshot.saved_pages, snapshot.saved_pages);
    assert_eq!(restored.snapshot.collections, snapshot.collections);
}

#[test]
fn single_row_reads_reject_a_row_with_invalid_timestamps_like_restore() {
    // Regression (review F3): single-row reads must apply the same row-data
    // invariant validation as the list/restore load, so an externally
    // corrupt same-version row (updated_at below created_at) is refused
    // everywhere on the read surface, never silently served.
    let db = TempDb::new("single-row-invalid-ts");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let record = saved_page_record(
        1000,
        "https://example.test/page",
        "Title",
        None,
        100,
        100,
        None,
    );
    repository.store_saved_page(&record).unwrap();
    let connection = rusqlite::Connection::open(&db.0).unwrap();
    connection
        .execute_batch("UPDATE saved_pages SET updated_at = 50 WHERE id = 1000")
        .unwrap();
    drop(connection);
    let expect = |result: Result<Option<SavedPageRecord>, PersistenceError>| {
        matches!(
            result,
            Err(PersistenceError::Corruption(
                CorruptionIssue::InvalidRequiredValue { table, .. }
            )) if table == "saved_pages"
        )
    };
    assert!(
        expect(repository.saved_page_by_id(1000)),
        "by-id refuses the corrupt row"
    );
    assert!(
        expect(repository.saved_page_by_url("https://example.test/page")),
        "by-url refuses the corrupt row"
    );
    assert!(
        matches!(
            repository.restore_read_only(),
            Err(PersistenceError::Corruption(
                CorruptionIssue::InvalidRequiredValue { table, .. }
            )) if table == "saved_pages"
        ),
        "restore refuses the corrupt row"
    );
}

#[test]
fn single_row_reads_reject_an_over_long_title_like_restore() {
    // Regression (review F3): an over-long captured title is row-data
    // corruption that the list load rejects; the single-row reads must
    // reject it identically instead of serving it.
    let db = TempDb::new("single-row-long-title");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let record = saved_page_record(
        1000,
        "https://example.test/page",
        "Title",
        None,
        100,
        100,
        None,
    );
    repository.store_saved_page(&record).unwrap();
    let connection = rusqlite::Connection::open(&db.0).unwrap();
    // The captured-title bound is 1024 bytes (B1 contract mirror); one byte
    // over is row-data corruption.
    let over_long = "x".repeat(1024 + 1);
    connection
        .execute(
            "UPDATE saved_pages SET captured_title = ?1 WHERE id = 1000",
            rusqlite::params![over_long],
        )
        .unwrap();
    drop(connection);
    let expect = |result: Result<Option<SavedPageRecord>, PersistenceError>| {
        matches!(
            result,
            Err(PersistenceError::Corruption(
                CorruptionIssue::InvalidRequiredValue { table, .. }
            )) if table == "saved_pages"
        )
    };
    assert!(
        expect(repository.saved_page_by_id(1000)),
        "by-id refuses the over-long title"
    );
    assert!(
        expect(repository.saved_page_by_url("https://example.test/page")),
        "by-url refuses the over-long title"
    );
    assert!(
        matches!(
            repository.restore_read_only(),
            Err(PersistenceError::Corruption(
                CorruptionIssue::InvalidRequiredValue { table, .. }
            )) if table == "saved_pages"
        ),
        "restore refuses the over-long title"
    );
}

#[test]
fn single_row_reads_reject_a_duplicate_canonical_key_store_like_restore() {
    // Regression (review F3): a store whose rows share a canonical URL key
    // (as in the checked-in `schema_v5_duplicate_url` fixture) is refused
    // read-only by the full load; the single-row reads must refuse it the
    // same way instead of serving one of the duplicate rows.
    let db = TempDb::from_fixture("schema_v5_duplicate_url");
    let repository = db.repository();
    assert_eq!(
        repository
            .saved_page_by_url("https://example.test/dupe")
            .unwrap_err(),
        PersistenceError::Corruption(CorruptionIssue::DuplicateCanonicalUrl { id: 1001 })
    );
    assert_eq!(
        repository.saved_page_by_id(1000).unwrap_err(),
        PersistenceError::Corruption(CorruptionIssue::DuplicateCanonicalUrl { id: 1001 })
    );
    assert_eq!(
        repository.restore_read_only().unwrap_err(),
        PersistenceError::Corruption(CorruptionIssue::DuplicateCanonicalUrl { id: 1001 })
    );
}

/// A minimal browser-state graph for the isolation test (mirrors the
/// `repository_browser` sample but stays local to this module).
fn support_browser_state() -> bbr_persistence::BrowserState {
    use bbr_persistence::{GlyphName, Profile, ProfileHue, Space, SpaceGlyph, SpaceTint, Tab};
    bbr_persistence::BrowserState {
        profiles: vec![Profile {
            id: 1,
            title: "Personal".into(),
            hue: ProfileHue::Cobalt,
            spaces: vec![Space {
                id: 10,
                title: "Main".into(),
                tint: SpaceTint::Pine,
                paper: Default::default(),
                glyph: SpaceGlyph::Glyph(GlyphName::Code),
                tabs: vec![Tab {
                    id: 100,
                    title: "Pinned".into(),
                    url: "https://example.test/pinned".into(),
                    pinned: true,
                    closed: false,
                    pinned_page: None,
                }],
                active_tab: Some(100),
                recently_closed: Vec::new(),
                split: None,
                folders: Vec::new(),
            }],
        }],
        windows: Vec::new(),
        favorites: Vec::new(),
    }
}
