//! Whole-collection replace contracts of the global Saved Pages store
//! (schema v3, B2 packet `saved-pages-actions-command`): the in-memory
//! provider's one-transaction write path. A replace rewrites exactly the
//! Saved Page rows and leaves every other table (collections included)
//! untouched; a duplicate canonical URL or a dangling collection reference
//! rejects whole before any mutation, leaving the store byte-identical —
//! the same combined contract `replace_snapshot` enforces.

use bbr_persistence::{InvalidSnapshotIssue, PersistenceError};

use super::support::{inventory, saved_page_record, seeded_saved_pages, TempDb};

#[test]
fn replace_saved_pages_atomically_replaces_only_the_saved_page_rows() {
    // The B2 whole-collection write path: the in-memory provider replaces
    // the persisted Saved Page collection in one transaction, leaving the
    // collections (and every other table) untouched. A removed record can
    // never return from the store.
    let db = TempDb::new("replace-saved-pages");
    let repository = db.repository();
    let seeded = seeded_saved_pages(&db); // records 1000 (member of collection 1) + 1001
    assert_eq!(seeded.len(), 2);
    let collections_before = repository.load_collections().unwrap();
    assert_eq!(collections_before.len(), 1);

    // Replace with a single record (the provider's deterministic order).
    let replacement = vec![saved_page_record(
        2000,
        "https://example.test/gamma",
        "Gamma",
        None,
        300,
        300,
        Some(1),
    )];
    let stats = repository.replace_saved_pages(&replacement).unwrap();
    assert_eq!(stats.deleted_rows, 2);
    assert_eq!(stats.inserted_rows, 1);
    assert_eq!(
        repository.load_saved_pages().unwrap(),
        replacement,
        "the whole collection was replaced"
    );
    assert_eq!(
        repository.saved_page_by_id(1000).unwrap(),
        None,
        "the removed record is gone"
    );
    assert_eq!(
        repository.saved_page_by_id(2000).unwrap(),
        Some(replacement[0].clone())
    );
    // The collections table is untouched by a saved-pages write.
    assert_eq!(repository.load_collections().unwrap(), collections_before);
    // The full restore agrees with the replaced collection.
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(restored.snapshot.saved_pages, replacement);
}

#[test]
fn replace_saved_pages_rejects_duplicate_urls_and_dangling_collections_whole() {
    // The whole write validates before any mutation: a duplicate canonical
    // URL or a dangling collection reference rejects and leaves the store
    // byte-identical (the same combined contract `replace_snapshot`
    // enforces).
    let db = TempDb::new("replace-saved-pages-reject");
    let repository = db.repository();
    seeded_saved_pages(&db);
    let before = inventory(&db.0);

    let duplicate = vec![
        saved_page_record(1, "https://example.test/dup", "One", None, 1, 1, None),
        saved_page_record(2, "https://example.test/dup", "Two", None, 1, 1, None),
    ];
    assert!(matches!(
        repository.replace_saved_pages(&duplicate),
        Err(PersistenceError::InvalidSnapshot(
            InvalidSnapshotIssue::DuplicateCanonicalUrl { .. }
        ))
    ));
    assert_eq!(
        inventory(&db.0),
        before,
        "rejected write leaves the store intact"
    );

    let dangling = vec![saved_page_record(
        1,
        "https://example.test/ok",
        "Ok",
        None,
        1,
        1,
        Some(999),
    )];
    assert!(matches!(
        repository.replace_saved_pages(&dangling),
        Err(PersistenceError::InvalidSnapshot(
            InvalidSnapshotIssue::UnknownCollection { .. }
        ))
    ));
    assert_eq!(
        inventory(&db.0),
        before,
        "rejected write leaves the store intact"
    );
}
