//! Selected execution policy behind the existing actual-guest/install boundary.
//! The fixed host transport supplies bound requests; no normal CLI bypass.
#[cfg(test)]
use crate::selected_request::PATH as REQUEST;
use crate::{gallery, gallery_selection::Schedule, payload::Payload, selected_request::Request};
use anyhow::{ensure, Result};
#[cfg(test)]
use cargo_gpui_visual::{config::Config, selection::Selection};
use cargo_gpui_visual::{
    process::{Limits, Output},
    store,
};
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::Path,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "selected_tests.rs"]
mod tests;

#[cfg(test)]
const PLAN: &str = "scripts/visual/catalogs/gallery-plan.json";

pub(super) fn load(
    payload: &Payload,
    work: &Path,
    digest: &str,
    request_hash: &str,
) -> Result<Request> {
    Request::load(payload, work, digest, request_hash)
}

/// The Gallery inventory command constructs descriptors, not a rendering context.
/// Filter only its descriptive rows, never rendered images. All selected rows
/// must match the independently authored plan before the first capture command.
fn inventory(text: &str, plan: &gallery::Plan) -> Result<String> {
    let expected: BTreeSet<_> = plan
        .cases
        .iter()
        .flat_map(|c| {
            ["light", "dark"].map(|mode| format!("{}--{mode}\t{}\t{}", c.base, c.label, c.state))
        })
        .collect();
    let prefixes: BTreeSet<_> = plan
        .cases
        .iter()
        .map(|c| c.base.split("-state-").next().unwrap().to_owned() + "-state-")
        .collect();
    let mut selected = BTreeSet::new();
    let mut found = BTreeSet::new();
    for line in text.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        ensure!(
            fields.len() == 3 && fields.iter().all(|v| !v.is_empty()) && found.insert(fields[0]),
            "invalid/duplicate guest registry inventory"
        );
        if prefixes.iter().any(|prefix| fields[0].starts_with(prefix)) {
            selected.insert(line.to_owned());
        }
    }
    ensure!(
        selected == expected,
        "selected registry differs from independent plan"
    );
    Ok(selected.into_iter().collect::<Vec<_>>().join("\n") + "\n")
}

fn checkpoint(
    payload: &Payload,
    work: &Path,
    request: &Request,
    schedule: &Schedule,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> Result<()> {
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "selected guest capture cancelled"
    );
    ensure!(Instant::now() < deadline, "selected guest capture deadline");
    load(payload, work, request.payload_digest(), request.digest())?;
    schedule.check_sources(&payload.repo, &payload.sources)
}
fn regular_output(output: &Path) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    store::no_symlinks(output)?;
    ensure!(output.is_dir(), "selected story output missing");
    for entry in fs::read_dir(output)? {
        let meta = fs::symlink_metadata(entry?.path())?;
        ensure!(
            meta.is_file() && meta.nlink() == 1,
            "single-link story output required"
        );
    }
    Ok(())
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(bytes)?;
    Ok(())
}

/// Called only after actual guest/payload checks. Test-only callers inject the
/// same command boundary; production supplies upstream's owned-group executor.
pub(super) fn execute(
    payload: &Payload,
    work: &Path,
    request: &Request,
    cancelled: &AtomicBool,
    mut invoke: impl FnMut(&mut Command, &Limits) -> Result<Output>,
) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(600);
    request.check(payload)?;
    let schedule = request.schedule();
    checkpoint(payload, work, request, schedule, cancelled, deadline)?;
    let raw = work.join("raw");
    let stories = work.join("selected-stories");
    for path in [&raw, &stories, &work.join("live-inventory.tsv")] {
        store::no_symlinks(path)?;
        ensure!(!path.exists(), "fresh selected guest output required");
    }
    let mut run = |cmd: &mut Command| -> Result<Vec<u8>> {
        checkpoint(payload, work, request, schedule, cancelled, deadline)?;
        super::environment(cmd, &payload.repo);
        let output = invoke(
            cmd,
            &Limits {
                timeout: deadline.saturating_duration_since(Instant::now()),
                max_output_bytes: 8 * 1024 * 1024,
                ..Limits::default()
            },
        )?;
        checkpoint(payload, work, request, schedule, cancelled, deadline)?;
        ensure!(
            output.status.success(),
            "selected guest producer failed: {}",
            output.status
        );
        Ok(output.stdout)
    };
    let binary = payload.repo.join("target/release/bbr-gallery");
    let text = String::from_utf8(run(Command::new(&binary).arg("--capture-inventory"))?)?;
    let all_inventory = inventory(&text, schedule.plan())?;
    fs::create_dir(&stories)?;
    fs::create_dir(&raw)?;
    let mut coverage = String::new();
    for id in schedule.story_ids() {
        let output = stories.join(format!("story-{id:02}"));
        let stdout = run(Command::new(&binary)
            .arg("--capture-gallery")
            .arg(&output)
            .arg("--story")
            .arg(id.to_string()))?;
        std::io::stderr().write_all(&stdout)?;
        let plan = gallery::Plan {
            schema: schedule.plan().schema,
            cases: schedule
                .plan()
                .cases
                .iter()
                .filter(|c| c.base.starts_with(&format!("gallery-{id:02}-")))
                .cloned()
                .collect(),
        };
        let story_inventory = inventory(&text, &plan)?;
        regular_output(&output)?;
        let names = gallery::validate_raw(&output, &plan, &story_inventory)?;
        coverage += &fs::read_to_string(output.join("coverage.tsv"))?;
        for (name, _) in names {
            let name = format!("{name}.png");
            ensure!(!raw.join(&name).exists(), "duplicate selected story output");
            fs::rename(output.join(&name), raw.join(&name))?;
        }
    }
    write_new(&raw.join("coverage.tsv"), coverage.as_bytes())?;
    regular_output(&raw)?;
    gallery::validate_raw(&raw, schedule.plan(), &all_inventory)?;
    checkpoint(payload, work, request, schedule, cancelled, deadline)?;
    write_new(&work.join("live-inventory.tsv"), all_inventory.as_bytes())?;
    // Raw names stay unchanged for the host validator. This receipt is not a
    // capture seal; only upstream may seal or review selected evidence.
    write_new(
        &raw.join("selection.json"),
        &serde_json::to_vec(request.selection())?,
    )?;
    Ok(())
}
