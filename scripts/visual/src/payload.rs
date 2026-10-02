//! Source-allowlisted Tart transfer payload, not a capture seal or VM launcher.
//! Guest destinations preserve compile-time paths; host mbx target links are
//! resolved only for the exact Cargo-reported executables, never whole targets.
use crate::process;
#[path = "payload_purpose.rs"]
mod purpose;
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{
    config::Config,
    inventory,
    store::{self, Hashes},
};
pub use purpose::Purpose;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

const FILE_LIMIT: u64 = 1024 * 1024 * 1024;
const TOTAL_LIMIT: u64 = 4 * FILE_LIMIT;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub payload: String,
    pub guest: PathBuf,
    pub sha256: String,
    pub bytes: u64,
    pub executable: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    pub schema: u32,
    pub suite: String,
    pub repo: PathBuf,
    pub sources: Hashes,
    pub files: Vec<Entry>,
}

fn checkpoint(cancelled: &AtomicBool) -> Result<()> {
    ensure!(!cancelled.load(Ordering::SeqCst), "payload cancelled");
    Ok(())
}
fn relative(name: &str) -> Result<()> {
    store::safe_relative(name)?;
    ensure!(
        Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_))),
        "canonical relative path required"
    );
    ensure!(
        !name.chars().any(char::is_control),
        "control character in payload path"
    );
    ensure!(
        !name.split('/').any(|part| {
            part.starts_with('.')
                || matches!(
                    part,
                    "secrets" | "credentials" | "reports" | "baselines" | "gallery-baselines"
                )
                || part.ends_with(".pem")
                || part.ends_with(".key")
                || part.ends_with("@git")
        }),
        "non-source or sensitive payload path refused"
    );
    Ok(())
}
fn regular(path: &Path) -> Result<fs::Metadata> {
    let meta = fs::symlink_metadata(path)?;
    ensure!(
        meta.is_file() && meta.len() <= FILE_LIMIT,
        "payload requires a bounded regular file: {}",
        path.display()
    );
    Ok(meta)
}
pub(crate) fn hash(path: &Path, cancelled: &AtomicBool) -> Result<String> {
    regular(path)?;
    let mut input = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut total = 0;
    loop {
        checkpoint(cancelled)?;
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        ensure!(total <= FILE_LIMIT, "payload file grew beyond limit");
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

/// Take before invoking mbx and compare after it. Never infer source identity
/// from an executable timestamp or copy a mutable whole worktree into Tart.
pub fn source_snapshot(repo: &Path, config: &Config, cancelled: &AtomicBool) -> Result<Hashes> {
    checkpoint(cancelled)?;
    for scope in config
        .source_policy
        .scopes
        .iter()
        .chain(&config.source_policy.artifacts)
    {
        relative(scope)?;
    }
    ensure!(
        !config
            .source_policy
            .suffixes
            .iter()
            .any(|s| matches!(s.as_str(), ".pem" | ".key")),
        "sensitive source suffix refused"
    );
    let sources = store::source_hashes_with(
        repo,
        &config.source_policy,
        Some(&repo.join(&config.baselines)),
        &inventory::git_limits(),
        cancelled,
    )?;
    ensure!(!sources.is_empty(), "empty payload source inventory");
    for name in sources.keys() {
        relative(name)?;
    }
    Ok(sources)
}

pub fn stage(
    repo: &Path,
    config: &Config,
    before_build: &Hashes,
    artifacts: &[Value],
    purpose: impl Into<Purpose>,
    destination: &Path,
    cancelled: &AtomicBool,
) -> Result<Payload> {
    checkpoint(cancelled)?;
    ensure!(
        repo.is_absolute() && repo.canonicalize()? == repo,
        "canonical repository required"
    );
    store::disjoint(&[repo, destination])?;
    ensure!(
        destination.is_absolute() && !destination.exists(),
        "fresh absolute external payload directory required"
    );
    ensure!(
        &source_snapshot(repo, config, cancelled)? == before_build,
        "source changed during build; rebuild before transfer"
    );
    let purpose = purpose.into();
    let required = purpose.required();
    let mut files = Vec::new();
    for (name, digest) in before_build {
        relative(name)?;
        let path = repo.join(name);
        store::no_symlinks(&path)?;
        let meta = regular(&path)?;
        files.push((
            path.clone(),
            Entry {
                payload: format!("source/{name}"),
                guest: path,
                sha256: digest.clone(),
                bytes: meta.len(),
                executable: false,
            },
        ));
    }
    for (index, (name, test)) in required.iter().enumerate() {
        purpose.check_artifact(artifacts, name, *test)?;
        let guest = process::executable(artifacts, name, *test)?;
        let role_path = guest
            .strip_prefix(repo)
            .context("artifact outside compiled repository")?;
        let role = purpose::role(role_path.to_str().context("UTF-8 artifact role path")?)?;
        ensure!(
            purpose.roles()[index] == role,
            "artifact role differs from payload purpose"
        );
        let target_root = if *name == "bbr-visual" {
            repo.join("scripts/visual/target")
        } else {
            repo.join("target")
        };
        let suffix = guest
            .strip_prefix(target_root.join("release"))
            .context("artifact must retain its compiled repository release path")?;
        relative(suffix.to_str().context("UTF-8 artifact path")?)?;
        // A managed target root can be a symlink. Never follow a symlink file
        // or any producer-selected path outside the canonical target root.
        let target = target_root.canonicalize()?;
        let canonical = guest.canonicalize()?;
        ensure!(
            canonical.starts_with(&target),
            "artifact escapes managed target"
        );
        regular(&guest)?;
        store::no_symlinks(&canonical)?;
        let meta = regular(&canonical)?;
        ensure!(
            meta.permissions().mode() & 0o111 != 0,
            "artifact is not executable"
        );
        let digest = hash(&canonical, cancelled)?;
        files.push((
            canonical,
            Entry {
                payload: format!("executables/{index}"),
                guest,
                sha256: digest,
                bytes: meta.len(),
                executable: true,
            },
        ));
    }
    let total = files
        .iter()
        .try_fold(0u64, |sum, (_, entry)| sum.checked_add(entry.bytes))
        .context("payload size overflow")?;
    ensure!(total <= TOTAL_LIMIT, "payload exceeds total transfer limit");
    fs::create_dir(destination)?;
    fs::set_permissions(destination, fs::Permissions::from_mode(0o700))?;
    // Partial failures deliberately retain diagnostics without a completion
    // manifest. Do not resume or overwrite a failed payload.
    for (source, entry) in &files {
        checkpoint(cancelled)?;
        let output = destination.join(&entry.payload);
        fs::create_dir_all(output.parent().context("payload parent")?)?;
        let mut input = File::open(source)?;
        let mut output_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)?;
        let mut buffer = [0u8; 65536];
        let mut bytes = 0;
        loop {
            checkpoint(cancelled)?;
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            bytes += count as u64;
            ensure!(bytes <= entry.bytes, "payload source grew during transfer");
            output_file.write_all(&buffer[..count])?;
        }
        ensure!(
            bytes == entry.bytes,
            "payload source shrank during transfer"
        );
        fs::set_permissions(
            &output,
            fs::Permissions::from_mode(if entry.executable { 0o500 } else { 0o400 }),
        )?;
        ensure!(
            hash(&output, cancelled)? == entry.sha256,
            "payload bytes changed while copying"
        );
        ensure!(
            hash(source, cancelled)? == entry.sha256,
            "source/artifact changed after copying"
        );
    }
    ensure!(
        &source_snapshot(repo, config, cancelled)? == before_build,
        "source changed after transfer"
    );
    let payload = Payload {
        schema: 1,
        suite: purpose.id().into(),
        repo: repo.into(),
        sources: before_build.clone(),
        files: files.into_iter().map(|(_, entry)| entry).collect(),
    };
    verify_files(destination, &payload, cancelled, false)?;
    checkpoint(cancelled)?;
    // Publish the completion manifest only after all checks. It identifies
    // transfer bytes, never guest environment, native execution or acceptance.
    let mut manifest = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination.join("payload.json"))?;
    manifest.write_all(&serde_json::to_vec_pretty(&payload)?)?;
    Ok(payload)
}

