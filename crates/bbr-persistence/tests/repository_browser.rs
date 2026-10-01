//! Contract tests for the focused browser-state repository write
//! (`Repository::store_browser_state`, Phase 7 warm-restore slice): the
//! browser tables round-trip with the global stores untouched, dense 500-tab
//! state round-trips, invalid browser state rejects whole with no partial
//! write, the sidebar contract matches the implemented shell range
//! (200..=400 pt, default 300), and a mismatched store is discarded and
//! recreated without migration or backward compatibility.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use bbr_persistence::{
    BrowserState, CommandHistoryEntry, DownloadOutcome, DownloadRecord, GlyphName, Profile,
    ProfileHue, RecentlyClosedTab, RecoveryWarning, Repository, SearchEngineId, SearchSettings,
    SidebarState, Space, SpaceGlyph, SpacePaper, SpaceTint, Split, Tab, TintIntensity,
    WindowGeometry, WindowState,
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

struct TempDb(PathBuf);

impl TempDb {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "bbr-persistence-browser-{name}-{}-{}.sqlite",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(sidecar(&path, "-wal"));
        let _ = fs::remove_file(sidecar(&path, "-shm"));
        Self(path)
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

/// A canonical small browser state: profile 1 with spaces 10/11 (tabs
/// 100..102 and 110), a pinned prefix, an active tab, one recently-closed
/// entry per space, a split, and one window with geometry + sidebar.
fn sample_browser_state() -> BrowserState {
    BrowserState {
        profiles: vec![Profile {
            id: 1,
            title: "Work".into(),
            hue: ProfileHue::Violet,
            spaces: vec![
                Space {
                    id: 10,
                    title: "Main".into(),
                    tint: SpaceTint::Pine,
                    paper: SpacePaper {
                        intensity: TintIntensity::Rich,
                        grain: true,
                    },
                    glyph: SpaceGlyph::Glyph(GlyphName::Code),
                    tabs: vec![
                        Tab {
                            id: 100,
                            title: "Pinned".into(),
                            url: "https://a.example/".into(),
                            pinned: true,
                            closed: false,
                            pinned_page: None,
                        },
                        Tab {
                            id: 101,
                            title: "One".into(),
                            url: "https://b.example/".into(),
                            pinned: false,
                            closed: false,
                            pinned_page: None,
                        },
                        Tab {
                            id: 102,
                            title: "Two".into(),
                            url: "https://c.example/".into(),
                            pinned: false,
                            closed: false,
                            pinned_page: None,
                        },
                    ],
                    active_tab: Some(101),
                    recently_closed: vec![RecentlyClosedTab {
                        tab_id: 900,
                        title: String::new(),
                        url: "https://closed.example/".into(),
                        pinned: false,
                        former_position: 2,
                        in_place: false,
                    }],
                    split: Some(Split {
                        primary_tab: 101,
                        secondary_tab: 102,
                    }),
                    folders: Vec::new(),
                },
                Space {
                    id: 11,
                    title: "Second".into(),
                    tint: SpaceTint::Slate,
                    paper: Default::default(),
                    glyph: SpaceGlyph::Emoji("⚙".into()),
                    tabs: vec![Tab {
                        id: 110,
                        title: "Solo".into(),
                        url: "https://d.example/".into(),
                        pinned: false,
                        closed: false,
                        pinned_page: None,
                    }],
                    active_tab: Some(110),
                    recently_closed: Vec::new(),
                    split: None,
                    folders: Vec::new(),
                },
            ],
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
                width: 300.0,
                collapsed: false,
                last_expanded_width: 300.0,
            },
        }],
        favorites: Vec::new(),
    }
}

/// The global-store rows a browser-state write must never touch.
fn global_rows() -> (
    SearchSettings,
    Vec<DownloadRecord>,
    Vec<CommandHistoryEntry>,
) {
    let settings = SearchSettings {
        engine: SearchEngineId::Google,
        custom_template: None,
    };
    let downloads = vec![DownloadRecord {
        id: 700,
        outcome: DownloadOutcome::Succeeded,
        size: 4096,
        destination: Some("/tmp/report.pdf".into()),
        terminal_at: 1000,
        url: "https://example.test/report.pdf".into(),
        filename: "report.pdf".into(),
        retry_of: None,
        attempt: 1,
    }];
    let history = vec![CommandHistoryEntry {
        id: 800,
        title: "Example".into(),
        url: "https://example.test/".into(),
        visit_count: 3,
        last_visited: 2000,
    }];
    (settings, downloads, history)
}

