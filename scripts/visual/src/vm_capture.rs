//! Fresh supervised transport -> raw preparation -> immutable upstream sealing.
//! No CLI, raw-directory import, renderer, environment fallback or acceptance.
use crate::{
    browser_output, gallery, gallery_selection::bound_bytes, payload, runner, selected_request,
    vm_transport, Suite,
};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{capture, config::Config, gate, store};
use std::{
    cell::RefCell,
    fs,
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "vm_capture_tests.rs"]
mod tests;

/// Internal only: network permission and real pilot remain external prerequisites.
/// Upstream snapshots current source BEFORE the one fresh transport invocation.
pub fn run(
    input: vm_transport::Input<'_>,
    paths: capture::Paths<'_>,
    options: &gate::Options,
    cancelled: &AtomicBool,
) -> Result<store::Manifest> {
    capture_with(input, paths, cancelled, |input| {
        vm_transport::run(input, options, cancelled)
    })
}

fn checkpoint(cancelled: &AtomicBool, deadline: Instant) -> Result<()> {
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "returned capture cancelled"
    );
    ensure!(
        Instant::now() < deadline,
        "returned capture preparation deadline"
    );
    Ok(())
}

fn copy(
    path: &Path,
    destination: &Path,
    record: &vm_transport::ReturnedFile,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> Result<()> {
    use sha2::{Digest, Sha256};
    checkpoint(cancelled, deadline)?;
    store::no_symlinks(path)?;
    let mut source = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let meta = source.metadata()?;
    ensure!(
        meta.is_file()
            && meta.nlink() == 1
            && meta.len() == record.bytes
            && (1..=64 * 1024 * 1024).contains(&meta.len()),
        "invalid returned copy source"
    );
    let mut target = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let mut digest = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        checkpoint(cancelled, deadline)?;
        let n = source.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count += n as u64;
        ensure!(count <= record.bytes, "returned copy grew");
        digest.update(&buffer[..n]);
        target.write_all(&buffer[..n])?;
    }
    ensure!(
        count == record.bytes && format!("{:x}", digest.finalize()) == record.sha256,
        "returned copy bytes changed"
    );
    Ok(())
}