/// Verify exactly the staged file set before sharing it. Guest installation
/// will additionally verify each explicit destination; this does not boot Tart.
pub fn verify(root: &Path, payload: &Payload, cancelled: &AtomicBool) -> Result<()> {
    verify_files(root, payload, cancelled, true)?;
    store::no_symlinks(&root.join("payload.json"))?;
    regular(&root.join("payload.json"))?;
    let manifest: Payload = store::read_json(&root.join("payload.json"))?;
    ensure!(
        serde_json::to_value(manifest)? == serde_json::to_value(payload)?,
        "payload manifest changed"
    );
    Ok(())
}

fn verify_files(
    root: &Path,
    payload: &Payload,
    cancelled: &AtomicBool,
    completed: bool,
) -> Result<()> {
    use std::collections::BTreeSet;
    checkpoint(cancelled)?;
    store::no_symlinks(root)?;
    ensure!(
        payload.schema == 1 && !payload.sources.is_empty(),
        "invalid payload schema/sources"
    );
    let purpose = Purpose::parse(&payload.suite)?;
    ensure!(
        payload.repo.is_absolute() && store::absolute(&payload.repo)? == payload.repo,
        "canonical absolute guest repo required"
    );
    let mut expected = BTreeSet::new();
    if completed {
        expected.insert("payload.json".to_owned());
    }
    let mut guests = BTreeSet::new();
    let mut sources = Hashes::new();
    let mut executables = BTreeSet::new();
    let mut total = 0u64;
    for entry in &payload.files {
        relative(&entry.payload)?;
        let guest = entry
            .guest
            .strip_prefix(&payload.repo)
            .context("guest path outside repository")?;
        let guest = guest.to_str().context("UTF-8 guest path")?;
        relative(guest)?;
        ensure!(
            entry.sha256.len() == 64 && entry.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid payload hash"
        );
        total = total
            .checked_add(entry.bytes)
            .context("payload size overflow")?;
        ensure!(total <= TOTAL_LIMIT, "payload exceeds total transfer limit");
        if let Some(name) = entry.payload.strip_prefix("source/") {
            ensure!(!entry.executable && guest == name, "source mapping differs");
            sources.insert(name.to_owned(), entry.sha256.clone());
        } else {
            let index = entry
                .payload
                .strip_prefix("executables/")
                .context("invalid payload namespace")?;
            ensure!(
                index.parse::<usize>().is_ok_and(|n| n.to_string() == index)
                    && entry.executable
                    && (guest.starts_with("target/release/")
                        || guest == "scripts/visual/target/release/bbr-visual"),
                "invalid executable mapping"
            );
            let role = purpose::role(guest)?;
            ensure!(executables.insert(role), "duplicate executable role");
        }
        ensure!(
            expected.insert(entry.payload.clone()) && guests.insert(entry.guest.clone()),
            "duplicate payload/guest path"
        );
        let path = root.join(&entry.payload);
        store::no_symlinks(&path)?;
        let meta = regular(&path)?;
        ensure!(
            meta.len() == entry.bytes && hash(&path, cancelled)? == entry.sha256,
            "payload digest/size differs"
        );
        ensure!(
            (meta.permissions().mode() & 0o111 != 0) == entry.executable,
            "payload executable mode differs"
        );
    }
    ensure!(
        sources == payload.sources,
        "payload does not contain exactly sealed source inputs"
    );
    let required: BTreeSet<_> = purpose.roles().iter().copied().collect();
    ensure!(executables == required, "payload executable suite differs");
    let mut actual = BTreeSet::new();
    let mut queue = vec![root.to_path_buf()];
    while let Some(directory) = queue.pop() {
        checkpoint(cancelled)?;
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            ensure!(!entry.file_type()?.is_symlink(), "symlink in payload");
            if entry.file_type()?.is_dir() {
                queue.push(entry.path());
            } else {
                regular(&entry.path())?;
                actual.insert(
                    entry
                        .path()
                        .strip_prefix(root)?
                        .to_str()
                        .context("UTF-8 payload path")?
                        .to_owned(),
                );
            }
        }
    }
    ensure!(actual == expected, "missing or unexpected payload file");
    Ok(())
}
