//! Read-only projections of the immutable tool APIs. Never native authorization.
use crate::{provenance, Suite};
use anyhow::{ensure, Result};
use cargo_gpui_visual::{
    capture::Paths,
    collection::catalog::{Catalog, State},
    config::Config,
    plan, selection, store,
};
use serde_json::{json, Value};
use std::path::Path;

fn sources(repo: &Path, config: &Config) -> Result<store::Hashes> {
    store::source_hashes(
        repo,
        &config.source_policy,
        Some(&repo.join(&config.baselines)),
    )
}

pub fn doctor(repo: &Path) -> Result<Value> {
    let export = provenance::verify(repo)?;
    let mut suites = Vec::new();
    for suite in [Suite::Gallery, Suite::Browser] {
        let config = Config::load(&repo.join(suite.config()))?;
        ensure!(!config.capture.is_empty(), "capture command is empty");
        let sources = sources(repo, &config)?;
        ensure!(!sources.is_empty(), "no capture sources");
        let relative = format!("scripts/visual/catalogs/{}.json", suite.id());
        let path = repo.join(&relative);
        store::no_symlinks(&path)?;
        let catalog: Catalog = store::read_json(&path)?;
        catalog.validate()?;
        ensure!(
            sources.get(&relative) == Some(&store::digest(&path)?),
            "Coverage catalog must be a sealed source input"
        );
        let pending: Vec<_> = catalog
            .snapshots
            .iter()
            .filter(|s| s.state != State::Required)
            .map(|s| json!({"base":s.base,"state":s.state,"reason":s.reason}))
            .collect();
        suites.push(json!({
            "id":suite.id(), "source_inputs":sources.len(),
            "capture_argv":config.capture,
            "story_catalog_configured":config.story_catalog.is_some(),
            "producer_selection_verified":false,
            "coverage": {
                "catalog":path,
                "required_images":catalog.snapshots.iter().filter(|s| s.state == State::Required).count()*2,
                "pending":pending,
                "current_registry_verified":false,
                "framing_verified":false,
                "warning":"Authored requirements only, not current capture or completeness evidence"
            }
        }));
    }
    Ok(json!({
        "consumer":"bbr-visual", "tool_revision":export.source_revision,
        "suites":suites, "producer_executed":false,"environment_probed":false,
        "execution":"tart_vm", "baseline_writes":false
    }))
}

pub fn list(repo: &Path, suite: Suite) -> Result<Value> {
    provenance::verify(repo)?;
    let config = Config::load(&repo.join(suite.config()))?;
    let (path, catalog) = selection::Catalog::load(repo, &config)?;
    ensure!(
        sources(repo, &config)?.get(&path) == Some(&store::digest(&repo.join(&path))?),
        "Story catalog must be a sealed source input"
    );
    Ok(json!({"suite":suite.id(), "catalog":catalog,
        "producer_executed":false, "producer_selection_verified":false,
        "coverage":"consumer_owned_not_verified"}))
}

pub fn preview(
    repo: &Path,
    suite: Suite,
    run: &Path,
    report: &Path,
    stories: &[String],
) -> Result<Value> {
    provenance::verify(repo)?;
    let config = Config::load(&repo.join(suite.config()))?;
    let config = if stories.is_empty() {
        config
    } else {
        config.select(repo, stories)?
    };
    // Explicit paths make the preview reproducible, not an implicit latest or
    // persisted execution plan. Do not acquire/probe a gate or environment.
    let preview = plan::check(
        Paths {
            repo,
            run,
            baselines: &repo.join(&config.baselines),
            report,
        },
        &config,
        false,
        &[],
    )?;
    let mut value = serde_json::to_value(preview)?;
    value["suite"] = json!(suite.id());
    value["execution"] = json!("tart_vm");
    value["warning"] = json!("Configured legacy commands are diagnostic only; native host capture is forbidden. Live capture runs through `check <suite>` in a disposable Tart guest. This preview grants no execution or acceptance permission.");
    Ok(value)
}
