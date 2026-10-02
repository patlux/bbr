//! Portable selection/plan contracts; these fixtures never invoke GPUI or Tart.
use bbr_visual::{gallery, gallery_selection::Schedule};
use cargo_gpui_visual::{config::Config, selection::Selection, store};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::Command};

struct Fixture {
    _temp: tempfile::TempDir,
    repo: PathBuf,
    config: Config,
    sources: store::Hashes,
    selection: Selection,
}
impl Fixture {
    fn new(ids: &[&str]) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().canonicalize().unwrap();
        assert!(Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .status()
            .unwrap()
            .success());
        // Deliberately nonnumeric authored order and unequal Light/Dark children.
        let plan = json!({"schema":1,"cases":[
            {"base":"gallery-44-state-00-standard-normal","label":"Sidebar","state":"default","group":"Composition","ranges":[[0,2],[2,3]],"dark_ranges":[[0,1],[1,2]]},
            {"base":"gallery-00-state-00-standard-normal","label":"Tokens","state":"default","group":"Foundations","ranges":[[0,1]]}
        ]});
        store::write_json(&repo.join("plan.json"), &plan).unwrap();
        store::write_json(&repo.join("stories.json"), &json!({"schema":1,"stories":[
            {"id":"gallery-44","label":"Sidebar","cases":["gallery-44-state-00-standard-normal-page-00","gallery-44-state-00-standard-normal-page-01"]},
            {"id":"gallery-00","label":"Tokens","cases":["gallery-00-state-00-standard-normal-page-00"]}
        ]})).unwrap();
        let config = Config::from_toml("schema=1\ncapture=['never-executed']\nenvironment_command=['never-probed']\nbaselines='refs'\nreport='review'\nruns='runs'\nstory_catalog='stories.json'\n[source_policy]\nscopes=['stories.json','plan.json']\nsuffixes=['.json']\n").unwrap();
        let selection = Selection::resolve(
            &repo,
            &config,
            &ids.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
        .unwrap();
        let sources = store::source_hashes(&repo, &config.source_policy, None).unwrap();
        Self {
            _temp: temp,
            repo,
            config,
            sources,
            selection,
        }
    }
    fn schedule(&self) -> anyhow::Result<Schedule> {
        Schedule::resolve(&self.repo, &self.sources, "plan.json", &self.selection)
    }
    fn mutate_plan(&mut self, change: impl FnOnce(&mut Value)) {
        let path = self.repo.join("plan.json");
        let mut plan: Value = store::read_json(&path).unwrap();
        change(&mut plan);
        store::write_json(&path, &plan).unwrap();
        self.sources
            .insert("plan.json".into(), store::digest(&path).unwrap());
    }
    fn mutate_selection(&mut self, change: impl FnOnce(&mut Value)) {
        let path = self.repo.join("stories.json");
        let mut catalog: Value = store::read_json(&path).unwrap();
        change(&mut catalog);
        store::write_json(&path, &catalog).unwrap();
        self.sources
            .insert("stories.json".into(), store::digest(&path).unwrap());
        self.selection =
            Selection::resolve(&self.repo, &self.config, &["gallery-44".into()]).unwrap();
    }
}

#[test]
fn selection_is_exact_ordered_complete_and_does_not_schedule_unselected_stories() {
    let f = Fixture::new(&["gallery-00", "gallery-44"]);
    let schedule = f.schedule().unwrap();
    assert_eq!(schedule.story_ids(), &[44, 0]);
    assert_eq!(schedule.plan().cases.len(), 2);
    assert_eq!(
        schedule.plan().cases[0].dark_ranges,
        Some(vec![[0, 1], [1, 2]])
    );
    let f = Fixture::new(&["gallery-44"]);
    let schedule = f.schedule().unwrap();
    // Observe the invocation schedule before any fixture output is generated,
    // not by post-filtering a complete set of images. Native wiring is separate.
    let mut invoked = Vec::new();
    for id in schedule.story_ids() {
        invoked.push(*id);
    }
    assert_eq!(invoked, vec![44]);
    assert_eq!(schedule.plan().cases.len(), 1);
    assert!(!f.repo.join("refs").exists() && !f.repo.join("review").exists());
}

