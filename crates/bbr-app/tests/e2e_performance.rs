//! Optimized real-app performance E2E — short PR/local regression gate.
//!
//! One cold fresh-profile launch of the real optimized `bbr` app measures:
//! process-spawn → first usable presentation (sidebar + selected native view),
//! a distinct deterministic-page-ready milestone, and the warm interaction
//! matrix (tab switch, sidebar toggle, New-Tab command bar, new-tab open,
//! active-tab close). Timings come from in-process production-path probes with
//! testkit transport excluded.
//!
//! Absolute ceilings are always enforced. The >15% p95 relative rule is
//! enforced only on the explicitly identifiable canonical machine; elsewhere
//! it stays report-only. On success the summary goes to the persistent trend;
//! on failure JSON + CSV + event ring + app stderr are retained.

#![cfg(target_os = "macos")]

mod perf_common;

use perf_common::scenarios::{
    aggregate, evaluate_aggregate, run_cold_trial, run_persisted_warm_trial, TrialConfig,
};
use perf_common::{artifacts_dir, baseline_path, short_config, trend_dir};
use std::panic::{catch_unwind, AssertUnwindSafe};

fn allow_gui() -> bool {
    !std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false")
}

#[test]
#[ignore]
fn startup_warm_tab_switch_and_sidebar_meet_initial_budgets() {
    if !allow_gui() {
        return;
    }

    let config: TrialConfig = short_config();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let sample = run_cold_trial(&config, true, "short");
        // Phase-7 persisted warm-startup scenario (warm-restore slice): one
        // warm sample through the actual repository restore path against the
        // dense persisted fixture, with the semantic fast-but-wrong guards
        // asserted inside the trial.
        let (warm_startup, warm_page_ready) = run_persisted_warm_trial(&config, "short");
        let mut agg = aggregate(vec![sample]);
        agg.warm_startup_ms.push(warm_startup);
        agg.warm_page_ready_ms.push(warm_page_ready);
        let report = evaluate_aggregate(
            &agg,
            &config,
            &baseline_path(),
            &artifacts_dir().as_path().join("short"),
            &trend_dir(),
            "cold-fresh",
        );

        let canonical_note = if report.machine.canonical {
            "CANONICAL MACHINE — relative baseline enforced"
        } else {
            "REPORT-ONLY machine — relative baseline reported, absolute ceilings enforced"
        };
        eprintln!("PERFORMANCE MACHINE: {canonical_note}");
        eprintln!(
            "PERFORMANCE RESULTS\n{}",
            serde_json::to_string_pretty(&report).unwrap()
        );

        if !report.regressions.is_empty() {
            for r in &report.regressions {
                eprintln!("REGRESSION {:?} {}: {}", r.kind, r.rule, r.detail);
            }
            panic!(
                "performance regressions: {}",
                serde_json::to_string(&report.regressions).unwrap()
            );
        }
        eprintln!("PERFORMANCE E2E PASSED");
    }));

    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}
