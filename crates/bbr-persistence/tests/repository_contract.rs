use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use bbr_persistence::{
    InvalidSnapshotIssue, PersistenceError, Profile, ProfileHue, RecentlyClosedTab,
    RecoveryWarning, Repository, SidebarState, Snapshot, Space, Split, Tab, WindowGeometry,
    WindowState,
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

struct TempDb(PathBuf);

impl TempDb {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "bbr-persistence-{name}-{}-{}.sqlite",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(sidecar(&path, "-wal"));
        let _ = fs::remove_file(sidecar(&path, "-shm"));
        Self(path)
    }

    fn from_fixture(name: &str) -> Self {
        let db = Self::new(name);
        let sql = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(format!("{name}.sql")),
        )
        .unwrap();
        let connection = rusqlite::Connection::open(&db.0).unwrap();
        connection.execute_batch(&sql).unwrap();
        drop(connection);
        db
    }

    fn repository(&self) -> Repository {
        Repository::new(&self.0)
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let _ = fs::remove_file(sidecar(&self.0, "-wal"));
        let _ = fs::remove_file(sidecar(&self.0, "-shm"));
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Inventory {
    main: Option<Vec<u8>>,
    wal: Option<Vec<u8>>,
    shm: Option<Vec<u8>>,
    user_version: i64,
    logical: Vec<(String, Vec<Vec<String>>)>,
}

fn inventory(path: &Path) -> Inventory {
    let connection = rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .unwrap();
    let user_version = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    let tables = connection
        .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let logical = tables
        .into_iter()
        .map(|table| {
            let mut statement = connection
                .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
                .unwrap();
            let column_count = statement.column_count();
            let rows = statement
                .query_map([], |row| {
                    (0..column_count)
                        .map(|index| {
                            let value = row.get_ref(index)?;
                            Ok(match value {
                                rusqlite::types::ValueRef::Null => "NULL".into(),
                                rusqlite::types::ValueRef::Integer(value) => value.to_string(),
                                rusqlite::types::ValueRef::Real(value) => {
                                    value.to_bits().to_string()
                                }
                                rusqlite::types::ValueRef::Text(value) => {
                                    String::from_utf8_lossy(value).into_owned()
                                }
                                rusqlite::types::ValueRef::Blob(value) => format!("blob:{value:?}"),
                            })
                        })
                        .collect::<Result<Vec<_>, rusqlite::Error>>()
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, rows)
        })
        .collect();
    drop(connection);
    Inventory {
        main: fs::read(path).ok(),
        wal: fs::read(sidecar(path, "-wal")).ok(),
        shm: fs::read(sidecar(path, "-shm")).ok(),
        user_version,
        logical,
    }
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}{suffix}", path.display()))
}

fn sample_snapshot(tab_count: usize) -> Snapshot {
    let tabs = (0..tab_count)
        .map(|index| Tab {
            id: 100 + index as u64,
            title: format!("Tab {index}"),
            url: format!("https://example.test/{index}"),
            pinned: index < 2,
        })
        .collect::<Vec<_>>();
    Snapshot {
        profiles: vec![Profile {
            id: 1,
            title: "Personal".into(),
            hue: ProfileHue::Blue,
            spaces: vec![Space {
                id: 10,
                title: "Main".into(),
                active_tab: tabs.first().map(|tab| tab.id),
                split: (tabs.len() > 1).then(|| Split {
                    primary_tab: tabs[0].id,
                    secondary_tab: tabs[1].id,
                }),
                tabs,
                recently_closed: vec![RecentlyClosedTab {
                    tab_id: 900,
                    title: String::new(),
                    url: "https://closed.test/".into(),
                    pinned: false,
                    former_position: 3,
                }],
            }],
        }],
        windows: vec![WindowState {
            id: 50,
            geometry: Some(WindowGeometry {
                x: -200.0,
                y: 10.0,
                width: 1080.0,
                height: 700.0,
            }),
            sidebar: SidebarState {
                width: 280.0,
                collapsed: true,
                last_expanded_width: 300.0,
            },
        }],
    }
}

#[test]
fn first_run_and_schema_v0_migrate_then_round_trip() {
    for db in [TempDb::new("first-run"), TempDb::from_fixture("schema_v0")] {
        let initial = db.repository().initialize_and_restore().unwrap();
        assert_eq!(initial.snapshot, Snapshot::default());
        assert!(initial.warnings.is_empty());
        let snapshot = sample_snapshot(3);
        let stats = db.repository().replace_snapshot(&snapshot).unwrap();
        assert_eq!(stats.committed_transactions, 1);
        assert_eq!(stats.sql_statements, 14);
        assert_eq!(stats.inserted_rows, 8);
        assert_eq!(
            db.repository().restore_read_only().unwrap().snapshot,
            snapshot
        );
    }
}

#[test]
fn schema_v1_fixture_restores_without_writes() {
    let db = TempDb::from_fixture("schema_v1");
    let before = inventory(&db.0);
    let outcome = db.repository().restore_read_only().unwrap();
    assert_eq!(outcome.snapshot.profiles.len(), 1);
    assert_eq!(outcome.snapshot.profiles[0].spaces[0].tabs.len(), 2);
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn optional_corruption_projects_all_warnings_in_required_order_without_writes() {
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
        ]
    );
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn structural_corruption_is_fatal_and_unchanged() {
    let db = TempDb::from_fixture("invalid_structural");
    let before = inventory(&db.0);
    assert!(matches!(
        db.repository().restore_read_only(),
        Err(PersistenceError::Corruption(_))
    ));
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn newer_schema_is_rejected_unchanged() {
    let db = TempDb::from_fixture("newer_schema");
    let before = inventory(&db.0);
    assert_eq!(
        db.repository().restore_read_only().unwrap_err(),
        PersistenceError::IncompatibleSchema {
            found: 2,
            supported: 1
        }
    );
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn failed_migration_rolls_back_schema_data_and_version() {
    let db = TempDb::from_fixture("failed_migration");
    let before = inventory(&db.0);
    assert!(matches!(
        db.repository().initialize_and_restore(),
        Err(PersistenceError::Migration(_))
    ));
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn invalid_write_rejects_whole_snapshot() {
    let db = TempDb::new("invalid-write");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let valid = sample_snapshot(3);
    repository.replace_snapshot(&valid).unwrap();
    let before = inventory(&db.0);
    let mut invalid = valid;
    invalid.profiles[0].spaces[0].tabs[0].url = "bad url".into();
    assert_eq!(
        repository.replace_snapshot(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidUrl { tab_id: 100 })
    );
    assert_eq!(inventory(&db.0), before);
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
        (1, 14, 0, 8)
    );
    let second = repository.replace_snapshot(&snapshot).unwrap();
    assert_eq!(
        (
            second.committed_transactions,
            second.sql_statements,
            second.deleted_rows,
            second.inserted_rows
        ),
        (1, 14, 8, 8)
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
