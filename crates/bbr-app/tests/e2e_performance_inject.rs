//! Deterministic injected-delay self-test.
//!
//! Proves the performance gate actually detects real latency: arming the
//! testkit-only `--bbr-bench-inject-delay-ms` flag must make the relevant
//! interaction benchmark fail its absolute budget; removing the injection must
//! return green **without** changing any budget or baseline. Injection is
//! impossible in normal/release-without-testkit builds (armed only from the
//! `testkit`-gated CLI flag).

#![cfg(target_os = "macos")]

mod perf_common;

use perf_common::scenarios::{
    aggregate, evaluate_aggregate, run_cold_trial, TrialConfig, INTERACTION_P95_MS,
};
use perf_common::{artifacts_dir, baseline_path, short_config, trend_dir};
use std::panic::{catch_unwind, AssertUnwindSafe};

const INJECT_DELAY_MS: u64 = 150;

fn allow_gui() -> bool {
    !std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false")
}

/// Warm-tab-switch p95 from a fresh cold launch (drives a minimal scenario).
fn warm_tab_p95(inject_delay_ms: Option<u64>, tag: &str) -> (f64, Vec<String>) {
    let mut config: TrialConfig = short_config();
    config.inject_delay_ms = inject_delay_ms;
    let sample = run_cold_trial(&config, true, tag);
    let agg = aggregate(vec![sample]);
    let report = evaluate_aggregate(
        &agg,
        &config,
        &baseline_path(),
        &artifacts_dir().as_path().join(tag),
        &trend_dir(),
        "cold-fresh",
    );
    let p95 = report.results["warm_tab_switch"].p95_ms;
    (
        p95,
        report.regressions.iter().map(|r| r.rule.clone()).collect(),
    )
}

#[test]
#[ignore]
fn injected_delay_is_detected_and_removal_restores_green() {
    if !allow_gui() {
        return;
    }

    let result = catch_unwind(AssertUnwindSafe(|| {
        // 1) With the testkit-only delay armed, the gate MUST detect the
        //    inflation: warm-tab p95 exceeds the 50ms absolute budget and a
        //    regression is reported.
        let (injected_p95, injected_rules) = warm_tab_p95(Some(INJECT_DELAY_MS), "inject-bad");
        eprintln!(
            "SELF-TEST: injected warm-tab p95 {injected_p95:.2}ms against {INTERACTION_P95_MS:.0}ms budget"
        );
        assert!(
            injected_p95 > INTERACTION_P95_MS,
            "injected delay ({INJECT_DELAY_MS}ms) was not detected: p95 {injected_p95:.2}ms <= {INTERACTION_P95_MS:.0}ms",
        );
        assert!(
            injected_rules.iter().any(|r| r == "abs_p95"),
            "expected an abs_p95 regression, got {injected_rules:?}"
        );
        eprintln!("SELF-TEST: injected delay correctly fails the budget gate ✓");

        // 2) Removing the injection (same budget, same baseline) restores green.
        let (clean_p95, clean_rules) = warm_tab_p95(None, "inject-clean");
        eprintln!("SELF-TEST: clean warm-tab p95 {clean_p95:.2}ms");
        assert!(
            clean_p95 <= INTERACTION_P95_MS,
            "removing injection did not restore green: p95 {clean_p95:.2}ms > {INTERACTION_P95_MS:.0}ms",
        );
        assert!(
            clean_rules.is_empty(),
            "clean run reported unexpected regressions: {clean_rules:?}"
        );
        eprintln!(
            "SELF-TEST: removing injected delay restores green without budget/baseline change ✓"
        );
        eprintln!("SELF-TEST PASSED");
    }));
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}
