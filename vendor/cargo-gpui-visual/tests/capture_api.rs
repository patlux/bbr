//! Disposable process/artifact fixtures, not production GPUI screenshot evidence.
#![cfg(unix)]

use cargo_gpui_visual::{capture::capture, config::Config, store::*};
use image::{Rgba, RgbaImage};
use serde_json::json;
use std::{fs, path::PathBuf, process::Command};

struct Fixture {
    _temp: tempfile::TempDir,
    repo: PathBuf,
    config: Config,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().canonicalize().unwrap();
        assert!(Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .status()
            .unwrap()
            .success());
        fs::write(repo.join("renderer.rs"), "production input fixture").unwrap();
        fs::create_dir(repo.join("seed")).unwrap();
        let cases = ["example--light", "example--dark"];
        for case in cases {
            RgbaImage::from_pixel(8, 6, Rgba([20, 30, 40, 255]))
                .save(repo.join(format!("seed/{case}.png")))
                .unwrap();
        }
        write_json(&repo.join("seed/inventory.json"), &cases).unwrap();
        write_json(
            &repo.join("environment.json"),
            &json!({"layer":"gpui","os":"fixture"}),
        )
        .unwrap();
        let config = Config::from_toml(
            r#"
schema = 1
capture = ["sh", "-c", "set -eu; test \"$1\" = \"$GPUI_VISUAL_OUTPUT\"; mkdir \"$1\"; cp seed/* \"$1/\"", "--", "{run}"]
environment_command = ["cat", "environment.json"]
baselines = "references"
report = "review"
runs = "runs"
[source_policy]
scopes = ["renderer.rs"]
suffixes = [".rs", ".svg"]
"#,
        )
        .unwrap();
        Self {
            _temp: temp,
            repo,
            config,
        }
    }

    fn capture(&self) -> anyhow::Result<Manifest> {
        capture(
            &self.repo,
            &self.config,
            "runs/a".as_ref(),
            &self.config.baselines,
            &self.config.report,
        )
    }

    fn change_after_produce(&mut self, suffix: &str) {
        self.config.capture[2].push_str("; ");
        self.config.capture[2].push_str(suffix);
    }

    fn assert_no_review_or_baseline(&self) {
        assert!(!self.repo.join("references").exists());
        assert!(!self.repo.join("review").exists());
    }
}

#[test]
fn configuration_is_strict_and_loadable_without_capture() {
    let f = Fixture::new();
    let source = r#"
schema = 1
capture = []
baselines = "references"
report = "review"
runs = "runs"
[source_policy]
scopes = ["renderer.rs"]
suffixes = [".rs"]
"#;
    let path = f.repo.join("visual.toml");
    fs::write(&path, source).unwrap();
    assert!(Config::load(&path).unwrap().environment_command.is_none());
    assert!(Config::from_toml(&source.replace("schema = 1", "schema = 2")).is_err());
    assert!(Config::from_toml(&source.replace("schema = 1", "schema = 1\ntypo = 7")).is_err());
    assert!(Config::from_toml(&source.replace("scopes =", "scope =")).is_err());
}

#[test]
fn library_capture_resolves_paths_seals_and_does_not_compare_or_accept() {
    let f = Fixture::new();
    fs::create_dir(f.repo.join("runs")).unwrap();
    let m = f.capture().unwrap();
    let run = f.repo.join("runs/a");
    assert_eq!(m.cases, ["example--light", "example--dark"]);
    assert_eq!(m.environment, json!({"layer":"gpui","os":"fixture"}));
    check_sources(&f.repo, &m, Some(&f.repo.join("references"))).unwrap();
    validate_images(&run, &m).unwrap();
    assert_eq!(m.images, load_manifest(&run).unwrap().images);
    let seal = digest(&run.join("capture.json")).unwrap();
    assert!(f
        .capture()
        .unwrap_err()
        .to_string()
        .contains("Run already exists"));
    assert_eq!(seal, digest(&run.join("capture.json")).unwrap());
    f.assert_no_review_or_baseline();
}

#[test]
fn guarded_capture_seals_the_same_pixels_without_touching_references() {
    use cargo_gpui_visual::{
        capture::{capture_guarded, Execution, Paths},
        gate::{Gate, Options},
    };
    use std::{sync::atomic::AtomicBool, time::Duration};
    let f = Fixture::new();
    fs::create_dir(f.repo.join("runs")).unwrap();
    let direct = f.capture().unwrap();
    let options = Options {
        path: f.repo.join("host.lock"),
        compatibility: vec![],
        wait: Duration::from_secs(1),
    };
    let guarded = capture_guarded(
        Paths {
            repo: &f.repo,
            run: "runs/b".as_ref(),
            baselines: &f.config.baselines,
            report: &f.config.report,
        },
        &f.config,
        &Execution::default(),
        &AtomicBool::new(false),
        &options,
    )
    .unwrap();
    assert_eq!(direct.images, guarded.images);
    assert_eq!(direct.sources, guarded.sources);
    assert_eq!(direct.environment, guarded.environment);
    assert_eq!(direct.cases, guarded.cases);
    Gate::acquire(&options, &AtomicBool::new(false))
        .unwrap()
        .finish()
        .unwrap();
    f.assert_no_review_or_baseline();
}

