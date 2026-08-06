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
use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId, BrowserChange, Request};
use bbr_web::WebViewId;

use super::benchmarks::{
    self, benchmark_click, benchmark_keystroke, benchmark_reopen, benchmark_select,
    benchmark_sidebar, benchmark_space_switch, connect, deadline, wait_browser_change, wait_title,
};
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
            harness
                .raw_log
                .interactions
                .entry("warm_tab_switch".into())
                .or_default()
                .push(measured.duration_ns);
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
            harness
                .raw_log
                .interactions
                .entry("sidebar_toggle".into())
                .or_default()
                .push(measured.duration_ns);
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
            assert_eq!(
                staged.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::NewTab
            );
            assert!(staged.chrome_focused);
            assert_eq!(staged_snapshot.tabs.len(), before_tabs);
            assert_eq!(staged_snapshot.engine_sessions, before_sessions);
            assert_eq!(
                staged_snapshot.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::NewTab
            );
            new_tab_command_samples.push(staged.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("new_tab_command_bar".into())
                .or_default()
                .push(staged.duration_ns);

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
            assert_eq!(
                opened_snapshot.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::Inactive
            );
            assert!(!opened_snapshot.chrome_focused);
            assert_ne!(opened.active_web_view, Some(previous_view));
            new_tab_open_samples.push(opened.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("new_tab_open".into())
                .or_default()
                .push(opened.duration_ns);

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
            assert_eq!(
                closed.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::Inactive
            );
            active_close_samples.push(closed.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("active_tab_close".into())
                .or_default()
                .push(closed.duration_ns);
        }

        // The new-tab cycle always ends back on two tabs with the second
        // (fixture two) tab active in web focus. Pointer selection measures
        // the production row-listener path (chrome focus preserved), then a
        // reopen cycle closes the active tab via real Cmd-W and measures the
        // fresh-session presentation of the reopened tab.
        let mut pointer_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let (index, expected) = if sample % 2 == 0 {
                (0, first_view)
            } else {
                (1, second_view)
            };
            let measured = benchmark_click(&mut client, &mut since, operation, index, expected);
            pointer_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_select_pointer".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        let mut reopen_samples = Vec::with_capacity(config.new_tab_samples);
        for _ in 0..config.new_tab_samples {
            // Unmeasured setup: close the active tab through the production
            // Cmd-W path (active-close has its own scenario above).
            client
                .send_ok(&Request::DispatchKeystroke {
                    keystroke: "cmd-w".into(),
                })
                .expect("close active tab before reopen benchmark");
            let closed = client.snapshot().expect("closed before reopen benchmark");
            assert_eq!(closed.tabs.len(), 1);
            assert_eq!(closed.engine_sessions, 1);
            let neighbor = closed.active_web_view.expect("neighbor view");
            let expected_domain = closed.chrome_focused;
            let measured = benchmark_reopen(
                &mut client,
                &mut since,
                operation,
                neighbor,
                expected_domain,
            );
            operation += 1;
            reopen_samples.push(measured.0.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_reopen".into())
                .or_default()
                .push(measured.0.duration_ns);
            let _ = measured.1; // fresh view id retained for diagnostics
        }

        // Keyboard activation: Enter on the focused tab list activates the
        // selected tab — the chrome traversal commits and keyboard focus
        // moves into the page (interaction contract "Enter activates"). The
        // unmeasured production pointer path re-focuses the list between
        // samples; only the Enter transition itself is sampled. The reopen
        // cycle recreated tab 1, so capture the live view ids after it.
        let after_reopen = client.snapshot().expect("state after reopen cycle");
        let tab0_view = after_reopen.tabs[0].web_view.expect("tab 0 view");
        let tab1_view = after_reopen.tabs[1].web_view.expect("tab 1 view");
        let mut activate_samples = Vec::with_capacity(config.new_tab_samples);
        for sample in 0..config.new_tab_samples {
            let (index, expected) = if sample % 2 == 0 {
                (0, tab0_view)
            } else {
                (1, tab1_view)
            };
            client
                .send_ok(&Request::ClickTab { index })
                .expect("focus tab list before Enter");
            let focused = client.snapshot().expect("tab list focused");
            assert!(focused.chrome_focused, "pointer click focuses the list");
            assert_eq!(focused.active_web_view, Some(expected));
            let measured = benchmark_keystroke(
                &mut client,
                &mut since,
                operation,
                BenchmarkKind::TabActivateKeyboard,
                "enter",
            );
            operation += 1;
            assert_eq!(
                measured.active_web_view,
                Some(expected),
                "Enter keeps the selected tab's view"
            );
            assert!(
                !measured.chrome_focused,
                "Enter activates into the page (web focus)"
            );
            assert_eq!(
                measured.command_bar_mode,
                bbr_testkit::protocol::CommandBarMode::Inactive
            );
            let after = client.snapshot().expect("after Enter snapshot");
            assert_eq!(after.active_web_view, Some(expected));
            assert!(!after.chrome_focused);
            activate_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("tab_activate_keyboard".into())
                .or_default()
                .push(measured.duration_ns);
        }

        // Warm space switching: create a second space in the default profile
        // with its own warm tab, then measure alternating activation between
        // the two spaces through the production activation path (DESIGN.md
        // §11.3 / interaction contract "each space remembers its own active
        // tab"). Every measured presentation must show the destination
        // space's warm selected tab with the same native session (no
        // recreation), keep the chrome focus domain, and expose the
        // destination space row as selected (§11.3/§15).
        let before_space = client.snapshot().expect("state before space setup");
        let default_profile = before_space.active_profile;
        let default_space = before_space.active_space;
        let default_space_view = before_space.active_web_view.expect("default space view");
        client
            .send_ok(&Request::CreateSpace {
                profile: default_profile,
                title: "Benchmark".into(),
            })
            .expect("create second space");
        let (seq, created) = wait_browser_change(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let benchmark_space = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        // Activate the empty space and warm its own tab through the
        // production controller path (unmeasured setup).
        client
            .send_ok(&Request::ActivateSpace {
                space: benchmark_space,
            })
            .expect("activate benchmark space");
        let (seq, _) = wait_browser_change(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == benchmark_space),
            since,
        );
        since = seq;
        client
            .send_ok(&Request::OpenTab {
                url: page_two.clone(),
            })
            .expect("open benchmark space tab");
        let second_space = client.snapshot().expect("benchmark space tab snapshot");
        let benchmark_space_view = second_space.active_web_view.expect("benchmark space view");
        if second_space.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(&mut client, benchmark_space_view, PAGE_TWO_TITLE, since);
        }
        let expected_sessions = second_space.engine_sessions;
        client
            .send_ok(&Request::ActivateSpace {
                space: default_space,
            })
            .expect("back to default space");
        let (seq, _) = wait_browser_change(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == default_space),
            since,
        );
        since = seq;

        let mut space_samples = Vec::with_capacity(config.interaction_samples);
        for sample in 0..config.interaction_samples {
            let (space, expected) = if sample % 2 == 0 {
                (default_space, default_space_view)
            } else {
                (benchmark_space, benchmark_space_view)
            };
            let measured = benchmark_space_switch(
                &mut client,
                &mut since,
                operation,
                space,
                expected,
                expected_sessions,
            );
            space_samples.push(measured.duration_ns);
            harness
                .raw_log
                .interactions
                .entry("space_switch".into())
                .or_default()
                .push(measured.duration_ns);
            operation += 1;
        }

        client.quit().expect("quit");
        wait_for_exit(&mut harness.child, deadline());

        let mut raw = BTreeMap::new();
        raw.insert("warm_tab_switch".to_string(), tab_samples);
        raw.insert("tab_select_pointer".to_string(), pointer_samples);
        raw.insert("tab_activate_keyboard".to_string(), activate_samples);
        raw.insert("tab_reopen".to_string(), reopen_samples);
        raw.insert("sidebar_toggle".to_string(), sidebar_samples);
        raw.insert("new_tab_command_bar".to_string(), new_tab_command_samples);
        raw.insert("new_tab_open".to_string(), new_tab_open_samples);
        raw.insert("active_tab_close".to_string(), active_close_samples);
        raw.insert("space_switch".to_string(), space_samples);
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
