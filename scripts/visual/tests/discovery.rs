//! Native/build/probe commands are tripwires. Read-only routes must never run them.
use bbr_visual::{discovery, Suite};
use cargo_gpui_visual::{
    process::{self, Limits},
    store,
};
use serde_json::{json, Value};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};

fn cli(args: &[&str], root: &Path) -> (i32, Value) {
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    for name in ["mbx", "cargo", "rustc", "python3", "tart", "xcrun"] {
        let path = bin.join(name);
        fs::write(
            &path,
            "#!/bin/sh\nprintf invoked > \"$TRIPWIRE\"\nexit 99\n",
        )
        .unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let path = std::env::join_paths(
        std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let result = process::run(
        Command::new(env!("CARGO_BIN_EXE_bbr-visual"))
            .args(args)
            .arg("--json")
            .env("PATH", path)
            .env("TRIPWIRE", root.join("invoked"))
            .env("GPUI_VISUAL_OUTPUT", root.join("native-output"))
            .env("GPUI_VISUAL_HOST_GATE", root.join("host.lock")),
        &Limits::default(),
        &AtomicBool::new(false),
        true,
    )
    .unwrap();
    assert!(!root.join("invoked").exists());
    assert!(!root.join("native-output").exists());
    assert!(!root.join("host.lock").exists());
    (
        result.status.code().unwrap(),
        serde_json::from_slice(&result.stdout).unwrap(),
    )
}

#[test]
fn doctor_and_preview_are_structured_read_only_and_do_not_authorize_execution() {
    let temp = tempfile::tempdir().unwrap();
    let canonical = temp.path().canonicalize().unwrap();
    let root = canonical.as_path();
    let (code, doctor) = cli(&["doctor"], root);
    assert_eq!(code, 0);
    assert_eq!(doctor["schema"], 1);
    assert_eq!(doctor["data"]["producer_executed"], false);
    assert_eq!(doctor["data"]["environment_probed"], false);
    assert_eq!(doctor["data"]["execution"], "blocked_pending_tart_adapter");
    assert_eq!(doctor["data"]["suites"].as_array().unwrap().len(), 2);
    assert_eq!(
        doctor["data"]["suites"][0]["coverage"]["current_registry_verified"],
        false
    );
    let coverage = &doctor["data"]["suites"][0]["coverage"];
    assert_eq!(
        coverage["required_images"], 1872,
        "historical requirements stay intact"
    );
    let pending = coverage["pending"].as_array().unwrap();
    assert_eq!(pending.len(), 21);
    for base in [
        "gallery-31-address-production-gaps",
        "gallery-32-staging-edit-gaps",
        "gallery-33-favorite-image-pop-framing",
        "gallery-34-folder-hidden-count-states",
        "gallery-35-footer-focus-strip-framing",
        "gallery-37-plus-menu-renderer-drift",
        "gallery-22-stale-matrix-exclusions",
        "gallery-30-selection-collection-gaps",
        "gallery-30-windowed-framing",
        "gallery-07-state-projection",
        "gallery-08-iconography-gaps",
        "gallery-core-control-applicability",
        "gallery-18-radio-check-asset",
    ] {
        assert!(pending
            .iter()
            .any(|item| item["base"] == base && item["state"] == "withheld"));
    }
    for id in 49..=52 {
        assert!(
            pending
                .iter()
                .any(|item| item["base"] == format!("gallery-{id}-audit-pending")
                    && item["state"] == "withheld"
                    && item["reason"].as_str().is_some_and(|s| !s.is_empty())),
            "new story must remain visible in read-only diagnostics"
        );
    }
    let run = root.join("run");
    let report = root.join("report");
    let (code, preview) = cli(
        &[
            "check",
            "--dry-run",
            "--suite",
            "gallery",
            "--run",
            run.to_str().unwrap(),
            "--report",
            report.to_str().unwrap(),
        ],
        root,
    );
    assert_eq!(code, 0);
    assert_eq!(preview["data"]["dry_run"], true);
    assert_eq!(preview["data"]["producer_executed"], false);
    assert_eq!(preview["data"]["host_gate"]["acquired"], false);
    assert_eq!(preview["data"]["execution"], "blocked_pending_tart_adapter");
    assert!(!run.exists() && !report.exists());
}

#[test]
fn invalid_previews_and_unconfigured_selection_fail_without_fallback() {
    let temp = tempfile::tempdir().unwrap();
    for args in [
        vec!["check", "--dry-run"],
        vec!["check", "--dry-run", "--dry-run"],
        vec![
            "check",
            "--dry-run",
            "--suite",
            "gallery",
            "--run",
            "missing-report",
        ],
        vec![
            "check",
            "--dry-run",
            "--suite",
            "gallery",
            "--run",
            "same",
            "--report",
            "same",
        ],
        vec![
            "check",
            "--dry-run",
            "--suite",
            "gallery",
            "--run",
            "run",
            "--report",
            "report",
            "--story",
            "*",
        ],
        vec!["list", "--suite", "gallery"],
        vec!["doctor", "--story", "tokens"],
        vec!["inspect"],
        vec!["inspect", "--report", "missing", "--limit", "0"],
    ] {
        let (code, response) = cli(&args, temp.path());
        assert_eq!(code, 2, "{args:?}: {response}");
        assert_eq!(response["status"], "invalid");
        assert!(response["error"].is_object());
    }
}

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    // Copy only the immutable, allowlisted portable export, never other worktrees.
    let provenance: Value =
        store::read_json(&source.join("scripts/visual/tool-source.json")).unwrap();
    for name in provenance["files"].as_object().unwrap().keys() {
        let relative = PathBuf::from("vendor/cargo-gpui-visual").join(name);
        fs::create_dir_all(repo.join(&relative).parent().unwrap()).unwrap();
        fs::copy(source.join(&relative), repo.join(relative)).unwrap();
    }
    fs::create_dir_all(repo.join("scripts/visual")).unwrap();
    fs::copy(
        source.join("scripts/visual/tool-source.json"),
        repo.join("scripts/visual/tool-source.json"),
    )
    .unwrap();
    assert!(Command::new("git")
        .args(["init", "-q"])
        .arg(repo)
        .status()
        .unwrap()
        .success());
    fs::write(
        repo.join("visual.toml"),
        r#"schema = 1
capture = ["must-not-execute"]
environment_command = ["must-not-probe"]
baselines = "baselines"
runs = "runs"
report = "report"
story_catalog = "stories.json"
[source_policy]
scopes = ["visual.toml", "stories.json"]
suffixes = [".toml", ".json"]
"#,
    )
    .unwrap();
    store::write_json(
        &repo.join("stories.json"),
        &json!({"schema":1,"stories":[
            {"id":"tokens","label":"Tokens","cases":["tokens-page-00","tokens-page-01"]},
            {"id":"sidebar","label":"Sidebar","cases":["sidebar-page-00"]}
        ]}),
    )
    .unwrap();
    temp
}

#[test]
fn authored_discovery_and_selection_preview_preserve_complete_pairs_pages_and_order() {
    let temp = fixture();
    let canonical = temp.path().canonicalize().unwrap();
    let repo = canonical.as_path();
    let listed = discovery::list(repo, Suite::Browser).unwrap();
    assert_eq!(listed["catalog"]["stories"][0]["id"], "tokens");
    assert_eq!(listed["producer_selection_verified"], false);
    let run = repo.join("runs/preview");
    let report = repo.join("selected-review");
    let preview = discovery::preview(
        repo,
        Suite::Browser,
        &run,
        &report,
        &["sidebar".into(), "tokens".into()],
    )
    .unwrap();
    assert_eq!(preview["selection"]["stories"][0]["id"], "tokens");
    assert_eq!(preview["requirements"]["still_images"], 6);
    for requested in [
        vec!["unknown".into()],
        vec!["tokens".into(), "tokens".into()],
        vec!["".into()],
    ] {
        assert!(discovery::preview(repo, Suite::Browser, &run, &report, &requested).is_err());
    }
    assert!(!run.exists() && !report.exists() && !repo.join("baselines").exists());
    let config = repo.join("visual.toml");
    fs::write(
        &config,
        fs::read_to_string(&config).unwrap().replace(
            "scopes = [\"visual.toml\", \"stories.json\"]",
            "scopes = [\"visual.toml\"]",
        ),
    )
    .unwrap();
    assert!(discovery::list(repo, Suite::Browser).is_err());
    assert!(discovery::preview(repo, Suite::Browser, &run, &report, &["tokens".into()]).is_err());
}

#[test]
fn inspect_reads_only_explicit_stored_evidence_and_preserves_it_byte_for_byte() {
    let temp = tempfile::tempdir().unwrap();
    let report = temp.path().canonicalize().unwrap().join("report");
    fs::create_dir(&report).unwrap();
    let document = report.join("collection.json");
    store::write_json(&document, &json!({"schema":1,"passed":false,"suites":[]})).unwrap();
    let before = store::digest(&document).unwrap();
    let (code, response) = cli(
        &["inspect", "--report", report.to_str().unwrap()],
        temp.path(),
    );
    assert_eq!(code, 0, "{response}");
    assert_eq!(response["data"]["evidence"], "stored_unvalidated");
    assert_eq!(response["data"]["collection_integrity"], "not_revalidated");
    assert_eq!(before, store::digest(&document).unwrap());
    assert_eq!(fs::read_dir(&report).unwrap().count(), 1);
    fs::write(report.join(".review.lock"), "interrupted").unwrap();
    let (code, _) = cli(
        &["inspect", "--report", report.to_str().unwrap()],
        temp.path(),
    );
    assert_eq!(code, 2);
    assert_eq!(
        fs::read_to_string(report.join(".review.lock")).unwrap(),
        "interrupted"
    );
}