#[test]
fn independently_changed_story_requirements_cannot_hide_missing_pages_or_states() {
    for change in [
        "page",
        "cross-story",
        "label",
        "unknown",
        "sequence",
        "reorder",
    ] {
        let mut f = Fixture::new(&["gallery-44"]);
        f.mutate_selection(|catalog| {
            let s = &mut catalog["stories"][0];
            match change {
                "page" => {
                    s["cases"].as_array_mut().unwrap().pop();
                }
                "cross-story" => {
                    s["cases"][0] = json!("gallery-00-state-99-standard-normal-page-00");
                }
                "label" => s["label"] = json!("not the authored story"),
                "unknown" => {
                    s["cases"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!("gallery-44-state-01-standard-normal-page-00"));
                }
                "sequence" => {
                    s["sequences"] = json!([{"base":"sequence-fake-60hz","hz":60,"ticks":2}])
                }
                "reorder" => s["cases"].as_array_mut().unwrap().reverse(),
                _ => unreachable!(),
            }
        });
        assert!(f.schedule().is_err(), "{change}");
    }
    let mut f = Fixture::new(&["gallery-44"]);
    f.mutate_plan(|p| p["cases"].as_array_mut().unwrap().insert(1, json!({
        "base":"gallery-44-state-01-standard-normal","label":"Sidebar","state":"new family","group":"Composition","ranges":[[0,1]]
    })));
    assert!(
        f.schedule().is_err(),
        "an authored state absent from selection must fail"
    );
}

#[test]
fn unknown_story_and_rebinding_a_changed_plan_cannot_reuse_the_schedule() {
    let mut f = Fixture::new(&["gallery-44"]);
    let schedule = f.schedule().unwrap();
    f.mutate_plan(|p| p["cases"][0]["ranges"] = json!([[0, 1], [1, 3]]));
    // Independently refreshed caller sources are not permission to reuse an
    // older prepared schedule, even when IDs and page counts did not change.
    assert!(schedule.check_sources(&f.repo, &f.sources).is_err());
    let mut f = Fixture::new(&["gallery-44"]);
    let path = f.repo.join("stories.json");
    store::write_json(&path, &json!({"schema":1,"stories":[
        {"id":"gallery-53","label":"New story","cases":["gallery-53-state-00-standard-normal-page-00"]}
    ]})).unwrap();
    f.sources
        .insert("stories.json".into(), store::digest(&path).unwrap());
    f.selection = Selection::resolve(&f.repo, &f.config, &["gallery-53".into()]).unwrap();
    assert!(f.schedule().is_err());
    let mut f = Fixture::new(&["gallery-44"]);
    for relative in ["../plan.json", "/plan.json", "./plan.json"] {
        f.sources
            .insert(relative.into(), f.sources["plan.json"].clone());
        assert!(Schedule::resolve(&f.repo, &f.sources, relative, &f.selection).is_err());
    }
}

#[test]
fn malformed_independent_plans_fail_without_outputs() {
    for change in [
        "empty",
        "duplicate",
        "gap",
        "zero",
        "asymmetric-pages",
        "noncanonical",
        "label",
        "story-label",
        "story-group",
        "noncontiguous",
        "unsafe",
    ] {
        let mut f = Fixture::new(&["gallery-44"]);
        f.mutate_plan(|p| match change {
            "empty" => p["cases"] = json!([]),
            "duplicate" => {
                let c = p["cases"][0].clone();
                p["cases"].as_array_mut().unwrap().push(c);
            }
            "gap" => p["cases"][0]["ranges"] = json!([[0, 1], [2, 3]]),
            "zero" => p["cases"][0]["ranges"] = json!([[0, 0], [0, 3]]),
            "asymmetric-pages" => p["cases"][0]["dark_ranges"] = json!([[0, 2]]),
            "noncanonical" => p["cases"][0]["base"] = json!("gallery-044-state-00-standard-normal"),
            "label" => p["cases"][0]["state"] = json!("default\nforged"),
            "story-label" | "story-group" | "noncontiguous" => {
                let mut case = p["cases"][0].clone();
                case["base"] = json!("gallery-44-state-01-standard-normal");
                match change {
                    "story-label" => case["label"] = json!("different story label"),
                    "story-group" => case["group"] = json!("different group"),
                    _ => (),
                }
                let index = if change == "noncontiguous" { 2 } else { 1 };
                p["cases"].as_array_mut().unwrap().insert(index, case);
            }
            "unsafe" => p["cases"][0]["base"] = json!("../gallery-44-state-00-standard-normal"),
            _ => unreachable!(),
        });
        assert!(f.schedule().is_err(), "{change}");
        assert!(!f.repo.join("refs").exists());
    }
}

