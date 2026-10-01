use super::*;
use cargo_gpui_visual::{
    accept::{accept_all, accept_with_format, preview_all, store_format},
    baseline::{self, Baseline},
    codec::{self, Format},
};

fn convert(f: &Fixture, to: Format, dry_run: bool) -> anyhow::Result<baseline::Conversion> {
    let hash = baseline_state(&f.base)?["baseline.json"].clone();
    baseline::convert(&f.base, to, &hash, dry_run)
}

fn metadata(f: &Fixture) -> Value {
    read_json(&f.base.join("baseline.json")).unwrap()
}

fn edit_metadata(f: &Fixture, edit: impl FnOnce(&mut Value)) {
    let path = f.base.join("baseline.json");
    let mut value: Value = read_json(&path).unwrap();
    edit(&mut value);
    write_json(&path, &value).unwrap();
}

fn candidate(f: &Fixture, case: &str) -> RgbaImage {
    Format::Png
        .read(&image_path(&f.run, case).unwrap())
        .unwrap()
}

fn webp_path(f: &Fixture, case: &str) -> PathBuf {
    reference_path(&f.base, case, Format::Webp).unwrap()
}

#[test]
fn converted_store_is_pixel_identical_schema_3_and_compares_like_png() {
    let f = Fixture::new();
    f.establish();
    let png_review = f.compare().unwrap();
    let png_diffs: Vec<_> = f
        .cases
        .iter()
        .map(|c| fs::read(f.report.join(c).join("diff.png")).unwrap())
        .collect();
    let before = baseline_state(&f.base).unwrap();
    let before_meta = metadata(&f);
    let preview = convert(&f, Format::Webp, true).unwrap();
    assert!(preview.changed && !preview.written);
    assert_eq!(
        (preview.cases, preview.from, preview.to),
        (2, Format::Png, Format::Webp)
    );
    assert_eq!(
        baseline_state(&f.base).unwrap(),
        before,
        "dry run writes nothing"
    );
    let result = convert(&f, Format::Webp, false).unwrap();
    assert!(result.written);
    assert_eq!(result.new_hash, preview.new_hash);
    assert_eq!(result.new_image_bytes, preview.new_image_bytes);
    let meta = metadata(&f);
    assert_eq!(
        (meta["schema"].clone(), meta["image_format"].clone()),
        (json!(3), json!("webp"))
    );
    for case in &f.cases {
        assert!(
            !image_path(&f.base, case).unwrap().exists(),
            "old PNG removed"
        );
        let stored = Format::Webp.read(&webp_path(&f, case)).unwrap();
        assert_eq!(stored.as_raw(), candidate(&f, case).as_raw());
        assert_eq!(
            meta["cases"][case]["pixel_hash"],
            codec::pixel_hash(&stored)
        );
        assert_eq!(
            meta["cases"][case]["image_hash"],
            digest(&webp_path(&f, case)).unwrap()
        );
    }
    // Environment, capture provenance and history are retained exactly.
    for key in ["environment", "captures"] {
        assert_eq!(meta[key], before_meta[key], "{key}");
    }
    for case in &f.cases {
        assert_eq!(
            meta["cases"][case]["capture_hash"],
            before_meta["cases"][case]["capture_hash"]
        );
    }
    let review = f.compare().unwrap();
    assert!(review.passed);
    let strip = |r: &Review| serde_json::to_value(&r.results).unwrap();
    assert_eq!(
        strip(&review),
        strip(&png_review),
        "fingerprint path yields identical findings"
    );
    for (case, png_diff) in f.cases.iter().zip(&png_diffs) {
        assert_eq!(
            &fs::read(f.report.join(case).join("diff.png")).unwrap(),
            png_diff
        );
        let shown = Format::Png
            .read(&f.report.join(case).join("expected.png"))
            .unwrap();
        assert_eq!(
            shown.as_raw(),
            candidate(&f, case).as_raw(),
            "report assets stay PNG"
        );
    }
    // Stored-report inspection is format-agnostic: its assets are PNG.
    let details = cargo_gpui_visual::inspect::inspect(
        &f.report,
        &cargo_gpui_visual::inspect::Options {
            case: Some(f.cases[0].clone()),
            ..Default::default()
        },
    )
    .unwrap()
    .to_string();
    assert!(details.contains("expected.png"), "{details}");
    let again = convert(&f, Format::Webp, false).unwrap();
    assert!(!again.changed && !again.written, "idempotent");
    // Converting back is exact too; schema 3 keeps the fingerprints.
    convert(&f, Format::Png, false).unwrap();
    assert_eq!(metadata(&f)["image_format"], "png");
    assert!(f.cases.iter().all(|c| !webp_path(&f, c).exists()));
    assert!(f.compare().unwrap().passed);
}

