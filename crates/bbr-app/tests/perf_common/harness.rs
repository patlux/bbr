//! Shared real-app harness: spawn the optimized `bbr` binary, capture stderr,
#![allow(dead_code)]
//! and emit robust failure artifacts (JSON + CSV + event ring + stderr) and
//! success trend output.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;

use super::machine::MachineInfo;
use super::stats::Stats;

/// A distribution summary per benchmark kind (serde-compatible for JSON/CSV).
#[derive(Debug, Clone, Serialize)]
pub struct KindStats {
    pub samples: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
}

impl From<&Stats> for KindStats {
    fn from(s: &Stats) -> Self {
        Self {
            samples: s.samples,
            p50_ms: s.p50_ms,
            p95_ms: s.p95_ms,
            max_ms: s.max_ms,
        }
    }
}

/// The full per-run performance report persisted to artifacts.
#[derive(Debug, Clone, Serialize)]
pub struct PerfReport {
    pub machine: MachineInfo,
    pub fixture_mode: String,
    /// Percentile summaries keyed by benchmark-kind string.
    pub results: BTreeMap<String, KindStats>,
    /// Raw nanosecond samples keyed by benchmark-kind string.
    pub raw_ns: BTreeMap<String, Vec<u64>>,
    /// Baseline regressions tripped (empty == green).
    pub regressions: Vec<super::baseline::Regression>,
    pub baseline_generated_revision: String,
    /// Absolute interaction p95/max ceilings (ms), always enforced.
    pub interaction_p95_ms: f64,
    pub interaction_max_ms: f64,
    pub kept_at: String,
    /// Set only on a failure-path (partial) artifact, explaining why the full
    /// post-trial evaluation never ran. Success runs omit the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

/// Raw samples accumulated live by a trial so a panic inside it (before
/// [`write_artifacts`] over the aggregate ever runs) can still persist JSON +
/// CSV. This is the data that matters most on a slow machine that trips the
/// startup/page-ready ceilings.
#[derive(Default, Clone)]
pub struct RawLog {
    pub startup_ms: Vec<f64>,
    pub page_ready_ms: Vec<f64>,
    pub interactions: BTreeMap<String, Vec<u64>>,
}

/// Spawns and supervises one `bbr` process for a benchmark run.
pub struct AppHarness {
    pub child: Child,
    pub root: PathBuf,
    stderr_buf: Arc<Mutex<String>>,
    /// Live sample accumulation so failure-path artifacts retain raw data even
    /// when a trial panics before [`write_artifacts`] over the aggregate runs.
    pub raw_log: RawLog,
    pub success: bool,
}

impl AppHarness {
    pub fn spawn(bin: &str, args: &[&str], root: &Path) -> Self {
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
            stderr_buf,
            raw_log: RawLog::default(),
            success: false,
        }
    }

    pub fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }

    /// Dump the event ring + stderr into the run root and stop the app. Used
    /// on failure so diagnostics survive the panic unwind.
    pub fn diagnostics(&mut self, socket: &Path, token: &str) {
        if let Ok(mut client) = bbr_testkit::Client::connect(socket) {
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
        let stderr = self
            .stderr_buf
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default();
        let _ = std::fs::write(self.root.join("app-stderr.txt"), &stderr);
        eprintln!("--- app stderr ---\n{stderr}");
        eprintln!("performance artifacts kept at {}", self.root.display());
    }
}

impl Drop for AppHarness {
    fn drop(&mut self) {
        self.terminate();
        if self.success {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

/// Persist JSON + CSV summaries and raw samples to `dir`.
pub fn write_artifacts(dir: &Path, report: &PerfReport) {
    std::fs::create_dir_all(dir).ok();
    let json = serde_json::to_vec_pretty(report).expect("serialize report");
    let _ = std::fs::write(dir.join("performance.json"), json);

    // Raw samples CSV: one row per sample across kinds.
    let mut csv = String::from("kind,sample_index,duration_ns\n");
    for (kind, samples) in &report.raw_ns {
        for (index, ns) in samples.iter().enumerate() {
            csv.push_str(&format!("{kind},{index},{ns}\n"));
        }
    }
    let _ = std::fs::write(dir.join("raw_samples.csv"), csv);
}

/// Append the summary to the persistent trend JSON (latest run retained, full
/// history kept as a line-oriented record). Returns the trend dir path.
pub fn append_trend(report: &PerfReport, trend_dir: &Path) -> PathBuf {
    std::fs::create_dir_all(trend_dir).ok();
    let latest = trend_dir.join("latest.json");
    let summary = serde_json::to_value(&report.results).expect("results serialize");
    let frame = serde_json::json!({
        "revision": report.machine.build_revision,
        "fixture_mode": report.fixture_mode,
        "canonical": report.machine.canonical,
        "results": summary,
    });
    let _ = std::fs::write(&latest, serde_json::to_vec_pretty(&frame).unwrap());
    // Line-oriented history.
    let history = trend_dir.join("history.jsonl");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", serde_json::to_string(&frame).unwrap());
    }
    eprintln!(
        "TREND {} at {}",
        report.machine.build_revision,
        trend_dir.display()
    );
    latest
}

/// Waits for the app process to exit, or panics past `deadline`.
pub fn wait_for_exit(child: &mut Child, deadline: Instant) -> std::process::ExitStatus {
    loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            return status;
        }
        assert!(Instant::now() < deadline, "app did not exit");
        std::thread::sleep(Duration::from_millis(20));
    }
}
