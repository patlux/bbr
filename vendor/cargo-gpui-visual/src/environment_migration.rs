//! Explicit metadata-only classification of an existing approved environment.
//! No renderer knowledge, capture, source refresh, image write or adoption.
use crate::{baseline::Baseline, environment, store::*};
use anyhow::{ensure, Result};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Debug, Serialize)]
pub struct Migration {
    pub old_hash: String,
    pub new_hash: String,
    pub cases: usize,
    pub written: bool,
    pub environment_before: Value,
    pub environment_after: Value,
}

/// `target` must reconstruct the exact old environment. Only classification and
/// the explicitly reviewed rendering contract are added; old facts never change.
/// Hash/state checks and the ordinary store lock bracket the atomic metadata write.
pub fn migrate(
    root: &Path,
    expected_hash: &str,
    target: &Value,
    dry_run: bool,
) -> Result<Migration> {
    no_symlinks(root)?;
    ensure!(root.is_dir(), "Existing baseline store required");
    ensure!(
        !root.join(".accept.lock").try_exists()?,
        "Baseline acceptance is locked; inspect owner"
    );
    for entry in fs::read_dir(root)? {
        ensure!(
            entry?.path().extension().is_none_or(|ext| ext != "pending"),
            "Unfinished baseline write; inspect retained evidence"
        );
    }
    let before = baseline_state(root)?;
    ensure!(
        before
            .get("baseline.json")
            .is_some_and(|h| h == expected_hash),
        "Baseline metadata changed; expected hash does not match"
    );
    let baseline = Baseline::load(root)?;
    ensure!(
        baseline.environment.get("comparison_schema").is_none(),
        "Environment already partitioned"
    );
    ensure!(
        environment::observation(target)? == baseline.environment,
        "Partition must preserve the complete original environment"
    );
    ensure!(
        baseline.image_errors(root, &before)?.is_empty(),
        "Invalid baseline images/inventory"
    );
    let cases: Vec<_> = baseline.cases.iter().collect();
    crate::parallel::map(&cases, |(id, evidence)| {
        let pixels = baseline
            .format()
            .read(&baseline.reference_path(root, id)?)?;
        if let Some(hash) = &evidence.pixel_hash {
            ensure!(
                crate::codec::pixel_hash(&pixels) == *hash,
                "Reference pixel fingerprint differs: {id}"
            );
        }
        Ok(())
    })?;
    let path = root.join("baseline.json");
    // Preserve the original stored schema and all historical records verbatim as
    // JSON values, including legacy inline sources. This is not compaction.
    let mut stored: Value = read_json(&path)?;
    stored["environment"] = target.clone();
    let mut bytes = serde_json::to_vec_pretty(&stored)?;
    bytes.push(b'\n');
    let migration = Migration {
        old_hash: expected_hash.into(),
        new_hash: format!("{:x}", Sha256::digest(&bytes)),
        cases: baseline.cases.len(),
        written: !dry_run,
        environment_before: baseline.environment,
        environment_after: target.clone(),
    };
    ensure!(
        before == baseline_state(root)?,
        "Baselines changed during migration preflight"
    );
    if !dry_run {
        let _lock = crate::accept::BaselineLock::acquire(root)?;
        ensure!(
            before == baseline_state(root)?,
            "Baselines changed before migration"
        );
        crate::accept::atomic_write(&path, &bytes)?;
    }
    Ok(migration)
}
