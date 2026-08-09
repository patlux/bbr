//! Repository contracts for the download-specific persistence path (the
//! settled post-MVP decisions, DESIGN.md §17.7/§17.8/§17.10): the
//! `replace_downloads` round trip, its whole-write invalid rejection and
//! isolation of the other tables, the deterministic persisted order and
//! soft `retry_of` links, and the same-version legacy downloads shape that
//! is refused read-only, discarded as corrupt on initialize, and
//! immediately rewritable. These contracts were split out of the legacy
//! aggregate `repository_contract.rs`; both targets share the narrowly
//! scoped `support` module.

mod support;

use bbr_persistence::{
    DownloadOutcome, DownloadRecord, InvalidSnapshotIssue, PersistenceError, Snapshot,
    StoreDiscardReason, StoreLoad,
};

use support::{download_record, inventory, sample_snapshot, TempDb};

#[test]
fn prior_schema_downloads_shape_is_discarded_and_immediately_rewritable() {
    // The downloads table shape grew within schema version 2 (the settled
    // post-MVP persistence decision added `url`/`filename`/`retry_of`/
    // `attempt`). Under schema v3 that legacy store is simply a prior-schema
    // store: the read-only path refuses it with a typed SchemaMismatch and
    // leaves it untouched, and the write path discards it as a version
    // mismatch — never a partial load — and is immediately rewritable with
    // the current shape. (Same-version structural corruption is proven by
    // the schema_v3_corrupt fixture.)
    let db = TempDb::from_fixture("schema_v2_legacy_downloads");
    let before = inventory(&db.0);
    assert_eq!(
        db.repository().restore_read_only().unwrap_err(),
        PersistenceError::SchemaMismatch {
            found: 2,
            supported: 3
        }
    );
    assert_eq!(inventory(&db.0), before, "read-only never writes");

    let outcome = db.repository().initialize_and_restore().unwrap();
    assert_eq!(outcome.snapshot, Snapshot::default());
    assert_eq!(
        outcome.store,
        StoreLoad::Discarded {
            reason: StoreDiscardReason::VersionMismatch { found: 2 }
        }
    );
    // Immediately rewritable: the full snapshot round-trips with the new
    // downloads shape.
    let snapshot = sample_snapshot(3);
    db.repository().replace_snapshot(&snapshot).unwrap();
    assert_eq!(
        db.repository().restore_read_only().unwrap().snapshot,
        snapshot
    );
}

#[test]
fn replace_downloads_round_trips_only_the_download_history() {
    let db = TempDb::new("replace-downloads");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    // Seed a full snapshot (the store's other tables), then replace only
    // the downloads history through the download-specific write path.
    let snapshot = sample_snapshot(3);
    repository.replace_snapshot(&snapshot).unwrap();

    let records = vec![
        DownloadRecord {
            url: "https://example.test/fail.bin".into(),
            filename: "fail.bin".into(),
            ..download_record(900, DownloadOutcome::Failed, 10, None, 3000)
        },
        DownloadRecord {
            url: "https://example.test/archive.zip".into(),
            filename: "archive.zip".into(),
            retry_of: Some(900),
            attempt: 2,
            ..download_record(901, DownloadOutcome::Cancelled, 12, None, 4000)
        },
    ];
    let stats = repository.replace_downloads(&records).unwrap();
    assert_eq!(stats.committed_transactions, 1);
    assert_eq!(stats.deleted_rows, 1, "the old download row is deleted");
    assert_eq!(stats.inserted_rows, 2);

    // Only the downloads table changed; the rest of the snapshot survives.
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(restored.snapshot.downloads, records);
    assert_eq!(restored.snapshot.profiles, snapshot.profiles);
    assert_eq!(restored.snapshot.windows, snapshot.windows);
    assert_eq!(restored.snapshot.settings, snapshot.settings);
    assert_eq!(restored.snapshot.command_history, snapshot.command_history);

    // Clearing the history is `replace_downloads(&[])`: the controller
    // emptied its history, so the store must too — deleted history cannot
    // return from the repository.
    let clear = repository.replace_downloads(&[]).unwrap();
    assert_eq!(clear.deleted_rows, 2);
    assert_eq!(clear.inserted_rows, 0);
    assert!(repository
        .restore_read_only()
        .unwrap()
        .snapshot
        .downloads
        .is_empty());
}

