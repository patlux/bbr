use super::*;

#[test]
fn legacy_schema_cannot_silently_read_a_selected_run_as_full() {
    let f = Fixture::new();
    f.capture("selected", &["tokens"], "none").unwrap();
    let path = f.repo.join("selected/capture.json");
    let mut m: store::Manifest = store::read_json(&path).unwrap();
    assert_eq!(m.schema, 2);
    m.schema = 1;
    store::write_json(&path, &m).unwrap();
    assert!(store::load_manifest(&f.repo.join("selected")).is_err());
}

#[test]
fn missing_catalog_legacy_producer_empty_request_and_stale_resolution_fail_before_producing() {
    let mut f = Fixture::new();
    assert!(f.config.select(&f.repo, &[]).is_err());
    f.config.story_catalog = None;
    assert!(f.capture("bad", &["tokens"], "none").is_err());
    f.config.story_catalog = Some("stories.json".into());
    let mut c = f.config.select(&f.repo, &["tokens".into()]).unwrap();
    c.environment_command = None;
    assert!(capture::capture(
        &f.repo,
        &c,
        Path::new("bad"),
        Path::new("refs"),
        Path::new("review")
    )
    .unwrap_err()
    .to_string()
    .contains("raw producer"));
    c.environment_command = f.config.environment_command.clone();
    fs::write(f.repo.join("stories.json"), "{}\n").unwrap();
    assert!(capture::capture(
        &f.repo,
        &c,
        Path::new("bad"),
        Path::new("refs"),
        Path::new("review")
    )
    .is_err());
    assert!(!f.repo.join("bad").exists());
}

#[test]
fn even_selecting_every_story_requires_explicit_producer_acknowledgement() {
    let f = Fixture::new();
    assert!(f
        .capture("bad", &["tokens", "opening", "other"], "receipt")
        .unwrap_err()
        .to_string()
        .contains("acknowledge"));
    assert!(!f.repo.join("bad/capture.json").exists());
}

#[test]
fn unrelated_baseline_corruption_and_environment_drift_are_not_hidden_by_selection() {
    let f = Fixture::new();
    f.establish();
    f.capture("selected", &["tokens"], "none").unwrap();
    let unrelated = f.repo.join("refs/other--light.png");
    let old = fs::read(&unrelated).unwrap();
    fs::write(&unrelated, b"corrupted").unwrap();
    let r = compare::compare(
        &f.repo,
        &f.repo.join("selected"),
        &f.repo.join("refs"),
        &f.repo.join("selected-review"),
    )
    .unwrap();
    assert!(!r.passed);
    assert!(r
        .errors
        .iter()
        .any(|e| e.contains("metadata/image mismatch")));
    fs::write(unrelated, old).unwrap();
    let metadata = f.repo.join("refs/baseline.json");
    let mut value: serde_json::Value = store::read_json(&metadata).unwrap();
    value["environment"]["old"] = json!(true);
    store::write_json(&metadata, &value).unwrap();
    let r = compare::compare(
        &f.repo,
        &f.repo.join("selected"),
        &f.repo.join("refs"),
        &f.repo.join("selected-review"),
    )
    .unwrap();
    assert!(!r.passed);
    assert!(r.environment_mismatch.is_some());
    let html = fs::read_to_string(f.repo.join("selected-review/index.html")).unwrap();
    assert!(!html.contains("accept --adopt-environment"));
}

#[test]
fn full_cli_clears_inherited_scope_and_selected_compare_requires_explicit_report() {
    let f = Fixture::new();
    let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"));
    command
        .arg("--repo")
        .arg(&f.repo)
        .args(["check", "--run", "full"])
        .env(ENVIRONMENT, "not-json");
    let output = cargo_gpui_visual::process::run(
        &mut command,
        &cargo_gpui_visual::process::Limits {
            timeout: std::time::Duration::from_secs(15),
            ..Default::default()
        },
        &AtomicBool::new(false),
        true,
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let m = store::load_manifest(&f.repo.join("full")).unwrap();
    assert_eq!(m.schema, 1);
    assert!(m.selection.is_none());
    assert_eq!(m.cases.len(), 10);
    assert!(!f.repo.join("full/selection.json").exists());
    let report_hash = store::digest(&f.repo.join("review/index.html")).unwrap();
    f.capture("selected", &["tokens"], "none").unwrap();
    assert_eq!(
        f.cli(&["compare", "--run", "selected"]).status.code(),
        Some(2)
    );
    assert_eq!(
        report_hash,
        store::digest(&f.repo.join("review/index.html")).unwrap()
    );
    assert_eq!(
        f.cli(&[
            "compare",
            "--run",
            "selected",
            "--report",
            "selected-review"
        ])
        .status
        .code(),
        Some(1)
    );
}

#[test]
fn catalog_paths_fail_closed_without_reading_escaping_or_symlinked_inputs() {
    let mut f = Fixture::new();
    for path in ["../stories.json", ".env", "/stories.json"] {
        f.config.story_catalog = Some(path.into());
        assert!(f.config.select(&f.repo, &["tokens".into()]).is_err());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(f.repo.join("stories.json"), f.repo.join("linked.json"))
            .unwrap();
        f.config.story_catalog = Some("linked.json".into());
        assert!(f.config.select(&f.repo, &["tokens".into()]).is_err());
    }
}

#[test]
fn partial_review_fixture_preserves_pairs_pages_groups_and_full_sequences() {
    let f = Fixture::new();
    f.establish();
    f.capture("selected", &["opening", "tokens"], "none")
        .unwrap();
    let output = std::env::var_os("GPUI_SELECTION_REVIEW_OUTPUT").map(PathBuf::from);
    let report = output
        .clone()
        .unwrap_or_else(|| f.repo.join("selected-review"));
    assert!(!report.exists(), "Choose a fresh fixture report directory");
    let r = compare::compare_with_options(
        &f.repo,
        &f.repo.join("selected"),
        &f.repo.join("refs"),
        &report,
        &cargo_gpui_visual::report::Options {
            groups: vec![cargo_gpui_visual::report::Group {
                title: "Tokens & pages".into(),
                prefixes: vec!["tokens-".into()],
            }],
        },
    )
    .unwrap();
    assert!(r.passed);
    assert_eq!(r.schema, 2);
    assert!(r.selection.is_some());
    assert_eq!(r.results.len(), 8);
    assert_eq!(r.sequences.len(), 2);
    let html = fs::read_to_string(report.join("index.html")).unwrap();
    assert!(html.contains("TEILLAUF · AUSWAHL PASS"));
    assert!(html.contains("Tokens &amp; pages"));
    assert!(!html.contains("other--light"));
    if output.is_some() {
        println!(
            "Synthetic scoped report (not app evidence): {}",
            report.display()
        );
    }
}
