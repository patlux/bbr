//! Fixed semantic action behind guest identity/installed-byte guards only.
use crate::{payload::Payload, semantic};
use anyhow::{ensure, Result};
use cargo_gpui_visual::{
    process::{Limits, Output},
    store,
};
use std::{
    fs,
    io::Write,
    path::Path,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
fn execute(
    payload: &Payload,
    work: &Path,
    digest: &str,
    cancelled: &AtomicBool,
    invoke: impl FnMut(&mut Command, &Limits) -> Result<Output>,
) -> Result<()> {
    execute_until(
        payload,
        work,
        digest,
        cancelled,
        Instant::now() + Duration::from_secs(600),
        invoke,
    )
}

pub(super) fn execute_until(
    payload: &Payload,
    work: &Path,
    digest: &str,
    cancelled: &AtomicBool,
    deadline: Instant,
    mut invoke: impl FnMut(&mut Command, &Limits) -> Result<Output>,
) -> Result<()> {
    let checkpoint = || -> Result<()> {
        ensure!(
            !cancelled.load(Ordering::SeqCst),
            "semantic execution cancelled"
        );
        ensure!(Instant::now() < deadline, "semantic execution deadline");
        semantic::Requirements::load(payload)?;
        Ok(())
    };
    checkpoint()?;
    ensure!(
        digest.len() == 64
            && digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "invalid semantic payload digest"
    );
    let plan = semantic::Requirements::load(payload)?;
    let binary = semantic::executable(payload)?;
    let raw = work.join("raw");
    store::no_symlinks(&raw)?;
    ensure!(!raw.exists(), "fresh semantic output required");
    super::semantic_diagnostics::require_fresh(&payload.repo)?;
    let mut run = |args: &[&str]| -> Result<String> {
        checkpoint()?;
        let mut command = Command::new(&binary.guest);
        command.args(args);
        super::environment(&mut command, &payload.repo);
        let output = invoke(
            &mut command,
            &Limits {
                timeout: deadline.saturating_duration_since(Instant::now()),
                max_output_bytes: semantic::OUTPUT_LIMIT,
                ..Limits::default()
            },
        )?;
        // Retain diagnostics even for failed/invalid runs in the bounded parent log.
        ensure!(
            output.stdout.len() as u64 <= semantic::OUTPUT_LIMIT,
            "semantic stdout exceeded bound"
        );
        std::io::stderr().write_all(&output.stdout)?;
        checkpoint()?;
        ensure!(
            output.status.success(),
            "semantic harness failed: {}",
            output.status
        );
        Ok(String::from_utf8(output.stdout)?)
    };
    let listing = run(&[
        "--ignored",
        "--list",
        "--format",
        "pretty",
        "--color",
        "never",
    ])?;
    plan.validate_listing(&listing)?;
    super::semantic_diagnostics::require_fresh(&payload.repo)?;
    let execution = run(&[
        "--ignored",
        "--nocapture",
        "--test-threads=1",
        "--format",
        "pretty",
        "--color",
        "never",
    ])?;
    let record = semantic::Record::new(payload, digest, listing, execution)?;
    checkpoint()?;
    fs::create_dir(&raw)?;
    super::semantic_diagnostics::collect(&payload.repo, &raw, checkpoint)?;
    checkpoint()?;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(raw.join(semantic::RESULT))?
        .write_all(&serde_json::to_vec(&record)?)?;
    // Dispatcher repeats full installed/shared identity before it returns success.
    Ok(())
}

#[cfg(test)]
#[path = "semantic_tests.rs"]
mod tests;
