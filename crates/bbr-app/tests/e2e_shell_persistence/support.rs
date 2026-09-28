//! Persistence + warm-restore E2E support: the deterministic fixtures and
//! helpers the `e2e_shell_persistence` scenarios share. Every fixture is
//! written through the actual `bbr-persistence` repository
//! (`Repository::store_browser_state`) — never a test-only in-memory seed
//! or product bypass — and the comparison helpers assert the chrome-first
//! restore contract on the live testkit snapshot.

use bbr_persistence::{
    BrowserState, GlyphName, Profile, ProfileHue, RecentlyClosedTab, Repository, SidebarState,
    Space, SpaceGlyph, SpaceTint, Split, Tab, WindowGeometry, WindowState,
};
use bbr_testkit::protocol::RestoreDisposition;

/// The persistence store file name the app opens at startup (mirrors
/// `crate::store::STORE_FILE_NAME`; the E2E crate cannot import the bin
/// crate's modules).
const STORE_FILE_NAME: &str = "bbr.sqlite";

/// Seed a schema-v2 store at `{profile_dir}/bbr.sqlite` with a browser
/// structural state, exactly as the app's startup restore reads it: through
/// the real repository's transactional browser-state write.
pub fn seed_browser_state(profile_dir: &std::path::Path, state: &BrowserState) {
    std::fs::create_dir_all(profile_dir).unwrap();
    let repository = Repository::new(profile_dir.join(STORE_FILE_NAME));
    repository.initialize_and_restore().unwrap();
    repository.store_browser_state(state).unwrap();
}

/// The canonical dense persisted fixture (mirrors the warm-startup scenario
/// shape): two profiles, three spaces, per-space active tabs, pinned
/// prefixes, one recently-closed entry, and a 500-tab dense space. The
/// active space (the deterministic policy default: the first space with an
/// active tab) is single-pane, so the expected session count is exactly one
/// — never 500 — and the dense space is fully deferred.
pub fn dense_browser_state(page_url: &str) -> BrowserState {
    BrowserState {
        profiles: vec![
            Profile {
                id: 1,
                title: "Work".into(),
                hue: ProfileHue::Cobalt,
                spaces: vec![
                    Space {
                        id: 10,
                        title: "Main".into(),
                        tint: SpaceTint::Cobalt,
                        paper: Default::default(),
                        glyph: SpaceGlyph::Glyph(GlyphName::Code),
                        tabs: vec![
                            Tab {
                                id: 100,
                                title: "Pinned".into(),
                                url: page_url.into(),
                                pinned: true,
                            },
                            Tab {
                                id: 101,
                                title: "One".into(),
                                url: page_url.into(),
                                pinned: false,
                            },
                        ],
                        active_tab: Some(101),
                        recently_closed: vec![RecentlyClosedTab {
                            tab_id: 900,
                            title: "Closed".into(),
                            url: "https://closed.example/".into(),
                            pinned: false,
                            former_position: 0,
                        }],
                        split: None,
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
                        }],
                        active_tab: Some(110),
                        recently_closed: Vec::new(),
                        split: None,
                    },
                ],
            },
            Profile {
                id: 2,
                title: "Dense".into(),
                hue: ProfileHue::Violet,
                spaces: vec![Space {
                    id: 20,
                    title: "Huge".into(),
                    tint: SpaceTint::Violet,
                    paper: Default::default(),
                    glyph: SpaceGlyph::Glyph(GlyphName::Code),
                    tabs: (0..500)
                        .map(|index| Tab {
                            id: 1000 + index,
                            title: format!("Tab {index}"),
                            url: format!("https://dense{index}.example/"),
                            pinned: index < 3,
                        })
                        .collect(),
                    active_tab: Some(1000),
                    recently_closed: Vec::new(),
                    split: None,
                }],
            },
        ],
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
    }
}

/// The dense fixture with the active space carrying a split: the visible set
/// is two panes, so the restore must create exactly two sessions.
pub fn split_active_browser_state(page_url: &str) -> BrowserState {
    let mut state = dense_browser_state(page_url);
    let space = &mut state.profiles[0].spaces[0];
    space.tabs.push(Tab {
        id: 102,
        title: "Two".into(),
        url: page_url.into(),
        pinned: false,
    });
    space.split = Some(Split {
        primary_tab: 101,
        secondary_tab: 102,
    });
    state
}

