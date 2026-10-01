//! Read-only capture planning. A preview is not execution permission or a cached plan.
use crate::{capture::Paths, config::Config, diagnostic::Code, store};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub dry_run: bool,
    pub producer_executed: bool,
    pub capture_argv: Vec<String>,
    pub environment_argv: Option<Vec<String>>,
    pub paths: Value,
    pub source_inputs: usize,
    pub selection: Option<crate::selection::Selection>,
    pub requirements: Value,
    pub host_gate: Value,
    pub unverified: Vec<String>,
    pub effects_on_execution: Vec<String>,
}
pub fn check(
    paths: Paths<'_>,
    config: &Config,
    host_gate: bool,
    compatibility: &[std::path::PathBuf],
) -> Result<Plan> {
    config.validate_schema()?;
    let repo = paths.repo.canonicalize()?;
    let run = store::absolute(&repo.join(paths.run))?;
    let baselines = store::absolute(&repo.join(paths.baselines))?;
    let report = store::absolute(&repo.join(paths.report))?;
    store::disjoint(&[&run, &baselines, &report])?;
    ensure!(
        !run.exists(),
        "Run already exists; never overwrite a capture"
    );
    ensure!(!config.capture.is_empty(), "Capture command is empty");
    ensure!(
        config
            .environment_command
            .as_ref()
            .is_none_or(|a| !a.is_empty()),
        "Environment command is empty"
    );
    ensure!(
        host_gate || compatibility.is_empty(),
        "Compatibility gates require host gate"
    );
    let mut gates = std::collections::BTreeSet::new();
    for gate in compatibility {
        ensure!(
            gate.is_absolute(),
            "Compatibility gate path must be absolute"
        );
        let path = store::absolute(gate)?;
        store::no_symlinks(&path)?;
        ensure!(gates.insert(path), "Duplicate compatibility gate");
    }
    #[cfg(unix)]
    if host_gate {
        ensure!(
            !gates.contains(&crate::gate::Options::host()?.path),
            "Duplicate host/compatibility gate path"
        );
    }
    let sources = store::source_hashes(&repo, &config.source_policy, Some(&baselines))?;
    ensure!(!sources.is_empty(), "No capture sources");
    if let Some(selection) = &config.selection {
        if config.environment_command.is_none() {
            return Err(Code::SelectionUnsupported.error("Story selection requires a raw producer; legacy self-sealed captures are unsupported").into());
        }
        selection.check_sources(&repo, &sources)?;
    }
    let requirements = config.selection.as_ref().map(|s| json!({
        "still_images":s.stories.iter().map(|s| s.cases.len()*2).sum::<usize>(),
        "sequences":s.stories.iter().map(|s| s.sequences.len()*2).sum::<usize>(),
        "timeline_ticks":s.stories.iter().flat_map(|s| &s.sequences).map(|s| s.ticks*2).sum::<usize>()
    })).unwrap_or_else(|| json!({"scope":"full","coverage":"consumer_owned_not_verified"}));
    Ok(Plan {
        dry_run: true,
        producer_executed: false,
        capture_argv: config
            .capture
            .iter()
            .map(|a| a.replace("{run}", &run.to_string_lossy()))
            .collect(),
        environment_argv: config.environment_command.clone(),
        paths: json!({"repo":repo,"run":run,"baselines":baselines,"report":report}),
        source_inputs: sources.len(),
        selection: config.selection.clone(),
        requirements,
        host_gate: json!({"requested":host_gate,"compatibility":compatibility,"acquired":false,"availability_checked":false}),
        unverified: vec![
            "producer implementation and executable readiness".into(),
            "environment and native/device authorization".into(),
            "host/compatibility gate availability".into(),
            "independent full-suite coverage".into(),
        ],
        effects_on_execution: vec![
            "invoke configured environment probes and producer".into(),
            "write a fresh capture and original report".into(),
            "never write baselines or accept changes".into(),
        ],
    })
}
