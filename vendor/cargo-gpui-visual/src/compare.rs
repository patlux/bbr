use crate::{codec::Format, report, store::*};
use anyhow::{ensure, Context, Result};
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

/// Diff colour of every differing RGBA pixel; the report reads it back for row extents.
pub const CHANGED: [u8; 4] = [255, 0, 160, 255];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Finding {
    pub case: String,
    pub status: String,
    pub changed_pixels: Option<u64>,
    pub bounds: Option<[u32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_size: Option<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_size: Option<[u32; 2]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Review {
    pub schema: u32,
    pub capture_hash: String,
    pub baseline_state: Hashes,
    pub environment: Value,
    pub environment_mismatch: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_difference: Option<Value>,
    pub results: Vec<Finding>,
    pub errors: Vec<String>,
    /// A selected run can pass its scope, never the full suite.
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<crate::selection::Selection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sequences: Vec<crate::sequence::SequenceReview>,
}
pub fn changed_rows(rows: &[u32], limit: usize) -> Vec<[u32; 2]> {
    let mut ranges: Vec<[u32; 2]> = Vec::new();
    for &row in rows {
        if let Some(last) = ranges.last_mut().filter(|last| last[1] == row) {
            last[1] = row + 1;
        } else {
            ranges.push([row, row + 1]);
        }
    }
    let mut gap = 8u64;
    loop {
        let mut merged: Vec<[u32; 2]> = Vec::new();
        for &[top, bottom] in &ranges {
            if let Some(last) = merged
                .last_mut()
                .filter(|last| u64::from(top) <= u64::from(last[1]) + gap)
            {
                last[1] = bottom;
            } else {
                merged.push([top, bottom]);
            }
        }
        if merged.len() <= limit.max(1) {
            return merged;
        }
        gap *= 2;
    }
}
pub fn compare(repo: &Path, run: &Path, baselines: &Path, report: &Path) -> Result<Review> {
    compare_with_options(
        repo,
        run,
        baselines,
        report,
        &crate::report::Options::default(),
    )
}

/// The exact comparator with consumer-defined gallery presentation.
pub fn compare_with_options(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
    options: &crate::report::Options,
) -> Result<Review> {
    compare_timed(
        repo,
        run,
        baselines,
        report,
        options,
        &mut crate::timing::Timings::default(),
    )
}

/// Same comparator and report with observational wall times, outside the review seal.
pub fn compare_timed(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
    options: &crate::report::Options,
    timings: &mut crate::timing::Timings,
) -> Result<Review> {
    options.validate()?;
    compare_inner(repo, run, baselines, report, Some(options), timings)
}

/// Collection assets share the exact comparator without publishing an alternate
/// single-suite acceptance seal or HTML beside the collection's review.
pub(crate) fn compare_assets(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
) -> Result<Review> {
    compare_inner(
        repo,
        run,
        baselines,
        report,
        None,
        &mut crate::timing::Timings::default(),
    )
}

fn compare_inner(
    repo: &Path,
    run: &Path,
    baselines: &Path,
    report: &Path,
    publish: Option<&crate::report::Options>,
    timings: &mut crate::timing::Timings,
) -> Result<Review> {
    let started = std::time::Instant::now();
    disjoint(&[run, baselines, report])?;
    let m = load_manifest(run)?;
    check_sources(repo, &m, Some(baselines))?;
    validate_images(run, &m)?;
    let state = baseline_state(baselines)?;
    let mut errors = Vec::new();
    let mut environment_mismatch = None;
    let mut provenance_difference = None;
    let metadata = baselines.join("baseline.json");
    let mut expected_sequences: Vec<crate::sequence::Sequence> = Vec::new();
    let mut stored_cases = BTreeMap::new();
    let mut format = Format::Png;
    if metadata.exists() {
        let stored = crate::baseline::Baseline::load(baselines)?;
        provenance_difference =
            crate::environment::provenance_difference(&stored.environment, &m.environment)?;
        if !crate::environment::equivalent(&stored.environment, &m.environment)? {
            environment_mismatch =
                Some(json!({"baselines":stored.environment,"capture":m.environment}));
        }
        errors.extend(stored.image_errors(baselines, &state)?);
        format = stored.format();
        stored_cases = stored.cases;
        expected_sequences = stored.sequences;
    } else if !state.is_empty() {
        errors.push("Baseline images lack environment metadata".into());
    }
    let extension = format!(".{}", format.extension());
    let expected: BTreeSet<_> = m.cases.iter().map(|c| format!("{c}{extension}")).collect();
    // Frames removed by an extant journey are reviewed with that complete sequence.
    // Removing a whole journey (or an unrelated case) still fails closed.
    let replaced_frames: BTreeSet<_> = expected_sequences
        .iter()
        .filter(|s| m.sequences.iter().any(|a| a.id == s.id))
        .flat_map(|s| s.images().map(|image| format!("{image}{extension}")))
        .collect();
    let obsolete: Vec<_> = state
        .keys()
        .filter(|n| {
            n.ends_with(&extension) && !expected.contains(*n) && !replaced_frames.contains(*n)
        })
        .filter(|n| {
            m.selection.as_ref().is_none_or(|s| {
                s.includes_baseline(n.trim_end_matches(&extension), &expected_sequences)
            })
        })
        .collect();
    if !obsolete.is_empty() {
        errors.push(format!(
            "Baseline cases absent from capture inventory: {obsolete:?}"
        ));
    }
    fs::create_dir_all(report)?;
    let _report_lock = report::ReportLock::acquire(report)?;
    // Invalidate the old review before touching any displayed pixels. A failed
    // replacement must never leave an acceptable seal beside partially new images.
    for name in ["review.json", "index.html"] {
        let path = report.join(name);
        no_symlinks(&path)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    // Cases own disjoint report folders and read only their own images, so they
    // run concurrently; findings keep inventory order and the first failing case
    // in that order is reported.
    let results = crate::parallel::map(&m.cases, |case| {
        let folder = report.join(case);
        no_symlinks(&folder)?;
        fs::create_dir_all(&folder)?;
        for name in ["actual.png", "expected.png", "diff.png"] {
            let path = folder.join(name);
            no_symlinks(&path)?;
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        let candidate = image_path(run, case)?;
        let baseline = reference_path(baselines, case, format)?;
        fs::copy(&candidate, folder.join("actual.png"))?;
        let mut finding = Finding {
            case: case.clone(),
            status: "missing-baseline".into(),
            changed_pixels: None,
            bounds: None,
            actual_size: None,
            expected_size: None,
            rows: Vec::new(),
            reason: None,
        };
        if baseline.exists() {
            let actual = read_png(&candidate)?;
            // Fast path, only for an intact reference (file digest equals its
            // recorded hash): identical file bytes, or an identical library-
            // computed pixel fingerprint, means identical pixels. Otherwise the
            // reference is decoded and every pixel compared exactly.
            let recorded = stored_cases.get(case).filter(|evidence| {
                state
                    .get(&format!("{case}{extension}"))
                    .is_some_and(|hash| hash == &evidence.image_hash)
            });
            let known_same = recorded.is_some_and(|evidence| {
                evidence.image_hash == m.images[case]
                    || evidence
                        .pixel_hash
                        .as_ref()
                        .is_some_and(|hash| hash == &crate::codec::pixel_hash(&actual))
            });
            let expected = if known_same {
                // Pixels are not needed, but the file must still be a reference
                // of the declared format (for WebP: the exact lossless container).
                format
                    .check(&fs::read(&baseline)?)
                    .with_context(|| format!("Invalid reference: {case}"))?;
                None
            } else {
                let pixels = format.read(&baseline)?;
                if let Some(hash) = stored_cases.get(case).and_then(|c| c.pixel_hash.as_ref()) {
                    ensure!(
                        &crate::codec::pixel_hash(&pixels) == hash,
                        "Reference pixels differ from the recorded fingerprint: {case}"
                    );
                }
                Some(pixels)
            };
            // Report assets stay PNG. A PNG reference is copied as before; a WebP
            // reference is shown as its (identical) candidate or re-encoded PNG.
            match (format, &expected) {
                (Format::Png, _) => {
                    fs::copy(&baseline, folder.join("expected.png"))?;
                }
                (_, None) => {
                    fs::copy(&candidate, folder.join("expected.png"))?;
                }
                (_, Some(pixels)) => pixels.save(folder.join("expected.png"))?,
            }
            let expected = expected.as_ref().unwrap_or(&actual);
            finding.actual_size = Some([actual.width(), actual.height()]);
            finding.expected_size = Some([expected.width(), expected.height()]);
            if actual.dimensions() != expected.dimensions() {
                finding.status = "changed".into();
                finding.reason = Some("dimensions".into());
                RgbaImage::from_pixel(
                    actual.width().max(expected.width()),
                    actual.height().max(expected.height()),
                    Rgba(CHANGED),
                )
                .save(folder.join("diff.png"))?;
            } else {
                let mut changed = 0;
                let mut bounds = [actual.width(), actual.height(), 0, 0];
                let mut rows = Vec::new();
                let mut diff = RgbaImage::new(actual.width(), actual.height());
                for y in 0..actual.height() {
                    let mut row_changed = false;
                    for x in 0..actual.width() {
                        let a = actual.get_pixel(x, y);
                        let e = expected.get_pixel(x, y);
                        let pixel = if a != e {
                            changed += 1;
                            row_changed = true;
                            bounds = [
                                bounds[0].min(x),
                                bounds[1].min(y),
                                bounds[2].max(x + 1),
                                bounds[3].max(y + 1),
                            ];
                            Rgba(CHANGED)
                        } else {
                            let gray = ((u32::from(e[0]) * 299
                                + u32::from(e[1]) * 587
                                + u32::from(e[2]) * 114
                                + 500)
                                / 1000) as u8;
                            Rgba([gray, gray, gray, 255])
                        };
                        diff.put_pixel(x, y, pixel);
                    }
                    if row_changed {
                        rows.push(y);
                    }
                }
                finding.changed_pixels = Some(changed);
                if changed > 0 {
                    finding.bounds = Some(bounds);
                    finding.rows = changed_rows(&rows, 40);
                }
                finding.status = if changed == 0 { "same" } else { "changed" }.into();
                diff.save(folder.join("diff.png"))?;
            }
        }
        Ok(finding)
    })?;
    // Keep full-store integrity above, but render/compare only selected complete
    // journeys. Unselected baseline cases are not missing candidate evidence.
    if let Some(selection) = &m.selection {
        expected_sequences.retain(|s| selection.includes_sequence(&s.id));
    }
    if !expected_sequences.is_empty() {
        let folder = report.join("sequence-expected");
        no_symlinks(&folder)?;
        fs::create_dir_all(&folder)?;
        let same: BTreeSet<_> = results
            .iter()
            .filter(|r| r.status == "same")
            .map(|r| &r.case)
            .collect();
        let frames: Vec<_> = expected_sequences
            .iter()
            .flat_map(|s| s.images())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        crate::parallel::map(&frames, |image| {
            let target = image_path(&folder, image)?;
            let reference = reference_path(baselines, image, format)?;
            if format == Format::Png {
                fs::copy(reference, target)?;
            } else if same.contains(image) {
                // Proven pixel-identical to the candidate frame above.
                fs::copy(image_path(run, image)?, target)?;
            } else {
                format.read(&reference)?.save(target)?;
            }
            Ok(())
        })?;
    }
    // Draw/hold drift can reference different frame IDs at the same tick. Compare
    // their pixels too, once per unique pair, instead of showing an empty diff.
    let mut pairs = BTreeSet::new();
    for actual in &m.sequences {
        if let Some(expected) = expected_sequences.iter().find(|s| s.id == actual.id) {
            for (a, e) in actual.ticks.iter().zip(&expected.ticks) {
                if a.image != e.image && pairs.insert((&a.image, &e.image)) {
                    let path = report.join(crate::sequence_report::diff_path(&a.image, &e.image));
                    no_symlinks(&path)?;
                    fs::create_dir_all(path.parent().unwrap())?;
                    let a = read_png(&image_path(run, &a.image)?)?;
                    let e = format.read(&reference_path(baselines, &e.image, format)?)?;
                    let mut diff =
                        RgbaImage::new(a.width().max(e.width()), a.height().max(e.height()));
                    for (x, y, pixel) in diff.enumerate_pixels_mut() {
                        let a = a.get_pixel_checked(x, y);
                        let e = e.get_pixel_checked(x, y);
                        *pixel = if a != e {
                            Rgba(CHANGED)
                        } else {
                            e.copied().unwrap_or(Rgba([0, 0, 0, 0]))
                        };
                    }
                    diff.save(path)?;
                }
            }
        }
    }
    // A concurrent editor or acceptance cannot produce a sealable green review.
    check_sources(repo, &m, Some(baselines))?;
    validate_images(run, &m)?;
    ensure!(
        state == baseline_state(baselines)?,
        "Baselines changed during comparison"
    );
    let sequences: Vec<_> = m
        .sequences
        .iter()
        .map(|actual| {
            let expected = expected_sequences
                .iter()
                .find(|s| s.id == actual.id)
                .cloned();
            let metadata_difference = expected
                .as_ref()
                .and_then(|e| crate::sequence::first_difference(actual, e));
            let pixel_difference = actual.ticks.iter().position(|tick| {
                results
                    .iter()
                    .any(|r| r.case == tick.image && r.status != "same")
            });
            let first_difference = metadata_difference
                .into_iter()
                .chain(pixel_difference)
                .min();
            crate::sequence::SequenceReview {
                actual: actual.clone(),
                expected,
                first_difference,
            }
        })
        .collect();
    for sequence in &expected_sequences {
        if !m.sequences.iter().any(|s| s.id == sequence.id) {
            errors.push(format!(
                "Baseline sequence absent from capture: {}",
                sequence.id
            ));
        }
    }
    let passed = sequences
        .iter()
        .all(|s| s.expected.is_some() && s.first_difference.is_none())
        && errors.is_empty()
        && environment_mismatch.is_none()
        && results.iter().all(|r| r.status == "same");
    let review = Review {
        schema: m.schema,
        capture_hash: digest(&run.join("capture.json"))?,
        baseline_state: state,
        environment: m.environment,
        environment_mismatch,
        provenance_difference,
        results,
        errors,
        passed,
        selection: m.selection,
        sequences,
    };
    for name in ["review.json", "index.html"] {
        no_symlinks(&report.join(name))?;
    }
    timings.record("compare_validation_assets_ms", started);
    let started = std::time::Instant::now();
    if let Some(options) = publish {
        let paths = report::AcceptPaths {
            repo,
            run,
            baselines,
            report,
        };
        report::write_review(report, &review, options, Some(paths))?;
        // The acceptance seal is published last, only after every report asset exists.
        write_json(&report.join("review.json"), &review)?;
    }
    timings.record("report_ms", started);
    Ok(review)
}
