//! Ordered, source-sealed render journeys. A held frame is not a new rendering.
use crate::store::{image_path, Manifest};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    pub id: String,
    pub hz: u64,
    pub ticks: Vec<Tick>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tick {
    pub tick: u64,
    pub time_ns: u64,
    pub drawn: bool,
    pub dirty_before_callbacks: bool,
    pub callbacks_pending: bool,
    pub image: String,
    pub events: Vec<String>,
    pub state: Value,
}
impl Sequence {
    pub fn images(&self) -> impl Iterator<Item = &String> {
        self.ticks.iter().filter(|t| t.drawn).map(|t| &t.image)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequenceReview {
    pub actual: Sequence,
    pub expected: Option<Sequence>,
    pub first_difference: Option<usize>,
}

impl SequenceReview {
    pub fn status(&self) -> &'static str {
        if self.expected.is_none() {
            "missing-baseline"
        } else if self.first_difference.is_some() {
            "changed"
        } else {
            "same"
        }
    }
}

pub fn validate(run: &Path, manifest: &Manifest) -> Result<()> {
    validate_inventory(run, &manifest.cases, &manifest.sequences, true)
}

pub fn validate_inventory(
    run: &Path,
    cases: &[String],
    sequences: &[Sequence],
    require_pairs: bool,
) -> Result<()> {
    let mut ids = BTreeSet::new();
    let mut images = BTreeSet::new();
    for sequence in sequences {
        image_path(run, &sequence.id)?;
        ensure!(
            sequence.id.starts_with("sequence-") && !cases.contains(&sequence.id),
            "Invalid sequence namespace"
        );
        ensure!(ids.insert(&sequence.id), "Duplicate sequence ID");
        ensure!(
            [60, 120].contains(&sequence.hz),
            "Unsupported sequence cadence"
        );
        ensure!(
            !sequence.ticks.is_empty() && sequence.ticks.len() <= 120 * 60,
            "Invalid sequence length"
        );
        let (base, theme) = sequence.id.rsplit_once("--").unwrap();
        let mut previous = None;
        for (index, tick) in sequence.ticks.iter().enumerate() {
            ensure!(
                tick.tick == index as u64
                    && tick.time_ns == tick.tick * 1_000_000_000 / sequence.hz,
                "Missing/reordered sequence tick: {}",
                sequence.id
            );
            ensure!(
                cases.contains(&tick.image),
                "Sequence references uncaptured image"
            );
            if tick.drawn {
                ensure!(
                    tick.image == format!("{base}-frame-{index:04}--{theme}"),
                    "Frame identity must match sequence tick"
                );
                ensure!(images.insert(&tick.image), "Duplicate rendered frame");
                previous = Some(&tick.image);
            } else {
                ensure!(
                    previous == Some(&tick.image),
                    "Held tick must reference preceding rendered frame"
                );
            }
        }
        let counterpart = format!(
            "{base}--{}",
            if theme == "light" { "dark" } else { "light" }
        );
        ensure!(
            !require_pairs || sequences.iter().any(|s| s.id == counterpart),
            "Missing sequence appearance counterpart"
        );
    }
    for case in cases {
        if case.starts_with("sequence-") {
            ensure!(
                images.contains(case),
                "Sequence frame lacks timeline receipt"
            );
        }
    }
    Ok(())
}

pub fn first_difference(actual: &Sequence, expected: &Sequence) -> Option<usize> {
    if actual.hz != expected.hz || actual.id != expected.id {
        return Some(0);
    }
    actual
        .ticks
        .iter()
        .zip(&expected.ticks)
        .position(|(a, b)| a != b)
        .or_else(|| {
            (actual.ticks.len() != expected.ticks.len())
                .then_some(actual.ticks.len().min(expected.ticks.len()))
        })
}
