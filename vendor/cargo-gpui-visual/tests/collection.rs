#[path = "collection/acceptance.rs"]
mod acceptance;
#[path = "collection/environment.rs"]
mod environment;
#[path = "collection/presentation.rs"]
mod presentation;
#[path = "collection/summary.rs"]
mod summary;
#[path = "collection/withheld.rs"]
mod withheld;

use cargo_gpui_visual::{
    collection::{self, Collection, Suite},
    report,
    store::*,
};
use image::{Rgba, RgbaImage};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::Command};

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    suites: Vec<Suite>,
    report: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let mut suites = Vec::new();
        for (id, layer, color) in [("gallery", "gpui", 30), ("browser", "ios", 80)] {
            let repo = root.join(id);
            fs::create_dir(&repo).unwrap();
            assert!(Command::new("git")
                .args(["init", "-q"])
                .arg(&repo)
                .status()
                .unwrap()
                .success());
            fs::write(repo.join("renderer.rs"), "production input").unwrap();
            fs::create_dir(repo.join("run")).unwrap();
            write_json(&repo.join("catalog.json"), &json!({
                "schema":1, "snapshots":[
                    {"base":"panel-page-one","label":"First page", "group":"Panels", "sequence":"Open panel", "moment":"Open", "page":{"set":"panel","number":1,"total":2}, "state":"required"},
                    {"base":"panel-page-two","label":"Second page", "group":"Panels", "sequence":"Open panel", "moment":"Open", "page":{"set":"panel","number":2,"total":2}, "state":"required"}
                ]
            })).unwrap();
            for base in ["panel-page-one", "panel-page-two"] {
                for theme in ["light", "dark"] {
                    RgbaImage::from_pixel(20, 12, Rgba([color, 20, 40, 255]))
                        .save(repo.join(format!("run/{base}--{theme}.png")))
                        .unwrap();
                }
            }
            write_json(
                &repo.join("environment.json"),
                &json!({"layer":layer,"os":id,"scale":2}),
            )
            .unwrap();
            suites.push(Suite {
                id: id.into(),
                label: id.into(),
                repo: repo.clone(),
                run: repo.join("run"),
                baselines: repo.join("baselines"),
                catalog: "catalog.json".into(),
            });
        }
        let f = Self {
            report: root.join("report"),
            root,
            _temp: temp,
            suites,
        };
        f.seal(0);
        f.seal(1);
        f
    }
    fn seal(&self, index: usize) {
        let s = &self.suites[index];
        let policy = Policy {
            scopes: vec!["renderer.rs".into(), "catalog.json".into()],
            suffixes: vec![".rs".into(), ".json".into()],
            artifacts: vec![],
        };
        let images: Hashes = fs::read_dir(&s.run)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "png"))
            .map(|p| {
                (
                    p.file_stem().unwrap().to_str().unwrap().to_owned(),
                    digest(&p).unwrap(),
                )
            })
            .collect();
        write_json(
            &s.run.join("capture.json"),
            &Manifest {
                schema: 1,
                cases: images.keys().cloned().collect(),
                images,
                environment: read_json(&s.repo.join("environment.json")).unwrap(),
                sources: source_hashes(&s.repo, &policy, Some(&s.baselines)).unwrap(),
                source_policy: policy,
                sequences: vec![],
                selection: None,
            },
        )
        .unwrap();
    }
    fn catalog(&self, index: usize, change: impl FnOnce(&mut Value)) {
        let path = self.suites[index].repo.join("catalog.json");
        let mut v: Value = read_json(&path).unwrap();
        change(&mut v);
        write_json(&path, &v).unwrap();
    }
    fn compare(&self) -> anyhow::Result<Collection> {
        collection::compare(&self.suites, &self.report)
    }
    fn html(&self) -> String {
        fs::read_to_string(self.report.join("index.html")).unwrap()
    }
}

