//! Supervised fixed Tart transport. Not wired to normal capture entrypoints:
//! external VM approval, pilot/coverage and real return evidence are still required.
use crate::{payload, vm_cleanup, vm_environment, vm_runtime, vm_supervision, Suite};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{config::Config, gate, store};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReturnedFile {
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Returned {
    pub schema: u32,
    pub payload_sha256: String,
    pub raw: PathBuf,
    pub files: BTreeMap<String, ReturnedFile>,
}
#[derive(Debug)]
pub struct Captured {
    pub raw: PathBuf,
    pub environment: Value,
}

/// Only transport integrity/receipt binding, never coverage or a capture seal.
/// Called after vm_supervision's independent cleanup check and terminal result.
pub fn validate_return(
    raw: &[u8],
    workspace: &Path,
    suite: Suite,
    staged: &payload::Payload,
    digest: &str,
    host: &vm_environment::HostBuild,
    cancelled: &AtomicBool,
) -> Result<Captured> {
    ensure!(
        raw.len() <= 8 * 1024 * 1024,
        "transport evidence exceeds bound"
    );
    let returned: Returned = serde_json::from_slice(raw)?;
    ensure!(
        returned.schema == 1
            && returned.payload_sha256 == digest
            && returned.raw == workspace.join("raw")
            && !returned.files.is_empty()
            && returned.files.len() <= 20000
            && returned.files.contains_key("guest-receipt.json"),
        "return transport identity/inventory differs"
    );
    store::no_symlinks(&returned.raw)?;
    let expected: std::collections::BTreeSet<_> = returned
        .files
        .keys()
        .map(String::as_str)
        .chain(std::iter::once("return.json"))
        .collect();
    let mut found = std::collections::BTreeSet::new();
    let mut total = 0u64;
    for entry in fs::read_dir(&returned.raw)? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("non-UTF8 returned name"))?;
        ensure!(
            found.len() <= 20000 && expected.contains(name.as_str()),
            "unexpected returned file"
        );
        found.insert(name);
    }
    ensure!(
        found
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>()
            == expected,
        "incomplete returned files"
    );
    for (name, record) in &returned.files {
        ensure!(
            Path::new(name).file_name().and_then(|n| n.to_str()) == Some(name.as_str())
                && !name.starts_with('.')
                && !name.chars().any(char::is_control),
            "nonflat returned artifact"
        );
        let path = returned.raw.join(name);
        store::no_symlinks(&path)?;
        let meta = fs::symlink_metadata(&path)?;
        ensure!(
            meta.is_file()
                && meta.nlink() == 1
                && meta.len() == record.bytes
                && (1..=64 * 1024 * 1024).contains(&record.bytes),
            "invalid returned file size/type"
        );
        total = total
            .checked_add(record.bytes)
            .context("return size overflow")?;
        ensure!(
            total <= 4 * 1024 * 1024 * 1024,
            "return total exceeds bound"
        );
        ensure!(
            payload::hash(&path, cancelled)? == record.sha256,
            "returned bytes differ"
        );
    }
    let marker_path = returned.raw.join("return.json");
    let marker_meta = fs::symlink_metadata(&marker_path)?;
    ensure!(
        marker_meta.is_file() && marker_meta.nlink() == 1 && marker_meta.len() <= 8 * 1024 * 1024,
        "invalid return completion marker"
    );
    store::no_symlinks(&marker_path)?;
    let marker: Value = store::read_json(&marker_path)?;
    let evidence: Value = serde_json::from_slice(raw)?;
    ensure!(
        marker == serde_json::json!({"schema":1,"payload_sha256":digest,"files":evidence["files"]}),
        "return marker differs from transport evidence"
    );
    // Parse the same bounded bytes whose hash is bound by transport evidence,
    // not a second unchecked filename read after the inventory hash pass.
    use sha2::{Digest, Sha256};
    use std::{io::Read, os::unix::fs::OpenOptionsExt};
    let mut receipt_file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(returned.raw.join("guest-receipt.json"))?
        .take(64 * 1024 * 1024 + 1);
    let meta = receipt_file.get_ref().metadata()?;
    ensure!(
        meta.is_file() && meta.nlink() == 1,
        "invalid guest receipt file"
    );
    let mut receipt_bytes = Vec::new();
    receipt_file.read_to_end(&mut receipt_bytes)?;
    let receipt_record = &returned.files["guest-receipt.json"];
    ensure!(
        receipt_bytes.len() as u64 == receipt_record.bytes
            && format!("{:x}", Sha256::digest(&receipt_bytes)) == receipt_record.sha256,
        "guest receipt bytes changed before binding"
    );
    let receipt: vm_environment::Receipt = serde_json::from_slice(&receipt_bytes)?;
    let environment = vm_environment::bind(suite, host, staged, digest, &receipt)?;
    Ok(Captured {
        raw: returned.raw,
        environment,
    })
}