#[test]
fn webp_acceptance_writes_verified_references_and_changes_compare_exactly() {
    let f = Fixture::new();
    f.establish();
    convert(&f, Format::Webp, false).unwrap();
    let original = candidate(&f, &f.cases[0]);
    let mut changed = original.clone();
    changed.put_pixel(3, 4, Rgba([21, 30, 40, 255]));
    changed
        .save(image_path(&f.run, &f.cases[0]).unwrap())
        .unwrap();
    f.seal();
    let review = f.compare().unwrap();
    let finding = review
        .results
        .iter()
        .find(|r| r.case == f.cases[0])
        .unwrap();
    assert_eq!(finding.status, "changed");
    assert_eq!(finding.changed_pixels, Some(1));
    assert_eq!(finding.bounds, Some([3, 4, 4, 5]));
    let shown = Format::Png
        .read(&f.report.join(&f.cases[0]).join("expected.png"))
        .unwrap();
    assert_eq!(
        shown.as_raw(),
        original.as_raw(),
        "reference shown as exact PNG"
    );
    f.accept(&f.cases[..1], false).unwrap();
    let meta = metadata(&f);
    let stored = Format::Webp.read(&webp_path(&f, &f.cases[0])).unwrap();
    assert_eq!(stored.as_raw(), changed.as_raw());
    assert_eq!(
        meta["cases"][&f.cases[0]]["pixel_hash"],
        codec::pixel_hash(&changed)
    );
    assert!(!image_path(&f.base, &f.cases[0]).unwrap().exists());
    assert!(f.compare().unwrap().passed);
    // accept --all into the WebP store.
    f.image(&f.cases[1], 20, 12, [1, 2, 3, 255]);
    f.seal();
    f.compare().unwrap();
    let preview = preview_all(&f.run, &f.report).unwrap();
    assert_eq!(preview.changed, vec![f.cases[1].clone()]);
    accept_all(&f.repo, &f.run, &f.base, &f.report, &preview, false).unwrap();
    assert!(f.compare().unwrap().passed);
    let files: Vec<_> = baseline_state(&f.base).unwrap().into_keys().collect();
    assert!(
        files
            .iter()
            .all(|n| n == "baseline.json" || n.ends_with(".webp")),
        "{files:?}"
    );
}

#[test]
fn lossy_corrupt_or_mismatched_references_fail_closed() {
    for tamper in ["lossy", "other-pixels"] {
        let f = Fixture::new();
        f.establish();
        convert(&f, Format::Webp, false).unwrap();
        let reference = webp_path(&f, &f.cases[0]);
        let bytes = match tamper {
            "lossy" => {
                let mut bytes = fs::read(&reference).unwrap();
                bytes[12..16].copy_from_slice(b"VP8 ");
                bytes
            }
            _ => Format::Webp
                .encode(&RgbaImage::from_pixel(20, 12, Rgba([7, 7, 7, 255])))
                .unwrap(),
        };
        fs::write(&reference, bytes).unwrap();
        // Only the file hash is updated: the recorded fingerprint no longer matches.
        let hash = digest(&reference).unwrap();
        edit_metadata(&f, |m| m["cases"][&f.cases[0]]["image_hash"] = json!(hash));
        let needle = if tamper == "lossy" {
            "lossless"
        } else {
            "recorded fingerprint"
        };
        // Unchanged candidate: the fingerprint path does not decode. It still
        // refuses a non-lossless container; a valid lossless file whose pixels
        // no longer match its recorded fingerprint is only found by decoding
        // (compact/convert, or any comparison that needs the pixels). Manual
        // metadata edits are outside the acceptance protocol.
        if tamper == "lossy" {
            fails(f.compare(), needle);
        } else {
            assert!(
                f.compare().unwrap().passed,
                "fingerprint path skips decoding"
            );
        }
        let state = baseline_state(&f.base).unwrap();
        fails(
            baseline::compact(&f.base, &state["baseline.json"], true),
            needle,
        );
        fails(convert(&f, Format::Png, true), needle);
        // A changed candidate forces the exact decode path in comparison.
        f.image(&f.cases[0], 20, 12, [99, 30, 40, 255]);
        f.seal();
        fails(f.compare(), needle);
    }
}

#[test]
fn conversion_refuses_drift_locks_pending_writes_and_stray_files() {
    let f = Fixture::new();
    f.establish();
    fails(
        baseline::convert(&f.base, Format::Webp, &"0".repeat(64), true),
        "expected hash",
    );
    for (name, needle) in [(".accept.lock", "locked"), ("x.pending", "Unfinished")] {
        fs::write(f.base.join(name), "retained").unwrap();
        fails(convert(&f, Format::Webp, false), needle);
        fs::remove_file(f.base.join(name)).unwrap();
    }
    assert!(f
        .cases
        .iter()
        .all(|c| image_path(&f.base, c).unwrap().exists()));
    convert(&f, Format::Webp, false).unwrap();
    // A stray file of the other format is a store inconsistency, never ignored.
    fs::copy(
        image_path(&f.run, &f.cases[0]).unwrap(),
        image_path(&f.base, &f.cases[0]).unwrap(),
    )
    .unwrap();
    assert!(f
        .compare()
        .unwrap()
        .errors
        .iter()
        .any(|e| e.contains("disagree")));
    fails(convert(&f, Format::Png, true), "Invalid baseline images");
}

