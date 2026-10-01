use crate::{
    baseline::{Baseline, Case},
    compare::Review,
    store::*,
};
use anyhow::{ensure, Result};
use sha2::{Digest, Sha256};
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
    accept_with_format(
        repo,
        run,
        baselines,
        report,
        selected,
        adopt,
        crate::codec::Format::Png,
    )
}

/// The format acceptance writes: an existing store's declared format, else the
/// configured one (PNG by default). A configured format that differs from an
/// existing store is refused; convert the store explicitly instead.
pub fn store_format(
    baselines: &Path,
    configured: Option<crate::codec::Format>,
) -> Result<crate::codec::Format> {
    if !baselines.join("baseline.json").exists() {
        return Ok(configured.unwrap_or_default());
    }
    let store = Baseline::load(baselines)?.format();
    if let Some(configured) = configured {
        ensure!(
            configured == store,
            "Baselines keep {} references but the configuration declares {}; convert explicitly with convert-baselines",
            store.extension(),
            configured.extension()
        );
    }
    Ok(store)
}

/// `accept`, creating a missing store in `new_store` format (for example the
/// consumer's configured `baseline_format`). Existing stores keep their format.
pub fn accept_with_format(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
    selected: &[String],
    adopt: bool,
    new_store: crate::codec::Format,
) -> Result<()> {
    ensure!(
        selected.iter().collect::<BTreeSet<_>>().len() == selected.len()
            && (!selected.is_empty() || adopt),
        "Explicit unique case IDs required; no accept-all default"
    );
    accept_reviewed(repo, run, baselines, report, adopt, new_store, |_, _| {
        Ok(selected.to_vec())
    })
}

