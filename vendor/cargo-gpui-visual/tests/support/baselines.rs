use super::*;
use cargo_gpui_visual::baseline::{self, Baseline};

fn legacy(f: &Fixture) -> Value {
    let mut value: Value = read_json(&f.base.join("baseline.json")).unwrap();
    let captures = value.as_object_mut().unwrap().remove("captures").unwrap();
    value["schema"] = json!(1);
    for case in value["cases"].as_object_mut().unwrap().values_mut() {
        case["sources"] = captures[case["capture_hash"].as_str().unwrap()]["sources"].clone();
    }
    write_json(&f.base.join("baseline.json"), &value).unwrap();
    value
}

fn metadata(f: &Fixture, edit: impl FnOnce(&mut Value)) {
    let path = f.base.join("baseline.json");
    let mut value: Value = read_json(&path).unwrap();
    edit(&mut value);
    write_json(&path, &value).unwrap();
}

#[test]
fn accepted_frames_store_sources_once_per_capture() {
    let f = sequences::sequences();
    sequences::establish(&f);
    let metadata: Value = read_json(&f.base.join("baseline.json")).unwrap();
    assert_eq!(metadata["schema"], 2);
    let manifest = load_manifest(&f.run).unwrap();
    let capture = digest(&f.run.join("capture.json")).unwrap();
    assert_eq!(metadata["captures"].as_object().unwrap().len(), 1);
    assert_eq!(
        metadata["captures"][&capture]["sources"],
        json!(manifest.sources)
    );
    for case in &f.cases {
        assert_eq!(metadata["cases"][case]["capture_hash"], capture);
        assert!(metadata["cases"][case].get("sources").is_none());
    }
    assert!(f.compare().unwrap().passed);
}

#[test]
fn compact_legacy_provenance_is_lossless_readonly_by_default_and_idempotent() {
    let f = sequences::sequences();
    sequences::establish(&f);
    let old = legacy(&f);
    assert!(f.compare().unwrap().passed); // legacy reads never rewrite
    let before = baseline_state(&f.base).unwrap();
    assert_eq!(
        read_json::<Value>(&f.base.join("baseline.json")).unwrap(),
        old
    );
    // Historical evidence must never be replaced with current source hashes.
    fs::write(f.repo.join("renderer.rs"), "new unreviewed source").unwrap();
    let preview = baseline::compact(&f.base, &before["baseline.json"], true).unwrap();
    assert!(!preview.written);
    assert_eq!(baseline_state(&f.base).unwrap(), before);
    let result = baseline::compact(&f.base, &before["baseline.json"], false).unwrap();
    assert!(result.written);
    assert_eq!(result.new_hash, preview.new_hash);
    let new = Baseline::load(&f.base).unwrap();
    assert_eq!(new.environment, old["environment"]);
    assert_eq!(json!(new.sequences), old["sequences"]);
    for (id, case) in &new.cases {
        assert_eq!(case.image_hash, old["cases"][id]["image_hash"]);
        assert_eq!(case.capture_hash, old["cases"][id]["capture_hash"]);
        assert_eq!(
            json!(new.captures[&case.capture_hash].sources),
            old["cases"][id]["sources"]
        );
    }
    let after = baseline_state(&f.base).unwrap();
    for (name, hash) in before.iter().filter(|(n, _)| n.ends_with(".png")) {
        assert_eq!(after.get(name), Some(hash));
    }
    assert_eq!(before.len(), after.len());
    assert!(
        !baseline::compact(&f.base, &after["baseline.json"], false)
            .unwrap()
            .written
    );
    assert_eq!(after, baseline_state(&f.base).unwrap());
    fails(
        f.accept(&sequences::ids(&f), false),
        "source inventory changed",
    );
}

#[test]
fn compaction_invalidates_existing_acceptance_reports() {
    let f = Fixture::new();
    f.establish();
    legacy(&f);
    f.image(&f.cases[0], 20, 12, [99; 4]);
    f.seal();
    f.compare().unwrap();
    let hash = digest(&f.base.join("baseline.json")).unwrap();
    baseline::compact(&f.base, &hash, false).unwrap();
    fails(f.accept(&f.cases[..1], false), "Baselines changed");
    f.compare().unwrap();
    f.accept(&f.cases[..1], false).unwrap();
    assert!(f.compare().unwrap().passed);
    let b = Baseline::load(&f.base).unwrap();
    assert_eq!(b.captures.len(), 2);
}

#[test]
fn selective_acceptance_retains_unselected_legacy_provenance_and_prunes_only_unused_captures() {
    let f = Fixture::new();
    f.establish();
    let old = legacy(&f);
    for case in &f.cases {
        f.image(case, 20, 12, [99; 4]);
    }
    f.seal();
    f.compare().unwrap();
    f.accept(&f.cases[..1], false).unwrap();
    let b = Baseline::load(&f.base).unwrap();
    assert_eq!(b.captures.len(), 2);
    let unchanged = &b.cases[&f.cases[1]];
    assert_eq!(
        unchanged.image_hash,
        old["cases"][&f.cases[1]]["image_hash"]
    );
    assert_eq!(
        unchanged.capture_hash,
        old["cases"][&f.cases[1]]["capture_hash"]
    );
    assert_eq!(
        json!(b.captures[&unchanged.capture_hash].sources),
        old["cases"][&f.cases[1]]["sources"]
    );
    f.compare().unwrap();
    f.accept(&f.cases[1..], false).unwrap();
    assert_eq!(Baseline::load(&f.base).unwrap().captures.len(), 1);
    assert!(f.compare().unwrap().passed);
}

