use cargo_gpui_visual::{
    accept::accept,
    compare::{changed_rows, compare, Review},
    report,
    store::*,
};
use image::{Rgba, RgbaImage};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
struct Fixture {
    _temp: tempfile::TempDir,
    repo: PathBuf,
    run: PathBuf,
    base: PathBuf,
    report: PathBuf,
    cases: Vec<String>,
    env: Value,
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
        fs::write(repo.join("renderer.rs"), "production source").unwrap();
        let run = repo.join("run");
        fs::create_dir(&run).unwrap();
        let f = Self {
            run,
            base: repo.join("baselines"),
            report: repo.join("report"),
            repo,
            _temp: temp,
            cases: vec![
                "component-link--light".into(),
                "component-link--dark".into(),
            ],
            env: json!({"layer":"gpui","os":"pinned-test-os","scale":2}),
        };
        for c in &f.cases {
            f.image(c, 20, 12, [20, 30, 40, 255]);
        }
        f.seal();
        f
    }
    fn image(&self, c: &str, w: u32, h: u32, p: [u8; 4]) {
        RgbaImage::from_pixel(w, h, Rgba(p))
            .save(self.run.join(format!("{c}.png")))
            .unwrap();
    }
    fn seal(&self) {
        let policy = Policy {
            scopes: vec![".".into()],
            suffixes: vec![".rs".into()],
            artifacts: vec![],
        };
        write_json(
            &self.run.join("capture.json"),
            &Manifest {
                schema: 1,
                cases: self.cases.clone(),
                environment: self.env.clone(),
                images: self
                    .cases
                    .iter()
                    .map(|c| {
                        (
                            c.clone(),
                            digest(&self.run.join(format!("{c}.png"))).unwrap(),
                        )
                    })
                    .collect(),
                sources: source_hashes(&self.repo, &policy).unwrap(),
                source_policy: policy,
            },
        )
        .unwrap();
    }
    fn compare(&self) -> anyhow::Result<Review> {
        compare(&self.repo, &self.run, &self.base, &self.report)
    }
    fn accept(&self, cases: &[String], adopt: bool) -> anyhow::Result<()> {
        accept(
            &self.repo,
            &self.run,
            &self.base,
            &self.report,
            cases,
            adopt,
        )
    }
    fn establish(&self) {
        self.compare().unwrap();
        self.accept(&self.cases, false).unwrap();
    }
    fn doc(&self) -> String {
        fs::read_to_string(self.report.join("index.html")).unwrap()
    }
    fn manifest(&self, f: impl FnOnce(&mut Value)) {
        let p = self.run.join("capture.json");
        let mut m: Value = read_json(&p).unwrap();
        f(&mut m);
        write_json(&p, &m).unwrap();
    }
}
fn fails<T: std::fmt::Debug>(r: anyhow::Result<T>, needle: &str) {
    let e = format!("{:#}", r.unwrap_err());
    assert!(e.contains(needle), "{e} lacks {needle}");
}
#[test]
fn missing_baselines_never_write() {
    let f = Fixture::new();
    assert!(!f.compare().unwrap().passed);
    assert!(!f.base.exists());
    let d = f.doc();
    assert!(d.contains("keine Referenz"));
    assert!(!d.contains(">Wischen</label>"));
}
#[test]
fn selective_acceptance_and_repeatable_comparison() {
    let f = Fixture::new();
    f.compare().unwrap();
    f.accept(&f.cases[..1], false).unwrap();
    assert!(!f.base.join(format!("{}.png", f.cases[1])).exists());
    fails(f.accept(&f.cases[1..], false), "Baselines changed");
    f.compare().unwrap();
    f.accept(&f.cases[1..], false).unwrap();
    let before = baseline_state(&f.base).unwrap();
    for _ in 0..2 {
        assert!(f.compare().unwrap().passed);
    }
    assert_eq!(before, baseline_state(&f.base).unwrap());
}
#[test]
fn alpha_pixel_is_a_regression() {
    let f = Fixture::new();
    f.establish();
    let p = f.run.join(format!("{}.png", f.cases[0]));
    let mut image = read_png(&p).unwrap();
    image.put_pixel(3, 4, Rgba([20, 30, 40, 254]));
    image.save(p).unwrap();
    f.seal();
    let r = f.compare().unwrap();
    assert!(!r.passed);
    assert_eq!(r.results[0].changed_pixels, Some(1));
    assert_eq!(r.results[0].bounds, Some([3, 4, 4, 5]));
    assert_eq!(r.results[0].rows, vec![[4, 5]]);
}
#[test]
fn dimensions_fail() {
    let f = Fixture::new();
    f.establish();
    f.image(&f.cases[0], 21, 12, [0; 4]);
    f.seal();
    assert_eq!(
        f.compare().unwrap().results[0].reason.as_deref(),
        Some("dimensions")
    );
}
#[test]
fn missing_and_unexpected_candidate_fail() {
    let f = Fixture::new();
    f.image("other--light", 1, 1, [0; 4]);
    fails(f.compare(), "Incomplete capture");
    fs::remove_file(f.run.join("other--light.png")).unwrap();
    fs::remove_file(f.run.join(format!("{}.png", f.cases[0]))).unwrap();
    fails(f.compare(), "Incomplete capture");
}
#[test]
fn modified_candidate_and_reseal_invalidate_review() {
    let f = Fixture::new();
    f.compare().unwrap();
    f.image(&f.cases[0], 20, 12, [9; 4]);
    fails(f.accept(&f.cases, false), "Candidate changed");
    assert!(!f.base.exists());
    f.seal();
    fails(f.accept(&f.cases, false), "Capture changed");
}
#[test]
fn source_edits_and_additions_invalidate_review() {
    for name in ["renderer.rs", "new.rs"] {
        let f = Fixture::new();
        f.compare().unwrap();
        fs::write(f.repo.join(name), "edited").unwrap();
        fails(f.compare(), "source inventory changed");
        fails(f.accept(&f.cases, false), "source inventory changed");
        assert!(!f.base.exists());
    }
}
#[test]
fn baseline_edits_invalidate_review() {
    let f = Fixture::new();
    f.establish();
    f.compare().unwrap();
    RgbaImage::new(1, 1)
        .save(f.base.join(format!("{}.png", f.cases[0])))
        .unwrap();
    fails(f.accept(&f.cases, false), "Baselines changed");
    let r = f.compare().unwrap();
    assert!(!r.passed);
    assert!(!r.errors.is_empty());
}
#[test]
fn environment_requires_explicit_reviewed_adoption() {
    let mut f = Fixture::new();
    f.establish();
    f.env["os"] = json!("next-build");
    f.seal();
    let r = f.compare().unwrap();
    assert!(!r.passed);
    assert!(r.environment_mismatch.is_some());
    fails(f.accept(&[], false), "Explicit");
    fails(f.accept(&f.cases, false), "adopt-environment");
    f.accept(&[], true).unwrap();
    assert!(f.compare().unwrap().passed);
}
#[test]
fn environment_adoption_refuses_unreviewed_drift() {
    let mut f = Fixture::new();
    f.establish();
    let before = baseline_state(&f.base).unwrap();
    f.env["os"] = json!("next");
    for c in &f.cases {
        f.image(c, 20, 12, [90; 4]);
    }
    f.seal();
    f.compare().unwrap();
    fails(f.accept(&f.cases[..1], true), "Unreviewed");
    assert_eq!(before, baseline_state(&f.base).unwrap());
}
#[test]
fn adoption_without_mismatch_fails() {
    let f = Fixture::new();
    f.compare().unwrap();
    fails(f.accept(&f.cases, true), "requires a reviewed");
}
#[test]
fn separate_native_namespaces() {
    let mut f = Fixture::new();
    f.establish();
    for layer in ["ios", "android"] {
        f.env["layer"] = json!(layer);
        f.seal();
        f.base = f.repo.join(layer);
        f.establish();
        assert!(f.compare().unwrap().passed);
    }
}
#[test]
fn invalid_selection_has_no_writes() {
    for mode in 0..3 {
        let f = Fixture::new();
        f.compare().unwrap();
        let selected = match mode {
            0 => vec![],
            1 => vec![f.cases[0].clone(), f.cases[0].clone()],
            _ => vec![f.cases[0].clone(), "unknown--light".into()],
        };
        assert!(f.accept(&selected, false).is_err());
        assert!(!f.base.exists());
    }
}
#[test]
fn obsolete_cases_never_disappear_silently() {
    let mut f = Fixture::new();
    f.establish();
    for c in &f.cases {
        fs::remove_file(f.run.join(format!("{c}.png"))).unwrap();
    }
    f.cases = vec!["new--light".into(), "new--dark".into()];
    for c in &f.cases {
        f.image(c, 1, 1, [0; 4]);
    }
    f.seal();
    assert!(!f.compare().unwrap().errors.is_empty());
    fails(f.accept(&f.cases, false), "invalid comparison");
}
#[test]
fn schema_pairs_paths_and_policy_fail_closed() {
    for mode in 0..6 {
        let f = Fixture::new();
        f.manifest(|m| match mode {
            0 => {
                m.as_object_mut().unwrap().remove("source_policy");
            }
            1 => m["cases"] = json!([f.cases[0]]),
            2 => m["cases"] = json!(["../outside--light", "../outside--dark"]),
            3 => m["source_policy"]["scopes"] = json!(["../other"]),
            4 => m["sources"] = json!({"/outside.rs":"bad"}),
            _ => m["schema"] = json!(9),
        });
        assert!(f.compare().is_err(), "mode {mode}");
    }
}
#[test]
fn directories_must_be_disjoint_for_compare_and_accept() {
    let f = Fixture::new();
    for p in [&f.run, &f.base, &f.base.join("report")] {
        fails(compare(&f.repo, &f.run, &f.base, p), "disjoint");
        fails(
            accept(&f.repo, &f.run, &f.base, p, &f.cases, false),
            "disjoint",
        );
    }
}
#[test]
fn stale_report_images_removed() {
    let mut f = Fixture::new();
    f.establish();
    f.compare().unwrap();
    let p = f.report.join(&f.cases[0]);
    assert!(p.join("expected.png").exists());
    f.base = f.repo.join("new-base");
    f.compare().unwrap();
    assert!(!p.join("expected.png").exists());
    assert!(!p.join("diff.png").exists());
}
#[test]
fn unchanged_cannot_be_blindly_accepted() {
    let f = Fixture::new();
    f.establish();
    f.compare().unwrap();
    fails(f.accept(&f.cases, false), "Not a reviewed");
}
#[test]
fn report_preserves_controls_and_csp() {
    let f = Fixture::new();
    f.establish();
    f.image(&f.cases[0], 20, 12, [90; 4]);
    f.seal();
    f.compare().unwrap();
    let d = f.doc();
    for text in [
        ">2-up</label>",
        ">Wischen</label>",
        ">Überblenden</label>",
        ">Diff</label>",
        "class=\"handle\" role=\"slider\"",
        "class=\"blend\" type=\"range\"",
        "class=\"m-crop\"",
        "--w:",
        "id=\"crop-all\"",
        "class=\"group global-mode\" hidden",
        "--shot:70vh",
        "Alle Snapshots",
        "default-src 'none'",
    ] {
        assert!(d.contains(text), "{text}");
    }
    assert_eq!(d.matches("<script").count(), 1);
    assert!(d.contains(&format!("script-src {}\"", report::script_hash())));
    assert!(d.contains(&format!("<script>{}</script>", report::SCRIPT)));
    assert!(!d.contains("name=\"mode-component-link--dark\""));
    assert_eq!(d.matches("<span class=\"marks\">").count(), 5);
    for id in [
        "h-small",
        "h-medium",
        "h-fit",
        "h-tall",
        "h-full",
        "only-findings",
    ] {
        assert!(d.contains(id));
    }
}
#[test]
fn report_missing_baseline_and_gallery() {
    let f = Fixture::new();
    f.compare().unwrap();
    let d = f.doc();
    for c in &f.cases {
        assert!(d.contains(&format!("alt=\"Neu: {c}\"")));
        assert!(d.contains(&format!("src=\"{c}/actual.png\"")));
    }
    assert!(!d.contains("expected.png"));
    assert!(!d.contains("id=\"crop-all\""));
}
#[test]
fn row_marks_merge_and_bound() {
    assert_eq!(
        changed_rows(&[10, 11, 15, 120], 40),
        vec![[10, 16], [120, 121]]
    );
    assert!(changed_rows(&(0..400).step_by(10).collect::<Vec<_>>(), 5).len() <= 5);
}
#[test]
fn source_inventory_excludes_secrets_and_baselines() {
    let f = Fixture::new();
    fs::write(f.repo.join(".env.rs"), "not read").unwrap();
    fs::create_dir(&f.base).unwrap();
    fs::write(f.base.join("other.rs"), "not a source").unwrap();
    let m = load_manifest(&f.run).unwrap();
    check_sources(&f.repo, &m).unwrap();
    for p in ["../other", "/absolute", ".env"] {
        let policy = Policy {
            scopes: vec![p.into()],
            suffixes: vec![".rs".into()],
            artifacts: vec![],
        };
        assert!(source_hashes(&f.repo, &policy).is_err());
    }
}
#[cfg(unix)]
#[test]
fn symlink_sources_images_reports_refused() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    symlink(f.repo.join("renderer.rs"), f.repo.join("link.rs")).unwrap();
    fails(f.compare(), "Symlink");
    fs::remove_file(f.repo.join("link.rs")).unwrap();
    symlink(&f.run, &f.report).unwrap();
    assert!(f.compare().is_err());
}
#[test]
fn command_line_cargo_prefix_and_exit_codes() {
    let f = Fixture::new();
    let binary = env!("CARGO_BIN_EXE_cargo-gpui-visual");
    let output = Command::new(binary)
        .args(["gpui-visual", "compare", "--repo"])
        .arg(&f.repo)
        .arg("--run")
        .arg(&f.run)
        .arg("--baselines")
        .arg(&f.base)
        .arg("--report")
        .arg(&f.report)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    f.accept(&f.cases, false).unwrap();
    assert!(f.compare().unwrap().passed);
    let invalid = Command::new(binary)
        .args(["accept", "--repo"])
        .arg(&f.repo)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(2));
}
#[test]
fn submodule_revision_and_dirty_sources_are_inputs() {
    let f = Fixture::new();
    let child = f.repo.join("sdk");
    fs::create_dir(&child).unwrap();
    assert!(Command::new("git")
        .args(["init", "-q"])
        .arg(&child)
        .status()
        .unwrap()
        .success());
    fs::write(child.join("lib.rs"), "source").unwrap();
    git(&child, &["add", "lib.rs"]);
    git(
        &child,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "fixture",
        ],
    );
    git(&f.repo, &["add", "sdk"]);
    let policy = Policy {
        scopes: vec!["sdk".into()],
        suffixes: vec![".rs".into()],
        artifacts: vec![],
    };
    let before = source_hashes(&f.repo, &policy).unwrap();
    assert!(before.contains_key("sdk@git"));
    assert!(before.contains_key("sdk/lib.rs"));
    fs::write(child.join("lib.rs"), "dirty").unwrap();
    assert_ne!(before, source_hashes(&f.repo, &policy).unwrap());
}
#[cfg(unix)]
#[test]
fn raw_capture_command_seals_only_stable_inputs() {
    for mutation in ["", "source", "environment", "fail"] {
        let mut f = Fixture::new();
        f.run = f.repo.join("fresh");
        fs::write(
            f.repo.join("environment.json"),
            serde_json::to_vec(&f.env).unwrap(),
        )
        .unwrap();
        let change = match mutation {
            "source" => "printf changed > renderer.rs",
            "environment" => "printf '{}' > environment.json",
            "fail" => "exit 3",
            _ => "true",
        };
        fs::write(f.repo.join("produce.sh"), format!(
            "set -eu\nmkdir \"$GPUI_VISUAL_OUTPUT\"\ncp run/*.png \"$GPUI_VISUAL_OUTPUT/\"\nprintf '%s' '{}' > \"$GPUI_VISUAL_OUTPUT/inventory.json\"\n{change}\n",
            serde_json::to_string(&f.cases).unwrap()
        )).unwrap();
        fs::write(
            f.repo.join("visual.toml"),
            r#"
schema = 1
capture = ["sh", "produce.sh"]
environment_command = ["cat", "environment.json"]
baselines = "baselines"
report = "report"
runs = "runs"
[source_policy]
scopes = ["."]
suffixes = [".rs"]
"#,
        )
        .unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"))
            .args(["check", "--repo"])
            .arg(&f.repo)
            .args(["--run", "fresh"])
            .output()
            .unwrap();
        if mutation.is_empty() {
            assert_eq!(
                result.status.code(),
                Some(1),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let m = load_manifest(&f.run).unwrap();
            check_sources(&f.repo, &m).unwrap();
            validate_images(&f.run, &m).unwrap();
            assert!(!f.base.exists());
        } else {
            assert_eq!(result.status.code(), Some(2));
            assert!(!f.run.join("capture.json").exists());
            assert!(!f.report.exists());
        }
    }
}
#[cfg(unix)]
#[test]
fn sealed_producer_cannot_narrow_the_configured_source_policy() {
    let f = Fixture::new();
    // Both policies currently see renderer.rs, but the producer omits future .svg inputs.
    fs::write(
        f.repo.join("visual.toml"),
        r#"
schema = 1
capture = ["sh", "-c", "mkdir \"$GPUI_VISUAL_OUTPUT\"; cp run/* \"$GPUI_VISUAL_OUTPUT/\""]
baselines = "baselines"
report = "report"
runs = "runs"
[source_policy]
scopes = ["."]
suffixes = [".rs", ".svg"]
"#,
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"))
        .args(["check", "--repo"])
        .arg(&f.repo)
        .args(["--run", "fresh"])
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains("source policy"));
    assert!(!f.report.exists());
}
#[test]
fn validate_resolves_run_relative_to_selected_repo() {
    let f = Fixture::new();
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"))
        .args(["validate", "--repo"])
        .arg(&f.repo)
        .args(["--run", "run"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn ignored_package_artifacts_are_explicit_inputs() {
    let f = Fixture::new();
    fs::write(f.repo.join(".gitignore"), "packages/\n").unwrap();
    fs::create_dir(f.repo.join("packages")).unwrap();
    fs::write(f.repo.join("packages/lib.rs"), "helper").unwrap();
    let mut m = load_manifest(&f.run).unwrap();
    m.source_policy.artifacts = vec!["packages".into()];
    m.sources = source_hashes(&f.repo, &m.source_policy).unwrap();
    assert!(m.sources.contains_key("packages/lib.rs"));
    write_json(&f.run.join("capture.json"), &m).unwrap();
    f.compare().unwrap();
    fs::write(f.repo.join("packages/new-input"), "unlisted").unwrap();
    fails(f.compare(), "source inventory changed");
    fails(f.accept(&f.cases, false), "source inventory changed");
    for path in [".", "./", "././", "../elsewhere", ".env.rs", "baselines"] {
        m.source_policy.artifacts = vec![path.into()];
        assert!(source_hashes(&f.repo, &m.source_policy).is_err());
    }
}
fn git(root: &Path, args: &[&str]) {
    assert!(Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
        .status
        .success());
}
