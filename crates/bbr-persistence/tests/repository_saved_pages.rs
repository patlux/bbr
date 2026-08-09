//! Repository contracts for the global Saved Pages store (schema v3, B1
//! packet `saved-pages-portable-persistence`), split by contract group so
//! every module stays below the ~500-line budget: this module owns the
//! record-lifecycle writes (idempotent save with the settled title/label
//! rule, field updates, idempotent remove) plus the timestamp-invariant and
//! true-no-op regressions; the sibling modules own search, collections, and
//! the read/recovery surface.

#[path = "modules/repository_saved_pages_collections.rs"]
mod repository_saved_pages_collections;
#[path = "modules/repository_saved_pages_reads.rs"]
mod repository_saved_pages_reads;
#[path = "modules/repository_saved_pages_search.rs"]
mod repository_saved_pages_search;
mod support;

use bbr_persistence::{
    InvalidSnapshotIssue, PersistenceError, SavedPageSaveOutcome, SavedPageUpdate, StoreLoad,
};

use support::{inventory, saved_page_record, seeded_saved_pages, TempDb};

#[test]
fn store_saved_page_round_trips_every_field_and_is_readable_by_id_and_url() {
    let db = TempDb::new("round-trip");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let record = saved_page_record(
        42,
        "https://example.test/path?q=1#frag",
        "Captured title",
        Some("My label"),
        1000,
        2000,
        None,
    );
    let outcome = repository.store_saved_page(&record).unwrap();
    assert_eq!(
        outcome,
        SavedPageSaveOutcome {
            id: 42,
            existed: false
        }
    );
    assert_eq!(
        repository.saved_page_by_id(42).unwrap(),
        Some(record.clone())
    );
    assert_eq!(
        repository
            .saved_page_by_url("https://example.test/path?q=1#frag")
            .unwrap(),
        Some(record.clone())
    );
    // The full restore preserves every field and the deterministic order.
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(restored.snapshot.saved_pages, vec![record]);
    assert_eq!(restored.store, StoreLoad::Loaded);
}

#[test]
fn canonical_duplicate_returns_existing_id_creates_no_second_row_and_follows_the_title_rule() {
    let db = TempDb::new("duplicate");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();

    // First save creates the record.
    let first = saved_page_record(
        1000,
        "https://example.test/page",
        "First title",
        None,
        100,
        100,
        None,
    );
    assert_eq!(
        repository.store_saved_page(&first).unwrap(),
        SavedPageSaveOutcome {
            id: 1000,
            existed: false
        }
    );

    // Re-saving the same canonical URL returns the SAME BookmarkId, creates
    // no second row, and updates the captured title (no user label exists);
    // created_at is preserved and updated_at advances from the re-save.
    let resave = saved_page_record(
        9999,                       // a different passed id: the existing id must win
        "https://example.com/page", // not a duplicate — different canonical key
        "never written",
        None,
        1,
        1,
        None,
    );
    assert_eq!(
        repository.store_saved_page(&resave).unwrap(),
        SavedPageSaveOutcome {
            id: 9999,
            existed: false
        }
    );
    let duplicate = saved_page_record(
        7777,
        "https://example.test/page",
        "Second title",
        None,
        1, // ignored on the duplicate path
        300,
        None,
    );
    let outcome = repository.store_saved_page(&duplicate).unwrap();
    assert_eq!(
        outcome,
        SavedPageSaveOutcome {
            id: 1000,
            existed: true
        }
    );
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(
        restored.snapshot.saved_pages.len(),
        2,
        "no second row for the duplicate URL"
    );
    let page = restored
        .snapshot
        .saved_pages
        .iter()
        .find(|page| page.canonical_url == "https://example.test/page")
        .unwrap();
    assert_eq!(page.id, 1000);
    assert_eq!(
        page.captured_title, "Second title",
        "title updates when no user label exists"
    );
    assert_eq!(page.created_at, 100, "created_at is preserved on re-save");
    assert_eq!(
        page.updated_at, 300,
        "updated_at advances when the title changes"
    );

    // A byte-identical re-save performs no write at all (idempotent no-op).
    let identical = saved_page_record(
        7777,
        "https://example.test/page",
        "Second title",
        None,
        1,
        300,
        None,
    );
    let before = inventory(&db.0);
    let outcome = repository.store_saved_page(&identical).unwrap();
    assert_eq!(
        outcome,
        SavedPageSaveOutcome {
            id: 1000,
            existed: true
        }
    );
    assert_eq!(
        inventory(&db.0),
        before,
        "identical re-save performs no write"
    );
}

