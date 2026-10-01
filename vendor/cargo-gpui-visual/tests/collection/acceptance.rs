use super::Fixture;
use cargo_gpui_visual::{collection, store::*};
use serde_json::{json, Value};
use std::{fs, process::Command};

const CASE: &str = "panel-page-one--light";
fn selection() -> Vec<String> {
    vec![format!("gallery:{CASE}"), format!("browser:{CASE}")]
}
fn no_references(f: &Fixture) {
    for s in &f.suites {
        assert!(baseline_state(&s.baselines).unwrap().is_empty());
    }
}
#[test]
fn exact_selection_routes_to_separate_authorities_and_cannot_replay() {
    let f = Fixture::new();
    f.compare().unwrap();
    collection::accept(&f.report, &selection(), &[]).unwrap();
    for s in &f.suites {
        let state = baseline_state(&s.baselines).unwrap();
        assert_eq!(state.len(), 2);
        assert_eq!(
            digest(&image_path(&s.baselines, CASE).unwrap()).unwrap(),
            digest(&image_path(&s.run, CASE).unwrap()).unwrap()
        );
        let m: Value = read_json(&s.baselines.join("baseline.json")).unwrap();
        assert_eq!(m["environment"]["os"], s.id);
    }
    let attempt: Value = read_json(&f.report.join("acceptance.json")).unwrap();
    assert_eq!(attempt["state"], "complete");
    assert_eq!(attempt["completed"], json!(["browser", "gallery"]));
    assert!(collection::accept(&f.report, &["gallery:panel-page-two--dark".into()], &[]).is_err());
    let next = f.root.join("next");
    collection::compare(&f.suites, &next).unwrap();
    collection::accept(&next, &["gallery:panel-page-two--dark".into()], &[]).unwrap();
}
#[test]
fn malformed_duplicate_unknown_and_unqualified_selections_never_write() {
    let f = Fixture::new();
    f.compare().unwrap();
    for cases in [
        vec![],
        vec![CASE.into()],
        vec!["gallery:*".into()],
        vec!["*:panel-page-one--light".into()],
        vec!["gallery:unknown--light".into()],
        vec![format!("gallery:{CASE}"), format!("gallery:{CASE}")],
        vec![
            format!("browser:{CASE}"),
            "unknown:panel-page-one--light".into(),
        ],
    ] {
        assert!(
            collection::accept(&f.report, &cases, &[]).is_err(),
            "{cases:?}"
        );
        no_references(&f);
        assert!(!f.report.join("acceptance.json").exists());
    }
}
#[test]
fn changed_review_assets_inventory_and_authorities_fail_closed() {
    for path in [
        "index.html",
        "collection.json",
        "collection-seal.json",
        "suites/gallery/panel-page-one--light/actual.png",
        "extra.txt",
    ] {
        let f = Fixture::new();
        f.compare().unwrap();
        fs::write(f.report.join(path), "changed").unwrap();
        assert!(
            collection::accept(&f.report, &selection(), &[]).is_err(),
            "{path}"
        );
        no_references(&f);
    }
}
#[test]
fn later_selected_invalid_suite_cannot_partially_accept_the_first() {
    for mutation in [
        "source",
        "catalog",
        "capture",
        "image",
        "baseline",
        "baseline-lock",
        "report-lock",
        "suite-lock",
    ] {
        let f = Fixture::new();
        f.compare().unwrap();
        let s = &f.suites[0]; // gallery sorts after browser in the acceptance order
        match mutation {
            "source" => fs::write(s.repo.join("renderer.rs"), "changed").unwrap(),
            "catalog" => f.catalog(0, |v| v["snapshots"][0]["label"] = json!("new label")),
            "capture" => fs::write(s.run.join("capture.json"), "{}").unwrap(),
            "image" => fs::write(image_path(&s.run, CASE).unwrap(), "bad").unwrap(),
            "baseline" => {
                fs::create_dir(&s.baselines).unwrap();
                fs::write(s.baselines.join("baseline.json"), "{}").unwrap();
            }
            "baseline-lock" => {
                fs::create_dir(&s.baselines).unwrap();
                fs::write(s.baselines.join(".accept.lock"), "foreign owner").unwrap();
            }
            "suite-lock" => fs::write(
                f.report.join("suites/gallery/.review.lock"),
                "foreign owner",
            )
            .unwrap(),
            _ => fs::write(f.report.join(".review.lock"), "foreign owner").unwrap(),
        }
        let before: Vec<_> = f
            .suites
            .iter()
            .map(|s| baseline_state(&s.baselines).unwrap())
            .collect();
        assert!(
            collection::accept(&f.report, &selection(), &[]).is_err(),
            "{mutation}"
        );
        assert!(!f.report.join("acceptance.json").exists());
        for (s, before) in f.suites.iter().zip(before) {
            assert_eq!(baseline_state(&s.baselines).unwrap(), before);
        }
        if mutation == "baseline-lock" {
            assert_eq!(
                fs::read_to_string(s.baselines.join(".accept.lock")).unwrap(),
                "foreign owner"
            );
        }
    }
}
#[test]
fn invalid_unselected_suite_stays_unaccepted_but_does_not_block_valid_selection() {
    let f = Fixture::new();
    f.catalog(0,|v|v["snapshots"].as_array_mut().unwrap().push(json!({"base":"native","label":"Native","group":"Native","state":"withheld","reason":"not authorized"})));
    f.seal(0);
    f.compare().unwrap();
    assert!(collection::accept(&f.report, &selection(), &[]).is_err());
    no_references(&f);
    collection::accept(&f.report, &[format!("browser:{CASE}")], &[]).unwrap();
    assert!(!f.suites[0].baselines.exists());
}
#[test]
fn environment_adoption_is_exact_per_suite_and_never_implicit() {
    let f = Fixture::new();
    f.compare().unwrap();
    let all: Vec<_> = f
        .suites
        .iter()
        .flat_map(|s| {
            load_manifest(&s.run)
                .unwrap()
                .cases
                .into_iter()
                .map(|case| format!("{}:{case}", s.id))
        })
        .collect();
    collection::accept(&f.report, &all, &[]).unwrap();
    let before = baseline_state(&f.suites[1].baselines).unwrap();
    let path = f.suites[0].repo.join("environment.json");
    let mut environment: Value = read_json(&path).unwrap();
    environment["os"] = json!("next-runtime");
    write_json(&path, &environment).unwrap();
    f.seal(0);
    let next = f.root.join("adoption");
    collection::compare(&f.suites, &next).unwrap();
    for adoption in [
        vec![],
        vec!["*".into()],
        vec!["browser".into()],
        vec!["gallery".into(), "gallery".into()],
    ] {
        assert!(collection::accept(&next, &[], &adoption).is_err());
    }
    collection::accept(&next, &[], &["gallery".into()]).unwrap();
    assert_eq!(before, baseline_state(&f.suites[1].baselines).unwrap());
    let metadata: Value = read_json(&f.suites[0].baselines.join("baseline.json")).unwrap();
    assert_eq!(metadata["environment"]["os"], "next-runtime");
}
#[test]
fn unfinished_baseline_write_is_detected_before_any_suite_mutation() {
    let f = Fixture::new();
    f.compare().unwrap();
    let blocked = &f.suites[0].baselines;
    fs::create_dir(blocked).unwrap();
    // Simulate an interrupted per-file write at the later authority. No override
    // or fault hook in production; create_new must refuse this retained evidence.
    fs::write(
        blocked.join(format!("{CASE}.pending")),
        "earlier write evidence",
    )
    .unwrap();
    assert!(collection::accept(&f.report, &selection(), &[]).is_err());
    assert!(!f.report.join("acceptance.json").exists());
    no_references(&f);
    assert!(collection::accept(&f.report, &selection(), &[]).is_err());
    assert_eq!(
        fs::read_to_string(blocked.join(format!("{CASE}.pending"))).unwrap(),
        "earlier write evidence"
    );
}
#[test]
fn retained_interrupted_attempt_always_prevents_replay() {
    for state in ["started", "failed", "complete"] {
        let f = Fixture::new();
        f.compare().unwrap();
        write_json(
            &f.report.join("acceptance.json"),
            &json!({"schema":1,"state":state,"completed":[]}),
        )
        .unwrap();
        assert!(collection::accept(&f.report, &selection(), &[]).is_err());
        no_references(&f);
    }
}

