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
use std::path::PathBuf;
use std::process::Child;
use std::time::{Duration, Instant};

use bbr_fixtures::FixtureServer;
use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId, Request};
use bbr_web::WebViewId;

use super::benchmarks::{self, connect, deadline};
use super::harness::{self, AppHarness};
use super::report::{self, Aggregated};

pub use super::report::{aggregate, evaluate_aggregate};

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
        );

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
