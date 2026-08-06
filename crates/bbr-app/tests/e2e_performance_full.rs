//! Full repeated canonical benchmark gate (`just e2e-perf-full`).
//!
//! Materially different from the short gate: it launches the real optimized
//! app `COLD_STARTUP_SAMPLES` times against fresh profiles to sample the cold
//! startup + deterministic-page-ready distributions (contract requires ≥10
//! startup samples per mode), and drives `INTERACTION_SAMPLES` warm
//! interactions once for the percentile-heavy interaction rules. Trend and
//! raw artifacts are persisted. Bounded by the recipe's longer timeout.

#![cfg(target_os = "macos")]

mod perf_common;

use perf_common::scenarios::{
    aggregate, evaluate_aggregate, run_cold_trial, ColdSample, TrialConfig, COLD_STARTUP_SAMPLES,
    INTERACTION_SAMPLES, NEW_TAB_SAMPLES,
};
use perf_common::{artifacts_dir, baseline_path, short_config, trend_dir};
use std::panic::{catch_unwind, AssertUnwindSafe};

fn allow_gui() -> bool {
    !std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false")
}

#[test]
#[ignore]
fn full_repeated_cold_benchmark_meets_budgets() {
    if !allow_gui() {
        return;
    }

    let config: TrialConfig = short_config();

    let result = catch_unwind(AssertUnwindSafe(|| {
        // Trial 0 drives the interaction matrix; the rest only measure cold
        // startup + page readiness so startup distributions reach the
        // contract-required sample count without repeating warm interactions.
        let mut samples: Vec<ColdSample> = Vec::with_capacity(COLD_STARTUP_SAMPLES);
        for i in 0..COLD_STARTUP_SAMPLES {
            let drive = i == 0;
            samples.push(run_cold_trial(&config, drive, &format!("full-{i}")));
        }

        let agg = aggregate(samples);
        let report = evaluate_aggregate(
            &agg,
            &config,
            &baseline_path(),
            &artifacts_dir().as_path().join("full"),
            &trend_dir(),
            "cold-fresh",
        );

        // Contract-required repetition counts (startup per mode, interactions).
        assert!(
            agg.startup_ms.len() >= COLD_STARTUP_SAMPLES,
            "cold startup samples {} < {COLD_STARTUP_SAMPLES}",
            agg.startup_ms.len()
        );
        assert!(
            agg.page_ready_ms.len() >= COLD_STARTUP_SAMPLES,
            "page-ready samples {} < {COLD_STARTUP_SAMPLES}",
            agg.page_ready_ms.len()
        );
        let warm = report.results["warm_tab_switch"].samples;
        let new_tab = report.results["new_tab_command_bar"].samples;
        assert!(
            warm >= INTERACTION_SAMPLES,
            "warm-tab samples {warm} < {INTERACTION_SAMPLES}"
        );
        assert!(
            new_tab >= NEW_TAB_SAMPLES,
            "new-tab samples {new_tab} < {NEW_TAB_SAMPLES}"
        );

        let canonical_note = if report.machine.canonical {
            "CANONICAL MACHINE — relative baseline enforced"
        } else {
            "REPORT-ONLY machine — relative baseline reported, absolute ceilings enforced"
        };
        eprintln!("PERFORMANCE-FULL MACHINE: {canonical_note}");
        eprintln!(
            "PERFORMANCE-FULL RESULTS\n{}",
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
        eprintln!("PERFORMANCE-FULL E2E PASSED");
    }));

    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}
