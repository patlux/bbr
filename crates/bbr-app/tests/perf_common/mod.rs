//! Shared support for the real-app performance E2E suites. See the individual
#![allow(dead_code)]
//! test files (`e2e_performance.rs`, `e2e_performance_full.rs`,
//! `e2e_performance_inject.rs`) for the two gate modes and the self-test.

mod auth;
mod auth_drive;
mod background_links;
mod background_links_drive;
pub mod baseline;
mod benchmarks;
mod benchmarks_u2;
mod command;
mod command_drive;
mod command_full_search;
mod context_menu;
mod context_menu_drive;
mod downloads;
mod downloads_drive;
mod drive;
mod errors_drive;
mod find;
mod find_drive;
pub mod harness;
mod idle;
mod library_drive;
pub mod machine;
mod peek;
mod peek_drive;
mod report;
mod responsiveness;
mod saved_pages_drive;
pub mod scenarios;
mod space_config_drive;
pub mod stats;
mod tab_menu;
mod tab_menu_drive;
mod zoom;
mod zoom_drive;

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
        // Phase-7 command-history provider scale (queue item 16): the
        // interaction matrix measures the command surface against the
        // history-rich corpus — the pre-spawned 10,000-entry provider feeds
        // every command ranking pass (never fixture setup inside a sample),
        // and the `command_bar_first_result` scenario adds the >=500 open
        // tabs on top.
        seed_history: 10_000,
        // B2 Saved Page provider scale (packet `saved-pages-actions-command`):
        // the interaction matrix measures the Saved Page command-text path
        // against the pre-spawned ≥1,000-record provider (never fixture
        // setup inside a sample).
        seed_saved_pages: 1_000,
    }
}
