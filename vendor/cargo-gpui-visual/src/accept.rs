use crate::{compare::Review, store::*};
use anyhow::{ensure, Result};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub fn accept(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
    selected: &[String],
    adopt: bool,
) -> Result<()> {
    disjoint(&[run, baselines, report])?;
    ensure!(
        selected.iter().collect::<BTreeSet<_>>().len() == selected.len()
            && (!selected.is_empty() || adopt),
        "Explicit unique case IDs required; no accept-all default"
    );
    let m = load_manifest(run)?;
    check_sources(repo, &m)?;
    validate_images(run, &m)?;
    let review: Review = read_json(&report.join("review.json"))?;
    ensure!(
        review.schema == 1 && review.capture_hash == digest(&run.join("capture.json"))?,
        "Capture changed since review; regenerate report"
    );
    ensure!(
        review.baseline_state == baseline_state(baselines)?,
        "Baselines changed since review; regenerate report"
    );
    ensure!(
        review.environment == m.environment,
        "Review environment mismatch"
    );
    ensure!(
        review.errors.is_empty(),
        "Cannot accept an invalid comparison"
    );
    ensure!(
        review.environment_mismatch.is_none() || adopt,
        "Environment differs; review and use --adopt-environment"
    );
    ensure!(
        !adopt || review.environment_mismatch.is_some(),
        "--adopt-environment requires a reviewed environment change"
    );
    let results: BTreeMap<_, _> = review
        .results
        .iter()
        .map(|r| (&r.case, &r.status))
        .collect();
    ensure!(
        results.len() == m.cases.len() && m.cases.iter().all(|c| results.contains_key(c)),
        "Review inventory mismatch"
    );
    for case in selected {
        ensure!(
            m.cases.contains(case)
                && results
                    .get(case)
                    .is_some_and(|s| matches!(s.as_str(), "changed" | "missing-baseline")),
            "Not a reviewed changed/missing candidate: {case}"
        );
    }
    if adopt {
        for r in &review.results {
            if selected.contains(&r.case) {
                continue;
            }
            let baseline = image_path(baselines, &r.case)?;
            ensure!(
                r.status != "changed",
                "Unreviewed references changed with the environment: {}",
                r.case
            );
            if baseline.exists() {
                ensure!(
                    digest(&baseline)? == m.images[&r.case],
                    "Unselected reference is not byte-identical: {}",
                    r.case
                );
            }
        }
    }
    let metadata_path = baselines.join("baseline.json");
    let mut metadata: Value = if metadata_path.exists() {
        read_json(&metadata_path)?
    } else {
        json!({"schema":1,"environment":m.environment,"cases":{}})
    };
    ensure!(
        metadata["schema"] == 1 && metadata["cases"].is_object(),
        "Invalid baseline metadata"
    );
    ensure!(
        metadata["environment"] == m.environment || adopt,
        "Baseline environment mismatch"
    );
    // Read and validate every payload before any baseline mutation.
    let payloads = selected
        .iter()
        .map(|c| Ok((c.clone(), fs::read(image_path(run, c)?)?)))
        .collect::<Result<Vec<_>>>()?;
    for (case, _) in &payloads {
        image_path(baselines, case)?;
    }
    for (case, _) in &payloads {
        metadata["cases"][case] = json!({"image_hash":m.images[case],"capture_hash":review.capture_hash,"sources":m.sources});
    }
    if adopt {
        metadata["environment"] = m.environment.clone();
    }
    // Cooperative lock + revalidation; interrupted writes remain detectably invalid,
    // never silently accepted or resealed on the next invocation.
    fs::create_dir_all(baselines)?;
    let lock = baselines.join(".accept.lock");
    let _file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)?;
    let result = (|| {
        ensure!(
            baseline_state(baselines)? == review.baseline_state,
            "Baselines changed since review"
        );
        check_sources(repo, &m)?;
        validate_images(run, &m)?;
        for (case, payload) in payloads {
            atomic_write(&image_path(baselines, &case)?, &payload)?;
        }
        let mut bytes = serde_json::to_vec_pretty(&metadata)?;
        bytes.push(b'\n');
        atomic_write(&metadata_path, &bytes)
    })();
    fs::remove_file(lock)?;
    result
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    no_symlinks(path)?;
    let pending = path.with_extension("pending");
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&pending)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(pending, path)?;
    Ok(())
}
