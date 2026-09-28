//! Repository contract tests for the legacy aggregate snapshot surface:
//! store lifecycle (first run, stamps, corruption), round trips, whole-write
//! validation, identity collisions, hues, write counts, and dense restores.
//!
//! The downloads-specific repository contracts (replace_downloads, the
//! downloads round trip/validation/ordering, and the same-version legacy
//! downloads shape) live in `repository_downloads.rs`; both targets share
//! the narrowly scoped `support` module.

#[path = "modules/contract_saved_pages.rs"]
mod contract_saved_pages;
mod support;

use bbr_persistence::{
    CommandHistoryEntry, DownloadOutcome, DownloadRecord, GlyphName, InvalidSnapshotIssue,
    PersistenceError, Profile, ProfileHue, RecentlyClosedTab, RecoveryWarning, SearchEngineId,
    SearchSettings, SidebarState, Snapshot, Space, SpaceGlyph, SpaceTint, StoreDiscardReason,
    StoreLoad,
};

use support::{download_record, inventory, sample_snapshot, TempDb};

#[test]
fn first_run_creates_v4_store_then_round_trips() {
    // A fresh file and an empty stamp-0 database are indistinguishable
    // first runs: both create the v4 schema, load as `Loaded`, and are
    // immediately writable.
    for db in [TempDb::new("first-run"), TempDb::from_fixture("schema_v0")] {
        let initial = db.repository().initialize_and_restore().unwrap();
        assert_eq!(initial.snapshot, Snapshot::default());
        assert!(initial.warnings.is_empty());
        assert_eq!(initial.store, StoreLoad::Loaded);
        let snapshot = sample_snapshot(3);
        let stats = db.repository().replace_snapshot(&snapshot).unwrap();
        assert_eq!(stats.committed_transactions, 1);
        // Schema v4: eleven DELETE statements (the nine v2 tables plus
        // saved_pages and collections) + 11 inserted rows.
        assert_eq!(stats.sql_statements, 22);
        assert_eq!(stats.inserted_rows, 11);
        assert_eq!(
            db.repository().restore_read_only().unwrap().snapshot,
            snapshot
        );
    }
}

#[test]
fn mismatched_stamp_discards_and_is_immediately_rewritable() {
    // A legacy v1 store: the read-only path refuses it untouched; the write
    // path discards it and yields an empty v2 store.
    let db = TempDb::from_fixture("schema_v1");
    let before = inventory(&db.0);
    assert_eq!(
        db.repository().restore_read_only().unwrap_err(),
        PersistenceError::SchemaMismatch {
            found: 1,
            supported: 4
        }
    );
    assert_eq!(inventory(&db.0), before);
    let outcome = db.repository().initialize_and_restore().unwrap();
    assert_eq!(outcome.snapshot, Snapshot::default());
    assert!(outcome.warnings.is_empty());
    assert_eq!(
        outcome.store,
        StoreLoad::Discarded {
            reason: StoreDiscardReason::VersionMismatch { found: 1 }
        }
    );
    // The discarded store is immediately rewritable and round-trips.
    let snapshot = sample_snapshot(3);
    db.repository().replace_snapshot(&snapshot).unwrap();
    assert_eq!(
        db.repository().restore_read_only().unwrap().snapshot,
        snapshot
    );

    // The write path discards a legacy store on its own: replacing the
    // snapshot of a v1 store first discards it, then writes the v2 shape.
    let db = TempDb::from_fixture("schema_v1");
    db.repository().replace_snapshot(&snapshot).unwrap();
    assert_eq!(
        db.repository().restore_read_only().unwrap().snapshot,
        snapshot
    );
}

#[test]
fn newer_stamp_is_refused_read_only_and_discarded_on_write() {
    let db = TempDb::from_fixture("newer_schema");
    let before = inventory(&db.0);
    assert_eq!(
        db.repository().restore_read_only().unwrap_err(),
        PersistenceError::SchemaMismatch {
            found: 5,
            supported: 4
        }
    );
    assert_eq!(inventory(&db.0), before);
    let outcome = db.repository().initialize_and_restore().unwrap();
    assert_eq!(outcome.snapshot, Snapshot::default());
    assert_eq!(
        outcome.store,
        StoreLoad::Discarded {
            reason: StoreDiscardReason::VersionMismatch { found: 5 }
        }
    );
}

