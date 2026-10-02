//! Exact functional libtest evidence, not an image seal or coverage catalog.
use crate::{
    gallery_selection::bound_bytes,
    payload::{Entry, Payload},
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PLAN: &str = "scripts/visual/catalogs/gallery-semantic.json";
pub const RESULT: &str = "semantic-results.json";
pub const OUTPUT_LIMIT: u64 = 1024 * 1024;
pub(crate) const DIAGNOSTICS: [(&str, &str); 2] = [
    ("every-story-renders", "semantic-every-story-renders.json"),
    ("mode-state-switching", "semantic-mode-state-switching.json"),
];

/// Report-only side effects of existing sweep tests, not product budget evidence.
pub(crate) fn validate_diagnostic(suite: &str, bytes: &[u8]) -> Result<()> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Report {
        suite: String,
        sanity_only: bool,
        note: String,
        observations_ms: std::collections::BTreeMap<String, f64>,
    }
    ensure!(
        (1..=OUTPUT_LIMIT as usize).contains(&bytes.len()),
        "bounded semantic diagnostic required"
    );
    let report: Report = serde_json::from_slice(bytes)?;
    ensure!(
        report.suite == suite
            && report.sanity_only
            && report.note == "report-only frame-time observations; never a product budget"
            && !report.observations_ms.is_empty()
            && report.observations_ms.len() <= 10000
            && report
                .observations_ms
                .iter()
                .all(|(name, value)| !name.is_empty() && value.is_finite() && *value >= 0.0),
        "invalid report-only semantic diagnostic"
    );
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirements {
    schema: u32,
    tests: Vec<String>,
}

fn digest(text: &str) -> Result<()> {
    ensure!(
        text.len() == 64
            && text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "invalid semantic digest"
    );
    Ok(())
}
fn lines(text: &str) -> Result<Vec<&str>> {
    ensure!(
        text.len() as u64 <= OUTPUT_LIMIT && !text.chars().any(|c| c.is_control() && c != '\n'),
        "invalid/bounded semantic stdout required"
    );
    Ok(text.lines().filter(|line| !line.is_empty()).collect())
}
fn number(text: &str) -> Result<usize> {
    let value: usize = text.parse().context("invalid libtest count")?;
    ensure!(value.to_string() == text, "noncanonical libtest count");
    Ok(value)
}

impl Requirements {
    pub fn load(payload: &Payload) -> Result<Self> {
        ensure!(
            payload.schema == 1 && payload.suite == "gallery-semantic",
            "semantic payload purpose differs"
        );
        let hash = payload
            .sources
            .get(PLAN)
            .context("unbound semantic requirements")?;
        let plan: Self = serde_json::from_slice(&bound_bytes(&payload.repo, PLAN, hash)?)?;
        ensure!(
            plan.schema == 1 && (1..=512).contains(&plan.tests.len()),
            "invalid semantic requirement schema/count"
        );
        ensure!(
            plan.tests.windows(2).all(|pair| pair[0] < pair[1]),
            "semantic names must be unique and sorted"
        );
        for name in &plan.tests {
            let parts: Vec<_> = name.split("::").collect();
            ensure!(
                parts.len() == 3
                    && parts[0] == "e2e"
                    && parts[1..].iter().all(|part| {
                        !part.is_empty()
                            && part
                                .bytes()
                                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
                    }),
                "invalid semantic test name"
            );
        }
        Ok(plan)
    }

    pub fn validate_listing(&self, text: &str) -> Result<()> {
        let lines = lines(text)?;
        ensure!(
            lines.len() == self.tests.len() + 1,
            "semantic listing count differs"
        );
        for (line, expected) in lines.iter().zip(&self.tests) {
            ensure!(
                *line == format!("{expected}: test"),
                "semantic listing differs from independent requirements"
            );
        }
        ensure!(
            lines.last() == Some(&format!("{} tests, 0 benchmarks", self.tests.len()).as_str()),
            "semantic listing summary differs"
        );
        Ok(())
    }

    pub fn validate_execution(&self, text: &str) -> Result<()> {
        let lines = lines(text)?;
        ensure!(
            lines.len() == self.tests.len() + 2,
            "semantic result count differs"
        );
        ensure!(
            lines[0] == format!("running {} tests", self.tests.len()),
            "semantic run count differs"
        );
        for (line, expected) in lines[1..].iter().zip(&self.tests) {
            ensure!(
                *line == format!("test {expected} ... ok"),
                "semantic test missing/failed/duplicate/unexpected"
            );
        }
        let prefix = format!(
            "test result: ok. {} passed; 0 failed; 0 ignored; 0 measured; ",
            self.tests.len()
        );
        let tail = lines
            .last()
            .unwrap()
            .strip_prefix(&prefix)
            .context("semantic success summary differs")?;
        let (filtered, duration) = tail
            .split_once(" filtered out; finished in ")
            .context("incomplete semantic summary")?;
        number(filtered)?; // Unit tests excluded by --ignored; they are not guest successes.
        let duration = duration
            .strip_suffix('s')
            .context("missing semantic duration")?;
        let parts: Vec<_> = duration.split('.').collect();
        ensure!(
            parts.len() == 2
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
                && duration
                    .parse::<f64>()
                    .is_ok_and(|v| v.is_finite() && (0.0..=600.0).contains(&v)),
            "invalid semantic duration"
        );
        Ok(())
    }
}

pub(crate) fn executable(payload: &Payload) -> Result<&Entry> {
    let prefix = payload.repo.join("target/release/deps");
    let matches: Vec<_> = payload
        .files
        .iter()
        .filter(|entry| {
            entry.executable
                && entry.guest.parent() == Some(prefix.as_path())
                && entry
                    .guest
                    .file_name()
                    .and_then(|n| n.to_str())
                    .and_then(|n| n.strip_prefix("bbr_gallery-"))
                    .is_some_and(|suffix| {
                        !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_alphanumeric())
                    })
        })
        .collect();
    ensure!(
        matches.len() == 1,
        "exactly one installed semantic harness required"
    );
    digest(&matches[0].sha256)?;
    Ok(matches[0])
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    schema: u32,
    origin: String,
    payload_sha256: String,
    plan_sha256: String,
    executable: PathBuf,
    executable_sha256: String,
    listing: String,
    execution: String,
}
impl Record {
    pub fn new(
        payload: &Payload,
        expected: &str,
        listing: String,
        execution: String,
    ) -> Result<Self> {
        let plan = Requirements::load(payload)?;
        plan.validate_listing(&listing)?;
        plan.validate_execution(&execution)?;
        digest(expected)?;
        let binary = executable(payload)?;
        Ok(Self {
            schema: 1,
            origin: "gallery-semantic-libtest".into(),
            payload_sha256: expected.into(),
            plan_sha256: payload.sources[PLAN].clone(),
            executable: binary.guest.clone(),
            executable_sha256: binary.sha256.clone(),
            listing,
            execution,
        })
    }

    pub fn validate(&self, payload: &Payload, expected: &str) -> Result<()> {
        let checked = Self::new(
            payload,
            expected,
            self.listing.clone(),
            self.execution.clone(),
        )?;
        ensure!(
            self.schema == checked.schema
                && self.origin == checked.origin
                && self.payload_sha256 == checked.payload_sha256
                && self.plan_sha256 == checked.plan_sha256
                && self.executable == checked.executable
                && self.executable_sha256 == checked.executable_sha256,
            "semantic result identity differs"
        );
        Ok(())
    }
}