#[test]
fn browser_state_round_trips_and_global_stores_survive() {
    let db = TempDb::new("round-trip");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    // Seed the global stores first; the browser-state write must preserve them.
    let (settings, _downloads, history) = global_rows();
    repository.store_settings(&settings).unwrap();
    repository.store_command_history(&history).unwrap();

    let state = sample_browser_state();
    let stats = repository.store_browser_state(&state).unwrap();
    assert_eq!(stats.committed_transactions, 1, "one atomic transaction");
    assert_eq!(
        stats.deleted_rows, 0,
        "fresh store: nothing to delete on first write"
    );
    assert_eq!(
        stats.inserted_rows,
        1 + 2 + 4 + 1 + 1 + 1,
        "1 profile + 2 spaces + 4 tabs (3+1) + 1 closed + 1 split + 1 window"
    );

    let outcome = repository.restore_read_only().unwrap();
    assert_eq!(outcome.snapshot.profiles, state.profiles);
    assert_eq!(outcome.snapshot.windows, state.windows);
    assert_eq!(
        outcome.snapshot.windows[0].sidebar,
        state.windows[0].sidebar
    );
    // The global stores are byte-identical after the browser-state write.
    assert_eq!(outcome.snapshot.settings, settings);
    assert!(
        outcome.snapshot.downloads.is_empty(),
        "no download rows seeded in this tree"
    );
    assert_eq!(outcome.snapshot.command_history, history);
    assert!(outcome.warnings.is_empty());
}

#[test]
fn dense_500_tab_browser_state_round_trips() {
    let db = TempDb::new("dense-500");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let tabs = (0..500)
        .map(|index| Tab {
            id: 100 + index,
            title: format!("Tab {index}"),
            url: format!("https://dense{index}.example/"),
            pinned: index < 3,
            closed: false,
            pinned_page: None,
        })
        .collect::<Vec<_>>();
    let state = BrowserState {
        profiles: vec![Profile {
            id: 1,
            title: "Dense".into(),
            hue: ProfileHue::Cobalt,
            spaces: vec![Space {
                id: 10,
                title: "Main".into(),
                tint: SpaceTint::Pine,
                paper: Default::default(),
                glyph: SpaceGlyph::Glyph(GlyphName::Code),
                tabs,
                active_tab: Some(100),
                recently_closed: vec![RecentlyClosedTab {
                    tab_id: 900,
                    title: String::new(),
                    url: "https://closed.example/".into(),
                    pinned: false,
                    former_position: 0,
                    in_place: false,
                }],
                split: None,
                folders: Vec::new(),
            }],
        }],
        windows: vec![WindowState {
            id: 50,
            geometry: None,
            sidebar: SidebarState::default(),
        }],
        favorites: Vec::new(),
    };
    repository.store_browser_state(&state).unwrap();
    let outcome = repository.restore_read_only().unwrap();
    assert_eq!(outcome.snapshot.profiles, state.profiles);
    assert_eq!(outcome.snapshot.profiles[0].spaces[0].tabs.len(), 500);
    assert_eq!(
        outcome.snapshot.profiles[0].spaces[0].tabs[0].id, 100,
        "identity order preserved"
    );
    assert_eq!(outcome.snapshot.profiles[0].spaces[0].tabs[499].id, 599);
    assert_eq!(outcome.snapshot.windows, state.windows);
}

#[test]
fn invalid_browser_state_rejects_whole_without_partial_write() {
    let db = TempDb::new("reject-invalid");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let (settings, _, _) = global_rows();
    repository.store_settings(&settings).unwrap();

    let mut state = sample_browser_state();
    // Duplicate identity across kinds: the tab id collides with a space id.
    state.profiles[0].spaces[0].tabs[0].id = 11;
    assert!(repository.store_browser_state(&state).is_err());
    // A pinned tab after a regular tab breaks the pinned prefix.
    let mut state = sample_browser_state();
    state.profiles[0].spaces[0].tabs[2].pinned = true;
    assert!(repository.store_browser_state(&state).is_err());
    // A non-member active tab.
    let mut state = sample_browser_state();
    state.profiles[0].spaces[0].active_tab = Some(777);
    assert!(repository.store_browser_state(&state).is_err());
    // A split with identical panes.
    let mut state = sample_browser_state();
    state.profiles[0].spaces[0].split = Some(Split {
        primary_tab: 101,
        secondary_tab: 101,
    });
    assert!(repository.store_browser_state(&state).is_err());
    // An invalid URL (ASCII control byte).
    let mut state = sample_browser_state();
    state.profiles[0].spaces[0].tabs[0].url = "https://a.example/\u{7}".into();
    assert!(repository.store_browser_state(&state).is_err());
    // Invalid window geometry (non-positive width) and out-of-range sidebar.
    let mut state = sample_browser_state();
    state.windows[0].geometry = Some(WindowGeometry {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 700.0,
    });
    assert!(repository.store_browser_state(&state).is_err());
    let mut state = sample_browser_state();
    state.windows[0].sidebar = SidebarState {
        width: 500.0,
        collapsed: false,
        last_expanded_width: 500.0,
    };
    assert!(repository.store_browser_state(&state).is_err());

    // Every rejected write left the store untouched: the settings row and an
    // empty browser model survive (no partial insert).
    let outcome = repository.restore_read_only().unwrap();
    assert_eq!(outcome.snapshot.settings, settings);
    assert!(outcome.snapshot.profiles.is_empty());
    assert!(outcome.snapshot.windows.is_empty());
}

