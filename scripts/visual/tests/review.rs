//! Stored-run review only: synthetic seals and command tripwires, no native pixels.
use bbr_visual::{process, review};
use cargo_gpui_visual::{
    collection::Collection,
    config::Config,
    selection::{Catalog, Story},
    store::{self, Manifest},
};
use serde_json::json;
use std::{fs, path::PathBuf, process::Command, sync::atomic::AtomicBool};

struct Fixture {
    _temp: tempfile::TempDir,
    repo: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().canonicalize().unwrap();
        process::output(
            Command::new("git").args(["init", "-q"]).arg(&repo),
            &AtomicBool::new(false),
        )
        .unwrap();
        fs::create_dir_all(repo.join("scripts/visual/catalogs")).unwrap();
        fs::write(repo.join("source.rs"), "// fixture\n").unwrap();
        let base = "gallery-00-state-00-standard-normal-page-00";
        for (suite, config, case) in [
            ("gallery", "visual-gallery.toml", base),
            ("browser", "visual.toml", "ordinary-full"),
        ] {
            fs::write(repo.join(config), format!(
                "schema=1\ncapture=['forbidden']\nenvironment_command=['forbidden']\nbaselines='{suite}-baselines'\nreport='unused-report'\nruns='unused-runs'\nstory_catalog='stories.json'\n[source_policy]\nscopes=['source.rs','visual.toml','visual-gallery.toml','stories.json','scripts/visual/catalogs']\nsuffixes=['.rs','.toml','.json']\n"
            )).unwrap();
            store::write_json(&repo.join(format!("scripts/visual/catalogs/{suite}.json")), &json!({
                "schema":1,"snapshots":[{"base":case,"label":"Fixture","group":"Fixture","sequence":null,"moment":null,"page":null,"state":"required","reason":null}]
            })).unwrap();
        }
        store::write_json(
            &repo.join("stories.json"),
            &Catalog {
                schema: 1,
                stories: vec![Story {
                    id: "gallery-00".into(),
                    label: "Tokens".into(),
                    cases: vec![base.into()],
                    sequences: vec![],
                }],
            },
        )
        .unwrap();
        store::write_json(&repo.join("scripts/visual/catalogs/gallery-plan.json"), &json!({
            "schema":1,"cases":[{"base":"gallery-00-state-00-standard-normal","label":"Tokens","state":"default","group":"Foundations","ranges":[[0,1]],"dark_ranges":null}]
        })).unwrap();
        Self { _temp: temp, repo }
    }
    fn seal(&self, suite: &str, selected: bool) -> PathBuf {
        let config_name = if suite == "gallery" {
            "visual-gallery.toml"
        } else {
            "visual.toml"
        };
        let config = Config::load(&self.repo.join(config_name)).unwrap();
        let selection = selected.then(|| {
            config
                .select(&self.repo, &["gallery-00".into()])
                .unwrap()
                .selection
                .unwrap()
        });
        let run = self.repo.join(format!("{suite}-run"));
        fs::create_dir(&run).unwrap();
        let base = if suite == "gallery" {
            "gallery-00-state-00-standard-normal-page-00"
        } else {
            "ordinary-full"
        };
        let cases: Vec<_> = ["light", "dark"]
            .iter()
            .map(|mode| format!("{base}--{mode}"))
            .collect();
        for case in &cases {
            image::RgbaImage::from_pixel(1, 1, image::Rgba([1, 2, 3, 255]))
                .save(run.join(format!("{case}.png")))
                .unwrap();
        }
        if let Some(selection) = &selection {
            store::write_json(&run.join("selection.json"), selection).unwrap();
        }
        let manifest = Manifest {
            schema: if selected { 2 } else { 1 },
            images: cases
                .iter()
                .map(|c| {
                    (
                        c.clone(),
                        store::digest(&run.join(format!("{c}.png"))).unwrap(),
                    )
                })
                .collect(),
            cases,
            sources: store::source_hashes(
                &self.repo,
                &config.source_policy,
                Some(&self.repo.join(&config.baselines)),
            )
            .unwrap(),
            source_policy: config.source_policy,
            environment: json!({"layer":"gpui","suite":suite}),
            selection,
            sequences: vec![],
        };
        store::write_json(&run.join("capture.json"), &manifest).unwrap();
        run
    }
    fn compare(
        &self,
        gallery: Option<PathBuf>,
        browser: Option<PathBuf>,
        name: &str,
    ) -> anyhow::Result<review::Outcome> {
        review::compare(
            &self.repo,
            &review::Request {
                gallery,
                browser,
                report: self.repo.join("reports/visual").join(name),
            },
            &AtomicBool::new(false),
        )
    }
}

