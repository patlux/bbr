//! Real-app benchmark trial orchestration.
//!
//! Each cold trial spawns the optimized real `bbr` process with a fresh test
//! profile, measures the process-spawn → first-usable-presentation and
//! process-spawn → deterministic-fixture-ready milestones on the production
//! path, then (optionally) drives the warm interaction matrix: tab selection,
//! sidebar toggle, New-Tab commands, pointer selection, reopen, keyboard
//! activation, and warm space switching. Ratios are never asserted on testkit
//! transport; the app reports presentation commits.
//!
//! Per-interaction benchmark drivers live in [`super::benchmarks`]; report
//! aggregation and baseline evaluation in [`super::report`].
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use bbr_fixtures::FixtureServer;
use bbr_persistence::{
    BrowserState, CommandHistoryEntry, GlyphName, Profile, ProfileHue, RecentlyClosedTab,
    Repository, SidebarState, Space, SpaceGlyph, SpaceTint, Tab, WindowGeometry, WindowState,
};
use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId, Request, RestoreDisposition};
use bbr_web::WebViewId;

use super::benchmarks::{self, connect, deadline};
use super::harness::{self, AppHarness};
use super::report::{self, Aggregated};

pub use super::report::{aggregate, evaluate_aggregate};

/// The persistence store file name the app opens at startup (mirrors
/// `crate::store::STORE_FILE_NAME`; the perf tests cannot import the bin
/// crate's modules).
const STORE_FILE_NAME: &str = "bbr.sqlite";

/// Phase-7 command-history provider seed (pre-spawn): writes `count`
/// deterministic history entries through the production repository so the
/// app's startup restore loads exactly the history-rich provider the
/// `command_bar_first_result` scenario measures. Entry 1 is the unique
/// query target ("alpha-history" matches only it); every other entry is
/// a distinct non-matching URL/title. Seeding happens outside any timed
/// region — the measured samples start after the app has loaded.
pub(crate) fn seed_history_store(profile: &Path, count: usize) {
    let repository = Repository::new(profile.join(STORE_FILE_NAME));
    repository.initialize_and_restore().unwrap();
    let entries = (0..count)
        .map(|index| {
            let id = (index + 1) as u64;
            if index == 0 {
                CommandHistoryEntry {
                    id,
                    title: "Alpha History Target".into(),
                    url: "https://alpha-history.example/".into(),
                    visit_count: 1,
                    last_visited: 0,
                }
            } else {
                CommandHistoryEntry {
                    id,
                    title: format!("History Entry {index}"),
                    url: format!("https://history-{index:05}.example/"),
                    visit_count: 1,
                    last_visited: index as u64,
                }
            }
        })
        .collect::<Vec<_>>();
    repository.store_command_history(&entries).unwrap();
}

/// B2 Saved Page provider seed (pre-spawn): writes `count` deterministic
/// global Saved Page records through the production repository so the app's
/// startup restore loads exactly the ≥1,000-record provider the
/// `saved_page_search_first_result` scenario measures. Entry 1 is the
/// unique query target ("Alpha Saved Target" — the query `alpha saved`
/// matches only it — with the deterministic navigable fixture page URL);
/// every other entry is a distinct non-matching title/URL. The records
/// carry no tab/session/space/profile identity (the B2 contract). Seeding
/// happens outside any timed region — the measured samples start after the
/// app has loaded.
pub(crate) fn seed_saved_pages_store(profile: &Path, count: usize, target_url: &str) {
    let repository = Repository::new(profile.join(STORE_FILE_NAME));
    repository.initialize_and_restore().unwrap();
    let pages = (0..count)
        .map(|index| {
            let id = (index + 1) as u64;
            if index == 0 {
                bbr_persistence::SavedPageRecord {
                    id,
                    canonical_url: target_url.into(),
                    captured_title: "Alpha Saved Target".into(),
                    user_label: None,
                    created_at: 0,
                    updated_at: 0,
                    collection_id: None,
                }
            } else {
                bbr_persistence::SavedPageRecord {
                    id,
                    canonical_url: format!("https://saved-{index:05}.example/"),
                    captured_title: format!("Saved Page {index}"),
                    user_label: None,
                    created_at: index as u64,
                    updated_at: index as u64,
                    collection_id: None,
                }
            }
        })
        .collect::<Vec<_>>();
    repository.replace_saved_pages(&pages).unwrap();
}

