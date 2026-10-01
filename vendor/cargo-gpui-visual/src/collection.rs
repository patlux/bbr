//! Explicit immutable review collections; suite capture/baseline authorities stay separate.
mod acceptance;
pub mod catalog;
mod seal;
use crate::{
    compare::{compare_assets, Review},
    report,
    store::*,
};
pub use acceptance::accept;
use anyhow::{ensure, Context, Result};
use catalog::Catalog;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    pub id: String,
    pub label: String,
    pub repo: PathBuf,
    pub run: PathBuf,
    pub baselines: PathBuf,
    /// Repository-relative canonical source path, included in the capture seal.
    pub catalog: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub schema: u32,
    pub suites: Vec<Suite>,
}
impl Input {
    /// Relative repository paths resolve from the input file; run/baselines from
    /// that repository. No inferred config, latest run, environment or producer.
    pub fn load(path: &Path) -> Result<Self> {
        no_symlinks(path)?;
        let mut input: Self = read_json(path)?;
        ensure!(input.schema == 1, "Unsupported collection input schema");
        let parent = absolute(path)?
            .parent()
            .context("Collection input parent")?
            .to_owned();
        for s in &mut input.suites {
            s.repo = parent.join(&s.repo);
            s.run = s.repo.join(&s.run);
            s.baselines = s.repo.join(&s.baselines);
        }
        Ok(input)
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteReview {
    pub suite: Suite,
    pub catalog: Option<Catalog>,
    pub catalog_hash: Option<String>,
    pub review: Option<Review>,
    pub errors: Vec<String>,
    pub passed: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    pub schema: u32,
    pub suites: Vec<SuiteReview>,
    pub passed: bool,
}

fn resolve(suites: &[Suite], root: &Path) -> Result<Vec<Suite>> {
    ensure!(!suites.is_empty(), "At least one explicit suite required");
    let mut ids = BTreeSet::new();
    let mut resolved = Vec::new();
    for s in suites {
        ensure!(
            !s.id.is_empty()
                && s.id.as_bytes()[0].is_ascii_alphanumeric()
                && s.id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
            "Invalid suite ID"
        );
        ensure!(ids.insert(&s.id), "Duplicate suite ID: {}", s.id);
        ensure!(!s.label.trim().is_empty(), "Suite label required");
        safe_relative(&s.catalog)?;
        ensure!(
            Path::new(&s.catalog)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
                && !s.catalog.contains('\\')
                && !s.catalog.contains("//"),
            "Canonical relative catalog path required"
        );
        let repo = absolute(&s.repo)?;
        no_symlinks(&repo)?;
        ensure!(
            repo.is_dir(),
            "Suite repository missing: {}",
            repo.display()
        );
        resolved.push(Suite {
            repo: repo.clone(),
            run: absolute(&repo.join(&s.run))?,
            baselines: absolute(&repo.join(&s.baselines))?,
            ..s.clone()
        });
    }
    let mut paths = vec![root];
    for s in &resolved {
        paths.extend([s.run.as_path(), s.baselines.as_path()]);
    }
    disjoint(&paths)?;
    for s in &resolved {
        // Output may live inside a repository but must never contain a repository
        // or overwrite the catalog/source authority.
        ensure!(
            !s.repo.starts_with(absolute(root)?),
            "Collection output contains a repository"
        );
        let catalog = s.repo.join(&s.catalog);
        no_symlinks(&catalog)?;
        for path in &paths {
            ensure!(
                !catalog.starts_with(absolute(path)?),
                "Catalog overlaps capture/baseline/output"
            );
        }
    }
    Ok(resolved)
}

pub fn compare(suites: &[Suite], root: &Path) -> Result<Collection> {
    let root = absolute(root)?;
    let suites = resolve(suites, &root)?;
    // Never replace another collection or pick an implicit latest. Failed output
    // remains for diagnosis without a published collection acceptance document.
    ensure!(
        !root.exists(),
        "Collection output already exists; choose a new explicit directory"
    );
    fs::create_dir_all(root.parent().context("Collection output parent")?)?;
    fs::create_dir(&root)?;
    let _lock = report::ReportLock::acquire(&root)?;
    let mut results = Vec::new();
    for suite in suites {
        let mut result = SuiteReview {
            suite,
            catalog: None,
            catalog_hash: None,
            review: None,
            errors: Vec::new(),
            passed: false,
        };
        if let Err(error) = compare_suite(&mut result, &root) {
            result.errors.push(format!("{error:#}"));
        }
        result.passed =
            result.errors.is_empty() && result.review.as_ref().is_some_and(|r| r.passed);
        results.push(result);
    }
    // Revalidate every successful suite after the entire batch, not merely when
    // that suite happened to finish. Never bless an earlier stale sibling.
    for result in &mut results {
        if result.review.is_some() {
            if let Err(error) = revalidate(result) {
                result.errors.push(format!("{error:#}"));
                result.passed = false;
            }
        }
    }
    let collection = Collection {
        schema: 1,
        passed: results.iter().all(|s| s.passed),
        suites: results,
    };
    report::write_collection(&root, &collection)?;
    write_json(&root.join("collection.json"), &collection)?;
    seal::publish(&root)?;
    Ok(collection)
}
fn compare_suite(result: &mut SuiteReview, root: &Path) -> Result<()> {
    let s = &result.suite;
    let m = load_manifest(&s.run)?;
    ensure!(
        m.selection.is_none(),
        "Selected-story development run is not full-suite collection evidence; review it alone"
    );
    // Timelines are only presented (and therefore reviewable) in the single-suite
    // report. Never let a collection hide or accept an unseen journey.
    ensure!(
        m.sequences.is_empty(),
        "Collections do not present sequence timelines yet; review this suite alone"
    );
    let catalog_path = s.repo.join(&s.catalog);
    no_symlinks(&catalog_path)?;
    let hash = digest(&catalog_path)?;
    ensure!(
        m.sources.get(&s.catalog) == Some(&hash),
        "Catalog must be a sealed source input"
    );
    let catalog: Catalog = read_json(&catalog_path)?;
    catalog.validate()?;
    result.errors.extend(catalog.coverage(&m.cases));
    result.catalog = Some(catalog);
    result.catalog_hash = Some(hash);
    result.review = Some(compare_assets(
        &s.repo,
        &s.run,
        &s.baselines,
        &root.join("suites").join(&s.id),
    )?);
    Ok(())
}
fn revalidate(result: &SuiteReview) -> Result<()> {
    let s = &result.suite;
    let review = result.review.as_ref().context("Missing suite review")?;
    let m = load_manifest(&s.run)?;
    ensure!(
        digest(&s.run.join("capture.json"))? == review.capture_hash,
        "Capture changed during collection"
    );
    check_sources(&s.repo, &m, Some(&s.baselines))?;
    validate_images(&s.run, &m)?;
    ensure!(
        baseline_state(&s.baselines)? == review.baseline_state,
        "Baselines changed during collection"
    );
    ensure!(
        Some(digest(&s.repo.join(&s.catalog))?) == result.catalog_hash,
        "Catalog changed during collection"
    );
    Ok(())
}
