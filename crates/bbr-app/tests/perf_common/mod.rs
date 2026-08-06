//! Shared support for the real-app performance E2E suites. See the individual
#![allow(dead_code)]
//! test files (`e2e_performance.rs`, `e2e_performance_full.rs`,
//! `e2e_performance_inject.rs`) for the two gate modes and the self-test.

pub mod baseline;
mod benchmarks;
mod drive;
pub mod harness;
pub mod machine;
mod report;
pub mod scenarios;
pub mod stats;

use scenarios::TrialConfig;
use std::path::PathBuf;

/// The checked-in accepted-baseline document path.
pub fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/performance-baseline.json")
}

/// Per-run artifact dir (raw JSON/CSV + event ring + stderr). Git-ignored.
pub fn artifacts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("bbr-perf-artifacts")
}

/// Persistent trend dir (latest.json + history.jsonl). Git-ignored.
pub fn trend_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("bbr-perf-trend")
}

/// A short-gate trial config (1 cold startup, 30 interactions, 10 new-tab).
pub fn short_config() -> TrialConfig {
    TrialConfig {
        bin: env!("CARGO_BIN_EXE_bbr"),
        engine_kind: "native-wkwebview",
        interaction_samples: scenarios::INTERACTION_SAMPLES,
        new_tab_samples: scenarios::NEW_TAB_SAMPLES,
        inject_delay_ms: None,
    }
}
