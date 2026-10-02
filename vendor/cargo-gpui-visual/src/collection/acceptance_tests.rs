use super::*;
use crate::{
    collection::{self, Suite},
    store::*,
};
use image::{Rgba, RgbaImage};
use serde_json::{json, Value};
use std::process::Command;

#[test]
fn interrupted_multi_authority_writes_retain_honest_attempt_evidence() {
    for (partial_current_suite, acknowledged_withheld) in [
        (false, vec![]),
        (true, vec![]),
        (false, vec!["second:remaining".to_owned()]),
        (true, vec!["second:remaining".to_owned()]),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let mut suites = Vec::new();
        for id in ["first", "second"] {
            let repo = root.join(id);
            fs::create_dir(&repo).unwrap();
            assert!(Command::new("git")
                .args(["init", "-q"])
                .arg(&repo)
                .status()
                .unwrap()
                .success());
            fs::write(repo.join("renderer.rs"), "fixture").unwrap();
            write_json(&repo.join("catalog.json"),&json!({"schema":1,"snapshots":[{"base":"panel","label":"Panel","group":"Panels","state":"required"}]})).unwrap();
            let run = repo.join("run");
            fs::create_dir(&run).unwrap();
            let mut images = Hashes::new();
            for theme in ["light", "dark"] {
                let case = format!("panel--{theme}");
                let path = image_path(&run, &case).unwrap();
                RgbaImage::from_pixel(2, 2, Rgba([10, 20, 30, 255]))
                    .save(&path)
                    .unwrap();
                images.insert(case, digest(&path).unwrap());
            }
            let policy = Policy {
                scopes: vec!["renderer.rs".into(), "catalog.json".into()],
                suffixes: vec![".rs".into(), ".json".into()],
                artifacts: vec![],
            };
            write_json(
                &run.join("capture.json"),
                &Manifest {
                    schema: 1,
                    cases: images.keys().cloned().collect(),
                    images,
                    environment: json!({"layer":"gpui","fixture":id}),
                    sources: source_hashes(&repo, &policy, None).unwrap(),
                    source_policy: policy,
                    sequences: vec![],
                    selection: None,
                },
            )
            .unwrap();
            suites.push(Suite {
                id: id.into(),
                label: id.into(),
                run,
                baselines: repo.join("baselines"),
                repo,
                catalog: "catalog.json".into(),
            });
        }
        let report = root.join("report");
        let collection = collection::compare(&suites, &report).unwrap();
        let mut prepared = Vec::new();
        let mut locks = Vec::new();
        for s in &collection.suites {
            let suite = &s.suite;
            let plan = accept::prepare(
                &suite.repo,
                &suite.run,
                &suite.baselines,
                &["panel--light".into()],
                false,
                crate::codec::Format::Png,
                load_manifest(&suite.run).unwrap(),
                s.review.clone().unwrap(),
            )
            .unwrap();
            locks.push(BaselineLock::acquire(&suite.baselines).unwrap());
            prepared.push((suite, plan));
        }
        let selected = vec!["first:panel--light".into(), "second:panel--light".into()];
        let result = execute(
            &report,
            &selected,
            &[],
            &acknowledged_withheld,
            &prepared,
            |index| {
                if index == 1 {
                    if partial_current_suite {
                        // A competing/interrupted output appears after all preflight.
                        fs::write(suites[1].baselines.join("baseline.pending"), "retained")?;
                    } else {
                        anyhow::bail!("injected write-boundary interruption");
                    }
                }
                Ok(())
            },
        );
        assert!(result.is_err());
        let journal: Value = read_json(&report.join("acceptance.json")).unwrap();
        assert_eq!(journal["state"], "failed");
        if !acknowledged_withheld.is_empty() {
            assert_eq!(journal["schema"], 2);
            assert_eq!(
                journal["acknowledged_withheld"],
                json!(acknowledged_withheld)
            );
        } else {
            assert_eq!(journal["schema"], 1);
            assert!(journal.get("acknowledged_withheld").is_none());
        }
        assert_eq!(journal["completed"], json!(["first"]));
        assert!(journal["error"].as_str().unwrap().contains("second"));
        assert!(suites[0].baselines.join("baseline.json").exists());
        assert_eq!(
            suites[1].baselines.join("panel--light.png").exists(),
            partial_current_suite
        );
        assert!(!suites[1].baselines.join("baseline.json").exists());
        assert!(collection::accept(&report, &selected, &[])
            .unwrap_err()
            .to_string()
            .contains("already has an acceptance attempt"));
        if partial_current_suite {
            let next = collection::compare(&suites, &root.join("next")).unwrap();
            assert!(!next.suites[1].review.as_ref().unwrap().errors.is_empty());
        }
        drop(locks);
    }
}
