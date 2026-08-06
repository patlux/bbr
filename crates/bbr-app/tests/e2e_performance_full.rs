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
        let pointer = report.results["tab_select_pointer"].samples;
        let activate = report.results["tab_activate_keyboard"].samples;
        let reopen = report.results["tab_reopen"].samples;
        let space = report.results["space_switch"].samples;
        let pin = report.results["tab_pin"].samples;
        let reorder = report.results["tab_reorder"].samples;
        let same_move = report.results["tab_move_same_profile"].samples;
        let cross_move = report.results["tab_move_cross_profile"].samples;
        let dense_scroll = report.results["dense_scroll"].samples;
        let dense_select = report.results["dense_select"].samples;
        let hover = report.results["tab_hover"].samples;
        let press = report.results["tab_press"].samples;
        assert!(
            warm >= INTERACTION_SAMPLES,
            "warm-tab samples {warm} < {INTERACTION_SAMPLES}"
        );
        assert!(
            pointer >= INTERACTION_SAMPLES,
            "pointer-select samples {pointer} < {INTERACTION_SAMPLES}"
        );
        assert!(
            space >= INTERACTION_SAMPLES,
            "space-switch samples {space} < {INTERACTION_SAMPLES}"
        );
        assert!(
            pin >= INTERACTION_SAMPLES,
            "pin samples {pin} < {INTERACTION_SAMPLES}"
        );
        assert!(
            reorder >= INTERACTION_SAMPLES,
            "reorder samples {reorder} < {INTERACTION_SAMPLES}"
        );
        assert!(
            same_move >= INTERACTION_SAMPLES,
            "same-profile move samples {same_move} < {INTERACTION_SAMPLES}"
        );
        assert!(
            cross_move >= INTERACTION_SAMPLES,
            "cross-profile move samples {cross_move} < {INTERACTION_SAMPLES}"
        );
        assert!(
            dense_scroll >= INTERACTION_SAMPLES,
            "dense-scroll samples {dense_scroll} < {INTERACTION_SAMPLES}"
        );
        assert!(
            dense_select >= INTERACTION_SAMPLES,
            "dense-select samples {dense_select} < {INTERACTION_SAMPLES}"
        );
        assert!(
            hover >= INTERACTION_SAMPLES,
            "hover samples {hover} < {INTERACTION_SAMPLES}"
        );
        assert!(
            press >= INTERACTION_SAMPLES,
            "press samples {press} < {INTERACTION_SAMPLES}"
        );
        assert!(
            activate >= NEW_TAB_SAMPLES,
            "keyboard-activate samples {activate} < {NEW_TAB_SAMPLES}"
        );
        assert!(
            new_tab >= NEW_TAB_SAMPLES,
            "new-tab samples {new_tab} < {NEW_TAB_SAMPLES}"
        );
        assert!(
            reopen >= NEW_TAB_SAMPLES,
            "reopen samples {reopen} < {NEW_TAB_SAMPLES}"
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
