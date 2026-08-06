//! Distribution statistics over raw nanosecond latency samples.
#![allow(dead_code)]

use serde::Serialize;

/// A summary distribution over one benchmark scenario.
#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub samples: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
}

impl Stats {
    pub fn from_ns(samples: &[u64]) -> Self {
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

/// The nearest-rank percentile (0..=100) over already-sorted samples.
pub fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    let index = ((sorted.len() * percentile).div_ceil(100)).saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

/// The nearest-rank percentile over already-sorted `f64` samples.
pub fn percentile_f64(sorted: &[f64], percentile: usize) -> f64 {
    let index = ((sorted.len() * percentile).div_ceil(100)).saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

pub fn ns_to_ms(ns: u64) -> f64 {
    ns as f64 / 1_000_000.0
}

/// Distribution over millisecond samples (e.g. startup/paged-ready which are
/// measured against the process-spawn monotonic clock reference).
pub fn stats_from_ms(samples: &[f64]) -> super::harness::KindStats {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    super::harness::KindStats {
        samples: sorted.len(),
        p50_ms: percentile_f64(&sorted, 50),
        p95_ms: percentile_f64(&sorted, 95),
        max_ms: *sorted.last().unwrap_or(&0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_edges() {
        assert_eq!(percentile(&[1, 2, 3, 4], 50), 2);
        assert_eq!(percentile(&[1, 2, 3, 4], 95), 4);
        assert_eq!(percentile(&[5], 50), 5);
    }

    #[test]
    fn stats_max_is_last_sorted() {
        let stats = Stats::from_ns(&[3, 1, 2, 10]);
        assert_eq!(stats.samples, 4);
        assert_eq!(stats.max_ms, 10.0 / 1_000_000.0);
        assert_eq!(stats.p50_ms, 2.0 / 1_000_000.0);
    }
}
