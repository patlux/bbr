//! Agent protocol regressions. All captures and references are disposable fixtures.
use cargo_gpui_visual::{process, store};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::Command, sync::atomic::AtomicBool, time::Duration};

struct Fixture {
    _temp: tempfile::TempDir,
    repo: PathBuf,
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
        fs::create_dir(repo.join("input")).unwrap();
        for mode in ["light", "dark"] {
            image::RgbaImage::from_pixel(2, 2, image::Rgba([1, 2, 3, 255]))
                .save(repo.join(format!("input/demo--{mode}.png")))
                .unwrap();
        }
        fs::write(repo.join("produce.sh"), "set -eu\necho noisy-producer-stdout\necho noisy-producer-stderr >&2\nmkdir -p \"$GPUI_VISUAL_OUTPUT\"\ncp input/*.png \"$GPUI_VISUAL_OUTPUT/\"\nprintf '%s' '[\"demo--light\",\"demo--dark\"]' > \"$GPUI_VISUAL_OUTPUT/inventory.json\"\nif [ -n \"${GPUI_VISUAL_SELECTION:-}\" ]; then printf '%s' \"$GPUI_VISUAL_SELECTION\" > \"$GPUI_VISUAL_OUTPUT/selection.json\"; fi\n").unwrap();
        store::write_json(
            &repo.join("stories.json"),
            &json!({"schema":1,"stories":[{"id":"demo","label":"Demo","cases":["demo"]}]}),
        )
        .unwrap();
        let config = json!({"schema":1,"capture":["sh","produce.sh"],
            "environment_command":["sh","-c","printf '%s' '{\"layer\":\"gpui\"}'"],
            "baselines":"refs","report":"review","runs":"runs","story_catalog":"stories.json",
            "source_policy":{"scopes":["input","produce.sh","stories.json","visual.toml"],"suffixes":[".png",".sh",".json",".toml"]}});
        fs::write(repo.join("visual.toml"), toml::to_string(&config).unwrap()).unwrap();
        Self { _temp: temp, repo }
    }
    fn run(&self, args: &[&str]) -> process::Output {
        process::run(
            Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"))
                .arg("--repo")
                .arg(&self.repo)
                .args(args),
            &process::Limits {
                timeout: Duration::from_secs(15),
                ..Default::default()
            },
            &AtomicBool::new(false),
            true,
        )
        .unwrap()
    }
    fn json(&self, args: &[&str], code: i32) -> Value {
        let mut arguments = args.to_vec();
        arguments.push("--json");
        let output = self.run(&arguments);
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        let result: Value =
            serde_json::from_slice(&output.stdout).expect("stdout must be exactly one JSON value");
        assert_eq!(result["schema"], 1);
        assert_eq!(result["exit_code"], code);
        result
    }
}

#[test]
fn json_success_review_and_errors_are_parseable_despite_noisy_producer() {
    let f = Fixture::new();
    let doctor = f.json(&["doctor"], 0);
    assert_eq!(doctor["data"]["producer_executed"], false);
    let catalog = f.json(&["list"], 0);
    assert_eq!(catalog["data"]["stories"][0]["id"], "demo");
    let check = f.json(&["check", "--run", "runs/one"], 1);
    assert_eq!(check["status"], "review_required");
    for phase in [
        "producer_ms",
        "environment_probe_ms",
        "capture_total_ms",
        "compare_validation_assets_ms",
        "report_ms",
        "total_ms",
    ] {
        assert!(
            check["timings_ms"][phase]
                .as_f64()
                .is_some_and(|v| v >= 0.0),
            "missing {phase}"
        );
    }
    assert_eq!(check["data"]["scope"]["kind"], "full");
    assert_eq!(check["data"]["coverage"], "not_independently_verified");
    assert_eq!(check["data"]["counts"]["missing_baseline"], 2);
    assert!(check["data"]["artifacts"]["review"]
        .as_str()
        .unwrap()
        .ends_with("review.json"));
    f.json(&["validate", "--run", "runs/one"], 0);
    fs::write(f.repo.join("produce.sh"), "exit 0\n").unwrap();
    let stale = f.json(&["validate", "--run", "runs/one"], 2);
    assert_eq!(stale["error"]["code"], "source_changed");
    assert!(!f.repo.join("refs").exists());
}

