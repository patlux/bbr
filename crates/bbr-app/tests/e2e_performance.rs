//! Optimized real-app performance E2E for startup, warm-tab switching, and
//! sidebar presentation. Timings come from in-process production-path probes;
//! testkit transport is excluded from interaction samples.

#![cfg(target_os = "macos")]

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bbr_fixtures::FixtureServer;
use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, Event, PresentationMeasurement, Request,
};
use bbr_testkit::Client;
use bbr_web::{WebEvent, WebViewId};
use serde_json::json;

const STARTUP_MAX_MS: f64 = 750.0;
const INTERACTION_P95_MS: f64 = 50.0;
const INTERACTION_MAX_MS: f64 = 100.0;
const INTERACTION_SAMPLES: usize = 30;
const NEW_TAB_SAMPLES: usize = 10;
const PAGE_ONE_TITLE: &str = "BBR Fixture One";
const PAGE_TWO_TITLE: &str = "BBR Fixture Two";

struct Harness {
    child: Child,
    root: PathBuf,
    stderr: Arc<Mutex<String>>,
    success: bool,
}

impl Harness {
    fn spawn(bin: &str, args: &[&str], root: &Path) -> Self {
        let mut child = Command::new(bin)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn optimized bbr");
        let stderr_buf = Arc::new(Mutex::new(String::new()));
        if let Some(mut stderr) = child.stderr.take() {
            let buf = stderr_buf.clone();
            std::thread::spawn(move || {
                use std::io::Read;
                let mut chunk = [0u8; 4096];
                while let Ok(n) = stderr.read(&mut chunk) {
                    if n == 0 {
                        break;
                    }
                    if let Ok(mut text) = buf.lock() {
                        text.push_str(&String::from_utf8_lossy(&chunk[..n]));
                        if text.len() > 131_072 {
                            let keep = text.len() - 131_072;
                            *text = text.split_off(keep);
                        }
                    }
                }
            });
        }
        Self {
            child,
            root: root.to_path_buf(),
            stderr: stderr_buf,
            success: false,
        }
    }

    fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }

    fn diagnostics(&mut self, socket: &Path, token: &str) {
        if let Ok(mut client) = Client::connect(socket) {
            if client.hello(token).is_ok() {
                if let Ok(events) = client.attach(None) {
                    let text = events
                        .into_iter()
                        .map(|(seq, event)| format!("{seq}: {event:?}\n"))
                        .collect::<String>();
                    let _ = std::fs::write(self.root.join("event-ring.txt"), text);
                }
            }
        }
        self.terminate();
        let stderr = self.stderr.lock().map(|s| s.clone()).unwrap_or_default();
        let _ = std::fs::write(self.root.join("app-stderr.txt"), &stderr);
        eprintln!("--- app stderr ---\n{stderr}");
        eprintln!("performance artifacts kept at {}", self.root.display());
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.terminate();
        if self.success {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

#[test]
#[ignore]
fn startup_warm_tab_switch_and_sidebar_meet_initial_budgets() {
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");

    let root = std::env::temp_dir().join(format!("bbr-e2e-perf-{}", std::process::id()));
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = profile.join("bbr-test.sock");
    let token = format!("perf-{}", std::process::id());
    let bin = env!("CARGO_BIN_EXE_bbr");
    let spawned_at_ns = monotonic_now_ns();
    let mut harness = Harness::spawn(
        bin,
        &[
            "--bbr-initial-url",
            &page_one,
            "--bbr-profile-dir",
            profile.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
        ],
        &root,
    );

    let result = catch_unwind(AssertUnwindSafe(|| {
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
            "startup presentation {startup_ms:.2}ms exceeded {STARTUP_MAX_MS:.0}ms"
        );

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        client
            .send_ok(&bbr_testkit::protocol::dsl::open_tab(&page_two))
            .expect("open second tab");
        let second = client.snapshot().expect("second tab snapshot");
        let second_view = second.active_web_view.expect("second native view");
        if second.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(&mut client, second_view, PAGE_TWO_TITLE, since);
        }

        // One warm-up round is intentionally discarded.
        let mut operation = 1_u64;
        benchmark_select(&mut client, &mut since, operation, 0, first_view);
        operation += 1;
        benchmark_select(&mut client, &mut since, operation, 1, second_view);
        operation += 1;

        let mut tab_samples = Vec::with_capacity(INTERACTION_SAMPLES);
        for sample in 0..INTERACTION_SAMPLES {
            let (index, expected) = if sample % 2 == 0 {
                (0, first_view)
            } else {
                (1, second_view)
            };
            let measured = benchmark_select(&mut client, &mut since, operation, index, expected);
            tab_samples.push(measured.duration_ns);
            operation += 1;
        }

        // Warm up sidebar geometry in both directions before collecting samples.
        benchmark_sidebar(&mut client, &mut since, operation, false);
        operation += 1;
        benchmark_sidebar(&mut client, &mut since, operation, true);
        operation += 1;

        let mut sidebar_samples = Vec::with_capacity(INTERACTION_SAMPLES);
        let mut expected_sidebar = true;
        for _ in 0..INTERACTION_SAMPLES {
            expected_sidebar = !expected_sidebar;
            let measured = benchmark_sidebar(&mut client, &mut since, operation, expected_sidebar);
            sidebar_samples.push(measured.duration_ns);
            operation += 1;
        }

        // Measure the real shortcut flow. Cmd-T must present only the command
        // bar; Enter creates the native session; Cmd-W presents the neighbor.
        let mut new_tab_command_samples = Vec::with_capacity(NEW_TAB_SAMPLES);
        let mut new_tab_open_samples = Vec::with_capacity(NEW_TAB_SAMPLES);
        let mut active_close_samples = Vec::with_capacity(NEW_TAB_SAMPLES);
        for _ in 0..NEW_TAB_SAMPLES {
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

        let tab_stats = Stats::from_ns(&tab_samples);
        let sidebar_stats = Stats::from_ns(&sidebar_samples);
        let new_tab_command_stats = Stats::from_ns(&new_tab_command_samples);
        let new_tab_open_stats = Stats::from_ns(&new_tab_open_samples);
        let active_close_stats = Stats::from_ns(&active_close_samples);
        assert_interaction_budget("warm tab switch", &tab_stats);
        assert_interaction_budget("sidebar toggle", &sidebar_stats);
        assert_interaction_budget("new-tab command bar", &new_tab_command_stats);
        assert_interaction_budget("active-tab close", &active_close_stats);

        let report = json!({
            "startup_ms": startup_ms,
            "warm_tab_switch": tab_stats,
            "sidebar_toggle": sidebar_stats,
            "new_tab_command_bar": new_tab_command_stats,
            "new_tab_open": new_tab_open_stats,
            "active_tab_close": active_close_stats,
            "samples": {
                "warm_tab_switch_ns": tab_samples,
                "sidebar_toggle_ns": sidebar_samples,
                "new_tab_command_bar_ns": new_tab_command_samples,
                "new_tab_open_ns": new_tab_open_samples,
                "active_tab_close_ns": active_close_samples,
            },
            "budgets_ms": {
                "startup_max": STARTUP_MAX_MS,
                "interaction_p95": INTERACTION_P95_MS,
                "interaction_max": INTERACTION_MAX_MS,
            },
            "binary": bin,
            "engine": "native-wkwebview",
        });
        std::fs::write(
            root.join("performance.json"),
            serde_json::to_vec_pretty(&report).expect("serialize performance report"),
        )
        .expect("write performance report");
        eprintln!(
            "PERFORMANCE RESULTS\n{}",
            serde_json::to_string_pretty(&report).unwrap()
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    match result {
        Ok(()) => {
            harness.success = true;
            eprintln!("PERFORMANCE E2E PASSED");
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
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

#[derive(Debug, serde::Serialize)]
struct Stats {
    samples: usize,
    p50_ms: f64,
    p95_ms: f64,
    max_ms: f64,
}

impl Stats {
    fn from_ns(samples: &[u64]) -> Self {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        Self {
            samples: sorted.len(),
            p50_ms: ns_to_ms(percentile(&sorted, 50)),
            p95_ms: ns_to_ms(percentile(&sorted, 95)),
            max_ms: ns_to_ms(*sorted.last().expect("at least one sample")),
        }
    }
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    let index = ((sorted.len() * percentile).div_ceil(100)).saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

fn ns_to_ms(ns: u64) -> f64 {
    ns as f64 / 1_000_000.0
}

fn assert_interaction_budget(label: &str, stats: &Stats) {
    assert!(
        stats.p95_ms <= INTERACTION_P95_MS,
        "{label} p95 {:.2}ms exceeded {:.0}ms",
        stats.p95_ms,
        INTERACTION_P95_MS
    );
    assert!(
        stats.max_ms <= INTERACTION_MAX_MS,
        "{label} max {:.2}ms exceeded {:.0}ms",
        stats.max_ms,
        INTERACTION_MAX_MS
    );
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