#[test]
fn replace_downloads_rejects_invalid_records_whole() {
    let db = TempDb::new("replace-downloads-invalid");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let valid = vec![DownloadRecord {
        url: "https://example.test/a.pdf".into(),
        filename: "a.pdf".into(),
        ..download_record(
            700,
            DownloadOutcome::Succeeded,
            4096,
            Some("/tmp/a.pdf"),
            1000,
        )
    }];
    repository.replace_downloads(&valid).unwrap();
    let before = inventory(&db.0);

    // A display filename must never be a path (no `/`).
    let mut invalid = valid.clone();
    invalid[0].filename = "sub/a.pdf".into();
    assert_eq!(
        repository.replace_downloads(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidDownload {
            id: 700,
            detail: "invalid display filename".into()
        })
    );
    assert_eq!(
        inventory(&db.0),
        before,
        "the rejected write changed nothing"
    );

    // An empty filename is not a display name.
    let mut invalid = valid.clone();
    invalid[0].filename = String::new();
    assert!(matches!(
        repository.replace_downloads(&invalid),
        Err(PersistenceError::InvalidSnapshot(
            InvalidSnapshotIssue::InvalidDownload { .. }
        ))
    ));

    // The source URL stays required and bounded.
    let mut invalid = valid.clone();
    invalid[0].url = String::new();
    assert!(matches!(
        repository.replace_downloads(&invalid),
        Err(PersistenceError::InvalidSnapshot(
            InvalidSnapshotIssue::InvalidDownload { .. }
        ))
    ));

    // The attempt count is 1-based.
    let mut invalid = valid.clone();
    invalid[0].attempt = 0;
    assert_eq!(
        repository.replace_downloads(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidDownload {
            id: 700,
            detail: "non-positive attempt count".into()
        })
    );
    assert_eq!(inventory(&db.0), before);

    // A `retry_of` link is a soft reference to a real persisted id.
    let mut invalid = valid.clone();
    invalid[0].retry_of = Some(0);
    assert!(matches!(
        repository.replace_downloads(&invalid),
        Err(PersistenceError::InvalidSnapshot(
            InvalidSnapshotIssue::InvalidDownload { .. }
        ))
    ));
}

#[test]
fn restore_preserves_download_history_order_and_soft_links() {
    // The persisted downloads order is the deterministic terminal order
    // (ascending `position`); restore returns it verbatim so the controller
    // can reassign terminal sequencing without inventing ties.
    let db = TempDb::new("downloads-order");
    let repository = db.repository();
    let records = vec![
        DownloadRecord {
            url: "https://example.test/a.pdf".into(),
            filename: "a.pdf".into(),
            ..download_record(
                700,
                DownloadOutcome::Succeeded,
                4096,
                Some("/tmp/a.pdf"),
                1000,
            )
        },
        DownloadRecord {
            url: "https://example.test/fail.bin".into(),
            filename: "fail.bin".into(),
            ..download_record(701, DownloadOutcome::Failed, 10, None, 2000)
        },
        DownloadRecord {
            url: "https://example.test/archive.zip".into(),
            filename: "archive.zip".into(),
            retry_of: Some(701),
            attempt: 2,
            ..download_record(702, DownloadOutcome::Cancelled, 12, None, 3000)
        },
    ];
    repository.replace_downloads(&records).unwrap();
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(restored.snapshot.downloads, records);
    // A deleted source row leaves the child's soft link dangling but intact
    // (no foreign key): deleting 701 does not cascade to 702.
    repository
        .replace_downloads(&[records[0].clone(), records[2].clone()])
        .unwrap();
    let after = repository.restore_read_only().unwrap();
    assert_eq!(after.snapshot.downloads[1].retry_of, Some(701));
}
