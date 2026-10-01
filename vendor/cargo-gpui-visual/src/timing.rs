//! Observational wall times, never readiness, FPS evidence or an acceptance input.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Instant};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Timings(pub BTreeMap<String, f64>);
impl Timings {
    pub fn record(&mut self, phase: &str, started: Instant) {
        *self.0.entry(phase.into()).or_default() += started.elapsed().as_secs_f64() * 1000.0;
    }
    pub fn measure<T>(&mut self, phase: &str, operation: impl FnOnce() -> T) -> T {
        let started = Instant::now();
        let result = operation();
        self.record(phase, started);
        result
    }
}