#[test]
fn stale_unsealed_forged_reordered_and_linked_requirements_fail_before_output() {
    let mut f = Fixture::new(&["gallery-44", "gallery-00"]);
    let schedule = f.schedule().unwrap();
    f.selection.stories.reverse();
    assert!(f.schedule().is_err());
    f.selection.stories.reverse();
    f.sources.remove("plan.json");
    assert!(f.schedule().is_err());
    assert!(schedule.check_sources(&f.repo, &f.sources).is_err());
    let f = Fixture::new(&["gallery-44"]);
    let schedule = f.schedule().unwrap();
    fs::write(f.repo.join("stories.json"), "{}").unwrap();
    assert!(f.schedule().is_err());
    assert!(schedule.check_sources(&f.repo, &f.sources).is_err());
    let f = Fixture::new(&["gallery-44"]);
    let schedule = f.schedule().unwrap();
    fs::write(f.repo.join("plan.json"), "{}").unwrap();
    assert!(schedule.check_sources(&f.repo, &f.sources).is_err());
    fs::rename(f.repo.join("plan.json"), f.repo.join("other.json")).unwrap();
    std::os::unix::fs::symlink(f.repo.join("other.json"), f.repo.join("plan.json")).unwrap();
    assert!(f.schedule().is_err());
    for name in ["plan.json", "stories.json"] {
        let f = Fixture::new(&["gallery-44"]);
        // Keep bytes and source hashes valid: rejection must be about the
        // linked requirement, not a coincidental malformed/stale fixture.
        fs::hard_link(f.repo.join(name), f.repo.join("alias.json")).unwrap();
        let error = f.schedule().unwrap_err();
        assert!(
            error.to_string().contains("single-link"),
            "{name}: {error:#}"
        );
    }
}

fn emit(output: &std::path::Path, plan: &gallery::Plan, png: &std::path::Path) -> String {
    fs::create_dir(output).unwrap();
    let mut inventory = String::new();
    let mut coverage = String::new();
    for c in &plan.cases {
        for mode in ["light", "dark"] {
            let ranges = if mode == "dark" {
                c.dark_ranges.as_ref().unwrap_or(&c.ranges)
            } else {
                &c.ranges
            };
            let id = format!("{}--{mode}", c.base);
            inventory += &format!("{id}\t{}\t{}\n", c.label, c.state);
            for (page, [start, end]) in ranges.iter().enumerate() {
                let raw = format!("{id}-page-{page:02}");
                fs::copy(png, output.join(format!("{raw}.png"))).unwrap();
                coverage += &format!(
                    "{raw}\t{id}\t{}\t{}\t{start}\t{end}\t{}\n",
                    c.label,
                    c.state,
                    ranges.last().unwrap()[1]
                );
            }
        }
    }
    fs::write(output.join("coverage.tsv"), coverage).unwrap();
    inventory
}

#[test]
fn upstream_seals_selected_gallery_as_partial_and_rejects_acceptance_and_collection_promotion() {
    use cargo_gpui_visual::{accept, capture, collection, compare};
    use std::sync::atomic::AtomicBool;
    // Even explicitly selecting every story stays partial/non-accepting.
    let mut f = Fixture::new(&["gallery-44", "gallery-00"]);
    // A static-only selection also works under plan schema 2, while upstream
    // partial sealing/nonacceptance remain unchanged. No endpoint is executed.
    f.mutate_plan(|p| p["schema"] = json!(2));
    let schedule = f.schedule().unwrap();
    let mut config = f.config.clone();
    config.selection = Some(f.selection.clone());
    let png = f.repo.join("fixture.png");
    image::RgbaImage::new(2400, 2000).save(&png).unwrap();
    let run = f.repo.join("selected-run");
    let refs = f.repo.join("refs");
    let report = f.repo.join("explicit-selected-review");
    let manifest = capture::capture_local(
        capture::Paths {
            repo: &f.repo,
            run: &run,
            baselines: &refs,
            report: &report,
        },
        &config,
        &AtomicBool::new(false),
        |output| {
            schedule.check_sources(&f.repo, &f.sources)?;
            let inventory = emit(output, schedule.plan(), &png);
            schedule.validate_and_prepare(&f.repo, &f.sources, output, &inventory)?;
            Ok(())
        },
        || Ok(json!({"layer":"gpui","evidence":"synthetic-portable-fixture-not-native"})),
    )
    .unwrap();
    assert_eq!(manifest.schema, 2);
    assert_eq!(manifest.selection, Some(f.selection.clone()));
    assert_eq!(manifest.cases.len(), 6);
    store::validate_images(&run, &manifest).unwrap();
    let review = compare::compare(&f.repo, &run, &refs, &report).unwrap();
    assert_eq!(review.schema, 2);
    assert_eq!(review.selection, Some(f.selection.clone()));
    assert!(!review.passed, "references remain absent");
    assert!(fs::read_to_string(report.join("index.html"))
        .unwrap()
        .contains("TEILLAUF"));
    for adopt in [false, true] {
        let error =
            accept::accept(&f.repo, &run, &refs, &report, &manifest.cases, adopt).unwrap_err();
        assert!(error.to_string().contains("development-only"), "{error:#}");
        assert!(store::baseline_state(&refs).unwrap().is_empty());
    }
    let collection_report = f.repo.join("explicit-collection");
    let result = collection::compare(
        &[collection::Suite {
            id: "gallery".into(),
            label: "Gallery fixture".into(),
            repo: f.repo.clone(),
            run: run.clone(),
            baselines: refs.clone(),
            catalog: "stories.json".into(),
        }],
        &collection_report,
    )
    .unwrap();
    assert!(!result.passed);
    assert!(result.suites[0].review.is_none());
    assert!(result.suites[0]
        .errors
        .iter()
        .any(|e| e.contains("not full-suite collection evidence")));
    // Actual consumer update router cannot convert it to accepted baselines.
    let result = cargo_gpui_visual::process::run(
        Command::new(env!("CARGO_BIN_EXE_bbr-visual"))
            .arg("update")
            .arg(&collection_report)
            .arg(format!("gallery:{}", manifest.cases[0])),
        &cargo_gpui_visual::process::Limits::default(),
        &AtomicBool::new(false),
        true,
    )
    .unwrap();
    assert!(!result.status.success());
    assert!(store::baseline_state(&refs).unwrap().is_empty());
}

