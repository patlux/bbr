//! Collection contracts of the global Saved Pages store (schema v3, B1
//! packet `saved-pages-portable-persistence`): whole-set replacement with
//! strict membership (removal while referenced rejects whole), membership
//! moves, and the whole-write rejection of duplicate canonical URLs and
//! dangling collection references with byte-identical inventory on
//! rejection.

use bbr_persistence::{InvalidSnapshotIssue, PersistenceError};

use super::support::{
    collection_record, inventory, sample_snapshot, saved_page_record, seeded_saved_pages, TempDb,
};

#[test]
fn collection_rename_and_moves_round_trip_and_remove_while_referenced_rejects_whole() {
    let db = TempDb::new("collections");
    let repository = db.repository();
    seeded_saved_pages(&db);

    // Rename through the whole-set replace; ids and membership survive.
    let stats = repository
        .replace_collections(&[collection_record(1, "Renamed Work")])
        .unwrap();
    assert_eq!(stats.committed_transactions, 1);
    assert_eq!(stats.deleted_rows, 1);
    assert_eq!(stats.inserted_rows, 1);
    let collections = repository.load_collections().unwrap();
    assert_eq!(collections, vec![collection_record(1, "Renamed Work")]);
    // The member record still references collection 1.
    assert_eq!(
        repository
            .saved_page_by_id(1000)
            .unwrap()
            .unwrap()
            .collection_id,
        Some(1)
    );

    // Move a record out of the collection (clear membership).
    repository.move_saved_page_collection(1000, None).unwrap();
    assert_eq!(
        repository
            .saved_page_by_id(1000)
            .unwrap()
            .unwrap()
            .collection_id,
        None
    );

    // Move a record into a collection.
    repository
        .move_saved_page_collection(1001, Some(1))
        .unwrap();
    assert_eq!(
        repository
            .saved_page_by_id(1001)
            .unwrap()
            .unwrap()
            .collection_id,
        Some(1)
    );

    // Removing a collection that is still referenced rejects whole.
    let before = inventory(&db.0);
    assert_eq!(
        repository.replace_collections(&[]).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::UnknownCollection { id: 1 })
    );
    assert_eq!(
        inventory(&db.0),
        before,
        "rejected collection removal leaves the store untouched"
    );

    // Moving to an unknown collection rejects whole.
    let before = inventory(&db.0);
    assert_eq!(
        repository
            .move_saved_page_collection(1001, Some(999))
            .unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::UnknownCollection { id: 999 })
    );
    assert_eq!(inventory(&db.0), before);

    // Moving an unknown record rejects whole.
    let before = inventory(&db.0);
    assert_eq!(
        repository
            .move_saved_page_collection(7777, None)
            .unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::UnknownBookmark { id: 7777 })
    );
    assert_eq!(inventory(&db.0), before);

    // Once membership is cleared, the collection can be removed.
    repository.move_saved_page_collection(1001, None).unwrap();
    repository.replace_collections(&[]).unwrap();
    assert!(repository.load_collections().unwrap().is_empty());
}

#[test]
fn duplicate_canonical_url_and_dangling_collection_cannot_enter_the_committed_store() {
    let db = TempDb::new("invalid-write");
    let repository = db.repository();
    seeded_saved_pages(&db);
    let before = inventory(&db.0);

    // Whole-snapshot write with a duplicate canonical URL rejects whole.
    let mut snapshot = sample_snapshot(3);
    snapshot.saved_pages = vec![
        saved_page_record(1000, "https://example.test/dupe", "A", None, 1, 1, None),
        saved_page_record(1001, "https://example.test/dupe", "B", None, 1, 1, None),
    ];
    assert_eq!(
        repository.replace_snapshot(&snapshot).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::DuplicateCanonicalUrl { id: 1001 })
    );
    assert_eq!(inventory(&db.0), before);

    // Whole-snapshot write with a dangling collection reference rejects
    // whole.
    let mut snapshot = sample_snapshot(3);
    snapshot.saved_pages = vec![saved_page_record(
        1000,
        "https://example.test/a",
        "A",
        None,
        1,
        1,
        Some(999),
    )];
    assert_eq!(
        repository.replace_snapshot(&snapshot).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::UnknownCollection { id: 999 })
    );
    assert_eq!(inventory(&db.0), before);

    // Incremental save with an unknown collection reference rejects whole.
    let bad = saved_page_record(2000, "https://example.test/b", "B", None, 1, 1, Some(999));
    assert_eq!(
        repository.store_saved_page(&bad).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::UnknownCollection { id: 999 })
    );
    assert_eq!(inventory(&db.0), before);

    // Incremental save with an invalid URL shape rejects whole.
    let bad = saved_page_record(2000, "bad url", "B", None, 1, 1, None);
    assert!(matches!(
        repository.store_saved_page(&bad).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidSavedPage { .. })
    ));
    assert_eq!(inventory(&db.0), before);

    // Incremental save with a duplicate id rejects whole (allocator bug).
    let bad = saved_page_record(1000, "https://example.test/new", "B", None, 1, 1, None);
    assert_eq!(
        repository.store_saved_page(&bad).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::DuplicateIdentity { id: 1000 })
    );
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn replace_saved_snapshot_atomically_replaces_both_tables_in_one_transaction() {
    let db = TempDb::new("saved-snapshot");
    let repository = db.repository();
    seeded_saved_pages(&db);

    // One combined write: rename the collection, move a record in, move the
    // other out, and drop one record — the delta the B3 Library batches
    // into a single poke. The order-independent combined contract must
    // accept the new set even though a page now references a freshly
    // renamed collection and the persisted store still references the old
    // one.
    let pages = vec![
        saved_page_record(
            1000,
            "https://example.test/alpha",
            "Alpha",
            None,
            100,
            100,
            None,
        ),
        saved_page_record(
            1001,
            "https://example.test/beta",
            "Beta",
            Some("work label"),
            200,
            300,
            Some(1),
        ),
    ];
    let stats = repository
        .replace_saved_snapshot(&pages, &[collection_record(1, "Renamed Work")])
        .unwrap();
    assert_eq!(stats.committed_transactions, 1);
    assert_eq!(stats.deleted_rows, 3, "one collection + two pages replaced");
    assert_eq!(stats.inserted_rows, 3, "one collection + two pages inserted");

    // Both tables round-trip the combined write exactly.
    assert_eq!(
        repository.load_saved_pages().unwrap(),
        vec![
            saved_page_record(
                1000,
                "https://example.test/alpha",
                "Alpha",
                None,
                100,
                100,
                None,
            ),
            saved_page_record(
                1001,
                "https://example.test/beta",
                "Beta",
                Some("work label"),
                200,
                300,
                Some(1),
            ),
        ]
    );
    assert_eq!(
        repository.load_collections().unwrap(),
        vec![collection_record(1, "Renamed Work")]
    );

    // A dangling membership in the combined write rejects whole and leaves
    // the store byte-identical (the page references a collection absent
    // from the new set).
    let before = inventory(&db.0);
    let dangling = vec![saved_page_record(
        1000,
        "https://example.test/alpha",
        "Alpha",
        None,
        100,
        100,
        Some(1),
    )];
    assert_eq!(
        repository
            .replace_saved_snapshot(&dangling, &[])
            .unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::UnknownCollection { id: 1 })
    );
    assert_eq!(inventory(&db.0), before, "rejected write leaves the store untouched");

    // An empty combined write clears both tables.
    repository.replace_saved_snapshot(&[], &[]).unwrap();
    assert!(repository.load_saved_pages().unwrap().is_empty());
    assert!(repository.load_collections().unwrap().is_empty());
}
