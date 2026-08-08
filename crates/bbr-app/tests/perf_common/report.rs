//! Benchmark report aggregation and evaluation.
//!
//! Combines one or more cold trials into raw distributions, evaluates the
//! absolute ceilings plus the canonical-machine >15% p95 regression rule,
//! and writes the JSON/CSV artifacts and the persistent trend frame.

use std::collections::BTreeMap;
use std::path::Path;

use super::baseline::{self, Baseline};
use super::harness::{self, PerfReport};
use super::machine::MachineInfo;
use super::scenarios::TrialConfig;
use super::stats::{stats_from_ms, Stats};

/// Aggregated measurements across one or more cold trials.
#[derive(Default)]
pub struct Aggregated {
    pub startup_ms: Vec<f64>,
    pub page_ready_ms: Vec<f64>,
    /// Persisted warm-startup samples (Phase 7, warm-restore slice).
    pub warm_startup_ms: Vec<f64>,
    /// Persisted warm-startup page-ready samples (same trials).
    pub warm_page_ready_ms: Vec<f64>,
    pub interactions: BTreeMap<String, Vec<u64>>,
}

impl Aggregated {
    fn push(&mut self, sample: super::scenarios::ColdSample) {
        self.startup_ms.push(sample.startup_ms);
        self.page_ready_ms.push(sample.page_ready_ms);
        if let Some(warm) = sample.warm_startup_ms {
            self.warm_startup_ms.push(warm);
        }
        if let Some(warm) = sample.warm_page_ready_ms {
            self.warm_page_ready_ms.push(warm);
        }
        for (kind, raw) in sample.interactions {
            self.interactions.entry(kind).or_default().extend(raw);
        }
    }
}

/// Aggregate the live sample log a harness keeps so a failure-path report can
/// include everything a panicking trial had already collected.
impl From<&super::harness::RawLog> for Aggregated {
    fn from(log: &super::harness::RawLog) -> Self {
        Self {
            startup_ms: log.startup_ms.clone(),
            page_ready_ms: log.page_ready_ms.clone(),
            warm_startup_ms: Vec::new(),
            warm_page_ready_ms: Vec::new(),
            interactions: log.interactions.clone(),
        }
    }
}

/// Aggregate one or more cold trials into raw distributions.
pub fn aggregate(samples: Vec<super::scenarios::ColdSample>) -> Aggregated {
    let mut agg = Aggregated::default();
    for sample in samples {
        agg.push(sample);
    }
    agg
}

/// Evaluate baseline regressions for an aggregated run and return the report.
pub fn evaluate_aggregate(
    agg: &Aggregated,
    config: &TrialConfig,
    baseline_path: &Path,
    artifacts_dir: &Path,
    trend_dir: &Path,
    fixture_mode: &str,
) -> PerfReport {
    let (report, _baseline) = report_from_agg(
        agg,
        config,
        baseline_path,
        artifacts_dir,
        fixture_mode,
        None,
    );
    harness::write_artifacts(artifacts_dir, &report);
    // Inject self-test runs deliberately inflate latencies; they must never be
    // indistinguishable from a real regression in the persistent trend.
    if config.inject_delay_ms.is_none() {
        harness::append_trend(&report, trend_dir);
    }
    report
}

/// Build a [`PerfReport`] from raw aggregate data, evaluating absolute and
/// canonical-relative baselines. Shared by the success path and the failure
/// path (which passes a partial log + a failure note).
pub fn report_from_agg(
    agg: &Aggregated,
    config: &TrialConfig,
    baseline_path: &Path,
    artifacts_dir: &Path,
    fixture_mode: &str,
    failure: Option<String>,
) -> (PerfReport, Baseline) {
    let baseline = Baseline::load_or_seed(baseline_path);
    let info = MachineInfo::collect(
        config.bin,
        config.engine_kind,
        fixture_mode,
        &baseline.canonical_host_model,
    );

    let mut results = BTreeMap::new();
    let mut raw_ns = BTreeMap::new();
    results.insert("startup".into(), stats_from_ms(&agg.startup_ms));
    results.insert("page_ready".into(), stats_from_ms(&agg.page_ready_ms));
    // Phase-7 persisted warm-startup scenario (warm-restore slice): the
    // distribution over the persisted-profile trials, evaluated against the
    // contract budgets (p95 ≤ 500 ms startup / ≤ 1,000 ms warm page-ready).
    if !agg.warm_startup_ms.is_empty() {
        results.insert(
            "persisted_warm_startup".into(),
            stats_from_ms(&agg.warm_startup_ms),
        );
        results.insert(
            "persisted_warm_page_ready".into(),
            stats_from_ms(&agg.warm_page_ready_ms),
        );
    }
    for (kind, raw) in &agg.interactions {
        results.insert(kind.clone(), harness::KindStats::from(&Stats::from_ns(raw)));
        raw_ns.insert(kind.clone(), raw.clone());
    }

    let measured = aggregate_measured(&results);
    let regressions = baseline::evaluate(&measured, &baseline, info.canonical);

    let kept_at = artifacts_dir.display().to_string();
    let report = PerfReport {
        machine: info,
        fixture_mode: fixture_mode.to_string(),
        results,
        raw_ns,
        regressions,
        baseline_generated_revision: baseline.generated_revision.clone(),
        interaction_p95_ms: super::scenarios::INTERACTION_P95_MS,
        interaction_max_ms: super::scenarios::INTERACTION_MAX_MS,
        kept_at,
        failure,
    };
    (report, baseline)
}

fn aggregate_measured(
    results: &BTreeMap<String, harness::KindStats>,
) -> BTreeMap<String, baseline::Measured> {
    results
        .iter()
        .map(|(kind, s)| {
            (
                kind.clone(),
                baseline::Measured {
                    samples: s.samples,
                    p50_ms: s.p50_ms,
                    p95_ms: s.p95_ms,
                    max_ms: s.max_ms,
                },
            )
        })
        .collect()
}

/// Best-effort human-readable message from a caught panic.
pub fn panic_message(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic".to_string()
    }
}