#[test]
fn public_capture_rejects_invalid_schema_or_overlapping_paths_before_execution() {
    let mut f = Fixture::new();
    f.config.capture = vec!["sh".into(), "-c".into(), "touch executed".into()];
    for schema in [0, 2] {
        f.config.schema = schema;
        assert!(f.capture().is_err());
        assert!(!f.repo.join("executed").exists());
    }
    f.config.schema = 1;
    for overlapping in ["runs/a", "runs", "runs/a/subdir"] {
        assert!(capture(
            &f.repo,
            &f.config,
            "runs/a".as_ref(),
            overlapping.as_ref(),
            "review".as_ref()
        )
        .is_err());
        assert!(!f.repo.join("executed").exists());
    }
    f.config.capture.clear();
    assert!(f
        .capture()
        .unwrap_err()
        .to_string()
        .contains("Capture command is empty"));
    assert!(!f.repo.join("executed").exists());
    f.config.environment_command = Some(vec![]);
    assert!(f
        .capture()
        .unwrap_err()
        .to_string()
        .contains("Environment command is empty"));
    f.assert_no_review_or_baseline();
}

#[test]
fn public_capture_refuses_unstable_or_failed_raw_producers_without_a_seal() {
    for change in [
        "printf changed > renderer.rs",
        "printf '{}' > environment.json",
        "exit 3",
        "printf '{}' > \"$1/capture.json\"",
        "rm \"$1/example--dark.png\"",
        "cp \"$1/example--dark.png\" \"$1/extra--dark.png\"",
        "printf '[\"example--light\",\"example--dark\",\"example--light\"]' > \"$1/inventory.json\"",
        "printf 'not-png' > \"$1/example--light.png\"",
    ] {
        let mut f = Fixture::new();
        fs::create_dir(f.repo.join("runs")).unwrap();
        f.change_after_produce(change);
        assert!(f.capture().is_err(), "{change}");
        // A raw producer's own forged seal is evidence, not an authorized CLI seal.
        if !change.contains("capture.json") {
            assert!(!f.repo.join("runs/a/capture.json").exists(), "{change}");
        }
        f.assert_no_review_or_baseline();
    }
}

#[test]
fn sealed_producer_must_match_the_complete_configured_policy_and_inventory() {
    for change in ["none", "policy", "sources", "pixels"] {
        let mut f = Fixture::new();
        fs::create_dir(f.repo.join("runs")).unwrap();
        let mut m = f.capture().unwrap();
        match change {
            "policy" => m.source_policy.suffixes = vec![".rs".into()],
            "sources" => {
                m.sources.insert("not-present.rs".into(), "0".repeat(64));
            }
            "pixels" => {
                m.images.insert("example--light".into(), "0".repeat(64));
            }
            _ => {}
        }
        write_json(&f.repo.join("seed/capture.json"), &m).unwrap();
        f.config.environment_command = None;
        let result = capture(
            &f.repo,
            &f.config,
            "runs/b".as_ref(),
            &f.config.baselines,
            &f.config.report,
        );
        if change == "none" {
            assert_eq!(result.unwrap().images, m.images);
        } else {
            assert!(result.is_err(), "{change}");
        }
        f.assert_no_review_or_baseline();
    }
}

/// Two-tick journeys (drawn, then held) in both appearances, as a raw producer emits them.
fn seed_sequences(f: &Fixture) -> serde_json::Value {
    let mut cases: Vec<String> = read_json(&f.repo.join("seed/inventory.json")).unwrap();
    let mut journeys = Vec::new();
    for theme in ["light", "dark"] {
        let frame = format!("sequence-demo-60hz-frame-0000--{theme}");
        RgbaImage::from_pixel(8, 6, Rgba([60, 70, 80, 255]))
            .save(f.repo.join(format!("seed/{frame}.png")))
            .unwrap();
        cases.push(frame.clone());
        let tick = |tick: u64, drawn: bool| {
            json!({"tick":tick,"time_ns":tick * 1_000_000_000 / 60,"drawn":drawn,
                "dirty_before_callbacks":drawn,"callbacks_pending":false,
                "image":frame,"events":[],"state":{"ready":tick > 0}})
        };
        journeys.push(json!({"id":format!("sequence-demo-60hz--{theme}"),"hz":60,
            "ticks":[tick(0, true), tick(1, false)]}));
    }
    write_json(&f.repo.join("seed/inventory.json"), &cases).unwrap();
    let journeys = json!(journeys);
    write_json(&f.repo.join("seed/sequences.json"), &journeys).unwrap();
    journeys
}

#[test]
fn raw_producer_timelines_are_sealed_with_their_frames() {
    let f = Fixture::new();
    fs::create_dir(f.repo.join("runs")).unwrap();
    let journeys = seed_sequences(&f);
    let m = f.capture().unwrap();
    assert_eq!(m.cases.len(), 4);
    assert_eq!(json!(m.sequences), journeys);
    let sealed: serde_json::Value = read_json(&f.repo.join("runs/a/capture.json")).unwrap();
    assert_eq!(sealed["sequences"], journeys);
    f.assert_no_review_or_baseline();
}

#[test]
fn malformed_or_linked_raw_timelines_leave_no_seal() {
    for mode in [
        "missing-tick",
        "orphaned-frames",
        "unrendered-hold",
        "unpaired",
        "symlink",
    ] {
        let mut f = Fixture::new();
        fs::create_dir(f.repo.join("runs")).unwrap();
        let mut journeys = seed_sequences(&f);
        match mode {
            "missing-tick" => journeys[0]["ticks"][1]["tick"] = json!(2),
            "orphaned-frames" => journeys = json!([]),
            "unrendered-hold" => journeys[0]["ticks"][0]["drawn"] = json!(false),
            "unpaired" => {
                journeys.as_array_mut().unwrap().pop();
            }
            _ => f.change_after_produce(
                r#"mv "$1/sequences.json" "$1/real.json"; ln -s real.json "$1/sequences.json""#,
            ),
        }
        write_json(&f.repo.join("seed/sequences.json"), &journeys).unwrap();
        assert!(f.capture().is_err(), "{mode}");
        assert!(!f.repo.join("runs/a/capture.json").exists(), "{mode}");
        f.assert_no_review_or_baseline();
    }
}
