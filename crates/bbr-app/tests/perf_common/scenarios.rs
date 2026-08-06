//! Real-app benchmark scenario driving and report aggregation.
#![allow(dead_code)]
//!
//! Each cold trial spawns the optimized real `bbr` process with a fresh test
//! profile, measures the process-spawn → first-usable-presentation and
//! process-spawn → deterministic-fixture-ready milestones on the production
//! path, then (optionally) drives the warm interaction matrix. Ratios are
//! never asserted on testkit transport; the app reports presentation commits.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use bbr_fixtures::FixtureServer;
use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, Event, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::{WebEvent, WebViewId};

use super::baseline::{self, Baseline};
use super::harness::{self, AppHarness, PerfReport};
use super::machine::MachineInfo;
use super::stats::{stats_from_ms, Stats};

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

/// Spawn configuration for a cold trial.
#[derive(Clone)]
pub struct TrialConfig {
    pub bin: &'static str,
    pub engine_kind: &'static str,
    pub interaction_samples: usize,
    pub new_tab_samples: usize,
    pub inject_delay_ms: Option<u64>,
}

/// One cold trial's measurements.
pub struct ColdSample {
    pub startup_ms: f64,
    pub page_ready_ms: f64,
    /// Interaction raw samples (empty when the trial only measured startup).
    pub interactions: BTreeMap<String, Vec<u64>>,
}

/// Aggregated measurements across one or more cold trials.
#[derive(Default)]
pub struct Aggregated {
    pub startup_ms: Vec<f64>,
    pub page_ready_ms: Vec<f64>,
    pub interactions: BTreeMap<String, Vec<u64>>,
}

impl Aggregated {
    fn push(&mut self, sample: ColdSample) {
        self.startup_ms.push(sample.startup_ms);
        self.page_ready_ms.push(sample.page_ready_ms);
        for (kind, raw) in sample.interactions {
            self.interactions.entry(kind).or_default().extend(raw);
        }
    }
}

