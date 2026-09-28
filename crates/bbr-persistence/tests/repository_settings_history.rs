//! Focused repository contract for the app-owned global stores (Phase 7,
//! command-history provider slice): the single global settings row and the
//! command-history collection. These row methods were added with the
//! command-history provider lane and live here so `repository_contract.rs`
//! stays at its pre-lane size; the support (TempDb/inventory) is a focused
//! copy of the same pattern that file uses.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use bbr_persistence::{
    CommandHistoryEntry, InvalidSnapshotIssue, PersistenceError, Repository, SearchEngineId,
    SearchSettings,
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

struct TempDb(PathBuf);

impl TempDb {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "bbr-persistence-rows-{name}-{}-{}.sqlite",
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

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}{suffix}", path.display()))
}

/// The persisted logical content and the on-disk bytes, so a rejected write
/// can be proven to have left the store untouched.
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
    let schema = connection
        .prepare(
            "SELECT type, name, sql FROM sqlite_schema \
             WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name",
        )
        .unwrap()
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tables = schema
        .iter()
        .filter(|(entry_type, ..)| entry_type == "table")
        .map(|(_, name, _)| name.clone())
        .collect::<Vec<_>>();
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
                        .collect::<Result<Vec<_>, _>>()
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, rows)
        })
        .collect();
    let bytes = |path: &Path| fs::read(path).ok();
    let inventory = Inventory {
        main: bytes(path),
        wal: bytes(&sidecar(path, "-wal")),
        shm: bytes(&sidecar(path, "-shm")),
        user_version,
        logical,
    };
    drop(connection);
    inventory
}

#[test]
fn settings_row_is_a_singleton_across_writes_and_restores() {
    // The schema seeds exactly one settings row with the deterministic
    // defaults; the row-level store upserts id 1 in place (never a second
    // row), and a standalone load returns the stored value.
    let db = TempDb::new("settings-singleton");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    assert_eq!(
        repository.load_settings().unwrap(),
        SearchSettings::default()
    );

    let custom = SearchSettings {
        engine: SearchEngineId::Custom,
        custom_template: Some("https://search.example.test/?q=%s".into()),
    };
    repository.store_settings(&custom).unwrap();
    assert_eq!(repository.load_settings().unwrap(), custom);
    // The full restore path agrees (single row, custom engine).
    assert_eq!(
        repository.restore_read_only().unwrap().snapshot.settings,
        custom
    );

    let google = SearchSettings {
        engine: SearchEngineId::Google,
        custom_template: None,
    };
    repository.store_settings(&google).unwrap();
    assert_eq!(repository.load_settings().unwrap(), google);
    // Still exactly one row after every upsert.
    let connection = rusqlite::Connection::open(&db.0).unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
        .unwrap();
    drop(connection);
    assert_eq!(count, 1);
}

#[test]
fn settings_write_rejects_incoherent_records_whole() {
    let db = TempDb::new("settings-invalid");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let before = inventory(&db.0);

    let custom_without_template = SearchSettings {
        engine: SearchEngineId::Custom,
        custom_template: None,
    };
    assert_eq!(
        repository
            .store_settings(&custom_without_template)
            .unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidSettings {
            detail: "custom engine requires a search template".into()
        })
    );
    let template_without_custom = SearchSettings {
        engine: SearchEngineId::DuckDuckGo,
        custom_template: Some("https://search.example.test/?q=%s".into()),
    };
    assert_eq!(
        repository
            .store_settings(&template_without_custom)
            .unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidSettings {
            detail: "search template without the custom engine".into()
        })
    );
    let malformed_template = SearchSettings {
        engine: SearchEngineId::Custom,
        custom_template: Some("javascript:alert(%s)".into()),
    };
    assert_eq!(
        repository.store_settings(&malformed_template).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidSettings {
            detail: "invalid custom search template".into()
        })
    );
    // The rejected writes left the persisted settings row untouched.
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn command_history_load_writes_order_and_round_trips() {
    let db = TempDb::new("history-order");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    assert!(repository.load_command_history().unwrap().is_empty());

    let entries = vec![
        CommandHistoryEntry {
            id: 800,
            title: "Alpha".into(),
            url: "https://alpha.test/".into(),
            visit_count: 2,
            last_visited: 100,
        },
        CommandHistoryEntry {
            id: 801,
            title: String::new(),
            url: "https://beta.test/path".into(),
            visit_count: 1,
            last_visited: 200,
        },
        CommandHistoryEntry {
            id: 802,
            title: "Gamma".into(),
            url: "https://gamma.test/".into(),
            visit_count: 4,
            last_visited: 300,
        },
    ];
    repository.store_command_history(&entries).unwrap();
    assert_eq!(repository.load_command_history().unwrap(), entries);
    // The full restore path agrees and keeps the caller's order.
    assert_eq!(
        repository
            .restore_read_only()
            .unwrap()
            .snapshot
            .command_history,
        entries
    );

    // Replacement re-orders deterministically (dense position keys).
    let reversed = {
        let mut reversed = entries.clone();
        reversed.reverse();
        reversed
    };
    repository.store_command_history(&reversed).unwrap();
    assert_eq!(repository.load_command_history().unwrap(), reversed);

    // An empty collection clears the table.
    repository.store_command_history(&[]).unwrap();
    assert!(repository.load_command_history().unwrap().is_empty());
}

#[test]
fn command_history_write_rejects_invalid_entries_whole() {
    let db = TempDb::new("history-invalid");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let valid = vec![CommandHistoryEntry {
        id: 800,
        title: "Alpha".into(),
        url: "https://alpha.test/".into(),
        visit_count: 1,
        last_visited: 100,
    }];
    repository.store_command_history(&valid).unwrap();
    let before = inventory(&db.0);

    let mut invalid = valid.clone();
    invalid[0].url = "bad url".into();
    assert_eq!(
        repository.store_command_history(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::InvalidHistory {
            id: 800,
            detail: "invalid URL".into()
        })
    );
    assert_eq!(inventory(&db.0), before);

    let mut invalid = valid.clone();
    invalid.push(invalid[0].clone()); // duplicate identity
    assert!(matches!(
        repository.store_command_history(&invalid).unwrap_err(),
        PersistenceError::InvalidSnapshot(InvalidSnapshotIssue::DuplicateIdentity { id: 800 })
    ));
    assert_eq!(inventory(&db.0), before);
}

#[test]
fn row_writes_discard_a_mismatched_store_then_succeed() {
    // Settings/history row writes run the same store disposition as every
    // write path: a legacy v1 store is discarded and recreated empty first,
    // then the row write lands on the fresh v2 schema.
    let db = TempDb::from_fixture("schema_v1");
    let repository = db.repository();
    assert_eq!(
        repository.load_settings().unwrap_err(),
        PersistenceError::SchemaMismatch {
            found: 1,
            supported: 4
        }
    );
    repository
        .store_settings(&SearchSettings::default())
        .unwrap();
    let entries = vec![CommandHistoryEntry {
        id: 800,
        title: "Alpha".into(),
        url: "https://alpha.test/".into(),
        visit_count: 1,
        last_visited: 100,
    }];
    repository.store_command_history(&entries).unwrap();
    assert_eq!(repository.load_command_history().unwrap(), entries);
}