#[test]
fn duplicate_save_never_touches_an_existing_user_label() {
    let db = TempDb::new("duplicate-label");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let first = saved_page_record(
        1000,
        "https://example.test/page",
        "Old title",
        Some("user"),
        100,
        100,
        None,
    );
    repository.store_saved_page(&first).unwrap();
    let before = inventory(&db.0);
    // Re-save with a different title AND a different label: the user label
    // is user-owned display truth — a save never overwrites it, and because
    // a label exists the captured title is left alone too.
    let resave = saved_page_record(
        1000,
        "https://example.test/page",
        "New title",
        Some("different label"),
        100,
        500,
        None,
    );
    repository.store_saved_page(&resave).unwrap();
    assert_eq!(
        inventory(&db.0),
        before,
        "a labelled record is untouched by re-save"
    );
    let page = repository.saved_page_by_id(1000).unwrap().unwrap();
    assert_eq!(page.captured_title, "Old title");
    assert_eq!(page.user_label.as_deref(), Some("user"));
    assert_eq!(page.updated_at, 100);
}

#[test]
fn update_title_and_label_are_idempotent_and_reject_unknown_identities_whole() {
    let db = TempDb::new("update");
    let repository = db.repository();
    seeded_saved_pages(&db);
    let before = inventory(&db.0);

    // Unknown identity rejects whole with the store untouched.
    let update = SavedPageUpdate {
        id: 9999,
        captured_title: Some("x".into()),
        user_label: None,
        updated_at: 1,
    };
    assert_eq!(
        repository.update_saved_page(&update).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::UnknownBookmark { id: 9999 })
    );
    assert_eq!(inventory(&db.0), before);

    // Invalid text rejects whole.
    let update = SavedPageUpdate {
        id: 1000,
        captured_title: Some("\u{0}".into()),
        user_label: None,
        updated_at: 1,
    };
    assert!(matches!(
        repository.update_saved_page(&update).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidSavedPage { .. })
    ));
    assert_eq!(inventory(&db.0), before);

    // Explicit updates apply title and label; updated_at advances.
    repository
        .update_saved_page(&SavedPageUpdate {
            id: 1000,
            captured_title: Some("Renamed".into()),
            user_label: Some(Some("new label".into())),
            updated_at: 500,
        })
        .unwrap();
    let page = repository.saved_page_by_id(1000).unwrap().unwrap();
    assert_eq!(page.captured_title, "Renamed");
    assert_eq!(page.user_label.as_deref(), Some("new label"));
    assert_eq!(page.updated_at, 500);

    // Clearing the label is explicit; a no-change update performs no write.
    repository
        .update_saved_page(&SavedPageUpdate {
            id: 1000,
            captured_title: None,
            user_label: Some(None),
            updated_at: 600,
        })
        .unwrap();
    assert_eq!(
        repository
            .saved_page_by_id(1000)
            .unwrap()
            .unwrap()
            .user_label,
        None
    );
    let before = inventory(&db.0);
    repository
        .update_saved_page(&SavedPageUpdate {
            id: 1000,
            captured_title: None,
            user_label: None,
            updated_at: 600,
        })
        .unwrap();
    assert_eq!(inventory(&db.0), before, "no-op update performs no write");
}