#[test]
fn duplicate_cases_keep_separate_assets_environments_and_original_report() {
    let f = Fixture::new();
    let c = f.compare().unwrap();
    assert!(!c.passed);
    assert_eq!(c.suites.len(), 2);
    assert_eq!(
        c.suites[0].review.as_ref().unwrap().environment["layer"],
        "gpui"
    );
    assert_eq!(
        c.suites[1].review.as_ref().unwrap().environment["layer"],
        "ios"
    );
    let html = f.html();
    for s in &f.suites {
        assert!(!s.baselines.exists());
        assert!(html.contains(&format!("id=\"f-{}:panel-page-one--light\"", s.id)));
        assert!(html.contains(&format!(
            "src=\"suites/{}/panel-page-one--light/actual.png\"",
            s.id
        )));
        assert!(!f
            .report
            .join(format!("suites/{}/review.json", s.id))
            .exists());
        assert!(!f
            .report
            .join(format!("suites/{}/index.html", s.id))
            .exists());
    }
    assert_ne!(
        digest(
            &f.report
                .join("suites/gallery/panel-page-one--light/actual.png")
        )
        .unwrap(),
        digest(
            &f.report
                .join("suites/browser/panel-page-one--light/actual.png")
        )
        .unwrap()
    );
    for text in [
        "Panels",
        "Open panel",
        "Open",
        "Seite 1 / 2",
        "Seite 2 / 2",
        "Alle Snapshots",
        "only-findings",
        "default-src 'none'",
        "keine Referenz",
    ] {
        assert!(html.contains(text), "{text}");
    }
    assert!(html.find("First page").unwrap() < html.find("Second page").unwrap());
    assert!(!html.contains("<iframe"));
    assert_eq!(html.matches("<script").count(), 1);
    assert!(html.contains(&format!("<script>{}</script>", report::SCRIPT)));
    assert!(html.contains(&format!("script-src {}\"", report::script_hash())));
    let saved: Collection = read_json(&f.report.join("collection.json")).unwrap();
    assert_eq!(saved.suites.len(), 2);
    assert!(
        f.compare().is_err(),
        "explicit collection directory is immutable"
    );
}

#[test]
fn required_coverage_is_not_derived_from_emitted_inventory() {
    let f = Fixture::new();
    for theme in ["light", "dark"] {
        fs::remove_file(f.suites[0].run.join(format!("panel-page-two--{theme}.png"))).unwrap();
    }
    f.seal(0);
    let c = f.compare().unwrap();
    assert!(c.suites[0]
        .errors
        .iter()
        .any(|e| e.contains("Missing required") && e.contains("panel-page-two")));
    assert!(!c.passed);
    assert!(f.html().contains("missing-capture"));
    assert!(!f.suites[0].baselines.exists());
}

#[test]
fn withheld_unsupported_and_unexpected_stay_distinct() {
    let f = Fixture::new();
    f.catalog(0, |v| {
        v["snapshots"][0]["state"] = json!("withheld");
        v["snapshots"][0]["reason"] = json!("native readiness not proven");
        v["snapshots"][1]["state"] = json!("unsupported");
        v["snapshots"][1]["reason"] = json!("not a host surface");
    });
    for theme in ["light", "dark"] {
        fs::remove_file(f.suites[0].run.join(format!("panel-page-two--{theme}.png"))).unwrap();
        RgbaImage::new(2, 2)
            .save(f.suites[0].run.join(format!("unplanned--{theme}.png")))
            .unwrap();
    }
    f.seal(0);
    let c = f.compare().unwrap();
    let errors = c.suites[0].errors.join("\n");
    for text in ["Withheld", "Non-required capture", "Unexpected capture"] {
        assert!(errors.contains(text), "{errors}");
    }
    let html = f.html();
    for text in [
        "withheld",
        "unsupported",
        "native readiness not proven",
        "not a host surface",
        "unplanned--light",
    ] {
        assert!(html.contains(text), "{text}");
    }
}

#[test]
fn one_invalid_suite_does_not_hide_its_valid_sibling() {
    let f = Fixture::new();
    fs::write(
        f.suites[0].repo.join("renderer.rs"),
        "changed after capture",
    )
    .unwrap();
    let c = f.compare().unwrap();
    assert!(c.suites[0].review.is_none());
    assert!(c.suites[0]
        .errors
        .join(" ")
        .contains("source inventory changed"));
    assert!(c.suites[1].review.is_some());
    assert!(f.html().contains("browser:panel-page-one--light"));
    assert!(f.html().contains("source inventory changed"));
}

#[test]
fn suites_with_timelines_fail_closed_until_collections_present_them() {
    let f = Fixture::new();
    let s = &f.suites[0];
    let mut m = load_manifest(&s.run).unwrap();
    for theme in ["light", "dark"] {
        let frame = format!("sequence-demo-60hz-frame-0000--{theme}");
        let path = s.run.join(format!("{frame}.png"));
        RgbaImage::from_pixel(20, 12, Rgba([1, 2, 3, 255]))
            .save(&path)
            .unwrap();
        m.cases.push(frame.clone());
        m.images.insert(frame.clone(), digest(&path).unwrap());
        m.sequences.push(
            serde_json::from_value(json!({"id":format!("sequence-demo-60hz--{theme}"),"hz":60,
                "ticks":[{"tick":0,"time_ns":0,"drawn":true,"dirty_before_callbacks":true,
                "callbacks_pending":false,"image":frame,"events":[],"state":null}]}))
            .unwrap(),
        );
    }
    write_json(&s.run.join("capture.json"), &m).unwrap();
    load_manifest(&s.run).unwrap();
    let c = f.compare().unwrap();
    assert!(!c.passed);
    assert!(c.suites[0].review.is_none());
    assert!(c.suites[0].errors.join(" ").contains("sequence timelines"));
    assert!(c.suites[1].review.is_some());
    assert!(collection::accept(
        &f.report,
        &["gallery:sequence-demo-60hz--light".into()],
        &[]
    )
    .is_err());
    assert!(!s.baselines.exists());
}

