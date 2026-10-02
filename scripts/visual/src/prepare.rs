//! Host-only mbx build and immutable payload staging. Never executes a producer.
use crate::{
    environment, payload, process, provenance, vm_environment::HostBuild, vm_runtime, Suite,
};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{config::Config, store};
use serde_json::Value;
use std::{path::Path, sync::atomic::AtomicBool};

pub fn host_build(probe: &Value, payload: &payload::Payload) -> Result<HostBuild> {
    let text = |key: &str| -> Result<String> {
        Ok(probe[key]
            .as_str()
            .with_context(|| format!("missing host build observation: {key}"))?
            .into())
    };
    let host = HostBuild {
        os: text("os")?,
        os_build: text("os_build")?,
        arch: text("arch")?,
        rustc: text("rustc")?,
        xcode: text("xcode")?,
        sdk: text("sdk")?,
        gpui_source: text("gpui_source")?,
        tool_revision: text("tool_revision")?,
        executable_hashes: payload
            .files
            .iter()
            .filter(|entry| entry.executable)
            .map(|entry| {
                (
                    entry.guest.to_string_lossy().into_owned(),
                    entry.sha256.clone(),
                )
            })
            .collect(),
    };
    host.validate()?;
    Ok(host)
}

pub fn prepare(
    repo: &Path,
    purpose: impl Into<payload::Purpose>,
    destination: &Path,
    cancelled: &AtomicBool,
) -> Result<HostBuild> {
    // Validate all static inputs before invoking a build, probe or creating files.
    ensure!(
        destination.is_absolute() && store::absolute(destination)? == destination,
        "canonical absolute payload destination required"
    );
    store::disjoint(&[repo, destination])?;
    ensure!(
        !destination.exists() && destination.parent().is_some_and(Path::is_dir),
        "fresh payload below an existing directory required"
    );
    provenance::verify(repo)?;
    let purpose = purpose.into();
    let suite = purpose.source_suite();
    let config = Config::load(&repo.join(suite.config()))?;
    let before = payload::source_snapshot(repo, &config, cancelled)?;
    let host_before = environment::probe(repo, suite, cancelled)?;
    let arguments: &[&str] = match suite {
        Suite::Gallery => &["-p", "bbr-gallery", "--features", "visual-capture"],
        Suite::Browser => &[
            "-p",
            "bbr",
            "--features",
            "visual-capture",
            "--bin",
            "bbr",
            "--test",
            "e2e_visual_shell_capture",
        ],
    };
    let mut artifacts = if purpose == payload::Purpose::GallerySemantic {
        process::build_gallery_semantic(repo, cancelled)?
    } else {
        process::build(repo, arguments, cancelled)?
    };
    artifacts.extend(process::build(
        repo,
        &[
            "--manifest-path",
            "scripts/visual/Cargo.toml",
            "--bin",
            "bbr-visual",
        ],
        cancelled,
    )?);
    ensure!(
        environment::probe(repo, suite, cancelled)? == host_before,
        "host build environment changed; rebuild before transfer"
    );
    let staged = payload::stage(
        repo,
        &config,
        &before,
        &artifacts,
        purpose,
        destination,
        cancelled,
    )?;
    let host = host_build(&host_before, &staged)?;
    vm_runtime::verify(repo, purpose, destination, &staged, cancelled)?;
    ensure!(
        payload::source_snapshot(repo, &config, cancelled)? == before,
        "source changed during runtime preflight; rebuild before transfer"
    );
    // Build evidence stays outside the shared payload so its exact inventory
    // remains unchanged. The caller records this typed return value explicitly.
    Ok(host)
}
