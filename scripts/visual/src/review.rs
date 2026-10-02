//! Explicit stored-run reviews. No capture, resealing, baseline adoption or process owner.
use crate::{gallery_selection::Schedule, provenance, Suite};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{
    agent::Response,
    collection::{self, Suite as CollectionSuite},
    compare as upstream_compare,
    config::Config,
    store,
};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug)]
pub struct Request {
    pub gallery: Option<PathBuf>,
    pub browser: Option<PathBuf>,
    pub report: PathBuf,
}
fn canonical(path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute() && store::absolute(path)?.as_os_str() == path.as_os_str(),
        "explicit canonical absolute path required"
    );
    store::no_symlinks(path)
}
impl Request {
    pub fn parse(args: &[String]) -> Result<Self> {
        let mut gallery = None;
        let mut browser = None;
        let mut report = None;
        let mut seen = BTreeSet::new();
        let mut args = args.iter();
        while let Some(flag) = args.next() {
            ensure!(seen.insert(flag), "duplicate option: {flag}");
            if flag == "--json" {
                continue;
            }
            let destination = match flag.as_str() {
                "--gallery-run" => &mut gallery,
                "--browser-run" => &mut browser,
                "--report" => &mut report,
                _ => anyhow::bail!("unknown compare option: {flag}"),
            };
            let value = args.next().context("compare option requires a path")?;
            ensure!(!value.starts_with("--"), "compare option requires a path");
            *destination = Some(PathBuf::from(value));
        }
        let request = Self {
            gallery,
            browser,
            report: report.context("--report required; no implicit latest")?,
        };
        request.validate()?;
        Ok(request)
    }
    fn validate(&self) -> Result<()> {
        ensure!(
            self.gallery.is_some() || self.browser.is_some(),
            "at least one explicit --gallery-run or --browser-run required"
        );
        let mut paths = vec![self.report.as_path()];
        paths.extend(
            self.gallery
                .iter()
                .chain(self.browser.iter())
                .map(PathBuf::as_path),
        );
        for path in &paths {
            canonical(path)?;
        }
        store::disjoint(&paths)?;
        ensure!(
            !self.report.exists(),
            "fresh explicit report directory required"
        );
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub scope: &'static str,
    pub passed: bool,
    pub report: PathBuf,
    pub producer_executed: bool,
    pub environment_probed: bool,
    pub baseline_writes: bool,
}
fn checkpoint(cancelled: &AtomicBool) -> Result<()> {
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "stored-run review cancelled"
    );
    Ok(())
}

/// Upstream remains the only source/image/comparison/report/acceptance authority.
/// Invalid full-suite captures remain visible beside valid siblings in Collections.
/// A valid selected Gallery capture has its own non-accepting development report.
/// Cancellation is checked at the operation boundaries; upstream comparison is
/// synchronous and has no cooperative cancellation API. Callers must retain an
/// outer hard deadline. A late cancellation can leave a completed report, never
/// baseline writes; it does not retroactively invalidate reviewed image bytes.
pub fn compare(repo: &Path, request: &Request, cancelled: &AtomicBool) -> Result<Outcome> {
    checkpoint(cancelled)?;
    canonical(repo)?;
    request.validate()?;
    ensure!(repo.is_dir(), "review repository missing");
    ensure!(
        !repo.starts_with(&request.report)
            && (!request.report.starts_with(repo)
                || (request.report.starts_with(repo.join("reports/visual"))
                    && request.report != repo.join("reports/visual"))),
        "report must be external or below repository reports/visual"
    );
    let mut suites = Vec::new();
    let mut selected = false;
    for (suite, run) in [
        (Suite::Gallery, &request.gallery),
        (Suite::Browser, &request.browser),
    ] {
        let Some(run) = run else { continue };
        let config_path = repo.join(suite.config());
        store::no_symlinks(&config_path)?;
        let config = Config::load(&config_path)?;
        let baselines = repo.join(&config.baselines);
        canonical(&baselines)?;
        // Missing/broken seals are upstream per-suite errors, not silently dropped.
        if let Ok(manifest) = store::load_manifest(run) {
            ensure!(
                manifest.source_policy.same_inputs(&config.source_policy)
                    && manifest.sources.get(suite.config())
                        == Some(&store::digest(&repo.join(suite.config()))?),
                "capture config/source policy differs; capture again"
            );
            ensure!(
                manifest.sequences.is_empty(),
                "sequence runs require a dedicated timeline report"
            );
            if let Some(selection) = &manifest.selection {
                ensure!(
                    suite == Suite::Gallery && request.browser.is_none(),
                    "selected Gallery run requires a separate single-suite report"
                );
                ensure!(
                    config.story_catalog.as_deref() == Some(Path::new(&selection.catalog)),
                    "selected capture catalog differs from current config"
                );
                Schedule::resolve(
                    repo,
                    &manifest.sources,
                    "scripts/visual/catalogs/gallery-plan.json",
                    selection,
                )?;
                selected = true;
            }
        }
        suites.push(CollectionSuite {
            id: suite.id().into(),
            label: match suite {
                Suite::Gallery => "Gallery",
                Suite::Browser => "Browser chrome",
            }
            .into(),
            repo: repo.into(),
            run: run.clone(),
            baselines,
            catalog: format!("scripts/visual/catalogs/{}.json", suite.id()),
        });
    }
    let mut paths = vec![request.report.as_path()];
    for suite in &suites {
        paths.extend([suite.run.as_path(), suite.baselines.as_path()]);
    }
    store::disjoint(&paths)?;
    checkpoint(cancelled)?;
    let passed = if selected {
        let suite = &suites[0];
        // The single-suite upstream API permits replacement. Validate before any
        // writes, then reserve this fresh directory atomically so an output that
        // appeared during preflight cannot be overwritten. Collections already
        // reserve their own fresh output in the unchanged upstream implementation.
        let manifest = store::load_manifest(&suite.run)?;
        store::check_sources(repo, &manifest, Some(&suite.baselines))?;
        store::validate_images(&suite.run, &manifest)?;
        checkpoint(cancelled)?;
        store::no_symlinks(&request.report)?;
        std::fs::create_dir_all(request.report.parent().context("report parent required")?)?;
        std::fs::create_dir(&request.report)?;
        upstream_compare::compare(repo, &suite.run, &suite.baselines, &request.report)?.passed
    } else {
        collection::compare(&suites, &request.report)?.passed
    };
    checkpoint(cancelled)?;
    Ok(Outcome {
        scope: if selected {
            "selected-development"
        } else {
            "collection"
        },
        passed,
        report: request.report.clone(),
        producer_executed: false,
        environment_probed: false,
        baseline_writes: false,
    })
}

pub fn dispatch(repo: &Path, args: &[String], cancelled: &AtomicBool) -> Result<Option<u8>> {
    if args.first().map(String::as_str) != Some("compare") {
        return Ok(None);
    }
    let result = (|| {
        let request = Request::parse(&args[1..])?;
        provenance::verify(repo)?;
        compare(repo, &request, cancelled)
    })();
    let response = match result {
        Ok(outcome) => Response::new(
            "compare",
            if outcome.passed { 0 } else { 1 },
            serde_json::to_value(outcome)?,
        ),
        Err(error) if args.iter().any(|arg| arg == "--json") => {
            Response::failure("compare", &error)
        }
        Err(error) => return Err(error),
    };
    if args.iter().any(|arg| arg == "--json") {
        println!("{}", serde_json::to_string(&response)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&response.data)?);
    }
    Ok(Some(response.exit_code))
}