#[test]
fn inspect_is_bounded_read_only_and_never_claims_current_validity() {
    let f = Fixture::new();
    f.json(&["check", "--run", "runs/one"], 1);
    let before = store::digest(&f.repo.join("review/review.json")).unwrap();
    let summary = f.json(&["inspect", "--report", "review", "--limit", "1"], 0);
    assert_eq!(summary["data"]["evidence"], "stored_unvalidated");
    assert_eq!(summary["data"]["findings"].as_array().unwrap().len(), 1);
    assert_eq!(summary["data"]["total_findings"], 2);
    assert_eq!(summary["data"]["next_offset"], 1);
    let detail = f.json(
        &["inspect", "--report", "review", "--case", "demo--dark"],
        0,
    );
    assert!(detail["data"]["detail"]["artifacts"]["actual"]
        .as_str()
        .unwrap()
        .ends_with("demo--dark/actual.png"));
    assert!(detail["data"]["detail"]["artifacts"]["expected"].is_null());
    fs::write(f.repo.join("produce.sh"), "changed source").unwrap();
    assert_eq!(
        f.json(&["inspect", "--report", "review"], 0)["data"]["evidence"],
        "stored_unvalidated"
    );
    f.json(
        &[
            "inspect",
            "--report",
            "review",
            "--case",
            "../escape--light",
        ],
        2,
    );
    assert_eq!(
        before,
        store::digest(&f.repo.join("review/review.json")).unwrap()
    );
    assert!(!f.repo.join("refs").exists());
    fs::write(f.repo.join("review/.review.lock"), "busy").unwrap();
    f.json(&["inspect", "--report", "review"], 2);
}

#[test]
fn dry_run_resolves_scope_without_probes_processes_locks_or_output() {
    let f = Fixture::new();
    fs::write(
        f.repo.join("produce.sh"),
        "echo invoked > forbidden; exit 9\n",
    )
    .unwrap();
    let text = fs::read_to_string(f.repo.join("visual.toml")).unwrap();
    let mut config: toml::Value = toml::from_str(&text).unwrap();
    config["environment_command"] =
        toml::Value::try_from(vec!["sh", "-c", "echo probed > forbidden; exit 9"]).unwrap();
    fs::write(
        f.repo.join("visual.toml"),
        toml::to_string(&config).unwrap(),
    )
    .unwrap();
    let p = f.json(
        &[
            "check",
            "--story",
            "demo",
            "--report",
            "planned",
            "--run",
            "runs/planned",
            "--dry-run",
            "--host-gate",
        ],
        0,
    );
    assert_eq!(p["data"]["dry_run"], true);
    assert_eq!(p["data"]["producer_executed"], false);
    assert_eq!(p["data"]["selection"]["stories"][0]["id"], "demo");
    assert_eq!(p["data"]["requirements"]["still_images"], 2);
    assert_eq!(p["data"]["host_gate"]["requested"], true);
    for path in ["forbidden", "runs", "planned", "refs"] {
        assert!(!f.repo.join(path).exists());
    }
    f.json(&["check", "--dry-run", "--run", "refs/overlap"], 2);
    fs::create_dir_all(f.repo.join("runs/existing")).unwrap();
    f.json(&["check", "--dry-run", "--run", "runs/existing"], 2);
    config.as_table_mut().unwrap().remove("environment_command");
    fs::write(
        f.repo.join("visual.toml"),
        toml::to_string(&config).unwrap(),
    )
    .unwrap();
    let unsupported = f.json(
        &[
            "check",
            "--story",
            "demo",
            "--report",
            "planned",
            "--dry-run",
        ],
        2,
    );
    assert_eq!(unsupported["error"]["code"], "selection_unsupported");
}

#[test]
fn bundled_skill_is_version_matched_available_without_a_repository_and_read_only() {
    let f = Fixture::new();
    fs::remove_file(f.repo.join("visual.toml")).unwrap();
    let skill = f.json(&["skill"], 0);
    assert_eq!(skill["data"]["content"], cargo_gpui_visual::skill::MAIN);
    assert!(cargo_gpui_visual::skill::MAIN.lines().count() < 150);
    assert!(cargo_gpui_visual::skill::MAIN.starts_with("---\nname: gpui-visual\ndescription:"));
    assert_eq!(
        f.json(&["skill", "--reference", "consumer"], 0)["data"]["content"],
        cargo_gpui_visual::skill::CONSUMER
    );
    let output = f.run(&["skill"]);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        cargo_gpui_visual::skill::MAIN
    );
    f.json(&["skill", "--reference", "unknown"], 2);
    assert!(!f.repo.join("runs").exists());
}

#[test]
fn explicit_disposable_acceptance_and_exact_compare_keep_json_protocol() {
    let f = Fixture::new();
    f.json(&["check", "--run", "runs/one"], 1);
    let accepted = f.json(
        &[
            "accept",
            "--run",
            "runs/one",
            "--case",
            "demo--light",
            "--case",
            "demo--dark",
        ],
        0,
    );
    assert_eq!(accepted["data"]["accepted"].as_array().unwrap().len(), 2);
    let references = store::baseline_state(&f.repo.join("refs")).unwrap();
    let compared = f.json(&["compare", "--run", "runs/one"], 0);
    assert_eq!(compared["data"]["counts"]["same"], 2);
    assert_eq!(compared["data"]["coverage"], "not_independently_verified");
    let config: cargo_gpui_visual::config::Config =
        cargo_gpui_visual::config::Config::load(&f.repo.join("visual.toml")).unwrap();
    store::write_json(&f.repo.join("policy.json"), &config.source_policy).unwrap();
    let sources = f.json(&["sources", "--policy", "policy.json"], 0);
    assert!(sources["data"]["produce.sh"].as_str().is_some());
    assert!(sources["data"]["timings_ms"].is_null());
    assert_eq!(
        references,
        store::baseline_state(&f.repo.join("refs")).unwrap()
    );
}