#[test]
fn sidebar_contract_matches_the_implemented_shell_range() {
    // The app's implemented sidebar range is 200..=400 pt with a 300 pt
    // default (the shell contract); persistence must restore exactly that.
    assert_eq!(SidebarState::default().width, 300.0);
    assert_eq!(SidebarState::default().last_expanded_width, 300.0);

    let db = TempDb::new("sidebar-range");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    // 400 pt (the app maximum) round-trips without a recovery warning.
    let mut state = sample_browser_state();
    state.windows[0].sidebar = SidebarState {
        width: 400.0,
        collapsed: true,
        last_expanded_width: 320.0,
    };
    repository.store_browser_state(&state).unwrap();
    let outcome = repository.restore_read_only().unwrap();
    assert_eq!(
        outcome.snapshot.windows[0].sidebar,
        SidebarState {
            width: 400.0,
            collapsed: true,
            last_expanded_width: 320.0,
        }
    );
    assert!(outcome.warnings.is_empty());

    // An out-of-range width recovers to the 300 pt default with the typed
    // warning (recovery, not corruption — the browser data is untouched).
    // The write path already rejects an out-of-range width, so the recovery
    // is exercised by corrupting the row directly.
    let db = TempDb::new("sidebar-recovery");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let state = sample_browser_state();
    assert!(repository.store_browser_state(&state).is_ok());
    let path = repository.path().to_path_buf();
    drop(repository);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE windows SET sidebar_width = 900.0, last_expanded_width = 900.0 WHERE id = 50",
            [],
        )
        .unwrap();
    drop(connection);
    let repository = Repository::new(&path);
    let outcome = repository.restore_read_only().unwrap();
    assert_eq!(outcome.snapshot.windows[0].sidebar, SidebarState::default());
    assert_eq!(
        outcome.warnings,
        vec![RecoveryWarning::Sidebar { window_id: 50 }]
    );
    assert_eq!(outcome.snapshot.profiles, state.profiles);
}

#[test]
fn browser_state_write_replaces_previous_browser_state_atomically() {
    let db = TempDb::new("replace");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let first = sample_browser_state();
    repository.store_browser_state(&first).unwrap();

    // A second write with an empty browser state clears the browser tables
    // and keeps the global stores (which were never written here).
    repository
        .store_browser_state(&BrowserState::default())
        .unwrap();
    let outcome = repository.restore_read_only().unwrap();
    assert!(outcome.snapshot.profiles.is_empty());
    assert!(outcome.snapshot.windows.is_empty());
    // The fresh-store defaults are still the single settings row.
    assert_eq!(outcome.snapshot.settings, SearchSettings::default());
}

#[test]
fn browser_state_write_to_mismatched_store_discards_without_migration() {
    let db = TempDb::new("discard-mismatch");
    let path = db.repository().path().to_path_buf();
    // Build a store carrying a stale v1 stamp with data.
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 1).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE profiles (id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, \
             hue TEXT NOT NULL, position INTEGER NOT NULL); \
             INSERT INTO profiles VALUES (1, 'Legacy', 'cobalt', 0);",
        )
        .unwrap();
    drop(connection);

    let repository = Repository::new(&path);
    let state = sample_browser_state();
    repository.store_browser_state(&state).unwrap();
    let outcome = repository.restore_read_only().unwrap();
    assert_eq!(outcome.snapshot.profiles, state.profiles);
    assert_eq!(
        outcome.snapshot.profiles[0].id, 1,
        "the discarded legacy row is gone; the write's browser state restored"
    );
}

#[test]
fn unknown_space_paper_values_are_corruption() {
    // The v4 space paper is closed identity data: an unknown intensity or a
    // non-boolean grain is refused read-only as corruption, never a
    // fallback.
    for update in [
        "UPDATE spaces SET tint_intensity = 'vivid' WHERE id = 10",
        "UPDATE spaces SET grain = 2 WHERE id = 10",
    ] {
        let db = TempDb::new("paper-corrupt");
        let repository = db.repository();
        repository.initialize_and_restore().unwrap();
        repository
            .store_browser_state(&sample_browser_state())
            .unwrap();
        let path = repository.path().to_path_buf();
        drop(repository);
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection.execute(update, []).unwrap();
        drop(connection);
        assert!(
            matches!(
                Repository::new(&path).restore_read_only(),
                Err(bbr_persistence::PersistenceError::Corruption(_))
            ),
            "{update}"
        );
    }
}