/// Internal route only. Does not imply network permission and has no CLI caller.
/// Caller supplies fresh host-built payload and gate options; fixed Python route
/// handles bootstrap/return while upstream owns the host process group and leases.
pub struct Input<'a> {
    pub repo: &'a Path,
    pub suite: Suite,
    pub share: &'a Path,
    pub staged: &'a payload::Payload,
    pub host: &'a vm_environment::HostBuild,
    pub workspace: &'a Path,
}

pub fn run(input: Input<'_>, options: &gate::Options, cancelled: &AtomicBool) -> Result<Captured> {
    let Input {
        repo,
        suite,
        share,
        staged,
        host,
        workspace,
    } = input;
    store::disjoint(&[repo, share, workspace])?;
    ensure!(
        workspace.is_absolute()
            && store::absolute(workspace)? == workspace
            && !workspace.exists()
            && workspace.parent().is_some_and(Path::is_dir),
        "fresh canonical transport workspace required"
    );
    payload::verify(share, staged, cancelled)?;
    let config = Config::load(&repo.join(suite.config()))?;
    ensure!(
        payload::source_snapshot(repo, &config, cancelled)? == staged.sources,
        "stale transport source"
    );
    vm_runtime::verify(repo, suite, share, staged, cancelled)?;
    host.validate()?;
    ensure!(
        host.executable_hashes
            == staged
                .files
                .iter()
                .filter(|entry| entry.executable)
                .map(|entry| (
                    entry.guest.to_string_lossy().into_owned(),
                    entry.sha256.clone()
                ))
                .collect(),
        "host build differs from transport payload"
    );
    let digest = store::digest(&share.join("payload.json"))?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let vm = format!(
        "bbr-visual-{}-{:012x}",
        std::process::id(),
        now & 0xffffffffffff
    );
    fs::create_dir(workspace)?;
    fs::set_permissions(workspace, fs::Permissions::from_mode(0o700))?;
    let artifacts = workspace.join("vm");
    fs::create_dir(&artifacts)?;
    fs::set_permissions(&artifacts, fs::Permissions::from_mode(0o700))?;
    let control = workspace.join("control");
    let output = vm_supervision::run(options,
        Command::new("python3").args(["-I", "-B", "-c", r#"
import json, os, sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
import vm_transport
try:
    result = vm_transport.run(Path(sys.argv[2]), sys.argv[3], Path(sys.argv[4]), sys.argv[5],
        Path(sys.argv[6]), sys.argv[7], Path(os.environ['BBR_VISUAL_CONTROL']), os.environ['BBR_VISUAL_REQUEST_SHA256'])
    print(json.dumps(result, sort_keys=True))
except BaseException as error:
    print(f'transport incomplete: {type(error).__name__}: {error}', file=sys.stderr)
    raise SystemExit(2)
"#]).arg(repo.join("scripts/visual")).arg(share).arg(&digest).arg(repo).arg(suite.id()).arg(workspace).arg(&vm),
        &control, &digest, &vm_supervision::Policy::default(), cancelled,
        |request| vm_cleanup::verify(repo, &vm_cleanup::Expected { artifacts: &artifacts, vm: &vm,
            request_sha256: request, payload_sha256: &digest }).map(|_| ()))?;
    payload::verify(share, staged, cancelled)?;
    ensure!(
        store::digest(&share.join("payload.json"))? == digest
            && payload::source_snapshot(repo, &config, cancelled)? == staged.sources,
        "source/payload changed during transport"
    );
    validate_return(
        &output.stdout,
        workspace,
        suite,
        staged,
        &digest,
        host,
        cancelled,
    )
}