/// Drive a cold fresh-profile trial and return its measurements.
pub fn run_cold_trial(config: &TrialConfig, drive_interactions: bool, tag: &str) -> ColdSample {
    let (startup_ms, page_ready_ms, interactions) =
        run_cold_trial_raw(config, drive_interactions, tag);
    ColdSample {
        startup_ms,
        page_ready_ms,
        interactions,
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

    let root = std::env::temp_dir().join(format!("bbr-e2e-perf-{tag}-{}", std::process::id()));
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
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
                measurement(event, BenchmarkKind::Startup, BenchmarkOperationId(0))
            })
            .unwrap_or_else(|| {
                wait_measurement(
                    &mut client,
                    BenchmarkKind::Startup,
                    BenchmarkOperationId(0),
                    0,
                )
                .1
            });
        let startup_ms = ns_to_ms(startup.monotonic_commit_ns.saturating_sub(spawned_at_ns));
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
        let (seq, page_ready) = wait_measurement(
            &mut client,
            BenchmarkKind::PageReady,
            BenchmarkOperationId(1),
            since,
        );
        since = seq;
        let page_ready_ms = ns_to_ms(page_ready.monotonic_commit_ns.saturating_sub(spawned_at_ns));
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

        let initial_snapshot = client.snapshot().expect("initial snapshot");
        let first_view = initial_snapshot.active_web_view.expect("first native view");
        if initial_snapshot.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        client
            .send_ok(&Request::OpenTab {
                url: page_two.clone(),
            })
            .expect("open second tab");
        let second = client.snapshot().expect("second tab snapshot");
        let second_view = second.active_web_view.expect("second native view");
        if second.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(&mut client, second_view, PAGE_TWO_TITLE, since);
        }

        let mut operation = 2_u64;
        // One discarded warm-up round.
        benchmark_select(&mut client, &mut since, operation, 0, first_view);
        operation += 1;

        let mut tab_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let (index, expected) = if sample % 2 == 0 {
                (0, first_view)
            } else {
                (1, second_view)
            };
            let measured = benchmark_select(&mut client, &mut since, operation, index, expected);
            tab_samples.push(measured.duration_ns);
            operation += 1;
        }

        benchmark_sidebar(&mut client, &mut since, operation, false);
        operation += 1;
        benchmark_sidebar(&mut client, &mut since, operation, true);
        operation += 1;

        let mut sidebar_samples = Vec::with_capacity(config.interaction_samples);
        let mut expected_sidebar = true;
        for _ in 0..config.interaction_samples {
            expected_sidebar = !expected_sidebar;
            let measured = benchmark_sidebar(&mut client, &mut since, operation, expected_sidebar);
            sidebar_samples.push(measured.duration_ns);
            operation += 1;
        }

        let mut new_tab_command_samples = Vec::with_capacity(config.new_tab_samples);
        let mut new_tab_open_samples = Vec::with_capacity(config.new_tab_samples);
        let mut active_close_samples = Vec::with_capacity(config.new_tab_samples);
        for _ in 0..config.new_tab_samples {
            let before = client.snapshot().expect("before Cmd-T benchmark");
            let before_tabs = before.tabs.len();
            let before_sessions = before.engine_sessions;
            let previous_view = before.active_web_view.expect("active view before Cmd-T");

            let staged = benchmark_keystroke(
                &mut client,
                &mut since,
                operation,
                BenchmarkKind::NewTabCommandBar,
                "cmd-t",
            );
            operation += 1;
            let staged_snapshot = client.snapshot().expect("staged Cmd-T snapshot");
            assert_eq!(staged.active_web_view, Some(previous_view));
            assert_eq!(staged.command_bar_mode, CommandBarMode::NewTab);
            assert!(staged.chrome_focused);
            assert_eq!(staged_snapshot.tabs.len(), before_tabs);
            assert_eq!(staged_snapshot.engine_sessions, before_sessions);
            assert_eq!(staged_snapshot.command_bar_mode, CommandBarMode::NewTab);
            new_tab_command_samples.push(staged.duration_ns);

            client
                .send_ok(&Request::SetCommandBarText {
                    text: page_one.clone(),
                })
                .expect("set staged destination");
            let opened = benchmark_keystroke(
                &mut client,
                &mut since,
                operation,
                BenchmarkKind::NewTabOpened,
                "enter",
            );
            operation += 1;
            let opened_snapshot = client.snapshot().expect("opened tab snapshot");
            assert_eq!(opened_snapshot.tabs.len(), before_tabs + 1);
            assert_eq!(opened_snapshot.engine_sessions, before_sessions + 1);
            assert_eq!(opened_snapshot.command_bar_mode, CommandBarMode::Inactive);
            assert!(!opened_snapshot.chrome_focused);
            assert_ne!(opened.active_web_view, Some(previous_view));
            new_tab_open_samples.push(opened.duration_ns);

            let closed = benchmark_keystroke(
                &mut client,
                &mut since,
                operation,
                BenchmarkKind::ActiveTabClose,
                "cmd-w",
            );
            operation += 1;
            let closed_snapshot = client.snapshot().expect("closed tab snapshot");
            assert_eq!(closed_snapshot.tabs.len(), before_tabs);
            assert_eq!(closed_snapshot.engine_sessions, before_sessions);
            assert_eq!(closed_snapshot.active_web_view, Some(previous_view));
            assert_eq!(closed.active_web_view, Some(previous_view));
            assert!(!closed.chrome_focused);
            assert_eq!(closed.command_bar_mode, CommandBarMode::Inactive);
            active_close_samples.push(closed.duration_ns);
        }

        client.quit().expect("quit");
        wait_for_exit(&mut harness.child, deadline());

        let mut raw = BTreeMap::new();
        raw.insert("warm_tab_switch".to_string(), tab_samples);
        raw.insert("sidebar_toggle".to_string(), sidebar_samples);
        raw.insert("new_tab_command_bar".to_string(), new_tab_command_samples);
        raw.insert("new_tab_open".to_string(), new_tab_open_samples);
        raw.insert("active_tab_close".to_string(), active_close_samples);
        (startup_ms, page_ready_ms, raw)
    }));

    match outcome {
        Ok(value) => {
            harness.success = true;
            value
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
}

/// Aggregate one or more cold trials into raw distributions.
pub fn aggregate(samples: Vec<ColdSample>) -> Aggregated {
    let mut agg = Aggregated::default();
    for sample in samples {
        agg.push(sample);
    }
    agg
}

/// Evaluate baseline regressions for an aggregated run and return the report.
pub fn evaluate_aggregate(
    agg: &Aggregated,
    config: &TrialConfig,
    baseline_path: &Path,
    artifacts_dir: &Path,
    trend_dir: &Path,
    fixture_mode: &str,
) -> PerfReport {
    let baseline = Baseline::load_or_seed(baseline_path);
    let info = MachineInfo::collect(
        config.bin,
        config.engine_kind,
        fixture_mode,
        &baseline.canonical_host_model,
    );

    let mut results = BTreeMap::new();
    let mut raw_ns = BTreeMap::new();
    results.insert("startup".into(), stats_from_ms(&agg.startup_ms));
    results.insert("page_ready".into(), stats_from_ms(&agg.page_ready_ms));
    for (kind, raw) in &agg.interactions {
        results.insert(kind.clone(), harness::KindStats::from(&Stats::from_ns(raw)));
        raw_ns.insert(kind.clone(), raw.clone());
    }

    // Build measured map for baseline evaluation (skip the interaction-only max
    // comparison duplication already covered by absolute rules below).
    let measured = aggregate_measured(&results);
    let regressions = baseline::evaluate(&measured, &baseline, info.canonical);

    let kept_at = artifacts_dir.display().to_string();
    let report = PerfReport {
        machine: info,
        fixture_mode: fixture_mode.to_string(),
        results,
        raw_ns,
        regressions,
        baseline_generated_revision: baseline.generated_revision.clone(),
        interaction_p95_ms: INTERACTION_P95_MS,
        interaction_max_ms: INTERACTION_MAX_MS,
        kept_at,
    };
    harness::write_artifacts(artifacts_dir, &report);
    harness::append_trend(&report, trend_dir);
    report
}

fn aggregate_measured(
    results: &BTreeMap<String, harness::KindStats>,
) -> BTreeMap<String, baseline::Measured> {
    results
        .iter()
        .map(|(kind, s)| {
            (
                kind.clone(),
                baseline::Measured {
                    samples: s.samples,
                    p50_ms: s.p50_ms,
                    p95_ms: s.p95_ms,
                    max_ms: s.max_ms,
                },
            )
        })
        .collect()
}

fn connect(socket: &Path) -> Client {
    let until = Instant::now() + Duration::from_secs(60);
    loop {
        match Client::connect(socket) {
            Ok(client) => return client,
            Err(_) if Instant::now() < until => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => panic!("could not connect to gateway: {error}"),
        }
    }
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

fn measurement(
    event: &Event,
    kind: BenchmarkKind,
    operation_id: BenchmarkOperationId,
) -> Option<PresentationMeasurement> {
    match event {
        Event::PresentationCommitted(measured)
            if measured.kind == kind && measured.operation_id == operation_id =>
        {
            Some(measured.clone())
        }
        _ => None,
    }
}

fn wait_measurement(
    client: &mut Client,
    kind: BenchmarkKind,
    operation_id: BenchmarkOperationId,
    since: u64,
) -> (u64, PresentationMeasurement) {
    let (seq, event) = client
        .wait_for_event_since(
            |event| measurement(event, kind, operation_id).is_some(),
            since,
            deadline(),
        )
        .expect("presentation measurement");
    (seq, measurement(&event, kind, operation_id).unwrap())
}

fn benchmark_select(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    index: usize,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkSelectTab {
            operation_id,
            index,
        })
        .expect("benchmark tab selection");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::WarmTabSwitch, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    let snapshot = client.snapshot().expect("selected snapshot");
    assert_eq!(snapshot.active_tab_index, Some(index));
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    measured
}

fn benchmark_keystroke(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    kind: BenchmarkKind,
    keystroke: &str,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDispatchKeystroke {
            operation_id,
            kind,
            keystroke: keystroke.into(),
        })
        .expect("benchmark keystroke");
    let (seq, measured) = wait_measurement(client, kind, operation_id, *since);
    *since = seq;
    measured
}

fn benchmark_sidebar(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_visible: bool,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkDispatchKeystroke {
            operation_id,
            kind: BenchmarkKind::SidebarToggle,
            keystroke: "cmd-b".into(),
        })
        .expect("benchmark sidebar toggle");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::SidebarToggle, operation_id, *since);
    *since = seq;
    assert_eq!(measured.sidebar_visible, expected_visible);
    let snapshot = client.snapshot().expect("sidebar snapshot");
    assert_eq!(snapshot.sidebar_visible, expected_visible);
    measured
}

fn wait_title(client: &mut Client, view: WebViewId, title: &str, since: u64) -> u64 {
    client
        .wait_for_event_since(
            |event| matches!(event, Event::Web(WebEvent::TitleChanged(id, value)) if *id == view && value == title),
            since,
            deadline(),
        )
        .expect("title event")
        .0
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