#[test]
fn malformed_or_ambiguous_provenance_never_compacts_or_compares() {
    for mutation in 0..9 {
        let f = Fixture::new();
        f.establish();
        if mutation == 0 {
            legacy(&f);
        }
        metadata(&f, |v| {
            let hash = v["cases"][&f.cases[0]]["capture_hash"]
                .as_str()
                .unwrap()
                .to_owned();
            match mutation {
                0 => v["cases"][&f.cases[0]]["sources"]["renderer.rs"] = json!("0".repeat(64)),
                1 => {
                    v["captures"].as_object_mut().unwrap().remove(&hash);
                }
                2 => v["captures"]["0".repeat(64)] = v["captures"][&hash].clone(),
                3 => v["cases"][&f.cases[0]]["sources"] = json!({}),
                4 => v["schema"] = json!(3),
                5 => v["captures"][&hash]["sources"] = json!({}),
                6 => v["captures"][&hash]["sources"] = json!({"../escape.rs":"0".repeat(64)}),
                7 => v["captures"][&hash]["sources"]["renderer.rs"] = json!("invalid"),
                _ => v["unknown_history"] = json!("must not silently discard"),
            }
        });
        let before = baseline_state(&f.base).unwrap();
        assert!(
            baseline::compact(&f.base, &before["baseline.json"], false).is_err(),
            "mutation {mutation}"
        );
        assert!(f.compare().is_err(), "mutation {mutation}");
        assert_eq!(baseline_state(&f.base).unwrap(), before);
    }
}

#[test]
fn compact_refuses_hash_drift_locks_pending_writes_and_invalid_images() {
    for mutation in 0..6 {
        let f = Fixture::new();
        f.establish();
        legacy(&f);
        let hash = digest(&f.base.join("baseline.json")).unwrap();
        match mutation {
            0 => metadata(&f, |v| v["environment"]["os"] = json!("changed")),
            1 => fs::write(f.base.join(".accept.lock"), "foreign owner").unwrap(),
            2 => fs::write(f.base.join("baseline.pending"), "interrupted").unwrap(),
            3 => fs::write(image_path(&f.base, &f.cases[0]).unwrap(), "bad").unwrap(),
            4 => {
                fs::copy(
                    image_path(&f.base, &f.cases[0]).unwrap(),
                    f.base.join("extra--light.png"),
                )
                .unwrap();
            }
            _ => fs::write(f.base.join("frame.pending"), "interrupted").unwrap(),
        }
        let before = baseline_state(&f.base).unwrap();
        for dry in [true, false] {
            assert!(
                baseline::compact(&f.base, &hash, dry).is_err(),
                "mutation {mutation}"
            );
            assert_eq!(baseline_state(&f.base).unwrap(), before);
        }
    }
}

#[test]
fn historical_submodule_git_ids_survive_compaction() {
    let f = Fixture::new();
    f.establish();
    legacy(&f);
    metadata(&f, |v| {
        for case in v["cases"].as_object_mut().unwrap().values_mut() {
            case["sources"]["vendor/sdk@git"] = json!("1".repeat(40));
            case["sources"]["vendor/sha256-sdk@git"] = json!("2".repeat(64));
        }
    });
    let hash = digest(&f.base.join("baseline.json")).unwrap();
    baseline::compact(&f.base, &hash, false).unwrap();
    let b = Baseline::load(&f.base).unwrap();
    for capture in b.captures.values() {
        assert_eq!(capture.sources["vendor/sdk@git"], "1".repeat(40));
        assert_eq!(capture.sources["vendor/sha256-sdk@git"], "2".repeat(64));
    }
    metadata(&f, |v| {
        for capture in v["captures"].as_object_mut().unwrap().values_mut() {
            capture["sources"]["ordinary.rs"] = json!("1".repeat(40));
        }
    });
    assert!(Baseline::load(&f.base).is_err());
}

#[test]
fn provenance_growth_tracks_captures_not_frame_count() {
    let mut f = Fixture::new();
    for i in 0..100 {
        fs::write(f.repo.join(format!("source-{i}.rs")), "source").unwrap();
    }
    for i in 0..40 {
        for theme in ["light", "dark"] {
            let case = format!("state-{i}--{theme}");
            f.image(&case, 1, 1, [1; 4]);
            f.cases.push(case);
        }
    }
    f.seal();
    f.establish();
    let compact_bytes = fs::metadata(f.base.join("baseline.json")).unwrap().len();
    legacy(&f);
    let legacy_bytes = fs::metadata(f.base.join("baseline.json")).unwrap().len();
    assert!(
        compact_bytes * 10 < legacy_bytes,
        "{compact_bytes} vs {legacy_bytes}"
    );
}