#[test]
fn schema_3_fields_are_required_together_and_only_in_schema_3() {
    let f = Fixture::new();
    f.establish();
    edit_metadata(&f, |m| m["image_format"] = json!("webp"));
    fails(Baseline::load(&f.base), "schema 3");
    edit_metadata(&f, |m| {
        m.as_object_mut().unwrap().remove("image_format");
        m["cases"][&f.cases[0]]["pixel_hash"] = json!("a".repeat(64));
    });
    fails(Baseline::load(&f.base), "pixel fingerprint");
    let g = Fixture::new();
    g.establish();
    convert(&g, Format::Webp, false).unwrap();
    edit_metadata(&g, |m| {
        m["cases"][&g.cases[0]]
            .as_object_mut()
            .unwrap()
            .remove("pixel_hash");
    });
    fails(Baseline::load(&g.base), "pixel fingerprint");
}

#[test]
fn png_stores_stay_schema_2_and_new_stores_follow_the_configuration() {
    let f = Fixture::new();
    f.establish();
    let meta = metadata(&f);
    assert_eq!(meta["schema"], 2);
    assert!(meta.get("image_format").is_none());
    for case in &f.cases {
        assert!(meta["cases"][case].get("pixel_hash").is_none());
        assert_eq!(
            digest(&image_path(&f.base, case).unwrap()).unwrap(),
            digest(&image_path(&f.run, case).unwrap()).unwrap(),
            "PNG references keep the exact capture bytes"
        );
    }
    assert_eq!(store_format(&f.base, None).unwrap(), Format::Png);
    assert_eq!(
        store_format(&f.base, Some(Format::Png)).unwrap(),
        Format::Png
    );
    fails(
        store_format(&f.base, Some(Format::Webp)),
        "convert explicitly",
    );
    let g = Fixture::new();
    assert_eq!(
        store_format(&g.base, Some(Format::Webp)).unwrap(),
        Format::Webp
    );
    g.compare().unwrap();
    accept_with_format(
        &g.repo,
        &g.run,
        &g.base,
        &g.report,
        &g.cases,
        false,
        Format::Webp,
    )
    .unwrap();
    assert_eq!(metadata(&g)["image_format"], "webp");
    assert!(g.cases.iter().all(|c| webp_path(&g, c).exists()));
    assert_eq!(store_format(&g.base, None).unwrap(), Format::Webp);
    assert!(g.compare().unwrap().passed);
}

#[test]
fn webp_sequences_keep_png_report_frames_and_accept_whole_journeys() {
    let f = sequences::sequences();
    sequences::establish(&f);
    convert(&f, Format::Webp, false).unwrap();
    assert!(f.compare().unwrap().passed);
    for case in &f.cases {
        let shown = Format::Png
            .read(
                &f.report
                    .join("sequence-expected")
                    .join(format!("{case}.png")),
            )
            .unwrap();
        assert_eq!(shown.as_raw(), candidate(&f, case).as_raw());
    }
    let c = f.cases[0].clone();
    f.image(&c, 2, 3, [99; 4]);
    f.manifest(|m| m["images"][&c] = json!(digest(&f.run.join(format!("{c}.png"))).unwrap()));
    let review = f.compare().unwrap();
    assert_eq!(review.sequences[0].first_difference, Some(0));
    let shown = Format::Png
        .read(&f.report.join("sequence-expected").join(format!("{c}.png")))
        .unwrap();
    assert_ne!(
        shown.as_raw(),
        candidate(&f, &c).as_raw(),
        "old reference frame shown"
    );
    f.accept(&sequences::ids(&f)[..1], false).unwrap();
    let stored = Format::Webp.read(&webp_path(&f, &c)).unwrap();
    assert_eq!(stored.as_raw(), candidate(&f, &c).as_raw());
    assert!(f.compare().unwrap().passed);
}

#[test]
fn cli_conversion_previews_savings_and_rejects_unknown_formats() {
    let f = Fixture::new();
    f.establish();
    let binary = env!("CARGO_BIN_EXE_cargo-gpui-visual");
    let run = |format: &str, dry_run: bool| {
        let hash = baseline_state(&f.base).unwrap()["baseline.json"].clone();
        let mut command = Command::new(binary);
        command
            .args(["convert-baselines", "--json", "--baselines"])
            .arg(&f.base)
            .args(["--format", format, "--expected-hash", &hash]);
        if dry_run {
            command.arg("--dry-run");
        }
        command.output().unwrap()
    };
    assert_ne!(run("jxl", true).status.code(), Some(0));
    let preview = run("webp", true);
    assert_eq!(preview.status.code(), Some(0));
    let reply: Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(reply["data"]["conversion"]["written"], false);
    assert_eq!(reply["data"]["accepted"], json!([]));
    assert!(f
        .cases
        .iter()
        .all(|c| image_path(&f.base, c).unwrap().exists()));
    let written = run("webp", false);
    assert_eq!(written.status.code(), Some(0));
    let reply: Value = serde_json::from_slice(&written.stdout).unwrap();
    assert_eq!(reply["data"]["conversion"]["to"], "webp");
    assert!(f.cases.iter().all(|c| webp_path(&f, c).exists()));
    assert!(f.compare().unwrap().passed);
}
