//! Compact versioned CLI protocol shared with Rust callers. Not an acceptance seal.
use crate::{compare::Review, diagnostic::Diagnostic};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub schema: u32,
    pub tool_version: String,
    pub command: String,
    pub status: String,
    pub exit_code: u8,
    pub data: Value,
    pub error: Option<Diagnostic>,
    #[serde(default)]
    pub timings_ms: crate::timing::Timings,
}
impl Response {
    pub fn new(command: &str, exit_code: u8, data: Value) -> Self {
        Self {
            schema: 1,
            tool_version: env!("CARGO_PKG_VERSION").into(),
            command: command.into(),
            status: match exit_code {
                0 => "ok",
                1 => "review_required",
                _ => "invalid",
            }
            .into(),
            exit_code,
            data,
            error: None,
            timings_ms: Default::default(),
        }
    }
    pub fn failure(command: &str, error: &anyhow::Error) -> Self {
        let mut result = Self::new(command, 2, Value::Null);
        result.error = Some(Diagnostic::from_error(error));
        result
    }
}

/// Counts logical stills and whole sequences, not thousands of frame findings.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Counts {
    pub same: usize,
    pub changed: usize,
    pub missing_baseline: usize,
    pub errors: usize,
}
impl Counts {
    fn add(&mut self, status: &str) {
        match status {
            "same" => self.same += 1,
            "changed" => self.changed += 1,
            "missing-baseline" => self.missing_baseline += 1,
            _ => self.errors += 1,
        }
    }
}
pub fn review_summary(review: &Review) -> Value {
    let mut counts = Counts {
        errors: review.errors.len(),
        ..Default::default()
    };
    for finding in review
        .results
        .iter()
        .filter(|r| !r.case.starts_with("sequence-"))
    {
        counts.add(&finding.status);
    }
    for sequence in &review.sequences {
        counts.add(sequence.status());
    }
    let ids: Vec<_> = review
        .selection
        .iter()
        .flat_map(|s| s.stories.iter().map(|s| &s.id))
        .collect();
    let scope = if review.selection.is_some() {
        "selected"
    } else {
        "full"
    };
    json!({
        "passed": review.passed,
        "scope": {"kind":scope,"stories":ids},
        "coverage":"not_independently_verified",
        "counts":counts,
        "environment_mismatch":review.environment_mismatch.is_some(),
        "acceptance":if review.selection.is_some() { "blocked_selected_scope" } else { "requires_explicit_review_and_revalidation" },
        "next_step":if !review.errors.is_empty() { "Inspect invalid comparison inputs; do not accept." }
            else if review.environment_mismatch.is_some() { "Investigate environment drift; do not automatically adopt." }
            else if !review.passed { "Inspect the changed or missing-reference images; references remain untouched." }
            else { "Exact match inside this scope only; consumer coverage and native gates remain separate." }
    })
}
pub fn review_result(
    review: &Review,
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
) -> Value {
    let mut value = review_summary(review);
    value["artifacts"] = json!({"repo":repo,"run":run,"baselines":baselines,"report":report,
        "review":report.join("review.json"),"html":report.join("index.html")});
    value
}