/// Deterministic fixture page titles (see `crates/bbr-fixtures/html`).
pub const PAGE_ONE_TITLE: &str = "BBR Fixture One";
pub const PAGE_TWO_TITLE: &str = "BBR Fixture Two";
/// Interaction budget ceilings from the product performance contract.
pub const INTERACTION_P95_MS: f64 = 50.0;
pub const INTERACTION_MAX_MS: f64 = 100.0;
/// Startup absolute max ceiling for the short single cold gate.
pub const STARTUP_MAX_MS: f64 = 750.0;
/// Contract-required repetition counts for the full canonical gate.
pub const COLD_STARTUP_SAMPLES: usize = 10;
pub const INTERACTION_SAMPLES: usize = 30;
pub const NEW_TAB_SAMPLES: usize = 10;
/// Contract-required persisted warm-startup samples (≥10 per the startup
/// contract: “at least 10 startup samples per mode”; the warm mode is the
/// persisted-profile launch).
pub const PERSISTED_WARM_STARTUP_SAMPLES: usize = 10;

/// Spawn configuration for a cold trial.
#[derive(Clone)]
pub struct TrialConfig {
    pub bin: &'static str,
    pub engine_kind: &'static str,
    pub interaction_samples: usize,
    pub new_tab_samples: usize,
    pub inject_delay_ms: Option<u64>,
    /// Phase-7 command-history provider scale: seed this many history
    /// entries into the profile store before the app starts (0 = no seed).
    /// The `command_bar_first_result` scenario sets 10,000 so the measured
    /// ranking corpus matches the command-core benchmark scale (500 tabs +
    /// 10k history).
    pub seed_history: usize,
    /// B2 Saved Page provider scale: seed this many global Saved Page
    /// records into the profile store before the app starts (0 = no seed).
    /// The `saved_page_search_first_result` scenario sets 1,000 so the
    /// measured ranking corpus matches the B2 contract scale (≥1,000
    /// loaded records).
    pub seed_saved_pages: usize,
}

/// One cold trial's measurements.
pub struct ColdSample {
    pub startup_ms: f64,
    pub page_ready_ms: f64,
    /// Persisted warm-startup sample (Phase 7, warm-restore slice):
    /// process-spawn → first usable presentation against a persisted
    /// profile. `None` on cold fresh-profile trials.
    pub warm_startup_ms: Option<f64>,
    /// Persisted warm-startup page-ready sample (same trial).
    pub warm_page_ready_ms: Option<f64>,
    /// Interaction raw samples (empty when the trial only measured startup).
    pub interactions: BTreeMap<String, Vec<u64>>,
}

/// Drive a cold fresh-profile trial and return its measurements.
pub fn run_cold_trial(config: &TrialConfig, drive_interactions: bool, tag: &str) -> ColdSample {
    let (startup_ms, page_ready_ms, interactions) =
        run_cold_trial_raw(config, drive_interactions, tag);
    ColdSample {
        startup_ms,
        page_ready_ms,
        warm_startup_ms: None,
        warm_page_ready_ms: None,
        interactions,
    }
}

/// The Phase-7 persisted warm-startup fixture (the packet's canonical dense
/// shape): 500+ tab identities across two profiles and three spaces with
/// per-space active tabs, pinned prefixes, and one recently-closed entry,
/// written through the real repository before the app starts. The active
/// space (the deterministic policy default: first space with an active tab)
/// is single-pane and its selected tab points at the deterministic fixture
/// page, so the expected session count is exactly **one** and the page-ready
/// milestone is network-independent. Seeding happens outside any timed
/// region — the measured samples start after the app has loaded.
pub(crate) fn seed_browser_store(profile: &std::path::Path, page_url: &str) {
    let repository = Repository::new(profile.join(STORE_FILE_NAME));
    repository.initialize_and_restore().unwrap();
    let state = BrowserState {
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
                                closed: false,
                                pinned_page: None,
                            },
                            Tab {
                                id: 101,
                                title: "One".into(),
                                url: page_url.into(),
                                pinned: false,
                                closed: false,
                                pinned_page: None,
                            },
                        ],
                        active_tab: Some(101),
                        recently_closed: vec![RecentlyClosedTab {
                            tab_id: 900,
                            title: "Closed".into(),
                            url: "https://closed.example/".into(),
                            pinned: false,
                            former_position: 0,
                            in_place: false,
                        }],
                        split: None,
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
                            closed: false,
                            pinned_page: None,
                        })
                        .collect(),
                    active_tab: Some(1000),
                    recently_closed: Vec::new(),
                    split: None,
                    folders: Vec::new(),
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
        favorites: Vec::new(),
    };
    repository.store_browser_state(&state).unwrap();
}