#[test]
fn compaction_cli_is_explicit_lossless_and_configuration_independent() {
    let f = Fixture::new();
    f.json(&["check", "--run", "runs/one"], 1);
    f.json(
        &[
            "accept",
            "--run",
            "runs/one",
            "--case",
            "demo--light",
            "--case",
            "demo--dark",
        ],
        0,
    );
    let root = f.repo.join("refs");
    let path = root.join("baseline.json");
    let mut legacy: Value = store::read_json(&path).unwrap();
    let captures = legacy.as_object_mut().unwrap().remove("captures").unwrap();
    legacy["schema"] = json!(1);
    for case in legacy["cases"].as_object_mut().unwrap().values_mut() {
        case["sources"] = captures[case["capture_hash"].as_str().unwrap()]["sources"].clone();
    }
    store::write_json(&path, &legacy).unwrap();
    let before = store::baseline_state(&root).unwrap();
    fs::write(
        f.repo.join("visual.toml"),
        "invalid config must not be loaded",
    )
    .unwrap();
    fs::remove_file(f.repo.join("produce.sh")).unwrap();
    f.json(&["compact-baselines"], 2);
    f.json(&["compact-baselines", "--baselines", "refs"], 2);
    f.json(
        &[
            "compact-baselines",
            "--expected-hash",
            &before["baseline.json"],
        ],
        2,
    );
    f.json(
        &[
            "compact-baselines",
            "--baselines",
            "refs",
            "--expected-hash",
            "wrong",
        ],
        2,
    );
    let args = [
        "compact-baselines",
        "--baselines",
        "refs",
        "--expected-hash",
        &before["baseline.json"],
    ];
    let mut preview_args = args.to_vec();
    preview_args.push("--dry-run");
    let preview = f.json(&preview_args, 0);
    assert_eq!(preview["data"]["compaction"]["written"], false);
    assert_eq!(preview["data"]["accepted"], json!([]));
    assert_eq!(before, store::baseline_state(&root).unwrap());
    let result = f.json(&args, 0);
    assert_eq!(result["data"]["compaction"]["written"], true);
    assert_eq!(
        result["data"]["compaction"]["new_hash"],
        store::digest(&path).unwrap()
    );
    let hash = store::digest(&path).unwrap();
    let human = f.run(&[
        "compact-baselines",
        "--baselines",
        "refs",
        "--expected-hash",
        &hash,
    ]);
    assert!(human.status.success());
    assert!(String::from_utf8(human.stdout)
        .unwrap()
        .contains("No images, environment or timelines accepted"));
    assert!(!root.join(".accept.lock").exists());
    for (name, hash) in before.iter().filter(|(name, _)| name.ends_with(".png")) {
        assert_eq!(*hash, store::digest(&root.join(name)).unwrap());
    }
}

#[test]
fn argument_errors_have_json_and_help_is_command_specific() {
    let f = Fixture::new();
    let invalid = f.json(&["doctor", "--case", "demo--light"], 2);
    assert_eq!(invalid["error"]["code"], "invalid_arguments");
    let help = f.run(&["doctor", "--help"]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(!text.contains("--case"));
    assert!(!text.contains("--story"));
    assert!(text.contains("--json"));
    assert!(!f.repo.join("runs").exists());
}

#[test]
fn selected_scope_and_actionable_failure_do_not_grant_full_suite_evidence() {
    let f = Fixture::new();
    let unknown = f.json(&["check", "--story", "absent", "--report", "selected"], 2);
    assert_eq!(unknown["error"]["code"], "unknown_story");
    assert!(!f.repo.join("runs").exists());
    let selected = f.json(
        &[
            "check",
            "--story",
            "demo",
            "--report",
            "selected",
            "--run",
            "runs/selected",
        ],
        1,
    );
    assert_eq!(selected["data"]["scope"]["kind"], "selected");
    assert_eq!(selected["data"]["acceptance"], "blocked_selected_scope");
    fs::write(f.repo.join("produce.sh"), "echo noise; exit 4\n").unwrap();
    let failed = f.json(&["check", "--run", "runs/failure"], 2);
    assert_eq!(failed["error"]["code"], "producer_failed");
    assert!(failed["timings_ms"]["producer_ms"].as_f64().is_some());
    assert!(failed["timings_ms"]["total_ms"].as_f64().is_some());
    assert!(!f.repo.join("runs/failure/capture.json").exists());
    let auto_failed = f.json(&["check"], 2);
    assert!(auto_failed["data"]["artifacts"]["run"]
        .as_str()
        .is_some_and(|p| p.contains("/runs/")));
}