#[test]
fn cli_accepts_only_the_explicit_collection_not_a_new_input_definition() {
    let f = Fixture::new();
    f.compare().unwrap();
    let invoke = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"))
            .arg("accept-collection")
            .arg("--report")
            .arg(&f.report)
            .args(args)
            .output()
            .unwrap()
    };
    assert_eq!(invoke(&[]).status.code(), Some(2));
    assert_eq!(
        invoke(&[
            "--case",
            "gallery:panel-page-one--light",
            "--adopt-environment"
        ])
        .status
        .code(),
        Some(2)
    );
    assert_eq!(
        invoke(&[
            "--collection",
            "another.json",
            "--case",
            "gallery:panel-page-one--light"
        ])
        .status
        .code(),
        Some(2)
    );
    let result = invoke(&["--case", "gallery:panel-page-one--light"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!f.suites[1].baselines.exists());
}
#[test]
fn collection_acceptance_writes_each_suite_store_in_its_declared_format() {
    use cargo_gpui_visual::{baseline, codec::Format};
    let f = Fixture::new();
    f.compare().unwrap();
    collection::accept(&f.report, &selection(), &[]).unwrap();
    // The native (layer "ios") suite converts; the host suite stays PNG.
    let native = f.suites.iter().find(|s| s.id == "browser").unwrap();
    let host = f.suites.iter().find(|s| s.id == "gallery").unwrap();
    let hash = baseline_state(&native.baselines).unwrap()["baseline.json"].clone();
    baseline::convert(&native.baselines, Format::Webp, &hash, false).unwrap();
    let next = f.root.join("next");
    collection::compare(&f.suites, &next).unwrap();
    let case = "panel-page-two--dark";
    collection::accept(
        &next,
        &[format!("gallery:{case}"), format!("browser:{case}")],
        &[],
    )
    .unwrap();
    let webp = reference_path(&native.baselines, case, Format::Webp).unwrap();
    assert!(!image_path(&native.baselines, case).unwrap().exists());
    assert_eq!(
        Format::Webp.read(&webp).unwrap().as_raw(),
        Format::Png
            .read(&image_path(&native.run, case).unwrap())
            .unwrap()
            .as_raw()
    );
    assert_eq!(
        digest(&image_path(&host.baselines, case).unwrap()).unwrap(),
        digest(&image_path(&host.run, case).unwrap()).unwrap(),
        "PNG suite store keeps exact capture bytes"
    );
    assert!(!reference_path(&host.baselines, case, Format::Webp)
        .unwrap()
        .exists());
}