#[test]
fn catalog_must_be_part_of_its_capture_source_seal() {
    let f = Fixture::new();
    let s = &f.suites[0];
    let mut m = load_manifest(&s.run).unwrap();
    m.sources.remove("catalog.json");
    m.source_policy.scopes = vec!["renderer.rs".into()];
    write_json(&s.run.join("capture.json"), &m).unwrap();
    let c = f.compare().unwrap();
    assert!(c.suites[0]
        .errors
        .join(" ")
        .contains("Catalog must be a sealed source input"));
    assert!(c.suites[0].review.is_none());
}

#[test]
fn unsafe_or_overlapping_suite_paths_fail_before_output() {
    for mode in 0..5 {
        let mut f = Fixture::new();
        match mode {
            0 => f.suites[1].id = f.suites[0].id.clone(),
            1 => f.suites[0].id = "../escape".into(),
            2 => f.suites[1].baselines = f.suites[0].baselines.clone(),
            3 => f.suites[1].run = f.report.join("run"),
            _ => f.suites[0].catalog = "../outside.json".into(),
        }
        assert!(f.compare().is_err(), "mode {mode}");
        assert!(!f.report.exists());
    }
}

#[test]
fn invalid_catalogs_are_explicit_suite_failures_and_text_is_escaped() {
    for mode in 0..8 {
        let f = Fixture::new();
        f.catalog(0, |v| match mode {
            0 => v["snapshots"][1] = v["snapshots"][0].clone(),
            1 => v["snapshots"][0]["page"]["number"] = json!(0),
            2 => v["snapshots"][0]["state"] = json!("withheld"),
            3 => v["snapshots"][0]["sequence"] = Value::Null,
            4 => v["snapshots"][0]["unknown"] = json!(true),
            5 => {
                v["snapshots"].as_array_mut().unwrap().pop();
            }
            6 => v["snapshots"].as_array_mut().unwrap().reverse(),
            _ => v["snapshots"][1]["page"]["total"] = json!(3),
        });
        f.seal(0);
        let c = f.compare().unwrap();
        assert!(c.suites[0].review.is_none(), "mode {mode}");
        assert!(!c.suites[0].errors.is_empty());
    }
    let mut f = Fixture::new();
    f.suites[0].label = "<script>alert('suite')</script>".into();
    f.catalog(0, |v| {
        v["snapshots"][0]["label"] = json!("<img src=x onerror=bad>")
    });
    f.seal(0);
    f.compare().unwrap();
    let html = f.html();
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&lt;img src=x onerror=bad&gt;"));
    assert_eq!(html.matches("<script").count(), 1);
}

