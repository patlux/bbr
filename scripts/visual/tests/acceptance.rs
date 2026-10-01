//! The actual consumer CLI routes only reviewed explicit collection selections.
use bbr_visual::process;
use cargo_gpui_visual::{
    collection::{self, Suite},
    store::{self, Manifest, Policy},
};
use serde_json::json;
use std::{fs, path::Path, process::Command, sync::atomic::AtomicBool};

fn fixture(root: &Path) -> Vec<Suite> {
    process::output(
        Command::new("git").args(["init", "-q"]).arg(root),
        &AtomicBool::new(false),
    )
    .unwrap();
    fs::write(root.join("source.rs"), "// deterministic fixture\n").unwrap();
    let catalog = json!({"schema":1,"snapshots":[{"base":"fixture","label":"Fixture","group":"Fixtures","sequence":null,"moment":null,"page":null,"state":"required","reason":null}]});
    store::write_json(&root.join("catalog.json"), &catalog).unwrap();
    let policy = Policy {
        scopes: vec!["source.rs".into(), "catalog.json".into()],
        suffixes: vec![".rs".into(), ".json".into()],
        artifacts: vec![],
    };
    let sources = store::source_hashes(root, &policy, None).unwrap();
    ["gallery", "browser"]
        .into_iter()
        .map(|id| {
            let run = root.join(format!("{id}-run"));
            fs::create_dir(&run).unwrap();
            let cases = vec!["fixture--light".to_owned(), "fixture--dark".to_owned()];
            for case in &cases {
                image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
                    .save(run.join(format!("{case}.png")))
                    .unwrap();
            }
            let images = cases
                .iter()
                .map(|case| {
                    (
                        case.clone(),
                        store::digest(&run.join(format!("{case}.png"))).unwrap(),
                    )
                })
                .collect();
            store::write_json(
                &run.join("capture.json"),
                &Manifest {
                    schema: 1,
                    cases,
                    images,
                    sources: sources.clone(),
                    source_policy: policy.clone(),
                    environment: json!({"layer":"gpui","suite":id}),
                    selection: None,
                    sequences: Vec::new(),
                },
            )
            .unwrap();
            Suite {
                id: id.into(),
                label: id.into(),
                repo: root.into(),
                run,
                baselines: root.join(format!("{id}-baselines")),
                catalog: "catalog.json".into(),
            }
        })
        .collect()
}
fn cli(report: &Path, cases: &[&str]) -> bool {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bbr-visual"));
    cmd.arg("update").arg(report).args(cases);
    process::output(&mut cmd, &AtomicBool::new(false)).is_ok()
}
#[test]
fn real_cli_selects_only_named_cases_and_rejects_replay() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let suites = fixture(&root);
    let report = root.join("review");
    collection::compare(&suites, &report).unwrap();
    for invalid in [
        vec![],
        vec!["fixture--light"],
        vec!["gallery:*"],
        vec!["gallery:fixture--light", "missing:fixture--light"],
    ] {
        assert!(!cli(&report, &invalid));
        for suite in &suites {
            assert!(store::baseline_state(&suite.baselines).unwrap().is_empty());
        }
    }
    assert!(cli(
        &report,
        &["gallery:fixture--light", "browser:fixture--dark"]
    ));
    assert!(suites[0].baselines.join("fixture--light.png").is_file());
    assert!(!suites[0].baselines.join("fixture--dark.png").exists());
    assert!(suites[1].baselines.join("fixture--dark.png").is_file());
    assert!(!cli(&report, &["gallery:fixture--dark"]));
    assert!(!suites[0].baselines.join("fixture--dark.png").exists());
}
#[test]
fn changed_report_or_source_never_writes_any_suite() {
    for mutation in ["index.html", "source.rs"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let suites = fixture(&root);
        let report = root.join("review");
        collection::compare(&suites, &report).unwrap();
        fs::write(
            if mutation == "source.rs" {
                root.join(mutation)
            } else {
                report.join(mutation)
            },
            "changed",
        )
        .unwrap();
        assert!(!cli(
            &report,
            &["browser:fixture--light", "gallery:fixture--light"]
        ));
        for suite in &suites {
            assert!(store::baseline_state(&suite.baselines).unwrap().is_empty());
        }
        assert!(!report.join("acceptance.json").exists());
    }
}