#[test]
fn unstamped_legacy_store_is_discarded() {
    // user_version 0 with existing data: a legacy pre-v2 store with no
    // stamp. The write path discards it (no partial load, no migration).
    let db = TempDb::from_fixture("discard_unstamped");
    let outcome = db.repository().initialize_and_restore().unwrap();
    assert_eq!(outcome.snapshot, Snapshot::default());
    assert_eq!(
        outcome.store,
        StoreLoad::Discarded {
            reason: StoreDiscardReason::Unstamped
        }
    );
}

#[test]
fn structural_corruption_is_fatal_read_only_and_discarded_on_write() {
    let db = TempDb::from_fixture("invalid_structural");
    let before = inventory(&db.0);
    assert!(matches!(
        db.repository().restore_read_only(),
        Err(PersistenceError::Corruption(_))
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
fn optional_corruption_projects_all_warnings_without_writes() {
    let db = TempDb::from_fixture("corrupt_optional");
    let before = inventory(&db.0);
    let outcome = db.repository().restore_read_only().unwrap();
    assert_eq!(outcome.snapshot.profiles[0].spaces[0].active_tab, None);
    assert_eq!(outcome.snapshot.profiles[0].spaces[0].split, None);
    assert_eq!(
        outcome.snapshot.profiles[0].spaces[0].recently_closed[0].title,
        ""
    );
    assert_eq!(outcome.snapshot.windows[0].geometry, None);
    assert_eq!(outcome.snapshot.windows[0].sidebar, SidebarState::default());
    // The incoherent settings record falls back to defaults; the valid
    // download and history rows survive the soft recovery.
    assert_eq!(outcome.snapshot.settings, SearchSettings::default());
    assert_eq!(outcome.snapshot.downloads.len(), 1);
    assert_eq!(outcome.snapshot.command_history.len(), 1);
    assert_eq!(
        outcome.warnings,
        vec![
            RecoveryWarning::ActiveTab {
                space_id: 10,
                persisted_tab_id: 999
            },
            RecoveryWarning::Split {
                space_id: 10,
                primary_tab_id: 100,
                secondary_tab_id: 999
            },
            RecoveryWarning::WindowGeometry { window_id: 50 },
            RecoveryWarning::Sidebar { window_id: 50 },
            RecoveryWarning::Settings {
                detail: "invalid custom search template".into()
            },
        ]
    );
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn v3_round_trip_preserves_every_new_field() {
    let db = TempDb::new("v3-round-trip");
    let repository = db.repository();
    let snapshot = Snapshot {
        profiles: vec![
            Profile {
                id: 1,
                title: "Work".into(),
                hue: ProfileHue::Violet,
                spaces: vec![Space {
                    id: 10,
                    title: "Emoji space".into(),
                    tint: SpaceTint::Plum,
                    paper: Default::default(),
                    glyph: SpaceGlyph::Emoji("🌊".into()),
                    tabs: Vec::new(),
                    active_tab: None,
                    recently_closed: Vec::new(),
                    split: None,
                }],
            },
            Profile {
                id: 2,
                title: "Personal".into(),
                hue: ProfileHue::Amber,
                spaces: vec![Space {
                    id: 20,
                    title: "Glyph space".into(),
                    tint: SpaceTint::Ink,
                    paper: Default::default(),
                    glyph: SpaceGlyph::Glyph(GlyphName::Globe),
                    tabs: Vec::new(),
                    active_tab: None,
                    recently_closed: Vec::new(),
                    split: None,
                }],
            },
        ],
        windows: Vec::new(),
        settings: SearchSettings {
            engine: SearchEngineId::Custom,
            custom_template: Some("https://search.example.test/q=%s&x=1".into()),
        },
        downloads: vec![
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
                url: "https://example.test/archive.zip".into(),
                filename: "archive.zip".into(),
                retry_of: Some(700),
                attempt: 2,
                ..download_record(701, DownloadOutcome::Cancelled, 12, None, 2000)
            },
        ],
        command_history: vec![
            CommandHistoryEntry {
                id: 800,
                title: "Alpha".into(),
                url: "https://alpha.test/".into(),
                visit_count: 2,
                last_visited: 500,
            },
            CommandHistoryEntry {
                id: 801,
                title: String::new(),
                url: "https://beta.test/path".into(),
                visit_count: 1,
                last_visited: 1500,
            },
        ],
        // The global Saved Pages/collections round trip moved to
        // `contract_saved_pages` with the other B1-owned contract tests.
        collections: Vec::new(),
        saved_pages: Vec::new(),
    };
    repository.replace_snapshot(&snapshot).unwrap();
    assert_eq!(repository.restore_read_only().unwrap().snapshot, snapshot);
    // And again after a discard: a store that was discarded and recreated
    // empty is immediately writable with the full v4 shape.
    let outcome = repository.initialize_and_restore().unwrap();
    assert_eq!(outcome.store, StoreLoad::Loaded);
    repository.replace_snapshot(&snapshot).unwrap();
    assert_eq!(repository.restore_read_only().unwrap().snapshot, snapshot);
}

#[test]
fn invalid_write_rejects_whole_snapshot() {
    let db = TempDb::new("invalid-write");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let valid = sample_snapshot(3);
    repository.replace_snapshot(&valid).unwrap();
    let before = inventory(&db.0);

    let mut invalid = valid.clone();
    invalid.profiles[0].spaces[0].tabs[0].url = "bad url".into();
    assert_eq!(
        repository.replace_snapshot(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidUrl { tab_id: 100 })
    );
    assert_eq!(inventory(&db.0), before);

    let mut invalid = valid.clone();
    invalid.profiles[0].spaces[0].glyph = SpaceGlyph::Emoji("🙈".into());
    assert_eq!(
        repository.replace_snapshot(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidEmoji { space_id: 10 })
    );
    assert_eq!(inventory(&db.0), before);

    let mut invalid = valid.clone();
    invalid.settings = SearchSettings {
        engine: SearchEngineId::Custom,
        custom_template: None,
    };
    assert_eq!(
        repository.replace_snapshot(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidSettings {
            detail: "custom engine requires a search template".into()
        })
    );
    assert_eq!(inventory(&db.0), before);

    let mut invalid = valid.clone();
    invalid.downloads[0].size = i64::MAX as u64 + 1;
    assert_eq!(
        repository.replace_snapshot(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidDownload {
            id: 700,
            detail: "size exceeds the SQLite integer range".into()
        })
    );
    assert_eq!(inventory(&db.0), before);

    let mut invalid = valid.clone();
    invalid.command_history[0].url = "bad url".into();
    assert_eq!(
        repository.replace_snapshot(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidHistory {
            id: 800,
            detail: "invalid URL".into()
        })
    );
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn recently_closed_identity_collisions_reject_whole_write_across_every_kind() {
    // Table-driven regression: every persisted identity kind — profile, space,
    // live tab, window, and another recently-closed entry — participates in the
    // same global identity set, so a recently-closed `tab_id` that collides
    // with any of them must be rejected with the typed InvalidSnapshot error
    // before any write, and the previously persisted complete inventory must
    // remain byte-for-byte untouched (main, -wal, -shm, user_version, schema,
    // and ordered logical rows).
    let cases = [
        ("profile", 1_u64),
        ("space", 10_u64),
        ("live tab", 100_u64),
        ("window", 50_u64),
        ("another recently closed", 900_u64),
    ];
    for (kind, colliding_id) in cases {
        let db = TempDb::new("closed-collision");
        let repository = db.repository();
        repository.initialize_and_restore().unwrap();
        let valid = sample_snapshot(3);
        repository.replace_snapshot(&valid).unwrap();
        let before = inventory(&db.0);

        let mut invalid = valid.clone();
        let space = &mut invalid.profiles[0].spaces[0];
        space.recently_closed.push(RecentlyClosedTab {
            tab_id: colliding_id,
            title: "Collision".into(),
            url: "https://collision.test/".into(),
            pinned: false,
            former_position: 0,
        });
        assert_eq!(
            repository.replace_snapshot(&invalid).unwrap_err(),
            PersistenceError::InvalidSnapshot(
                InvalidSnapshotIssue::RecentlyClosedIdentityCollision {
                    tab_id: colliding_id
                }
            )
        );
        // The rejected write left the persisted complete inventory unchanged:
        // main/-wal/-shm existence and bytes, user_version, schema, and the
        // ordered logical rows of every table.
        assert_eq!(inventory(&db.0), before, "kind: {kind}");
    }
}

#[test]
fn all_six_profile_hues_round_trip_without_domain_dependency() {
    let db = TempDb::new("hue-round-trip");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let hues = [
        (ProfileHue::Cobalt, "cobalt"),
        (ProfileHue::Violet, "violet"),
        (ProfileHue::Fuchsia, "fuchsia"),
        (ProfileHue::Amber, "amber"),
        (ProfileHue::Terracotta, "terracotta"),
        (ProfileHue::Slate, "slate"),
    ];
    assert_eq!(
        ProfileHue::ALL,
        hues.map(|(hue, _)| hue),
        "assignment order stays aligned with the persisted contract"
    );
    for (position, (hue, expected_name)) in hues.into_iter().enumerate() {
        let snapshot = Snapshot {
            profiles: vec![Profile {
                id: 1 + position as u64,
                title: format!("Hue {position}"),
                hue,
                spaces: Vec::new(),
            }],
            windows: Vec::new(),
            settings: SearchSettings::default(),
            downloads: Vec::new(),
            command_history: Vec::new(),
            saved_pages: Vec::new(),
            collections: Vec::new(),
        };
        repository.replace_snapshot(&snapshot).unwrap();
        let connection = rusqlite::Connection::open(&db.0).unwrap();
        let persisted_name = connection
            .query_row("SELECT hue FROM profiles", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap();
        drop(connection);
        assert_eq!(persisted_name, expected_name, "hue: {hue:?}");
        assert_eq!(
            repository.restore_read_only().unwrap().snapshot,
            snapshot,
            "hue: {hue:?}"
        );
    }
    // A stale legacy persisted name is no longer accepted: a database row
    // carrying the pre-remediation "blue" value fails restore as corruption.
    let connection = rusqlite::Connection::open(&db.0).unwrap();
    connection
        .execute("UPDATE profiles SET hue = 'blue'", [])
        .unwrap();
    drop(connection);
    assert!(matches!(
        db.repository().restore_read_only(),
        Err(PersistenceError::Corruption(_))
    ));
}

#[test]
fn deterministic_write_counts_include_deletes_and_inserts() {
    let db = TempDb::new("write-counts");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let snapshot = sample_snapshot(3);
    let first = repository.replace_snapshot(&snapshot).unwrap();
    assert_eq!(
        (
            first.committed_transactions,
            first.sql_statements,
            first.deleted_rows,
            first.inserted_rows
        ),
        // Schema v4: eleven DELETE statements + the sample's 11 inserts on
        // the first write (the settings row seeded by schema creation is
        // also deleted), then the same 11 again on the second write.
        (1, 22, 1, 11)
    );
    let second = repository.replace_snapshot(&snapshot).unwrap();
    assert_eq!(
        (
            second.committed_transactions,
            second.sql_statements,
            second.deleted_rows,
            second.inserted_rows
        ),
        (1, 22, 11, 11)
    );
}

#[test]
fn dense_500_live_tab_identities_restore_portably() {
    let db = TempDb::new("dense");
    let repository = db.repository();
    let snapshot = sample_snapshot(500);
    repository.replace_snapshot(&snapshot).unwrap();
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(restored.snapshot.profiles[0].spaces[0].tabs.len(), 500);
    assert_eq!(restored.snapshot.profiles[0].spaces[0].tabs[499].id, 599);
}
