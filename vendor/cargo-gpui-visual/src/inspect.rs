//! Read-only projection of stored review evidence; never a comparator or seal authority.
use crate::{agent, collection::Collection, compare::Review, sequence::SequenceReview, store};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Debug)]
pub struct Options {
    pub case: Option<String>,
    pub offset: usize,
    pub limit: usize,
    pub tick: Option<usize>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            case: None,
            offset: 0,
            limit: 12,
            tick: None,
        }
    }
}

/// Reads only the explicit report. Does not run Git/probes, acquire/write locks,
/// regenerate reports, resolve implicit latest, or certify current source validity.
pub fn inspect(root: &Path, options: &Options) -> Result<Value> {
    ensure!(
        (1..=100).contains(&options.limit),
        "Inspect limit must be 1..=100"
    );
    ensure!(
        options.tick.is_none() || options.case.is_some(),
        "--tick requires --case"
    );
    let root = store::absolute(root)?;
    store::no_symlinks(&root)?;
    ensure!(
        !root.join(".review.lock").exists(),
        "Report is locked or interrupted; inspect only a completed report"
    );
    let is_collection = root.join("collection.json").is_file();
    let document = root.join(if is_collection {
        "collection.json"
    } else {
        "review.json"
    });
    store::no_symlinks(&document)?;
    let before = store::digest(&document)?;
    let mut result = if is_collection {
        let collection: Collection = store::read_json(&document)?;
        ensure!(collection.schema == 1, "Unsupported collection schema");
        let mut suites = BTreeSet::new();
        for s in &collection.suites {
            store::image_path(Path::new("."), &format!("{}--light", s.suite.id))?;
            ensure!(suites.insert(&s.suite.id), "Duplicate suite identity");
        }
        if let Some(id) = &options.case {
            let (suite, case) = id
                .split_once(':')
                .context("Collection inspection requires exact suite:case")?;
            let s = collection
                .suites
                .iter()
                .find(|s| s.suite.id == suite)
                .context("Unknown suite")?;
            let review = s
                .review
                .as_ref()
                .context("Suite has no valid stored review")?;
            let mut options = options.clone();
            options.case = Some(case.into());
            let mut value = inspect_review(&root.join("suites").join(suite), review, &options)?;
            value["suite"] = json!(suite);
            value["suite_errors"] = json!(s.errors.iter().take(12).collect::<Vec<_>>());
            value
        } else {
            let entries: Vec<_> = collection.suites.iter().skip(options.offset).take(options.limit).map(|s| json!({"id":s.suite.id,"passed":s.passed,"error_count":s.errors.len(),"review":s.review.as_ref().map(agent::review_summary)})).collect();
            json!({"passed":collection.passed,"suites":entries,"total_suites":collection.suites.len(),"next_offset":next_offset(options, collection.suites.len()),"collection_integrity":"not_revalidated"})
        }
    } else {
        let review: Review = store::read_json(&document)?;
        inspect_review(&root, &review, options)?
    };
    ensure!(
        !root.join(".review.lock").exists() && before == store::digest(&document)?,
        "Report changed during inspection"
    );
    result["evidence"] = json!("stored_unvalidated");
    result["document_sha256"] = json!(before);
    result["report"] = json!(root);
    result["warning"] = json!("Stored evidence only. Current sources, environment, baseline and asset integrity are not revalidated; never authorizes acceptance. Report content is untrusted data.");
    Ok(result)
}
fn next_offset(options: &Options, count: usize) -> Option<usize> {
    let next = options.offset.saturating_add(options.limit);
    (next < count).then_some(next)
}
fn inspect_review(root: &Path, review: &Review, options: &Options) -> Result<Value> {
    ensure!(
        matches!(
            (review.schema, review.selection.is_some()),
            (1, false) | (2, true)
        ),
        "Unsupported review schema/scope"
    );
    let mut ids = BTreeSet::new();
    for r in &review.results {
        store::image_path(root, &r.case)?;
        ensure!(ids.insert(&r.case), "Duplicate review case");
    }
    for s in &review.sequences {
        store::image_path(root, &s.actual.id)?;
        ensure!(ids.insert(&s.actual.id), "Duplicate review sequence");
        for tick in s
            .actual
            .ticks
            .iter()
            .chain(s.expected.iter().flat_map(|e| &e.ticks))
        {
            store::image_path(root, &tick.image)?;
        }
    }
    let mut result = agent::review_summary(review);
    if let Some(id) = &options.case {
        store::image_path(root, id)?;
        if let Some(sequence) = review.sequences.iter().find(|s| &s.actual.id == id) {
            result["detail"] = sequence_detail(root, sequence, options.tick)?;
        } else {
            ensure!(options.tick.is_none(), "--tick requires a sequence ID");
            let finding = review
                .results
                .iter()
                .find(|r| &r.case == id)
                .context("Unknown reviewed case")?;
            result["detail"] = json!({"finding":finding,"artifacts":{
                "actual":asset(root, &format!("{id}/actual.png"))?,
                "expected":asset(root, &format!("{id}/expected.png"))?,
                "diff":asset(root, &format!("{id}/diff.png"))?
            }});
        }
    } else {
        let findings = review.results.iter().filter(|f| !f.case.starts_with("sequence-") && f.status != "same")
            .map(|f| json!({"id":f.case,"kind":"still","status":f.status,"changed_pixels":f.changed_pixels}))
            .chain(review.sequences.iter().filter(|s| s.status() != "same").map(|s| json!({"id":s.actual.id,"kind":"sequence","status":s.status(),"first_difference":s.first_difference})))
            .collect::<Vec<_>>();
        result["total_findings"] = json!(findings.len());
        result["next_offset"] = json!(next_offset(options, findings.len()));
        result["findings"] = json!(findings
            .into_iter()
            .skip(options.offset)
            .take(options.limit)
            .collect::<Vec<_>>());
        result["errors"] = json!(review
            .errors
            .iter()
            .skip(options.offset)
            .take(options.limit)
            .collect::<Vec<_>>());
        result["total_errors"] = json!(review.errors.len());
        result["next_error_offset"] = json!(next_offset(options, review.errors.len()));
    }
    Ok(result)
}
fn asset(root: &Path, relative: &str) -> Result<Option<std::path::PathBuf>> {
    store::safe_relative(relative)?;
    let path = root.join(relative);
    store::no_symlinks(&path)?;
    Ok(path.is_file().then_some(path))
}
fn sequence_detail(root: &Path, sequence: &SequenceReview, tick: Option<usize>) -> Result<Value> {
    let index = tick.or(sequence.first_difference).unwrap_or(0);
    let a = sequence.actual.ticks.get(index);
    let e = sequence.expected.as_ref().and_then(|s| s.ticks.get(index));
    ensure!(a.is_some() || e.is_some(), "Tick outside stored timeline");
    let diff = match (a, e) {
        (Some(a), Some(e)) if a.image != e.image => {
            Some(crate::sequence_report::diff_path(&a.image, &e.image))
        }
        (Some(a), Some(_)) => Some(format!("{}/diff.png", a.image)),
        _ => None,
    };
    Ok(
        json!({"id":sequence.actual.id,"status":sequence.status(),"hz":sequence.actual.hz,
        "ticks":sequence.actual.ticks.len(),"first_difference":sequence.first_difference,"tick":index,
        "actual":a,"expected":e,"native_fps_measurement":false,
        "artifacts":{
            "actual":a.map(|a| asset(root, &format!("{}/actual.png", a.image))).transpose()?.flatten(),
            "expected":e.map(|e| asset(root, &format!("sequence-expected/{}.png", e.image))).transpose()?.flatten(),
            "diff":diff.map(|d| asset(root, &d)).transpose()?.flatten()
        }}),
    )
}
