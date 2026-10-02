//! Functional semantic evidence only. No image suite, seal or baseline authority.
//! `run` reuses supervised transport; the pure validator alone attests no VM cleanup.
use crate::{
    gallery_selection::bound_bytes,
    payload::{Payload, Purpose},
    semantic,
    vm_environment::{self, HostBuild},
    vm_transport,
};
use anyhow::{ensure, Result};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

/// Internal only: callers still require explicit network approval. No CLI route.
pub struct Input<'a> {
    pub repo: &'a Path,
    pub share: &'a Path,
    pub staged: &'a Payload,
    pub host: &'a HostBuild,
    pub workspace: &'a Path,
}

pub fn run(
    input: Input<'_>,
    options: &cargo_gpui_visual::gate::Options,
    cancelled: &AtomicBool,
) -> Result<Verified> {
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "semantic transport cancelled"
    );
    semantic::Requirements::load(input.staged)?;
    let transfer = vm_transport::run_transfer(
        vm_transport::TransferInput {
            repo: input.repo,
            purpose: Purpose::GallerySemantic,
            share: input.share,
            staged: input.staged,
            host: input.host,
            workspace: input.workspace,
            selection: None,
        },
        options,
        cancelled,
    )?;
    validate_return(
        &transfer.evidence,
        input.workspace,
        input.staged,
        &transfer.digest,
        input.host,
        cancelled,
    )
}

#[derive(Debug)]
pub struct Verified {
    raw: PathBuf,
    workspace: PathBuf,
    evidence: Vec<u8>,
    // Retain host expectations too: a later caller cannot adopt another payload.
    payload_identity: Vec<u8>,
    digest: String,
    host: HostBuild,
}
impl Verified {
    pub fn raw(&self) -> &Path {
        &self.raw
    }

    pub fn recheck(
        &self,
        payload: &Payload,
        digest: &str,
        host: &HostBuild,
        cancelled: &AtomicBool,
    ) -> Result<()> {
        ensure!(
            serde_json::to_vec(payload)? == self.payload_identity
                && digest == self.digest
                && host == &self.host,
            "original semantic host identity changed"
        );
        let checked = validate_return(
            &self.evidence,
            &self.workspace,
            payload,
            digest,
            host,
            cancelled,
        )?;
        ensure!(checked.raw == self.raw, "semantic returned path changed");
        Ok(())
    }
}

/// Call only after successful fixed guest execution and independently verified
/// cleanup in the existing supervisor. Fixture success is not runtime evidence.
pub fn validate_return(
    evidence: &[u8],
    workspace: &Path,
    payload: &Payload,
    digest: &str,
    host: &HostBuild,
    cancelled: &AtomicBool,
) -> Result<Verified> {
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "semantic return cancelled"
    );
    semantic::Requirements::load(payload)?;
    let (returned, receipt) =
        vm_transport::verify_returned_bytes(evidence, workspace, digest, cancelled)?;
    vm_environment::validate_receipt(Purpose::GallerySemantic, host, payload, digest, &receipt)?;
    let expected: std::collections::BTreeSet<_> = [semantic::RESULT, "guest-receipt.json"]
        .into_iter()
        .chain(semantic::DIAGNOSTICS.map(|(_, name)| name))
        .collect();
    ensure!(
        returned
            .files
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>()
            == expected,
        "semantic return inventory differs"
    );
    let read = |name: &str, limit: u64| -> Result<Vec<u8>> {
        ensure!(
            !cancelled.load(Ordering::SeqCst),
            "semantic return cancelled"
        );
        let record = &returned.files[name];
        ensure!(record.bytes <= limit, "semantic result size exceeds bound");
        let bytes = bound_bytes(&returned.raw, name, &record.sha256)?;
        ensure!(
            bytes.len() as u64 == record.bytes,
            "semantic result size differs"
        );
        Ok(bytes)
    };
    let record: semantic::Record =
        serde_json::from_slice(&read(semantic::RESULT, 4 * 1024 * 1024)?)?;
    record.validate(payload, digest)?;
    for (suite, name) in semantic::DIAGNOSTICS {
        semantic::validate_diagnostic(suite, &read(name, semantic::OUTPUT_LIMIT)?)?;
    }
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "semantic return cancelled"
    );
    Ok(Verified {
        raw: returned.raw,
        workspace: workspace.into(),
        evidence: evidence.into(),
        payload_identity: serde_json::to_vec(payload)?,
        digest: digest.into(),
        host: host.clone(),
    })
}