#[test]
fn selected_gallery_output_requires_complete_scope_before_identity_renames_or_receipt() {
    let f = Fixture::new(&["gallery-44"]);
    let schedule = f.schedule().unwrap();
    let png = f.repo.join("fixture.png");
    image::RgbaImage::new(2400, 2000).save(&png).unwrap();
    for change in [
        "none",
        "missing",
        "extra",
        "appearance",
        "inventory",
        "duplicate",
        "ignored",
        "receipt",
        "hardlink",
        "output-link",
        "drift",
    ] {
        let output = f.repo.join(format!("output-{change}"));
        let plan = if change == "ignored" {
            store::read_json(&f.repo.join("plan.json")).unwrap()
        } else {
            schedule.plan().clone()
        };
        let mut inventory = emit(&output, &plan, &png);
        match change {
            "missing" => {
                fs::remove_file(
                    output.join("gallery-44-state-00-standard-normal--light-page-01.png"),
                )
                .unwrap();
            }
            "extra" => {
                fs::copy(&png, output.join("extra--light.png")).unwrap();
            }
            "appearance" => {
                for p in 0..2 {
                    fs::remove_file(output.join(format!(
                        "gallery-44-state-00-standard-normal--dark-page-{p:02}.png"
                    )))
                    .unwrap();
                }
            }
            "inventory" => inventory = String::new(),
            "duplicate" => inventory += &inventory.clone(),
            "receipt" => fs::write(output.join("selection.json"), "{}").unwrap(),
            "hardlink" => {
                let image = output.join("gallery-44-state-00-standard-normal--light-page-00.png");
                fs::remove_file(&image).unwrap();
                fs::hard_link(&png, &image).unwrap();
            }
            "output-link" => {
                let actual = f.repo.join("actual-output");
                fs::rename(&output, &actual).unwrap();
                std::os::unix::fs::symlink(actual, &output).unwrap();
            }
            "drift" => fs::write(f.repo.join("plan.json"), "{}").unwrap(),
            _ => (),
        }
        let result = schedule.validate_and_prepare(&f.repo, &f.sources, &output, &inventory);
        assert_eq!(result.is_ok(), change == "none", "{change}: {result:?}");
        if change == "none" {
            assert_eq!(result.unwrap().len(), 4);
            let receipt: Selection = store::read_json(&output.join("selection.json")).unwrap();
            assert_eq!(receipt, f.selection);
            assert_eq!(
                store::digest(&png).unwrap(),
                store::digest(
                    &output.join("gallery-44-state-00-standard-normal-page-00--light.png")
                )
                .unwrap()
            );
        } else {
            assert!(!output.join("inventory.json").exists());
            assert!(!output
                .join("gallery-44-state-00-standard-normal-page-00--light.png")
                .exists());
            if change != "receipt" {
                assert!(!output.join("selection.json").exists());
            }
        }
        assert!(!output.join("capture.json").exists());
    }
}