#[test]
fn remove_is_idempotent_and_recompacts_dense_positions() {
    let db = TempDb::new("remove");
    let repository = db.repository();
    seeded_saved_pages(&db);
    repository.remove_saved_page(1000).unwrap();
    assert!(repository.saved_page_by_id(1000).unwrap().is_none());
    assert_eq!(repository.load_saved_pages().unwrap().len(), 1);

    // The remaining record is the other saved page (position re-compacted:
    // a later restore passes the dense-order check).
    let remaining = repository.load_saved_pages().unwrap();
    assert_eq!(remaining[0].id, 1001);

    // Repeated remove of the same (and of an unknown) identity is a no-op
    // success that performs no further store mutation.
    let before = inventory(&db.0);
    repository.remove_saved_page(1000).unwrap();
    repository.remove_saved_page(7777).unwrap();
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn duplicate_save_clamps_a_stale_updated_at_to_the_existing_created_at() {
    // Regression (review F2): no public mutation may persist
    // `updated_at < created_at` for the stored row. A re-save carries the
    // caller's own timestamps; when they are stale against the existing
    // record's `created_at`, the store must clamp up to that `created_at`
    // (smallest consistent contract) so the monotonic invariant the loader
    // enforces can never be broken and the next write never discards data.
    let db = TempDb::new("duplicate-stale-timestamp");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let first = saved_page_record(
        1000,
        "https://example.test/page",
        "First title",
        None,
        200,
        200,
        None,
    );
    repository.store_saved_page(&first).unwrap();
    // Re-save with a changed title but a stale updated_at (50 < created 200).
    let resave = saved_page_record(
        1000,
        "https://example.test/page",
        "Second title",
        None,
        1,
        50,
        None,
    );
    let outcome = repository.store_saved_page(&resave).unwrap();
    assert_eq!(
        outcome,
        SavedPageSaveOutcome {
            id: 1000,
            existed: true
        }
    );
    let page = repository.saved_page_by_id(1000).unwrap().unwrap();
    assert_eq!(page.captured_title, "Second title");
    assert_eq!(
        page.updated_at, 200,
        "stale re-save updated_at clamps up to the existing created_at"
    );
    assert_eq!(page.created_at, 200);
    // The store must restore cleanly (no invalid-timestamp corruption).
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(restored.store, StoreLoad::Loaded);
}

#[test]
fn update_saved_page_clamps_a_stale_updated_at_to_the_created_at() {
    // Regression (review F2): the field-update path applies the caller's
    // `updated_at`; when it is stale against the stored row's `created_at`
    // the write must clamp up to that `created_at`, never persist an
    // invalid timestamp pair.
    let db = TempDb::new("update-stale-timestamp");
    let repository = db.repository();
    seeded_saved_pages(&db); // record 1000 has created_at = 100
    repository
        .update_saved_page(&SavedPageUpdate {
            id: 1000,
            captured_title: Some("Renamed".into()),
            user_label: None,
            updated_at: 50, // stale: below the stored created_at 100
        })
        .unwrap();
    let page = repository.saved_page_by_id(1000).unwrap().unwrap();
    assert_eq!(page.captured_title, "Renamed");
    assert_eq!(
        page.updated_at, 100,
        "stale update updated_at clamps up to the stored created_at"
    );
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(restored.store, StoreLoad::Loaded);
}

#[test]
fn byte_identical_update_performs_no_write_and_does_not_bump_updated_at() {
    // Regression (review F4): the documented "an update that changes
    // nothing performs no write" rule must hold for byte-identical values
    // too — passing the stored title/label back must not rewrite the row
    // or bump `updated_at`.
    let db = TempDb::new("byte-identical-update");
    let repository = db.repository();
    seeded_saved_pages(&db); // record 1001: title "Beta", label Some("work label")
    let before = inventory(&db.0);
    repository
        .update_saved_page(&SavedPageUpdate {
            id: 1001,
            captured_title: Some("Beta".into()),
            user_label: Some(Some("work label".into())),
            updated_at: 9000, // must not be applied: nothing changed
        })
        .unwrap();
    assert_eq!(
        inventory(&db.0),
        before,
        "byte-identical update performs no write"
    );
    let page = repository.saved_page_by_id(1001).unwrap().unwrap();
    assert_eq!(
        page.updated_at, 200,
        "updated_at is not bumped for a no-op update"
    );
    // A real update still writes and applies its updated_at.
    repository
        .update_saved_page(&SavedPageUpdate {
            id: 1001,
            captured_title: Some("Beta renamed".into()),
            user_label: None,
            updated_at: 9000,
        })
        .unwrap();
    let page = repository.saved_page_by_id(1001).unwrap().unwrap();
    assert_eq!(page.captured_title, "Beta renamed");
    assert_eq!(page.updated_at, 9000);
}
