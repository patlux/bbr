//! Host-only runtime preflight. Python owns Nix/Mach-O observation, while the
//! Rust caller binds it to the original staged sources/artifacts and cancellation.
//! This is not a rendering environment, VM launcher or capture seal.
use crate::{payload, Suite};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{process, store};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap, path::Path, process::Command, sync::atomic::AtomicBool, time::Duration,
};

const LOCK: &str = "scripts/visual/guest-runtime.json";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Python {
    pub path: String,
    pub version: String,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub schema: u32,
    pub origin: String,
    pub payload_sha256: String,
    pub runtime_lock_sha256: String,
    pub python: Python,
    pub dependencies: BTreeMap<String, Vec<String>>,
    pub closure_paths: usize,
    pub closure_nar_bytes: u64,
}

pub fn validate(raw: &[u8], share: &Path, staged: &payload::Payload) -> Result<Evidence> {
    ensure!(raw.len() <= 65536, "runtime evidence exceeds bound");
    let evidence: Evidence = serde_json::from_slice(raw)?;
    let expected = staged
        .sources
        .get(LOCK)
        .context("runtime lock missing from staged sources")?;
    let lock_path = share.join("source").join(LOCK);
    ensure!(
        store::digest(&lock_path)? == *expected,
        "staged runtime lock changed"
    );
    let lock: serde_json::Value = store::read_json(&lock_path)?;
    let paths = lock["paths"]
        .as_object()
        .context("runtime closure missing")?;
    let bytes = paths.values().try_fold(0u64, |sum, entry| {
        sum.checked_add(
            entry["narSize"]
                .as_u64()
                .context("runtime NAR size missing")?,
        )
        .context("runtime NAR size overflow")
    })?;
    ensure!(
        evidence.schema == 1
            && evidence.origin == "host-runtime-prerequisite"
            && evidence.payload_sha256 == store::digest(&share.join("payload.json"))?
            && evidence.runtime_lock_sha256 == *expected
            && serde_json::to_value(&evidence.python)? == lock["python"]
            && evidence.closure_paths == paths.len()
            && (1..=256).contains(&paths.len())
            && evidence.closure_nar_bytes == bytes
            && (1..=2 * 1024 * 1024 * 1024).contains(&bytes),
        "runtime evidence identity/closure differs"
    );
    let executables: std::collections::BTreeSet<_> = staged
        .files
        .iter()
        .filter(|entry| entry.executable)
        .map(|entry| entry.guest.to_string_lossy().into_owned())
        .collect();
    ensure!(
        evidence
            .dependencies
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            == executables,
        "runtime evidence omits/adds staged executable"
    );
    for dependencies in evidence.dependencies.values() {
        let mut seen = std::collections::BTreeSet::new();
        for dependency in dependencies {
            let parts: Vec<_> = dependency.split('/').collect();
            ensure!(
                parts.len() > 4
                    && parts[..3] == ["", "nix", "store"]
                    && parts[1..]
                        .iter()
                        .all(|part| !part.is_empty() && ![".", ".."].contains(part))
                    && !dependency
                        .chars()
                        .any(|c| c.is_whitespace() || c.is_control())
                    && paths.contains_key(&parts[..4].join("/"))
                    && seen.insert(dependency),
                "unlocked/ambiguous runtime dependency"
            );
        }
    }
    Ok(evidence)
}

pub fn verify(
    repo: &Path,
    suite: Suite,
    share: &Path,
    staged: &payload::Payload,
    cancelled: &AtomicBool,
) -> Result<Evidence> {
    payload::verify(share, staged, cancelled)?;
    ensure!(
        staged.repo == repo && staged.suite == suite.id(),
        "runtime payload origin differs"
    );
    ensure!(
        staged.sources.contains_key(LOCK),
        "runtime lock missing from staged sources"
    );
    let digest = store::digest(&share.join("payload.json"))?;
    let result = process::run(
        Command::new("python3")
            .args([
                "-I",
                "-B",
                "-c",
                r#"
import json, sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
import vm_runtime
print(json.dumps(vm_runtime.verify(Path(sys.argv[2]), sys.argv[3], Path(sys.argv[4]), sys.argv[5])))
"#,
            ])
            .arg(repo.join("scripts/visual"))
            .arg(share)
            .arg(&digest)
            .arg(repo)
            .arg(suite.id()),
        &process::Limits {
            timeout: Duration::from_secs(240),
            max_output_bytes: 65536,
            ..Default::default()
        },
        cancelled,
        true,
    )?;
    ensure!(
        result.status.success(),
        "offline runtime prerequisite failed"
    );
    payload::verify(share, staged, cancelled)?;
    ensure!(
        store::digest(&share.join("payload.json"))? == digest,
        "runtime payload identity changed"
    );
    validate(&result.stdout, share, staged)
}