/// The deferred identity count of the warm fixture: 503 live tabs across
/// the three spaces minus the one hydrated session (recently-closed entries
/// are closed identities, never session-less tabs).
pub(crate) const PERSISTED_WARM_DEFERRED: usize = 502;

/// Drive a persisted warm-startup trial and return its two startup
/// milestones: process-spawn → first usable presentation (sidebar + the
/// selected restored tab's native view, non-zero rect) and process-spawn →
/// deterministic fixture page ready. The profile's browser state was written
/// through the actual repository before spawn; the semantic fast-but-wrong
/// guards are asserted before any sample is accepted: exactly one live
/// engine session from 500+ persisted identities, the typed `Restored`
/// restore disposition with the deferred count, and the persisted
/// sidebar/geometry presented.
pub fn run_persisted_warm_trial(config: &TrialConfig, tag: &str) -> (f64, f64) {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");

    let root = PathBuf::from(format!(
        "/tmp/bbr-e2e-perf-warm-{tag}-{}",
        std::process::id()
    ));
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    seed_browser_store(&profile, &page_one);
    let socket = profile.join("bbr-test.sock");
    let token = format!("perf-warm-{tag}");

    let args: Vec<String> = vec![
        "--bbr-initial-url".into(),
        page_one.clone(),
        "--bbr-profile-dir".into(),
        profile.to_str().unwrap().into(),
        "--bbr-test-token".into(),
        token.clone(),
        "--bbr-ids=sequential".into(),
    ];

    let spawned_at_ns = monotonic_now_ns();
    let mut harness = AppHarness::spawn(
        config.bin,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
        &root,
    );

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let startup = replay
            .iter()
            .find_map(|(_, event)| {
                benchmarks::measurement(event, BenchmarkKind::Startup, BenchmarkOperationId(0))
            })
            .unwrap_or_else(|| {
                benchmarks::wait_measurement(
                    &mut client,
                    BenchmarkKind::Startup,
                    BenchmarkOperationId(0),
                    0,
                )
                .1
            });
        let startup_ms = ns_to_ms(startup.monotonic_commit_ns.saturating_sub(spawned_at_ns));
        harness.raw_log.startup_ms.push(startup_ms);
        // Semantic fast-but-wrong guards on the startup presentation sample:
        // the selected restored tab's native view is presented with a
        // non-zero rect and the sidebar is visible.
        assert_eq!(startup.active_web_view, Some(WebViewId(1)));
        assert!(startup.sidebar_visible);
        assert!(startup.content_rect[2] > 0.0 && startup.content_rect[3] > 0.0);
        assert!(
            startup_ms <= STARTUP_MAX_MS,
            "warm startup presentation {startup_ms:.2}ms exceeded {STARTUP_MAX_MS:.0}ms ceiling"
        );

        // The typed restore disposition and the visible-set session count:
        // exactly one live session from 500+ persisted identities, never the
        // persisted tab count — the fast-but-wrong guard.
        let snapshot = client.snapshot().expect("warm snapshot");
        assert_eq!(
            snapshot.restore,
            RestoreDisposition::Restored {
                deferred_tabs: PERSISTED_WARM_DEFERRED,
            },
            "the persisted model restored chrome-first with the deferred count"
        );
        assert_eq!(
            snapshot.engine_sessions, 1,
            "visible-set-only hydration: exactly one session, never 500+"
        );
        assert_eq!(
            snapshot.sidebar_width, 300.0,
            "the persisted sidebar width presented"
        );
        assert_eq!(snapshot.window_content_size, [1080.0, 700.0]);
        assert_eq!(snapshot.window_size[0], 1080.0);

        // Deterministic page-ready milestone on the restored selected tab.
        client
            .send_ok(&Request::BenchmarkPageReady {
                operation_id: BenchmarkOperationId(1),
                expected_view: WebViewId(1),
                expected_title: PAGE_ONE_TITLE.to_string(),
            })
            .expect("arm page ready");
        let (_seq, page_ready) = benchmarks::wait_measurement(
            &mut client,
            BenchmarkKind::PageReady,
            BenchmarkOperationId(1),
            since,
        );
        let page_ready_ms = ns_to_ms(page_ready.monotonic_commit_ns.saturating_sub(spawned_at_ns));
        harness.raw_log.page_ready_ms.push(page_ready_ms);
        assert_eq!(page_ready.page_title.as_deref(), Some(PAGE_ONE_TITLE));
        assert!(
            page_ready_ms <= 1500.0,
            "warm page ready {page_ready_ms:.2}ms exceeded the 1500ms ceiling"
        );

        client.quit().expect("quit");
        wait_for_exit(&mut harness.child, deadline());
        (startup_ms, page_ready_ms)
    }));

    match outcome {
        Ok(value) => {
            harness.success = true;
            value
        }
        Err(panic) => {
            let partial = super::report::Aggregated::from(&harness.raw_log);
            let failure = super::report::panic_message(&panic);
            let (report, _baseline) = super::report::report_from_agg(
                &partial,
                config,
                &super::baseline_path(),
                &root,
                "persisted-warm",
                Some(failure.clone()),
            );
            super::harness::write_artifacts(&root, &report);
            eprintln!("FAILED-TRIAL ARTIFACTS at {}", root.display());
            eprintln!("FAILURE: {failure}");
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
}

/// The raw tuple form of [`run_cold_trial`].
pub fn run_cold_trial_raw(
    config: &TrialConfig,
    drive_interactions: bool,
    tag: &str,
) -> (f64, f64, BTreeMap<String, Vec<u64>>) {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");
    // The deterministic multi-match fixture the find-in-page performance
    // scenarios drive (task packet `find-in-page-portal-bar`); loaded before
    // any sample, so fixture/network time never enters app timings.
    let page_find = fixtures.url("/find-repeated.html");

    // Keep the root under the fixed short `/tmp` prefix (macOS-only module):
    // the gateway socket must stay strictly below macOS's 104-byte `sun_path`
    // even when `$TMPDIR` is long (the default `/var/folders/…/T/` is 49
    // bytes, longer under direnv).
    let root = PathBuf::from(format!("/tmp/bbr-e2e-perf-{tag}-{}", std::process::id()));
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    // Phase-7 command-history provider seed: when the trial config requests
    // the history-rich corpus, the store is seeded BEFORE the app starts so
    // the measured ranking runs against the loaded provider (never fixture
    // setup inside a sample).
    if config.seed_history > 0 {
        seed_history_store(&profile, config.seed_history);
    }
    // B2 Saved Page provider seed: when the trial config requests the
    // ≥1,000-record provider, the store is seeded BEFORE the app starts so
    // the measured ranking runs against the loaded provider (never fixture
    // setup inside a sample).
    if config.seed_saved_pages > 0 {
        seed_saved_pages_store(&profile, config.seed_saved_pages, &page_one);
    }
    let socket = profile.join("bbr-test.sock");
    let token = format!("perf-{tag}");

    let mut args: Vec<String> = vec![
        "--bbr-initial-url".into(),
        page_one.clone(),
        "--bbr-profile-dir".into(),
        profile.to_str().unwrap().into(),
        "--bbr-test-token".into(),
        token.clone(),
        "--bbr-ids=sequential".into(),
    ];
    if let Some(ms) = config.inject_delay_ms {
        args.push(format!("--bbr-bench-inject-delay-ms={ms}"));
    }

    let spawned_at_ns = monotonic_now_ns();
    let mut harness = AppHarness::spawn(
        config.bin,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
        &root,
    );

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let startup = replay
            .iter()
            .find_map(|(_, event)| {
                benchmarks::measurement(event, BenchmarkKind::Startup, BenchmarkOperationId(0))
            })
            .unwrap_or_else(|| {
                benchmarks::wait_measurement(
                    &mut client,
                    BenchmarkKind::Startup,
                    BenchmarkOperationId(0),
                    0,
                )
                .1
            });
        let startup_ms = ns_to_ms(startup.monotonic_commit_ns.saturating_sub(spawned_at_ns));
        harness.raw_log.startup_ms.push(startup_ms);
        assert_eq!(startup.active_web_view, Some(WebViewId(1)));
        assert!(startup.sidebar_visible);
        assert!(startup.content_rect[2] > 0.0 && startup.content_rect[3] > 0.0);
        assert!(
            startup_ms <= STARTUP_MAX_MS,
            "startup presentation {startup_ms:.2}ms exceeded {STARTUP_MAX_MS:.0}ms ceiling"
        );

        // Deterministic page-ready milestone: measure process-spawn → fixture
        // content ready, separate from shell/native-view presentation.
        client
            .send_ok(&Request::BenchmarkPageReady {
                operation_id: BenchmarkOperationId(1),
                expected_view: WebViewId(1),
                expected_title: PAGE_ONE_TITLE.to_string(),
            })
            .expect("arm page ready");
        let (seq, page_ready) = benchmarks::wait_measurement(
            &mut client,
            BenchmarkKind::PageReady,
            BenchmarkOperationId(1),
            since,
        );
        since = seq;
        let page_ready_ms = ns_to_ms(page_ready.monotonic_commit_ns.saturating_sub(spawned_at_ns));
        harness.raw_log.page_ready_ms.push(page_ready_ms);
        assert_eq!(page_ready.page_title.as_deref(), Some(PAGE_ONE_TITLE));
        assert!(
            page_ready_ms <= 1500.0,
            "page ready {page_ready_ms:.2}ms exceeded the 1500ms cold ceiling"
        );

        if !drive_interactions {
            client.quit().expect("quit");
            wait_for_exit(&mut harness.child, deadline());
            return (startup_ms, page_ready_ms, BTreeMap::new());
        }

        // Warm interaction matrix (tab/session/sidebar/space scenarios).
        let raw = super::drive::drive_interaction_matrix(
            &mut client,
            &mut harness,
            &mut since,
            config,
            &page_one,
            &page_two,
            &page_find,
        );
        // v2 motion idle budget: the settled shell requests no animation
        // frames (zero animation elements across forced frames).
        super::idle::assert_idle_requests_no_frames(&mut client);

        client.quit().expect("quit");
        wait_for_exit(&mut harness.child, deadline());
        (startup_ms, page_ready_ms, raw)
    }));

    match outcome {
        Ok(value) => {
            harness.success = true;
            value
        }
        Err(panic) => {
            // Preserve the partial JSON + CSV summary and raw samples before
            // dropping the app: a startup/page-ready ceiling or wrong-state
            // assert trips inside the trial, before `evaluate_aggregate` would
            // have written full artifacts. Event ring + app stderr follow.
            let partial = Aggregated::from(&harness.raw_log);
            let failure = report::panic_message(&panic);
            let (report, _baseline) = report::report_from_agg(
                &partial,
                config,
                &super::baseline_path(),
                &root,
                "cold-fresh",
                Some(failure.clone()),
            );
            harness::write_artifacts(&root, &report);
            eprintln!("FAILED-TRIAL ARTIFACTS at {}", root.display());
            eprintln!("FAILURE: {failure}");
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
}

fn wait_for_exit(child: &mut Child, deadline: Instant) -> std::process::ExitStatus {
    loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            return status;
        }
        assert!(Instant::now() < deadline, "app did not exit");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn ns_to_ms(ns: u64) -> f64 {
    ns as f64 / 1_000_000.0
}

fn monotonic_now_ns() -> u64 {
    let mut info = mach2::mach_time::mach_timebase_info_data_t { numer: 0, denom: 0 };
    // SAFETY: these macOS clock calls have no memory aliasing or lifetime
    // requirements beyond the initialized plain data output pointer.
    unsafe {
        mach2::mach_time::mach_timebase_info(&mut info);
        let ticks = mach2::mach_time::mach_absolute_time() as u128;
        let nanos = ticks * u128::from(info.numer) / u128::from(info.denom.max(1));
        nanos.min(u64::MAX as u128) as u64
    }
}
