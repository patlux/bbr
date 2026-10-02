//! Acknowledging missing future coverage never turns it into a pass.
use super::Fixture;
use cargo_gpui_visual::{collection, store::*};
use serde_json::{json, Value};
use std::{fs, process::Command};

const CASE: &str = "panel-page-one--light";
const ACK: &str = "gallery:remaining";

fn fixture() -> Fixture {
    let f = Fixture::new();
    f.catalog(0, |v| {
        v["snapshots"].as_array_mut().unwrap().push(json!({
            "base":"remaining", "label":"Remaining views", "group":"Coverage",
            "state":"withheld", "reason":"Not implemented; coverage remains incomplete"
        }));
    });
    f.seal(0);
    f
}
fn invoke(
    f: &Fixture,
    cases: &[&str],
    acknowledgements: &[&str],
    adopt: &[&str],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"));
    command
        .arg("accept-collection")
        .arg("--report")
        .arg(&f.report)
        .arg("--json");
    for case in cases {
        command.args(["--case", case]);
    }
    for ack in acknowledgements {
        command.args(["--acknowledge-withheld", ack]);
    }
    for suite in adopt {
        command.args(["--adopt-suite", suite]);
    }
    command.output().unwrap()
}
fn all_cases(f: &Fixture) -> Vec<String> {
    load_manifest(&f.suites[0].run)
        .unwrap()
        .cases
        .into_iter()
        .map(|c| format!("gallery:{c}"))
        .collect()
}
fn rejected(f: &Fixture, cases: &[&str], acknowledgements: &[&str], adopt: &[&str]) {
    let before: Vec<_> = f
        .suites
        .iter()
        .map(|s| baseline_state(&s.baselines).unwrap())
        .collect();
    let result = invoke(f, cases, acknowledgements, adopt);
    assert_eq!(
        result.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    assert!(!f.report.join("acceptance.json").exists());
    for (s, expected) in f.suites.iter().zip(before) {
        assert_eq!(baseline_state(&s.baselines).unwrap(), expected);
    }
}
#[test]
fn acknowledged_images_are_accepted_but_fresh_coverage_still_fails() {
    let f = fixture();
    f.compare().unwrap();
    let selected = all_cases(&f);
    assert!(
        collection::accept(&f.report, &selected, &[]).is_err(),
        "default remains strict"
    );
    let html_hash = digest(&f.report.join("index.html")).unwrap();
    let seal_hash = digest(&f.report.join("collection-seal.json")).unwrap();
    let result = invoke(
        &f,
        &selected.iter().map(String::as_str).collect::<Vec<_>>(),
        &[ACK],
        &[],
    );
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let response: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(response["data"]["acknowledged_withheld"], json!([ACK]));
    let journal: Value = read_json(&f.report.join("acceptance.json")).unwrap();
    assert_eq!(journal["acknowledged_withheld"], json!([ACK]));
    assert_eq!(journal["state"], "complete");
    assert_eq!(journal["selected"], json!(selected));
    assert_eq!(html_hash, digest(&f.report.join("index.html")).unwrap());
    assert_eq!(
        seal_hash,
        digest(&f.report.join("collection-seal.json")).unwrap()
    );
    assert!(!f.suites[1].baselines.exists());
    for id in load_manifest(&f.suites[0].run).unwrap().cases {
        assert_eq!(
            digest(&image_path(&f.suites[0].run, &id).unwrap()).unwrap(),
            digest(&image_path(&f.suites[0].baselines, &id).unwrap()).unwrap()
        );
    }
    assert!(
        !invoke(&f, &["gallery:panel-page-one--light"], &[ACK], &[])
            .status
            .success(),
        "no replay"
    );
    let next = collection::compare(&f.suites, &f.root.join("next")).unwrap();
    assert!(!next.passed);
    assert!(!next.suites[0].passed);
    assert!(
        next.suites[0].review.as_ref().unwrap().passed,
        "all images match exactly"
    );
    assert_eq!(next.suites[0].errors.len(), 1);
    assert!(next.suites[0].errors[0].contains("Withheld: remaining"));
}
#[test]
fn acknowledgement_must_be_exact_complete_and_selected_without_granting_selection() {
    let f = fixture();
    f.catalog(0, |v| v["snapshots"].as_array_mut().unwrap().push(json!({
        "base":"native", "label":"Native", "group":"Coverage", "state":"unsupported", "reason":"outside renderer"
    })));
    f.seal(0);
    f.compare().unwrap();
    for acks in [
        vec![],
        vec![ACK, ACK],
        vec!["remaining"],
        vec!["gallery:*"],
        vec!["*:remaining"],
        vec!["gallery:remaining--light"],
        vec!["gallery:unknown"],
        vec!["gallery:panel-page-one"],
        vec!["gallery:native"],
        vec!["browser:remaining"],
        vec![ACK, "browser:remaining"],
    ] {
        rejected(&f, &["gallery:panel-page-one--light"], &acks, &[]);
    }
    rejected(&f, &[], &[ACK], &[]);
    rejected(&f, &["browser:panel-page-one--light"], &[ACK], &[]);
    rejected(&f, &["gallery:remaining--light"], &[ACK], &[]);
    rejected(
        &f,
        &[
            "gallery:panel-page-one--light",
            "gallery:panel-page-one--light",
        ],
        &[ACK],
        &[],
    );
    let g = fixture();
    g.catalog(0, |v| v["snapshots"].as_array_mut().unwrap().push(json!({
        "base":"another", "label":"Another", "group":"Coverage", "state":"withheld", "reason":"also absent"
    })));
    g.seal(0);
    g.compare().unwrap();
    rejected(&g, &["gallery:panel-page-one--light"], &[ACK], &[]);
    assert!(invoke(
        &g,
        &["gallery:panel-page-one--light"],
        &[ACK, "gallery:another"],
        &[]
    )
    .status
    .success());
}
#[test]
fn acknowledged_withholding_never_allows_invalid_capture_coverage_or_comparison() {
    for mutation in ["missing", "unexpected", "non-required", "bad-reference"] {
        let f = fixture();
        let s = &f.suites[0];
        match mutation {
            "missing" => {
                fs::remove_file(image_path(&s.run, "panel-page-two--dark").unwrap()).unwrap()
            }
            "unexpected" | "non-required" => {
                let case = if mutation == "unexpected" {
                    "extra--light"
                } else {
                    "remaining--light"
                };
                fs::copy(
                    image_path(&s.run, CASE).unwrap(),
                    image_path(&s.run, case).unwrap(),
                )
                .unwrap();
            }
            _ => {
                fs::create_dir(&s.baselines).unwrap();
                fs::write(s.baselines.join("baseline.json"), "{}").unwrap();
            }
        }
        f.seal(0);
        f.compare().unwrap();
        rejected(
            &f,
            &[
                "browser:panel-page-one--light",
                "gallery:panel-page-one--light",
            ],
            &[ACK],
            &[],
        );
    }
}
#[test]
fn acknowledged_withholding_keeps_all_authority_and_lock_checks() {
    for mutation in [
        "source",
        "catalog",
        "capture",
        "image",
        "report",
        "asset",
        "extra-report",
        "baseline",
        "report-lock",
        "suite-lock",
        "baseline-lock",
        "pending",
    ] {
        let f = fixture();
        f.compare().unwrap();
        let s = &f.suites[0];
        match mutation {
            "source" => fs::write(s.repo.join("renderer.rs"), "changed").unwrap(),
            "catalog" => f.catalog(0, |v| v["snapshots"][2]["reason"] = json!("changed")),
            "capture" => fs::write(s.run.join("capture.json"), "{}").unwrap(),
            "image" => fs::write(image_path(&s.run, CASE).unwrap(), "invalid").unwrap(),
            "report" => fs::write(f.report.join("index.html"), "changed").unwrap(),
            "asset" => fs::write(
                f.report
                    .join("suites/gallery/panel-page-one--light/actual.png"),
                "changed",
            )
            .unwrap(),
            "extra-report" => fs::write(f.report.join("extra.txt"), "changed").unwrap(),
            "report-lock" => fs::write(f.report.join(".review.lock"), "owned").unwrap(),
            "suite-lock" => {
                fs::write(f.report.join("suites/gallery/.review.lock"), "owned").unwrap()
            }
            _ => {
                fs::create_dir(&s.baselines).unwrap();
                let file = match mutation {
                    "baseline" => "baseline.json",
                    "pending" => "panel-page-one--light.pending",
                    _ => ".accept.lock",
                };
                fs::write(s.baselines.join(file), "{}").unwrap();
            }
        }
        rejected(
            &f,
            &[
                "browser:panel-page-one--light",
                "gallery:panel-page-one--light",
            ],
            &[ACK],
            &[],
        );
    }
}
#[test]
fn withheld_acknowledgement_never_adopts_environment_implicitly() {
    let f = fixture();
    f.compare().unwrap();
    let cases = all_cases(&f);
    let ids: Vec<_> = cases.iter().map(String::as_str).collect();
    assert!(invoke(&f, &ids, &[ACK], &[]).status.success());
    let environment = f.suites[0].repo.join("environment.json");
    write_json(
        &environment,
        &json!({"layer":"gpui", "os":"changed", "scale":2}),
    )
    .unwrap();
    f.seal(0);
    let f = Fixture {
        report: f.root.join("environment-report"),
        ..f
    };
    f.compare().unwrap();
    rejected(&f, &ids, &[ACK], &[]);
    rejected(&f, &ids, &[ACK], &["browser"]);
    assert!(
        invoke(&f, &[], &[ACK], &["gallery"]).status.success(),
        "explicit adoption of byte-identical unselected references"
    );
    let journal: Value = read_json(&f.report.join("acceptance.json")).unwrap();
    assert_eq!(journal["adopt_suites"], json!(["gallery"]));
}