#[test]
fn full_stills_use_collections_with_independent_coverage_and_invalid_sibling_visibility() {
    let f = Fixture::new();
    let gallery = f.seal("gallery", false);
    let browser = f.seal("browser", false);
    let capture_hash = store::digest(&gallery.join("capture.json")).unwrap();
    let result = f
        .compare(Some(gallery.clone()), Some(browser.clone()), "full")
        .unwrap();
    assert_eq!(result.scope, "collection");
    assert!(!result.passed); // missing baselines require review
    let report = f.repo.join("reports/visual/full");
    let collection: Collection = store::read_json(&report.join("collection.json")).unwrap();
    assert_eq!(collection.suites.len(), 2);
    assert!(collection
        .suites
        .iter()
        .all(|s| s.errors.is_empty() && s.review.is_some()));
    assert_eq!(
        store::digest(&gallery.join("capture.json")).unwrap(),
        capture_hash
    );
    assert!(!f.repo.join("gallery-baselines").exists());
    assert!(!f.repo.join("browser-baselines").exists());
    assert!(f.compare(Some(gallery.clone()), None, "full").is_err());

    fs::remove_file(browser.join("capture.json")).unwrap();
    f.compare(Some(gallery.clone()), Some(browser), "invalid")
        .unwrap();
    let collection: Collection =
        store::read_json(&f.repo.join("reports/visual/invalid/collection.json")).unwrap();
    assert!(collection.suites[0].review.is_some());
    assert!(!collection.suites[1].errors.is_empty());

    // The source-bound required catalog, not a well-formed emitted pair, wins.
    let mut m: Manifest = store::read_json(&gallery.join("capture.json")).unwrap();
    for mode in ["light", "dark"] {
        let old = format!("gallery-00-state-00-standard-normal-page-00--{mode}");
        let new = format!("different--{mode}");
        fs::rename(
            gallery.join(format!("{old}.png")),
            gallery.join(format!("{new}.png")),
        )
        .unwrap();
        let hash = m.images.remove(&old).unwrap();
        m.images.insert(new.clone(), hash);
        *m.cases.iter_mut().find(|c| **c == old).unwrap() = new;
    }
    store::write_json(&gallery.join("capture.json"), &m).unwrap();
    f.compare(Some(gallery), None, "coverage").unwrap();
    let collection: Collection =
        store::read_json(&f.repo.join("reports/visual/coverage/collection.json")).unwrap();
    assert!(!collection.suites[0].errors.is_empty());
    assert!(!collection.passed);
}

#[test]
fn selected_gallery_has_only_a_partial_report_and_cannot_join_or_accept_a_collection() {
    let f = Fixture::new();
    let gallery = f.seal("gallery", true);
    let browser = f.seal("browser", false);
    let result = f.compare(Some(gallery.clone()), None, "partial").unwrap();
    assert_eq!(result.scope, "selected-development");
    let report = f.repo.join("reports/visual/partial");
    assert!(!report.join("collection.json").exists());
    assert!(fs::read_to_string(report.join("index.html"))
        .unwrap()
        .contains("TEILLAUF"));
    let r: cargo_gpui_visual::compare::Review =
        store::read_json(&report.join("review.json")).unwrap();
    assert!(r.selection.is_some());
    for adopt in [false, true] {
        let error = cargo_gpui_visual::accept::accept(
            &f.repo,
            &gallery,
            &f.repo.join("gallery-baselines"),
            &report,
            &["gallery-00-state-00-standard-normal-page-00--light".into()],
            adopt,
        )
        .unwrap_err();
        assert!(error.to_string().contains("development-only"), "{error:#}");
    }
    assert!(f
        .compare(Some(gallery), Some(browser), "mixed")
        .unwrap_err()
        .to_string()
        .contains("selected"));
    assert!(!f.repo.join("reports/visual/mixed").exists());
    assert!(cargo_gpui_visual::collection::accept(
        &report,
        &["gallery:gallery-00-state-00-standard-normal-page-00--light".into()],
        &[]
    )
    .is_err());
    assert!(!f.repo.join("gallery-baselines").exists());
}

