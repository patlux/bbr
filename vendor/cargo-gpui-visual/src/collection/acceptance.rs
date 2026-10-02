//! Exact collection selection with all-selected preflight, not a filesystem transaction.
#[cfg(test)]
#[path = "acceptance_tests.rs"]
mod tests;
use super::{catalog::State, resolve, revalidate, seal, Collection};
use crate::{
    accept::{self, BaselineLock},
    report::ReportLock,
    store::*,
};
use anyhow::{ensure, Context, Result};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::Path,
};

#[derive(Serialize)]
struct Attempt<'a> {
    schema: u32,
    selected: &'a [String],
    adopt_suites: &'a [String],
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    acknowledged_withheld: &'a [String],
    completed: Vec<String>,
    state: &'static str,
    error: Option<String>,
}
/// Accept only explicit exact `suite:case` IDs from this reviewed directory.
/// Environment changes require exact suite IDs; no global implicit adoption.
pub fn accept(root: &Path, selected: &[String], adopt_suites: &[String]) -> Result<()> {
    accept_acknowledging_withheld(root, selected, adopt_suites, &[])
}

/// Accept reviewed references while explicitly acknowledging every withheld
/// `suite:base` of the selected authorities. Coverage remains failing. No missing
/// required capture, comparison error or integrity check can be acknowledged.
pub fn accept_acknowledging_withheld(
    root: &Path,
    selected: &[String],
    adopt_suites: &[String],
    acknowledged_withheld: &[String],
) -> Result<()> {
    let root = absolute(root)?;
    no_symlinks(&root)?;
    let _report_lock = ReportLock::acquire(&root)?;
    ensure!(
        !root.join("acceptance.json").exists(),
        "Collection already has an acceptance attempt; review a new explicit collection"
    );
    seal::validate(&root)?;
    let collection: Collection = read_json(&root.join("collection.json"))?;
    ensure!(collection.schema == 1, "Unsupported collection schema");
    resolve(
        &collection
            .suites
            .iter()
            .map(|r| r.suite.clone())
            .collect::<Vec<_>>(),
        &root,
    )?;
    ensure!(
        !selected.is_empty() || !adopt_suites.is_empty(),
        "Explicit unique suite:case IDs required; no accept-all default"
    );
    ensure!(
        selected.iter().collect::<BTreeSet<_>>().len() == selected.len(),
        "Duplicate selection"
    );
    ensure!(
        adopt_suites.iter().collect::<BTreeSet<_>>().len() == adopt_suites.len(),
        "Duplicate environment selection"
    );
    let mut selection = BTreeMap::<&str, Vec<String>>::new();
    for id in selected {
        let (suite, case) = id.split_once(':').context("Exact suite:case required")?;
        image_path(Path::new("."), case)?;
        selection.entry(suite).or_default().push(case.to_owned());
    }
    for id in adopt_suites {
        selection.entry(id).or_default();
    }
    if !acknowledged_withheld.is_empty() {
        ensure!(
            acknowledged_withheld.iter().collect::<BTreeSet<_>>().len()
                == acknowledged_withheld.len(),
            "Duplicate withheld acknowledgement"
        );
        let mut expected = BTreeSet::new();
        for id in selection.keys() {
            let result = collection
                .suites
                .iter()
                .find(|s| s.suite.id == *id)
                .context("Unknown selected suite")?;
            let catalog = result
                .catalog
                .as_ref()
                .context("Missing reviewed catalog")?;
            catalog.validate()?;
            for snapshot in &catalog.snapshots {
                if snapshot.state == State::Withheld {
                    expected.insert(format!("{id}:{}", snapshot.base));
                }
            }
        }
        ensure!(
            acknowledged_withheld
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
                == expected,
            "Acknowledge exactly every withheld suite:base of the selected suites"
        );
    }
    let mut prepared = Vec::new();
    let mut suite_locks = Vec::new();
    for (id, cases) in &selection {
        let result = collection
            .suites
            .iter()
            .find(|s| s.suite.id == *id)
            .context("Unknown selected suite")?;
        let suite = &result.suite;
        suite_locks.push(ReportLock::acquire(&root.join("suites").join(id))?);
        if acknowledged_withheld.is_empty() {
            ensure!(
                result.errors.is_empty(),
                "Cannot accept suite with invalid coverage/comparison: {id}"
            );
        }
        let catalog = result
            .catalog
            .as_ref()
            .context("Missing reviewed catalog")?;
        catalog.validate()?;
        let m = load_manifest(&suite.run)?;
        ensure!(
            catalog.capture_coverage(&m.cases).is_empty(),
            "Invalid selected suite capture coverage"
        );
        ensure!(
            result.errors == catalog.coverage(&m.cases),
            "Cannot accept suite with invalid coverage/comparison: {id}"
        );
        for case in cases {
            ensure!(
                catalog.snapshots.iter().any(|s| s.state == State::Required
                    && (case == &format!("{}--light", s.base)
                        || case == &format!("{}--dark", s.base))),
                "Not a required reviewed case: {id}:{case}"
            );
        }
        revalidate(result)?;
        let review = result
            .review
            .clone()
            .context("Missing reviewed comparison")?;
        let plan = accept::prepare(
            &suite.repo,
            &suite.run,
            &suite.baselines,
            cases,
            adopt_suites.iter().any(|s| s == id),
            // Collections accept into existing suite stores in their declared
            // format; a missing suite store starts as PNG.
            crate::codec::Format::Png,
            m,
            review,
        )?;
        prepared.push((suite, plan));
    }
    // Serial cooperating authorities: validate all before any payload mutation.
    let mut locks = Vec::new();
    for (suite, _) in &prepared {
        locks.push(BaselineLock::acquire(&suite.baselines)?);
    }
    seal::validate(&root)?;
    for (_, plan) in &prepared {
        plan.revalidate()?;
    }
    execute(
        &root,
        selected,
        adopt_suites,
        acknowledged_withheld,
        &prepared,
        |_| Ok(()),
    )?;
    drop(locks);
    Ok(())
}

/// Private seam for deterministic interruption tests. Production always passes
/// a no-op; no user-configurable fault injection or weakened validation.
fn execute(
    root: &Path,
    selected: &[String],
    adopt_suites: &[String],
    acknowledged_withheld: &[String],
    prepared: &[(&super::Suite, accept::Prepared)],
    before_write: impl Fn(usize) -> Result<()>,
) -> Result<()> {
    let path = root.join("acceptance.json");
    no_symlinks(&path)?;
    let mut attempt = Attempt {
        schema: if acknowledged_withheld.is_empty() {
            1
        } else {
            2
        },
        selected,
        adopt_suites,
        acknowledged_withheld,
        completed: Vec::new(),
        state: "started",
        error: None,
    };
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    file.write_all(&serde_json::to_vec_pretty(&attempt)?)?;
    file.sync_all()?;
    for (index, (suite, plan)) in prepared.iter().enumerate() {
        if let Err(error) = before_write(index)
            .and_then(|()| plan.revalidate())
            .and_then(|()| plan.write())
        {
            attempt.state = "failed";
            attempt.error = Some(format!("{}: {error:#}", suite.id));
            accept::atomic_write(&path, &serde_json::to_vec_pretty(&attempt)?)?;
            return Err(error.context(
                "Acceptance may be partial; inspect acceptance.json and generate a new review",
            ));
        }
        attempt.completed.push(suite.id.clone());
        accept::atomic_write(&path, &serde_json::to_vec_pretty(&attempt)?)?;
    }
    attempt.state = "complete";
    accept::atomic_write(&path, &serde_json::to_vec_pretty(&attempt)?)?;
    Ok(())
}
