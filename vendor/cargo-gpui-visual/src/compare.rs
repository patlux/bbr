use crate::{report, store::*};
use anyhow::{ensure, Result};
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::Path};

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
    pub results: Vec<Finding>,
    pub errors: Vec<String>,
    pub passed: bool,
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
    disjoint(&[run, baselines, report])?;
    let m = load_manifest(run)?;
    check_sources(repo, &m)?;
    validate_images(run, &m)?;
    let state = baseline_state(baselines)?;
    let mut errors = Vec::new();
    let mut environment_mismatch = None;
    let metadata = baselines.join("baseline.json");
    if metadata.exists() {
        let stored: Value = read_json(&metadata)?;
        if stored["schema"] != 1 {
            errors.push("Baseline schema mismatch".into());
        }
        if stored["environment"] != m.environment {
            environment_mismatch =
                Some(json!({"baselines":stored["environment"],"capture":m.environment}));
        }
        let cases = stored["cases"]
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid baseline case inventory"))?;
        for (case, evidence) in cases {
            let path = image_path(baselines, case)?;
            if !path.is_file() || digest(&path)? != evidence["image_hash"].as_str().unwrap_or("") {
                errors.push(format!("Baseline metadata/image mismatch: {case}"));
            }
        }
        let expected: BTreeSet<_> = cases.keys().map(|c| format!("{c}.png")).collect();
        let actual: BTreeSet<_> = state
            .keys()
            .filter(|n| n.ends_with(".png"))
            .cloned()
            .collect();
        if expected != actual {
            errors.push("Baseline files and baseline metadata disagree".into());
        }
    } else if !state.is_empty() {
        errors.push("Baseline images lack environment metadata".into());
    }
    let expected: BTreeSet<_> = m.cases.iter().map(|c| format!("{c}.png")).collect();
    let obsolete: Vec<_> = state
        .keys()
        .filter(|n| n.ends_with(".png") && !expected.contains(*n))
        .collect();
    if !obsolete.is_empty() {
        errors.push(format!(
            "Baseline cases absent from capture inventory: {obsolete:?}"
        ));
    }
    fs::create_dir_all(report)?;
    let mut results = Vec::new();
    for case in &m.cases {
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
        let baseline = image_path(baselines, case)?;
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
            fs::copy(&baseline, folder.join("expected.png"))?;
            let actual = read_png(&candidate)?;
            let expected = read_png(&baseline)?;
            finding.actual_size = Some([actual.width(), actual.height()]);
            finding.expected_size = Some([expected.width(), expected.height()]);
            if actual.dimensions() != expected.dimensions() {
                finding.status = "changed".into();
                finding.reason = Some("dimensions".into());
                RgbaImage::from_pixel(
                    actual.width().max(expected.width()),
                    actual.height().max(expected.height()),
                    Rgba([255, 0, 160, 255]),
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
                            Rgba([255, 0, 160, 255])
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
        results.push(finding);
    }
    // A concurrent editor or acceptance cannot produce a sealable green review.
    check_sources(repo, &m)?;
    validate_images(run, &m)?;
    ensure!(
        state == baseline_state(baselines)?,
        "Baselines changed during comparison"
    );
    let passed = errors.is_empty()
        && environment_mismatch.is_none()
        && results.iter().all(|r| r.status == "same");
    let review = Review {
        schema: 1,
        capture_hash: digest(&run.join("capture.json"))?,
        baseline_state: state,
        environment: m.environment,
        environment_mismatch,
        results,
        errors,
        passed,
    };
    for name in ["review.json", "index.html"] {
        no_symlinks(&report.join(name))?;
    }
    write_json(&report.join("review.json"), &review)?;
    report::write(report, &review)?;
    Ok(review)
}