/// A compact persisted fixture for the restart/force-quit legs: one profile,
/// one space, a pinned + active tab, one recently-closed entry, and a
/// persisted window row (geometry + sidebar width 300).
pub fn small_browser_state(page_url: &str) -> BrowserState {
    BrowserState {
        profiles: vec![Profile {
            id: 1,
            title: "Work".into(),
            hue: ProfileHue::Cobalt,
            spaces: vec![Space {
                id: 10,
                title: "Main".into(),
                tint: SpaceTint::Cobalt,
                paper: Default::default(),
                glyph: SpaceGlyph::Glyph(GlyphName::Code),
                tabs: vec![
                    Tab {
                        id: 100,
                        title: "Pinned".into(),
                        url: "https://pinned.example/".into(),
                        pinned: true,
                    },
                    Tab {
                        id: 101,
                        title: "One".into(),
                        url: page_url.into(),
                        pinned: false,
                    },
                ],
                active_tab: Some(101),
                recently_closed: vec![RecentlyClosedTab {
                    tab_id: 900,
                    title: "Closed".into(),
                    url: "https://closed.example/".into(),
                    pinned: false,
                    former_position: 0,
                }],
                split: None,
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
                width: 300.0,
                collapsed: false,
                last_expanded_width: 300.0,
            },
        }],
    }
}

/// The reorder-regression fixture: one profile, one space, a pinned tab and
/// two regular tabs with distinct urls, so the reorder mutation's effect on
/// the persisted order is unambiguous after a force quit.
pub fn reorder_browser_state(page_one: &str, page_two: &str) -> BrowserState {
    BrowserState {
        profiles: vec![Profile {
            id: 1,
            title: "Work".into(),
            hue: ProfileHue::Cobalt,
            spaces: vec![Space {
                id: 10,
                title: "Main".into(),
                tint: SpaceTint::Cobalt,
                paper: Default::default(),
                glyph: SpaceGlyph::Glyph(GlyphName::Code),
                tabs: vec![
                    Tab {
                        id: 100,
                        title: "Pinned".into(),
                        url: "https://pinned.example/".into(),
                        pinned: true,
                    },
                    Tab {
                        id: 101,
                        title: "One".into(),
                        url: page_one.into(),
                        pinned: false,
                    },
                    Tab {
                        id: 102,
                        title: "Two".into(),
                        url: page_two.into(),
                        pinned: false,
                    },
                ],
                active_tab: Some(101),
                recently_closed: Vec::new(),
                split: None,
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
                width: 300.0,
                collapsed: false,
                last_expanded_width: 300.0,
            },
        }],
    }
}

/// The committed browser state the repository currently holds: the
/// crash-recovery comparison source. Opened through the app's own startup
/// disposition ([`Repository::initialize_and_restore`]) — a force-quit
/// (SIGKILL) can leave a hot rollback journal that only a read-write open
/// can recover, exactly like the real app's next launch — so the comparison
/// measures precisely the state the app itself presents.
pub fn committed_browser_state(profile_dir: &std::path::Path) -> BrowserState {
    let repository = Repository::new(profile_dir.join(STORE_FILE_NAME));
    let outcome = repository.initialize_and_restore().unwrap();
    BrowserState {
        profiles: outcome.snapshot.profiles,
        windows: outcome.snapshot.windows,
    }
}

/// Assert the chrome-first restore contract on a live snapshot: the typed
/// restore disposition, exactly `expected_sessions` live engine sessions
/// (never the persisted tab count), every other identity session-less
/// (`Suspended`), and the restored window/sidebar presentation.
pub fn assert_restored_chrome_first(
    snapshot: &bbr_testkit::protocol::Snapshot,
    expected_sessions: usize,
    expected_deferred: usize,
) {
    assert_eq!(
        snapshot.restore,
        RestoreDisposition::Restored {
            deferred_tabs: expected_deferred,
        },
        "typed restore disposition with the deferred count"
    );
    assert_eq!(
        snapshot.engine_sessions, expected_sessions,
        "visible-set-only hydration: {expected_sessions} live session(s), never the persisted tab count"
    );
    let suspended = snapshot
        .all_tabs
        .iter()
        .filter(|tab| tab.lifecycle == "suspended")
        .count();
    assert_eq!(
        suspended, expected_deferred,
        "every deferred identity is session-less until activation"
    );
    assert!(
        snapshot.content_rect[2] > 0.0 && snapshot.content_rect[3] > 0.0,
        "the hydrated native view has a non-zero content rect"
    );
    assert!(
        snapshot.sidebar_visible && snapshot.sidebar_width == 300.0,
        "persisted sidebar state restored truthfully (visible, 300 pt)"
    );
}

/// The active-space tab urls in display order — the persisted tab order a
/// reorder mutation changes. The reorder fixture has a single space, so
/// `all_tabs` is exactly that space's tabs in order.
pub fn urls_in_order(snapshot: &bbr_testkit::protocol::Snapshot) -> Vec<String> {
    snapshot
        .all_tabs
        .iter()
        .map(|tab| tab.url.clone())
        .collect()
}
