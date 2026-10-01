//! Concrete host-side read-only cleanup verifier for the Tart supervisor callback.
//! Requires independent run identity, then repeats absence observations; never
//! deletes resources, reads a key or sends a signal to a recorded process.
use anyhow::{ensure, Result};
use cargo_gpui_visual::{process, store};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command, sync::atomic::AtomicBool, time::Duration};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub schema: u32,
    pub artifacts: std::path::PathBuf,
    pub vm: String,
    pub request_sha256: String,
    pub payload_sha256: String,
    pub ip: String,
    pub tart_pid: u32,
    pub tart_exit: i32,
    pub clone_absent: bool,
    pub disk_detached: bool,
    pub private_key_absent: bool,
    pub owned_process_reaped: bool,
    pub ssh_unreachable: bool,
}

pub struct Expected<'a> {
    pub artifacts: &'a Path,
    pub vm: &'a str,
    pub request_sha256: &'a str,
    pub payload_sha256: &'a str,
}
impl Expected<'_> {
    fn validate(&self) -> Result<()> {
        store::no_symlinks(self.artifacts)?;
        ensure!(
            self.artifacts.is_absolute() && store::absolute(self.artifacts)? == self.artifacts,
            "canonical cleanup diagnostics required"
        );
        let parts: Vec<_> = self.vm.split('-').collect();
        ensure!(
            parts.len() == 4
                && parts[..2] == ["bbr", "visual"]
                && !parts[2].is_empty()
                && parts[2].bytes().all(|b| b.is_ascii_digit())
                && parts[3].len() == 12
                && lower_hex(parts[3]),
            "invalid visual clone identity"
        );
        for digest in [self.request_sha256, self.payload_sha256] {
            ensure!(
                digest.len() == 64 && lower_hex(digest),
                "invalid cleanup binding digest"
            );
        }
        Ok(())
    }
}
fn lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn validate(raw: &[u8], expected: &Expected<'_>) -> Result<Evidence> {
    expected.validate()?;
    ensure!(raw.len() <= 16384, "cleanup evidence exceeds bound");
    let value: Evidence = serde_json::from_slice(raw)?;
    ensure!(
        value.schema == 1
            && value.artifacts == expected.artifacts
            && value.vm == expected.vm
            && value.request_sha256 == expected.request_sha256
            && value.payload_sha256 == expected.payload_sha256,
        "cleanup evidence identity differs"
    );
    let ip: std::net::Ipv4Addr = value.ip.parse()?;
    ensure!(
        ip.is_private() && ip.to_string() == value.ip,
        "private canonical guest IP required"
    );
    ensure!(
        value.tart_pid > 1
            && value.tart_pid <= i32::MAX as u32
            && (-255..=255).contains(&value.tart_exit)
            && value.clone_absent
            && value.disk_detached
            && value.private_key_absent
            && value.owned_process_reaped
            && value.ssh_unreachable,
        "incomplete cleanup evidence"
    );
    Ok(value)
}

/// Call after transport exit, while vm_supervision still holds both leases.
/// A fresh bounded process ignores user cancellation only to finish observation;
/// the supervisor subsequently rejects cancelled capture even if cleanup passes.
pub fn verify(repo: &Path, expected: &Expected<'_>) -> Result<Evidence> {
    expected.validate()?;
    store::disjoint(&[repo, expected.artifacts])?;
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
import vm_evidence
print(json.dumps(vm_evidence.verify(vm_evidence.Binding(Path(sys.argv[2]), *sys.argv[3:]))))
"#,
            ])
            .arg(repo.join("scripts/visual"))
            .arg(expected.artifacts)
            .arg(expected.vm)
            .arg(expected.request_sha256)
            .arg(expected.payload_sha256),
        &process::Limits {
            timeout: Duration::from_secs(45),
            max_output_bytes: 16384,
            ..Default::default()
        },
        &AtomicBool::new(false),
        true,
    )?;
    ensure!(
        result.status.success(),
        "independent host cleanup observation failed"
    );
    validate(&result.stdout, expected)
}