#[test]
fn malformed_paths_stale_config_and_weakened_policy_never_publish_a_review() {
    for mutation in [
        "config",
        "policy",
        "unbound",
        "cancelled",
        "source-output",
        "baseline-output",
        "linked-config",
    ] {
        let f = Fixture::new();
        let run = f.seal("gallery", false);
        let mut m: Manifest = store::read_json(&run.join("capture.json")).unwrap();
        match mutation {
            "config" => {
                fs::write(
                    f.repo.join("visual-gallery.toml"),
                    fs::read_to_string(f.repo.join("visual-gallery.toml")).unwrap()
                        + "\n# changed\n",
                )
                .unwrap();
            }
            "policy" => {
                m.source_policy.scopes.retain(|s| s != "source.rs");
                store::write_json(&run.join("capture.json"), &m).unwrap();
            }
            "unbound" => {
                m.sources.remove("visual-gallery.toml");
                store::write_json(&run.join("capture.json"), &m).unwrap();
            }
            "linked-config" => {
                let config = f.repo.join("visual-gallery.toml");
                let target = f.repo.join("saved-config");
                fs::rename(&config, &target).unwrap();
                std::os::unix::fs::symlink(target, config).unwrap();
            }
            _ => {}
        }
        let report = f.repo.join(match mutation {
            "source-output" => "scripts/report",
            "baseline-output" => "gallery-baselines",
            _ => "reports/visual/rejected",
        });
        let error = review::compare(
            &f.repo,
            &review::Request {
                gallery: Some(run),
                browser: None,
                report: report.clone(),
            },
            &AtomicBool::new(mutation == "cancelled"),
        )
        .unwrap_err();
        assert!(!error.to_string().is_empty());
        assert!(!report.exists(), "{mutation}");
    }
    for args in [
        vec![],
        vec!["--report", "/tmp/report"],
        vec!["--gallery-run", "relative", "--report", "/tmp/report"],
        vec!["--latest"],
        vec![
            "--gallery-run",
            "/tmp/a",
            "--gallery-run",
            "/tmp/b",
            "--report",
            "/tmp/r",
        ],
        vec!["--browser-run", "/tmp/a", "--report", "/tmp/a"],
        vec!["--story", "*"],
    ] {
        assert!(
            review::Request::parse(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                .is_err(),
            "{args:?}"
        );
    }
}

#[test]
fn source_image_and_report_drift_remain_unacceptable() {
    for mutation in ["source", "image", "extra-image", "report"] {
        let f = Fixture::new();
        let gallery = f.seal("gallery", false);
        let first = f.compare(Some(gallery.clone()), None, "first").unwrap();
        match mutation {
            "source" => fs::write(f.repo.join("source.rs"), "// drift\n").unwrap(),
            "image" => fs::write(
                gallery.join("gallery-00-state-00-standard-normal-page-00--light.png"),
                "changed",
            )
            .unwrap(),
            "extra-image" => fs::copy(
                gallery.join("gallery-00-state-00-standard-normal-page-00--light.png"),
                gallery.join("extra--light.png"),
            )
            .map(|_| ())
            .unwrap(),
            "report" => fs::write(first.report.join("index.html"), "changed").unwrap(),
            _ => unreachable!(),
        }
        assert!(
            cargo_gpui_visual::collection::accept(
                &first.report,
                &["gallery:gallery-00-state-00-standard-normal-page-00--light".into()],
                &[],
            )
            .is_err(),
            "{mutation}"
        );
        assert!(!f.repo.join("gallery-baselines").exists());
        assert!(!first.report.join("acceptance.json").exists());
        if mutation != "report" {
            let second = f.compare(Some(gallery), None, "second").unwrap();
            assert!(!second.passed);
            let c: Collection = store::read_json(&second.report.join("collection.json")).unwrap();
            assert!(!c.suites[0].errors.is_empty(), "{mutation}");
            assert!(c.suites[0].review.is_none(), "{mutation}");
        }
    }
}

#[test]
fn exact_comparison_and_environment_drift_use_existing_upstream_evidence() {
    let f = Fixture::new();
    let gallery = f.seal("gallery", false);
    let first = f.compare(Some(gallery.clone()), None, "first").unwrap();
    // This is an explicitly scoped disposable synthetic store, not BBR adoption.
    cargo_gpui_visual::collection::accept(
        &first.report,
        &[
            "gallery:gallery-00-state-00-standard-normal-page-00--light".into(),
            "gallery:gallery-00-state-00-standard-normal-page-00--dark".into(),
        ],
        &[],
    )
    .unwrap();
    let baselines = f.repo.join("gallery-baselines");
    let before = store::baseline_state(&baselines).unwrap();
    assert!(
        f.compare(Some(gallery.clone()), None, "same")
            .unwrap()
            .passed
    );
    let name = "gallery-00-state-00-standard-normal-page-00--light";
    let image = gallery.join(format!("{name}.png"));
    image::RgbaImage::from_pixel(1, 1, image::Rgba([1, 2, 4, 255]))
        .save(&image)
        .unwrap();
    let mut m: Manifest = store::read_json(&gallery.join("capture.json")).unwrap();
    m.images.insert(name.into(), store::digest(&image).unwrap());
    m.environment["fixture-change"] = json!(true);
    store::write_json(&gallery.join("capture.json"), &m).unwrap();
    let changed = f.compare(Some(gallery), None, "changed").unwrap();
    assert!(!changed.passed);
    let c: Collection = store::read_json(&changed.report.join("collection.json")).unwrap();
    let r = c.suites[0].review.as_ref().unwrap();
    assert!(r.environment_mismatch.is_some());
    let finding = r.results.iter().find(|r| r.case == name).unwrap();
    assert_eq!(finding.status, "changed");
    assert_eq!(finding.changed_pixels, Some(1));
    assert_eq!(store::baseline_state(&baselines).unwrap(), before);
}

#[test]
fn canonical_paths_and_linked_outputs_are_refused_before_writes() {
    let f = Fixture::new();
    let gallery = f.seal("gallery", false);
    for run in [
        PathBuf::from(format!("{}/./gallery-run", f.repo.display())),
        PathBuf::from(format!("{}//gallery-run", f.repo.display())),
        f.repo.join("x/../gallery-run"),
    ] {
        assert!(f.compare(Some(run), None, "alias").is_err());
        assert!(!f.repo.join("reports").exists());
    }
    let link = f.repo.join("linked");
    std::os::unix::fs::symlink(&gallery, &link).unwrap();
    assert!(f.compare(Some(link), None, "linked").is_err());
    let report_parent = f.repo.join("reports");
    std::os::unix::fs::symlink(&gallery, &report_parent).unwrap();
    assert!(f.compare(Some(gallery.clone()), None, "parent").is_err());
    assert!(!gallery.join("visual").exists());
}

#[test]
fn valid_sequence_and_selected_browser_runs_cannot_hide_scope_in_collections() {
    let f = Fixture::new();
    let gallery = f.seal("gallery", false);
    let mut m: Manifest = store::read_json(&gallery.join("capture.json")).unwrap();
    for mode in ["light", "dark"] {
        let old = format!("gallery-00-state-00-standard-normal-page-00--{mode}");
        let frame = format!("sequence-demo-frame-0000--{mode}");
        fs::rename(
            gallery.join(format!("{old}.png")),
            gallery.join(format!("{frame}.png")),
        )
        .unwrap();
        let hash = m.images.remove(&old).unwrap();
        m.images.insert(frame.clone(), hash);
        *m.cases.iter_mut().find(|c| **c == old).unwrap() = frame.clone();
        m.sequences.push(cargo_gpui_visual::sequence::Sequence {
            id: format!("sequence-demo--{mode}"),
            hz: 60,
            ticks: vec![cargo_gpui_visual::sequence::Tick {
                tick: 0,
                time_ns: 0,
                drawn: true,
                dirty_before_callbacks: false,
                callbacks_pending: false,
                image: frame,
                events: vec![],
                state: json!({}),
            }],
        });
    }
    store::write_json(&gallery.join("capture.json"), &m).unwrap();
    store::load_manifest(&gallery).unwrap(); // Valid upstream timeline, not incidental rejection.
    assert!(f
        .compare(Some(gallery), None, "timeline")
        .unwrap_err()
        .to_string()
        .contains("sequence"));
    assert!(!f.repo.join("reports").exists());

    let f = Fixture::new();
    let gallery = f.seal("gallery", true);
    assert!(f
        .compare(None, Some(gallery), "wrong-suite")
        .unwrap_err()
        .to_string()
        .contains("selected"));
    assert!(!f.repo.join("reports").exists());
}