/// What an explicit `accept --all` takes from one sealed review: every still it
/// flags as changed or missing and every complete journey it flags as changed or
/// new. Journey frames are never listed on their own; unchanged cases never are.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Reviewed {
    pub changed: Vec<String>,
    pub missing: Vec<String>,
    pub sequences: Vec<String>,
}
impl Reviewed {
    /// The explicit selection passed to the ordinary acceptance checks.
    pub fn ids(&self) -> Vec<String> {
        self.changed
            .iter()
            .chain(&self.missing)
            .chain(&self.sequences)
            .cloned()
            .collect()
    }
    pub fn len(&self) -> usize {
        self.changed.len() + self.missing.len() + self.sequences.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn reviewed(review: &Review, m: &Manifest) -> Reviewed {
    let frames: BTreeSet<&String> = m.sequences.iter().flat_map(|s| s.images()).collect();
    let mut result = Reviewed::default();
    for finding in review.results.iter().filter(|f| !frames.contains(&f.case)) {
        match finding.status.as_str() {
            "changed" => result.changed.push(finding.case.clone()),
            "missing-baseline" => result.missing.push(finding.case.clone()),
            _ => {}
        }
    }
    for sequence in &review.sequences {
        if sequence.expected.is_none() || sequence.first_difference.is_some() {
            result.sequences.push(sequence.actual.id.clone());
        }
    }
    result
}

/// Read-only: what `accept_all` would take from this exact run and report now.
pub fn preview_all(run: &Path, report: &Path) -> Result<Reviewed> {
    let m = load_manifest(run)?;
    let review: Review = read_json(&report.join("review.json"))?;
    ensure!(
        review.capture_hash == digest(&run.join("capture.json"))?,
        "Capture changed since review; regenerate report"
    );
    Ok(reviewed(&review, &m))
}

/// Accepts exactly the previously shown and confirmed list. It is derived again
/// from the review under the report lock and must still be identical; then every
/// ordinary acceptance check applies as for an explicit `--case` selection.
pub fn accept_all(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
    confirmed: &Reviewed,
    adopt: bool,
) -> Result<()> {
    accept_all_with_format(
        repo,
        run,
        baselines,
        report,
        confirmed,
        adopt,
        crate::codec::Format::Png,
    )
}

/// `accept_all`, creating a missing store in `new_store` format.
pub fn accept_all_with_format(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
    confirmed: &Reviewed,
    adopt: bool,
    new_store: crate::codec::Format,
) -> Result<()> {
    accept_reviewed(
        repo,
        run,
        baselines,
        report,
        adopt,
        new_store,
        |review, m| {
            let current = reviewed(review, m);
            ensure!(
                &current == confirmed,
                "Review changed since confirmation; nothing accepted"
            );
            ensure!(
                !current.is_empty(),
                "Nothing to accept: the review flags no changed or missing references"
            );
            Ok(current.ids())
        },
    )
}

/// Shows the exact list on stderr. On an interactive terminal it asks; otherwise
/// only the explicit `assume_yes` (`--yes`) confirms. Nothing is written here.
pub fn confirm_all(reviewed: &Reviewed, assume_yes: bool) -> Result<()> {
    use std::io::{IsTerminal, Write};
    ensure!(
        !reviewed.is_empty(),
        "Nothing to accept: the review flags no changed or missing references"
    );
    let mut err = std::io::stderr().lock();
    writeln!(
        err,
        "accept --all: {} changed stills, {} new stills, {} complete sequences",
        reviewed.changed.len(),
        reviewed.missing.len(),
        reviewed.sequences.len()
    )?;
    for (label, ids) in [
        ("changed", &reviewed.changed),
        ("new", &reviewed.missing),
        ("sequence", &reviewed.sequences),
    ] {
        for id in ids {
            writeln!(err, "  {label:<8} {id}")?;
        }
    }
    if assume_yes {
        return Ok(());
    }
    ensure!(
        std::io::stdin().is_terminal(),
        "accept --all needs an interactive confirmation or --yes; nothing accepted"
    );
    write!(err, "Accept all {} references? [y/N] ", reviewed.len())?;
    err.flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    ensure!(
        matches!(
            answer.trim().to_lowercase().as_str(),
            "y" | "yes" | "j" | "ja"
        ),
        "Not confirmed; nothing accepted"
    );
    Ok(())
}

fn accept_reviewed(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
    adopt: bool,
    new_store: crate::codec::Format,
    select: impl FnOnce(&Review, &Manifest) -> Result<Vec<String>>,
) -> Result<()> {
    disjoint(&[run, baselines, report])?;
    let m = load_manifest(run)?;
    check_sources(repo, &m, Some(baselines))?;
    validate_images(run, &m)?;
    // Hold through baseline mutation so a comparison cannot replace the images
    // after we have validated the review the user is accepting.
    let _report_lock = crate::report::ReportLock::acquire(report)?;
    let review: Review = read_json(&report.join("review.json"))?;
    let selected = select(&review, &m)?;
    let prepared = prepare(repo, run, baselines, &selected, adopt, new_store, m, review)?;
    let _baseline_lock = BaselineLock::acquire(baselines)?;
    prepared.revalidate()?;
    prepared.write()
}

/// One authority's validated, memory-owned payloads. Collection acceptance uses
/// this same policy, preparing every selected suite before acquiring write locks.
pub(crate) struct Prepared {
    repo: std::path::PathBuf,
    run: std::path::PathBuf,
    baselines: std::path::PathBuf,
    manifest: Manifest,
    review: Review,
    payloads: Vec<Payload>,
    obsolete: Vec<String>,
    metadata: Baseline,
}
/// One verified reference file to write, in the store's format.
struct Payload {
    case: String,
    image_hash: String,
    pixel_hash: Option<String>,
    bytes: Vec<u8>,
}
/// `new_store` only applies when no `baseline.json` exists yet; an existing
/// store always keeps its declared format (see `baseline::convert`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    selected: &[String],
    adopt: bool,
    new_store: crate::codec::Format,
    m: Manifest,
    review: Review,
) -> Result<Prepared> {
    ensure!(
        selected.iter().collect::<BTreeSet<_>>().len() == selected.len()
            && (!selected.is_empty() || adopt),
        "Explicit unique case IDs required; no accept-all default"
    );
    ensure!(
        m.selection.is_none() && review.selection.is_none(),
        "Selected-story runs are development-only; acceptance requires a full capture"
    );
    check_sources(repo, &m, Some(baselines))?;
    validate_images(run, &m)?;
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
    let sequence_ids: BTreeSet<_> = selected
        .iter()
        .filter(|id| m.sequences.iter().any(|s| &s.id == *id))
        .cloned()
        .collect();
    let mut expanded = Vec::new();
    for id in selected {
        if let Some(sequence) = m.sequences.iter().find(|s| &s.id == id) {
            expanded.extend(sequence.images().cloned());
        } else {
            ensure!(
                !id.starts_with("sequence-"),
                "Accept complete sequence IDs, never individual journey frames"
            );
            expanded.push(id.clone());
        }
    }
    let selected = &expanded;
    ensure!(
        review.sequences.len() == m.sequences.len()
            && m.sequences.iter().all(|s| review
                .sequences
                .iter()
                .filter(|r| &r.actual == s)
                .count()
                == 1),
        "Review sequence inventory mismatch"
    );
    for id in &sequence_ids {
        let sequence = review
            .sequences
            .iter()
            .find(|s| &s.actual.id == id)
            .unwrap();
        ensure!(
            sequence.expected.is_none() || sequence.first_difference.is_some(),
            "Not a reviewed changed/missing sequence: {id}"
        );
    }
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
                && results.get(case).is_some_and(|s| matches!(
                    s.as_str(),
                    "changed" | "missing-baseline"
                ) || (*s == "same"
                    && m.sequences
                        .iter()
                        .any(|sequence| sequence_ids.contains(&sequence.id)
                            && sequence.images().any(|image| image == case)))),
            "Not a reviewed changed/missing candidate: {case}"
        );
    }
    let metadata_path = baselines.join("baseline.json");
    let mut metadata = if metadata_path.exists() {
        Baseline::load(baselines)?
    } else {
        Baseline::with_format(m.environment.clone(), new_store)
    };
    ensure!(
        metadata
            .image_errors(baselines, &review.baseline_state)?
            .is_empty(),
        "Invalid baseline images/inventory"
    );
    let format = metadata.format();
    if adopt {
        for sequence in &review.sequences {
            ensure!(
                sequence_ids.contains(&sequence.actual.id)
                    || sequence.expected.is_none()
                    || sequence.first_difference.is_none(),
                "Unreviewed sequence changed with the environment: {}",
                sequence.actual.id
            );
        }
        for r in &review.results {
            if selected.contains(&r.case) {
                continue;
            }
            let baseline = metadata.reference_path(baselines, &r.case)?;
            ensure!(
                r.status != "changed",
                "Unreviewed references changed with the environment: {}",
                r.case
            );
            if baseline.exists() {
                // Byte-identical PNG, or (schema 3) identical decoded pixels.
                let recorded = metadata
                    .cases
                    .get(&r.case)
                    .and_then(|c| c.pixel_hash.as_ref());
                let identical = digest(&baseline)? == m.images[&r.case]
                    || match recorded {
                        Some(recorded) => {
                            &crate::codec::pixel_hash(&read_png(&image_path(run, &r.case)?)?)
                                == recorded
                        }
                        None => false,
                    };
                ensure!(
                    identical,
                    "Unselected reference is not byte-identical: {}",
                    r.case
                );
            }
        }
    }
    ensure!(
        metadata.environment == m.environment || adopt,
        "Baseline environment mismatch"
    );
    // Read, encode and prove every payload before any baseline mutation. PNG
    // stores keep the exact capture bytes; WebP references must decode back to
    // the capture's exact RGBA8 pixels (`Format::encode`).
    let records_pixels = metadata.records_pixels();
    let payloads = crate::parallel::map(selected, |c| {
        let bytes = fs::read(image_path(run, c)?)?;
        ensure!(
            format!("{:x}", Sha256::digest(&bytes)) == m.images[c],
            "Candidate changed while preparing payload: {c}"
        );
        if format == crate::codec::Format::Png && !records_pixels {
            return Ok(Payload {
                case: c.clone(),
                image_hash: m.images[c].clone(),
                pixel_hash: None,
                bytes,
            });
        }
        let pixels = crate::codec::Format::Png.decode(&bytes)?;
        let pixel_hash = Some(crate::codec::pixel_hash(&pixels));
        let bytes = match format {
            crate::codec::Format::Png => bytes,
            other => other.encode(&pixels)?,
        };
        Ok(Payload {
            case: c.clone(),
            image_hash: format!("{:x}", Sha256::digest(&bytes)),
            pixel_hash,
            bytes,
        })
    })?;
    for path in payloads
        .iter()
        .map(|p| metadata.reference_path(baselines, &p.case))
        .chain(std::iter::once(Ok(metadata_path.clone())))
    {
        let pending = path?.with_extension("pending");
        no_symlinks(&pending)?;
        ensure!(
            !pending.exists(),
            "Unfinished baseline write; inspect retained evidence: {}",
            pending.display()
        );
    }
    if !payloads.is_empty() {
        metadata.record_capture(&review.capture_hash, &m.sources)?;
    }
    for payload in &payloads {
        metadata.cases.insert(
            payload.case.clone(),
            Case {
                image_hash: payload.image_hash.clone(),
                capture_hash: review.capture_hash.clone(),
                pixel_hash: payload.pixel_hash.clone(),
            },
        );
    }
    let mut obsolete = Vec::new();
    if !sequence_ids.is_empty() {
        let sequences = &mut metadata.sequences;
        for sequence in sequences.iter().filter(|s| sequence_ids.contains(&s.id)) {
            for image in sequence.images().filter(|image| !selected.contains(image)) {
                reference_path(baselines, image, format)?;
                metadata.cases.remove(image);
                obsolete.push(image.clone());
            }
        }
        sequences.retain(|s| !sequence_ids.contains(&s.id));
        sequences.extend(
            m.sequences
                .iter()
                .filter(|s| sequence_ids.contains(&s.id))
                .cloned(),
        );
        sequences.sort_by(|a, b| a.id.cmp(&b.id));
    }
    metadata.prune_captures();
    if adopt {
        metadata.environment = m.environment.clone();
    }
    Ok(Prepared {
        repo: repo.into(),
        run: run.into(),
        baselines: baselines.into(),
        manifest: m,
        review,
        payloads,
        obsolete,
        metadata,
    })
}
impl Prepared {
    pub(crate) fn revalidate(&self) -> Result<()> {
        ensure!(
            baseline_state(&self.baselines)? == self.review.baseline_state,
            "Baselines changed since review"
        );
        ensure!(
            digest(&self.run.join("capture.json"))? == self.review.capture_hash,
            "Capture changed since review"
        );
        check_sources(&self.repo, &self.manifest, Some(&self.baselines))?;
        validate_images(&self.run, &self.manifest)
    }
    pub(crate) fn write(&self) -> Result<()> {
        for payload in &self.payloads {
            atomic_write(
                &self
                    .metadata
                    .reference_path(&self.baselines, &payload.case)?,
                &payload.bytes,
            )?;
        }
        // Only frames of a selected, reviewed journey that its new timeline no
        // longer renders; an entire removed journey still fails closed.
        for case in &self.obsolete {
            fs::remove_file(self.metadata.reference_path(&self.baselines, case)?)?;
        }
        let mut bytes = serde_json::to_vec_pretty(&self.metadata)?;
        bytes.push(b'\n');
        atomic_write(&self.baselines.join("baseline.json"), &bytes)
    }
}
/// Stale crash locks remain explicit recovery evidence, as before.
pub(crate) struct BaselineLock(std::path::PathBuf);
impl BaselineLock {
    pub(crate) fn acquire(root: &Path) -> Result<Self> {
        fs::create_dir_all(root)?;
        let path = root.join(".accept.lock");
        no_symlinks(&path)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(Self(path))
    }
}
impl Drop for BaselineLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
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
