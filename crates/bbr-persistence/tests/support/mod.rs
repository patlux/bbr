//! Shared test-support helpers for the bbr-persistence integration tests
//! (narrowly scoped: temp database lifecycle, byte-for-byte inventory
//! comparison, the full sample snapshot, and the terminal download-record
//! constructor). Several integration targets share this module via `mod
//! support;` so no test-support authority is duplicated; it never reaches
//! production code.
//!
//! Each helper is exercised by at least one target, but not every target
//! uses every helper; `dead_code` is allowed here so a focused target that
//! needs only a subset does not fail the `-D warnings` gates.
#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use bbr_persistence::{
    CollectionRecord, CommandHistoryEntry, DownloadOutcome, DownloadRecord, GlyphName, Profile,
    ProfileHue, RecentlyClosedTab, Repository, SavedPageRecord, SearchSettings, SidebarState,
    Snapshot, Space, SpaceGlyph, SpaceTint, Split, Tab, WindowGeometry, WindowState,
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

pub struct TempDb(pub PathBuf);

impl TempDb {
    pub fn new(name: &str) -> Self {
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

    pub fn from_fixture(name: &str) -> Self {
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

    pub fn repository(&self) -> Repository {
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
pub struct Inventory {
    main: Option<Vec<u8>>,
    wal: Option<Vec<u8>>,
    shm: Option<Vec<u8>>,
    user_version: i64,
    schema: Vec<(String, String, String)>,
    logical: Vec<(String, Vec<Vec<String>>)>,
}

pub fn inventory(path: &Path) -> Inventory {
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
        schema,
        logical,
    }
}

pub fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}{suffix}", path.display()))
}

pub fn sample_snapshot(tab_count: usize) -> Snapshot {
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
            hue: ProfileHue::Cobalt,
            spaces: vec![Space {
                id: 10,
                title: "Main".into(),
                tint: SpaceTint::Pine,
                glyph: SpaceGlyph::Glyph(GlyphName::Code),
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
        settings: SearchSettings::default(),
        downloads: vec![DownloadRecord {
            id: 700,
            outcome: DownloadOutcome::Succeeded,
            size: 4096,
            destination: Some("/tmp/report.pdf".into()),
            terminal_at: 1000,
            url: "https://example.test/report.pdf".into(),
            filename: "report.pdf".into(),
            retry_of: None,
            attempt: 1,
        }],
        command_history: vec![CommandHistoryEntry {
            id: 800,
            title: "Example".into(),
            url: "https://example.test/".into(),
            visit_count: 3,
            last_visited: 2000,
        }],
        // Schema v3: the sample snapshot deliberately seeds no Saved Pages
        // so the legacy write-count arithmetic stays small and exact; the
        // dedicated saved-pages tests build their own records.
        saved_pages: Vec::new(),
        collections: Vec::new(),
    }
}

/// A Saved Page record in persisted shape (schema v3). `canonical_url` is
/// the caller-provided canonical key — persistence treats it as opaque and
/// never re-canonicalizes.
pub fn saved_page_record(
    id: u64,
    canonical_url: &str,
    captured_title: &str,
    user_label: Option<&str>,
    created_at: u64,
    updated_at: u64,
    collection_id: Option<u64>,
) -> SavedPageRecord {
    SavedPageRecord {
        id,
        canonical_url: canonical_url.into(),
        captured_title: captured_title.into(),
        user_label: user_label.map(str::to_owned),
        created_at,
        updated_at,
        collection_id,
    }
}

/// A flat collection record in persisted shape (schema v3).
pub fn collection_record(id: u64, title: &str) -> CollectionRecord {
    CollectionRecord {
        id,
        title: title.into(),
    }
}

/// Seeds a store with the baseline browser snapshot plus one collection and
/// two Saved Pages (one member, one labelled), returning the saved-page
/// records that must survive every isolated write. Shared by the focused
/// saved-pages test modules.
pub fn seeded_saved_pages(db: &TempDb) -> Vec<SavedPageRecord> {
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let mut snapshot = sample_snapshot(3);
    snapshot.collections = vec![collection_record(1, "Work")];
    snapshot.saved_pages = vec![
        saved_page_record(
            1000,
            "https://example.test/alpha",
            "Alpha",
            None,
            100,
            100,
            Some(1),
        ),
        saved_page_record(
            1001,
            "https://example.test/beta",
            "Beta",
            Some("work label"),
            200,
            200,
            None,
        ),
    ];
    repository.replace_snapshot(&snapshot).unwrap();
    snapshot.saved_pages
}

/// A terminal download record in persisted-history shape. The base carries
/// the legacy persisted fields (identity, outcome, size, destination,
/// terminal timestamp) positionally; call sites fill the settled post-MVP
/// persistence facts — source `url`, display `filename`, soft `retry_of`
/// link, and the 1-based `attempt` count — with struct-update syntax so no
/// helper becomes a 9-argument soup.
pub fn download_record(
    id: u64,
    outcome: DownloadOutcome,
    size: u64,
    destination: Option<&str>,
    terminal_at: u64,
) -> DownloadRecord {
    DownloadRecord {
        id,
        outcome,
        size,
        destination: destination.map(str::to_owned),
        terminal_at,
        url: String::new(),
        filename: String::new(),
        retry_of: None,
        attempt: 1,
    }
}
