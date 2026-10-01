//! B1-owned repository contract tests (schema v3, packet
//! `saved-pages-portable-persistence`), extracted from `repository_contract`
//! so that target keeps its pre-B1 size: the prior-v2/v3 fixture
//! dispositions (v3/v4 are prior since the v5 kept places), the
//! same-version Saved Pages corruption fixtures
//! (broken shape, duplicate canonical keys, dangling collection
//! references), and the Saved Pages/collections round trip. The v2
//! aggregate-contract tests stay in `repository_contract.rs`.

use bbr_persistence::{
    CommandHistoryEntry, CorruptionIssue, DownloadOutcome, DownloadRecord, Folder,
    PersistenceError, PinnedPage, SearchEngineId, SearchSettings, Snapshot, SpaceGlyph, SpacePaper,
    SpaceTint, StoreDiscardReason, StoreLoad, TintIntensity,
};

use super::support::{
    collection_record, download_record, inventory, sample_snapshot, saved_page_record, TempDb,
};

#[test]
fn schema_v2_fixture_is_refused_read_only_untouched_as_prior_schema() {
    // The v2 fixture is now a prior-schema store: the read-only path
    // refuses it with a typed SchemaMismatch and leaves it byte-for-byte
    // untouched (the current-schema restore-without-writes proof lives in
    // the schema_v5 fixture test below).
    let db = TempDb::from_fixture("schema_v2");
    let before = inventory(&db.0);
    assert_eq!(
        db.repository().restore_read_only().unwrap_err(),
        PersistenceError::SchemaMismatch {
            found: 2,
            supported: 5
        }
    );
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn prior_schema_fixtures_are_discarded_by_the_v5_kept_places_bump() {
    // Schema v4 added the space paper columns (v2 D6/D7); schema v5 adds the
    // kept places of pinned tabs and the tab organization. A complete v3 or
    // v4 store is a prior-schema store under the closed discard-on-mismatch
    // policy: read-only refuses it untouched; the write path discards it
    // and the fresh store round-trips the new fields.
    for (fixture, found) in [("schema_v3", 3), ("schema_v4", 4)] {
        let db = TempDb::from_fixture(fixture);
        let before = inventory(&db.0);
        assert_eq!(
            db.repository().restore_read_only().unwrap_err(),
            PersistenceError::SchemaMismatch {
                found,
                supported: 5
            }
        );
        assert_eq!(inventory(&db.0), before);
        let outcome = db.repository().initialize_and_restore().unwrap();
        assert_eq!(outcome.snapshot, Snapshot::default());
        assert_eq!(
            outcome.store,
            StoreLoad::Discarded {
                reason: StoreDiscardReason::VersionMismatch { found }
            }
        );
        let mut snapshot = sample_snapshot(3);
        snapshot.profiles[0].spaces[0].paper = SpacePaper {
            intensity: TintIntensity::Subtle,
            grain: true,
        };
        db.repository().replace_snapshot(&snapshot).unwrap();
        assert_eq!(
            db.repository().restore_read_only().unwrap().snapshot,
            snapshot
        );
    }
}

#[test]
fn schema_v5_fixture_restores_without_writes() {
    // The current-schema fixture proves the full v5 shape restores as
    // `Loaded` with every field group — including the Saved Pages tables,
    // the kept places of pinned tabs and the organization — and that a
    // matching valid read performs zero writes (byte-identical inventory).
    let db = TempDb::from_fixture("schema_v5");
    let before = inventory(&db.0);
    let outcome = db.repository().restore_read_only().unwrap();
    assert_eq!(outcome.store, StoreLoad::Loaded);
    assert!(outcome.warnings.is_empty());
    let snapshot = outcome.snapshot;
    assert_eq!(snapshot.profiles.len(), 1);
    let space = &snapshot.profiles[0].spaces[0];
    assert_eq!(space.tabs.len(), 4);
    let mail = &space.tabs[1];
    assert!(mail.pinned && mail.closed);
    assert_eq!(
        mail.pinned_page,
        Some(PinnedPage {
            url: "https://mail.test/".into(),
            title: "Mail".into()
        })
    );
    let record = &space.recently_closed[1];
    assert!(record.in_place);
    assert_eq!(
        (record.tab_id, record.url.as_str()),
        (102, "https://mail.test/inbox")
    );
    assert_eq!(
        space.folders,
        vec![Folder {
            id: 5,
            title: "Work".into(),
            tabs: vec![103]
        }]
    );
    assert_eq!(snapshot.favorites, vec![100]);
    assert_eq!(space.tint, SpaceTint::Pine);
    assert_eq!(space.glyph, SpaceGlyph::Emoji("🚀".into()));
    assert_eq!(
        space.paper,
        SpacePaper {
            intensity: TintIntensity::Rich,
            grain: true
        }
    );
    assert_eq!(
        snapshot.settings,
        SearchSettings {
            engine: SearchEngineId::Custom,
            custom_template: Some("https://search.example.test/?q=%s".into()),
        }
    );
    assert_eq!(
        snapshot.downloads,
        vec![
            DownloadRecord {
                url: "https://example.test/report.pdf".into(),
                filename: "report.pdf".into(),
                ..download_record(
                    700,
                    DownloadOutcome::Succeeded,
                    4096,
                    Some("/tmp/report.pdf"),
                    1000
                )
            },
            DownloadRecord {
                url: "https://example.test/archive.zip".into(),
                filename: "archive.zip".into(),
                attempt: 2,
                ..download_record(701, DownloadOutcome::Cancelled, 12, None, 2000)
            },
        ]
    );
    assert_eq!(
        snapshot.command_history,
        vec![CommandHistoryEntry {
            id: 800,
            title: "Example".into(),
            url: "https://example.test/".into(),
            visit_count: 3,
            last_visited: 2000,
        }]
    );
    assert_eq!(
        snapshot.collections,
        vec![
            collection_record(1, "Work"),
            collection_record(2, "Reading"),
        ]
    );
    assert_eq!(
        snapshot.saved_pages,
        vec![
            saved_page_record(
                1000,
                "https://example.test/pinned",
                "Pinned",
                None,
                100,
                100,
                Some(1)
            ),
            saved_page_record(
                1001,
                "https://example.test/regular",
                "Regular",
                Some("My label"),
                200,
                300,
                Some(1)
            ),
            saved_page_record(
                1002,
                "https://archive.test/a",
                "Archive",
                None,
                400,
                400,
                None
            ),
        ]
    );
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn same_version_structural_corruption_is_discarded_on_write() {
    // A v5-stamped store with a broken shape (missing `collections` table)
    // is refused read-only and discarded as `Corrupt` on write — the
    // same-version corruption disposition of the development rule.
    let db = TempDb::from_fixture("schema_v5_corrupt");
    let before = inventory(&db.0);
    assert!(matches!(
        db.repository().restore_read_only(),
        Err(PersistenceError::Schema(_))
    ));
    assert_eq!(inventory(&db.0), before);
    let outcome = db.repository().initialize_and_restore().unwrap();
    assert_eq!(outcome.snapshot, Snapshot::default());
    assert!(matches!(
        outcome.store,
        StoreLoad::Discarded {
            reason: StoreDiscardReason::Corrupt { .. }
        }
    ));
    // Immediately rewritable: a fresh snapshot round-trips on the recreated
    // store without any migration or compatibility path.
    let snapshot = sample_snapshot(3);
    db.repository().replace_snapshot(&snapshot).unwrap();
    assert_eq!(
        db.repository().restore_read_only().unwrap().snapshot,
        snapshot
    );
}

#[test]
fn duplicate_canonical_url_store_is_corruption_read_only_and_discarded_on_write() {
    // Two rows sharing a canonical URL key violate the at-most-one-active-
    // record-per-key invariant: read-only refuses as typed corruption, the
    // write path discards under the development rule.
    let db = TempDb::from_fixture("schema_v5_duplicate_url");
    let before = inventory(&db.0);
    assert!(matches!(
        db.repository().restore_read_only(),
        Err(PersistenceError::Corruption(
            CorruptionIssue::DuplicateCanonicalUrl { id: 1001 }
        ))
    ));
    assert_eq!(inventory(&db.0), before);
    let outcome = db.repository().initialize_and_restore().unwrap();
    assert_eq!(outcome.snapshot, Snapshot::default());
    assert!(matches!(
        outcome.store,
        StoreLoad::Discarded {
            reason: StoreDiscardReason::Corrupt { .. }
        }
    ));
}

#[test]
fn dangling_collection_reference_is_corruption_read_only_and_discarded_on_write() {
    // A saved-page row referencing a collection that does not exist violates
    // the strict membership invariant: the structural foreign-key check
    // refuses it read-only as corruption, and the write path discards it.
    let db = TempDb::from_fixture("schema_v5_collection_integrity");
    let before = inventory(&db.0);
    assert!(matches!(
        db.repository().restore_read_only(),
        Err(PersistenceError::Corruption(
            CorruptionIssue::ForeignKeyViolation
        ))
    ));
    assert_eq!(inventory(&db.0), before);
    let outcome = db.repository().initialize_and_restore().unwrap();
    assert_eq!(outcome.snapshot, Snapshot::default());
    assert!(matches!(
        outcome.store,
        StoreLoad::Discarded {
            reason: StoreDiscardReason::Corrupt { .. }
        }
    ));
}

#[test]
fn saved_pages_and_collections_round_trip_preserve_every_field() {
    // The global Saved Pages shape round-trips through a whole-snapshot
    // write and restore like every other field group, including a labelled
    // record and a collection member (extracted from the v3 aggregate
    // round-trip so `repository_contract` keeps its pre-B1 size).
    let db = TempDb::new("saved-pages-round-trip");
    let repository = db.repository();
    let mut snapshot = sample_snapshot(3);
    snapshot.collections = vec![
        collection_record(1, "Work"),
        collection_record(2, "Reading"),
    ];
    snapshot.saved_pages = vec![
        saved_page_record(
            1000,
            "https://saved.test/a",
            "Alpha",
            None,
            100,
            100,
            Some(1),
        ),
        saved_page_record(
            1001,
            "https://saved.test/b",
            "Beta",
            Some("user label"),
            200,
            300,
            None,
        ),
    ];
    repository.replace_snapshot(&snapshot).unwrap();
    assert_eq!(repository.restore_read_only().unwrap().snapshot, snapshot);
    // And again after a discard: an empty recreated store round-trips the
    // full v3 Saved Pages shape too.
    let outcome = repository.initialize_and_restore().unwrap();
    assert_eq!(outcome.store, StoreLoad::Loaded);
    repository.replace_snapshot(&snapshot).unwrap();
    assert_eq!(repository.restore_read_only().unwrap().snapshot, snapshot);
}