// Private seam: production can only obtain Captured by fresh supervised run.
// Tests replace transport with newly generated synthetic files, never native code.
fn capture_with(
    input: vm_transport::Input<'_>,
    paths: capture::Paths<'_>,
    cancelled: &AtomicBool,
    mut transport: impl FnMut(vm_transport::Input<'_>) -> Result<vm_transport::Captured>,
) -> Result<store::Manifest> {
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "VM capture cancelled before preflight"
    );
    ensure!(
        paths.repo == input.repo
            && input.staged.repo == input.repo
            && input.staged.suite == input.suite.id(),
        "capture repository/suite differs"
    );
    runner::output_directory(input.repo, input.suite, paths.run)?;
    ensure!(
        !input.workspace.exists(),
        "fresh transport workspace required; never reseal old output"
    );
    store::disjoint(&[
        paths.run,
        paths.baselines,
        paths.report,
        input.share,
        input.workspace,
    ])?;
    let config_hash = input
        .staged
        .sources
        .get(input.suite.config())
        .context("unbound capture config")?;
    let bytes = bound_bytes(input.repo, input.suite.config(), config_hash)?;
    let mut config = Config::from_toml(std::str::from_utf8(&bytes)?)?;
    ensure!(
        paths.baselines == input.repo.join(&config.baselines),
        "capture baseline identity differs"
    );
    ensure!(
        payload::source_snapshot(input.repo, &config, cancelled)? == input.staged.sources,
        "stale prepared capture source"
    );
    ensure!(
        input.selection.is_none() || input.suite == Suite::Gallery,
        "only Gallery supports selected capture"
    );
    let digest = store::digest(&input.share.join("payload.json"))?;
    config.selection = input.selection.cloned();
    let selected = input
        .selection
        .map(|s| selected_request::Request::new(input.staged, &digest, s.clone()))
        .transpose()?;
    let expected = || vm_transport::ExpectedReturn {
        suite: input.suite,
        staged: input.staged,
        digest: &digest,
        host: input.host,
        selected: selected.as_ref(),
    };
    let plan_path = "scripts/visual/catalogs/gallery-plan.json";
    let plan: Option<gallery::Plan> = if input.suite == Suite::Gallery && selected.is_none() {
        Some(serde_json::from_slice(&bound_bytes(
            input.repo,
            plan_path,
            input
                .staged
                .sources
                .get(plan_path)
                .context("unbound independent Gallery plan")?,
        )?)?)
    } else {
        None
    };
    let captured = RefCell::new(None::<vm_transport::Captured>);
    // Only upstream writes/validates capture.json. Failed callbacks cannot reach
    // that boundary, and no import-old-raw public entrypoint exists.
    capture::capture_local(
        paths,
        &config,
        cancelled,
        |output| {
            let deadline = Instant::now() + Duration::from_secs(120);
            checkpoint(cancelled, deadline)?;
            let borrow = captured.borrow();
            let raw = borrow.as_ref().context("missing fresh guest capture")?;
            raw.recheck(expected(), cancelled)?;
            ensure!(
                raw.selection == config.selection,
                "returned capture scope differs"
            );
            let files = raw.files()?;
            // These evidence files are deferred until structural validation.
            // Every other entry is copied and rejected if unexpected.
            let metadata = ["guest-receipt.json", "live-inventory.tsv", "selection.json"];
            if input.suite == Suite::Browser {
                ensure!(
                    !files.contains_key("live-inventory.tsv"),
                    "browser returned Gallery inventory"
                );
            }
            ensure!(
                !files.contains_key("capture.json")
                    && !files.contains_key("inventory.json")
                    && !files.contains_key("sequences.json"),
                "raw transport must not supply seals/inventory/sequences"
            );
            fs::create_dir_all(output.parent().context("run parent")?)?;
            fs::create_dir(output)?;
            for (name, record) in &files {
                if !metadata.contains(&name.as_str()) {
                    copy(
                        &raw.raw.join(name),
                        &output.join(name),
                        record,
                        cancelled,
                        deadline,
                    )?;
                }
            }
            checkpoint(cancelled, deadline)?;
            match input.suite {
                Suite::Gallery => {
                    let record = files
                        .get("live-inventory.tsv")
                        .context("missing returned registry inventory")?;
                    let inventory = String::from_utf8(bound_bytes(
                        &raw.raw,
                        "live-inventory.tsv",
                        &record.sha256,
                    )?)?;
                    if let Some(request) = &selected {
                        request.schedule().validate_and_prepare(
                            input.repo,
                            &input.staged.sources,
                            output,
                            &inventory,
                        )?;
                    } else {
                        gallery::validate_and_prepare(
                            output,
                            plan.as_ref().context("missing independent plan")?,
                            &inventory,
                        )?;
                    }
                }
                Suite::Browser => {
                    browser_output::validate_and_prepare(output)?;
                }
            }
            checkpoint(cancelled, deadline)?;
            raw.recheck(expected(), cancelled)?;
            // Keep provenance in the run as unchanged ancillary bytes, only after
            // structural producer validation. Original transport marker stays in
            // raw diagnostics: canonical renames must not rewrite that evidence.
            for name in ["guest-receipt.json", "live-inventory.tsv"] {
                if let Some(record) = files.get(name) {
                    copy(
                        &raw.raw.join(name),
                        &output.join(name),
                        record,
                        cancelled,
                        deadline,
                    )?;
                }
            }
            verify_prepared(raw, output, input.suite, cancelled)?;
            checkpoint(cancelled, deadline)?;
            Ok(())
        },
        || {
            if captured.borrow().is_none() {
                // First upstream environment callback: source snapshot already taken.
                // The transport observes actual guest before/after facts and cleanup.
                *captured.borrow_mut() = Some(transport(input)?);
            }
            let borrow = captured.borrow();
            let raw = borrow.as_ref().context("missing guest environment")?;
            ensure!(
                !cancelled.load(Ordering::SeqCst),
                "VM capture cancelled after transport"
            );
            raw.recheck(expected(), cancelled)?;
            ensure!(
                payload::source_snapshot(input.repo, &config, cancelled)? == input.staged.sources
                    && store::digest(&input.share.join("payload.json"))? == digest,
                "source/payload changed before sealing"
            );
            if paths.run.exists() {
                verify_prepared(raw, paths.run, input.suite, cancelled)?;
            }
            Ok(raw.environment.clone())
        },
    )
}

fn verify_prepared(
    raw: &vm_transport::Captured,
    output: &Path,
    suite: Suite,
    cancelled: &AtomicBool,
) -> Result<()> {
    use std::collections::BTreeSet;
    let mut names = BTreeSet::new();
    let mut images = BTreeSet::new();
    for (name, record) in raw.files()? {
        let target = if suite == Suite::Gallery && name.ends_with(".png") {
            format!(
                "{}.png",
                gallery::canonical_id(name.strip_suffix(".png").unwrap())?
            )
        } else {
            name
        };
        let path = output.join(&target);
        store::no_symlinks(&path)?;
        let meta = fs::symlink_metadata(&path)?;
        ensure!(
            meta.is_file()
                && meta.nlink() == 1
                && meta.len() == record.bytes
                && payload::hash(&path, cancelled)? == record.sha256,
            "prepared bytes differ from transport"
        );
        if let Some(id) = target.strip_suffix(".png") {
            images.insert(id.to_owned());
        }
        ensure!(names.insert(target), "duplicate prepared identity");
    }
    names.insert("inventory.json".into());
    let actual = fs::read_dir(output)?
        .map(|entry| {
            entry?
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("non-UTF8 prepared name"))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(actual == names, "prepared output inventory differs");
    let path = output.join("inventory.json");
    store::no_symlinks(&path)?;
    let meta = fs::symlink_metadata(&path)?;
    ensure!(
        meta.is_file() && meta.nlink() == 1 && meta.len() <= 8 * 1024 * 1024,
        "invalid prepared inventory"
    );
    let ids: Vec<String> = store::read_json(&path)?;
    ensure!(
        ids.len() == images.len() && ids.into_iter().collect::<BTreeSet<_>>() == images,
        "prepared image inventory differs"
    );
    Ok(())
}