#[test]
fn changed_suites_reuse_controls_and_cannot_be_accepted_as_single_suite_reports() {
    let f = Fixture::new();
    for s in &f.suites {
        let report = s.repo.join("establish");
        let m = load_manifest(&s.run).unwrap();
        cargo_gpui_visual::compare::compare(&s.repo, &s.run, &s.baselines, &report).unwrap();
        cargo_gpui_visual::accept::accept(&s.repo, &s.run, &s.baselines, &report, &m.cases, false)
            .unwrap();
        let path = image_path(&s.run, "panel-page-one--light").unwrap();
        let mut image = read_png(&path).unwrap();
        image.put_pixel(3, 4, Rgba([0, 0, 0, 0]));
        image.save(path).unwrap();
    }
    f.seal(0);
    f.seal(1);
    // Only the second suite drifts its environment; it must not affect the first.
    let path = f.suites[1].repo.join("environment.json");
    let mut environment: Value = read_json(&path).unwrap();
    environment["os"] = json!("new-runtime");
    write_json(&path, &environment).unwrap();
    f.seal(1);
    let before: Vec<_> = f
        .suites
        .iter()
        .map(|s| baseline_state(&s.baselines).unwrap())
        .collect();
    let c = f.compare().unwrap();
    assert!(c.suites[0]
        .review
        .as_ref()
        .unwrap()
        .environment_mismatch
        .is_none());
    assert!(c.suites[1]
        .review
        .as_ref()
        .unwrap()
        .environment_mismatch
        .is_some());
    for (s, state) in f.suites.iter().zip(before) {
        let r = c
            .suites
            .iter()
            .find(|r| r.suite.id == s.id)
            .unwrap()
            .review
            .as_ref()
            .unwrap();
        assert_eq!(
            r.results
                .iter()
                .find(|r| r.case == "panel-page-one--light")
                .unwrap()
                .changed_pixels,
            Some(1)
        );
        assert_eq!(state, baseline_state(&s.baselines).unwrap());
        assert!(cargo_gpui_visual::accept::accept(
            &s.repo,
            &s.run,
            &s.baselines,
            &f.report.join("suites").join(&s.id),
            &["panel-page-one--light".into()],
            false
        )
        .is_err());
        assert!(f
            .html()
            .contains(&format!("name=\"mode-{}:panel-page-one--light\"", s.id)));
    }
    for text in [
        ">2-up</label>",
        ">Wischen</label>",
        ">Überblenden</label>",
        ">Diff</label>",
        "id=\"crop-all\"",
        "class=\"m-crop\"",
        "Aufnahmeumgebung weicht ab",
    ] {
        assert!(f.html().contains(text), "{text}");
    }
    // Optional retained disposable fixture for managed-browser verification only.
    if std::env::var_os("GPUI_VISUAL_KEEP_COLLECTION_FIXTURE").is_some() {
        println!("COLLECTION_FIXTURE={}", f._temp.keep().display());
    }
}

#[test]
fn complete_exact_collection_passes_but_withholding_cannot_hide_in_green() {
    let f = Fixture::new();
    for s in &f.suites {
        let report = s.repo.join("establish");
        let m = load_manifest(&s.run).unwrap();
        cargo_gpui_visual::compare::compare(&s.repo, &s.run, &s.baselines, &report).unwrap();
        cargo_gpui_visual::accept::accept(&s.repo, &s.run, &s.baselines, &report, &m.cases, false)
            .unwrap();
    }
    assert!(f.compare().unwrap().passed);
    f.catalog(0,|v|v["snapshots"].as_array_mut().unwrap().push(json!({"base":"gesture","label":"Gesture","group":"Native","state":"withheld","reason":"device not authorized"})));
    f.seal(0);
    let c = collection::compare(&f.suites, &f.root.join("withheld")).unwrap();
    assert!(!c.passed);
    assert!(c.suites[0].review.as_ref().unwrap().passed);
    assert!(!c.suites[0].passed);
}

#[test]
fn input_paths_resolve_from_input_file_not_the_current_directory() {
    let f = Fixture::new();
    let path = f.root.join("relative.json");
    let mut suites = f.suites.clone();
    for s in &mut suites {
        s.repo = s.id.clone().into();
        s.run = "run".into();
        s.baselines = "baselines".into();
    }
    write_json(&path, &json!({"schema":1,"suites":suites})).unwrap();
    let input = collection::Input::load(&path).unwrap();
    assert_eq!(input.suites[0].repo, f.suites[0].repo);
    assert_eq!(input.suites[1].run, f.suites[1].run);
    collection::compare(&input.suites, &f.report).unwrap();
}

#[cfg(unix)]
#[test]
fn symlinked_catalog_and_output_are_refused_without_reference_writes() {
    use std::os::unix::fs::symlink;
    for catalog in [true, false] {
        let f = Fixture::new();
        if catalog {
            let path = f.suites[0].repo.join("catalog.json");
            fs::rename(&path, path.with_extension("original")).unwrap();
            symlink(path.with_extension("original"), &path).unwrap();
        } else {
            let elsewhere = f.root.join("elsewhere");
            fs::create_dir(&elsewhere).unwrap();
            symlink(&elsewhere, &f.report).unwrap();
        }
        assert!(f.compare().is_err());
        for s in &f.suites {
            assert!(!s.baselines.exists());
        }
        assert!(!f.report.join("collection.json").exists());
    }
}

#[test]
fn cli_requires_explicit_new_report_and_emits_collection_verdict() {
    let f = Fixture::new();
    let input = f.root.join("suites.json");
    write_json(&input, &json!({"schema":1,"suites":f.suites})).unwrap();
    let run = |with_report| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"));
        command
            .args(["review-collection", "--collection"])
            .arg(&input);
        if with_report {
            command.arg("--report").arg(&f.report);
        }
        command.output().unwrap()
    };
    assert_eq!(run(false).status.code(), Some(2));
    assert!(!f.report.exists());
    let output = run(true);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(f.report.join("collection.json").exists());
    assert_eq!(run(true).status.code(), Some(2));
}
