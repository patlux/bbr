//! Transfer contracts use tiny executable fixtures, never a VM or native renderer.
use bbr_visual::{payload, Suite};
use cargo_gpui_visual::{config::Config, store};
use serde_json::{json, Value};
use std::{
    fs,
    os::unix::fs::{symlink, PermissionsExt},
    path::PathBuf,
    process::Command,
    sync::atomic::AtomicBool,
};

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    repo: PathBuf,
    config: Config,
    artifacts: Vec<Value>,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let repo = root.join("repo");
        fs::create_dir(&repo).unwrap();
        assert!(Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .status()
            .unwrap()
            .success());
        fs::write(repo.join("source.rs"), "// authored source\n").unwrap();
        // Neither this synthetic forbidden file nor reports/whole targets may be transferred.
        fs::write(
            repo.join(".env.fixture"),
            "synthetic tripwire, not credentials",
        )
        .unwrap();
        fs::write(repo.join("ignored.txt"), "unselected").unwrap();
        let target = root.join("managed-target");
        fs::create_dir_all(target.join("release/deps")).unwrap();
        symlink(&target, repo.join("target")).unwrap();
        let mut artifacts = Vec::new();
        for (name, test) in [
            ("bbr-gallery", false),
            ("bbr-visual", false),
            ("bbr", false),
            ("e2e_visual_shell_capture", true),
        ] {
            let artifact_target = if name == "bbr-visual" {
                let consumer_target = repo.join("scripts/visual/target");
                fs::create_dir_all(consumer_target.parent().unwrap()).unwrap();
                symlink(&target, &consumer_target).unwrap();
                consumer_target
            } else {
                repo.join("target")
            };
            let relative = if test {
                format!("release/deps/{name}-fixture")
            } else {
                format!("release/{name}")
            };
            fs::write(target.join(&relative), "#!/bin/sh\nexit 98\n").unwrap();
            fs::set_permissions(target.join(&relative), fs::Permissions::from_mode(0o700)).unwrap();
            artifacts.push(json!({"reason":"compiler-artifact","target":{"name":name},"profile":{"test":test},"executable":artifact_target.join(relative)}));
        }
        let config = Config::from_toml(
            r#"schema=1
capture=["not-executed"]
baselines="baselines"
report="report"
runs="runs"
[source_policy]
scopes=["source.rs"]
suffixes=[".rs"]
"#,
        )
        .unwrap();
        Self {
            _temp: temp,
            root,
            repo,
            config,
            artifacts,
        }
    }
    fn sources(&self) -> store::Hashes {
        payload::source_snapshot(&self.repo, &self.config, &AtomicBool::new(false)).unwrap()
    }
}

#[test]
fn copies_only_sealed_sources_and_exact_suite_artifacts_at_compiled_paths() {
    let f = Fixture::new();
    let sources = f.sources();
    for (suite, count) in [(Suite::Gallery, 3), (Suite::Browser, 4)] {
        let destination = f.root.join(suite.id());
        let manifest = payload::stage(
            &f.repo,
            &f.config,
            &sources,
            &f.artifacts,
            suite,
            &destination,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(manifest.files.len(), count);
        assert_eq!(manifest.sources, sources);
        for entry in &manifest.files {
            assert_eq!(
                store::digest(&destination.join(&entry.payload)).unwrap(),
                entry.sha256
            );
            assert!(entry.guest.starts_with(&f.repo));
        }
        assert!(!destination.join("source/.env.fixture").exists());
        assert!(!destination.join("source/ignored.txt").exists());
        assert!(!destination.join("source/target").exists());
        payload::verify(&destination, &manifest, &AtomicBool::new(false)).unwrap();
        assert!(payload::stage(
            &f.repo,
            &f.config,
            &sources,
            &f.artifacts,
            suite,
            &destination,
            &AtomicBool::new(false)
        )
        .is_err());
    }
}

#[test]
fn stale_source_or_invalid_artifact_never_creates_a_payload() {
    for mutation in [
        "source",
        "missing",
        "duplicate",
        "symlink",
        "outside",
        "cancelled",
        "sensitive",
    ] {
        let mut f = Fixture::new();
        let sources = f.sources();
        let binary = PathBuf::from(f.artifacts[0]["executable"].as_str().unwrap());
        match mutation {
            "source" => fs::write(f.repo.join("source.rs"), "changed after build started").unwrap(),
            "missing" => {
                f.artifacts.remove(0);
            }
            "duplicate" => f.artifacts.push(f.artifacts[0].clone()),
            "symlink" => {
                fs::remove_file(&binary).unwrap();
                symlink(f.repo.join("source.rs"), &binary).unwrap();
            }
            "outside" => {
                f.artifacts[0]["executable"] =
                    json!(f.root.join("managed-target/release/bbr-gallery"));
            }
            "sensitive" => {
                // Authored scope escalation is refused before reading a payload source.
                f.config.source_policy.scopes = vec!["secrets".into()];
                fs::create_dir(f.repo.join("secrets")).unwrap();
                fs::write(f.repo.join("secrets/synthetic.rs"), "not a credential").unwrap();
            }
            _ => {}
        }
        let destination = f.root.join("payload");
        assert!(
            payload::stage(
                &f.repo,
                &f.config,
                &sources,
                &f.artifacts,
                Suite::Gallery,
                &destination,
                &AtomicBool::new(mutation == "cancelled")
            )
            .is_err(),
            "{mutation}"
        );
        assert!(!destination.exists(), "{mutation}");
    }
}

#[test]
fn transfer_mutation_missing_extra_symlink_and_mode_changes_cannot_verify() {
    for mutation in [
        "bytes",
        "missing",
        "extra",
        "symlink",
        "mode",
        "manifest",
        "manifest-symlink",
    ] {
        let f = Fixture::new();
        let destination = f.root.join("payload");
        let manifest = payload::stage(
            &f.repo,
            &f.config,
            &f.sources(),
            &f.artifacts,
            Suite::Gallery,
            &destination,
            &AtomicBool::new(false),
        )
        .unwrap();
        let file = destination.join("source/source.rs");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
        match mutation {
            "bytes" => fs::write(&file, "changed").unwrap(),
            "missing" => fs::remove_file(&file).unwrap(),
            "extra" => fs::write(destination.join("unexpected"), "extra").unwrap(),
            "symlink" => {
                fs::remove_file(&file).unwrap();
                symlink(f.repo.join("source.rs"), &file).unwrap();
            }
            "mode" => fs::set_permissions(&file, fs::Permissions::from_mode(0o700)).unwrap(),
            "manifest" => fs::write(destination.join("payload.json"), "{}").unwrap(),
            "manifest-symlink" => {
                fs::rename(
                    destination.join("payload.json"),
                    f.root.join("manifest-copy"),
                )
                .unwrap();
                symlink(
                    f.root.join("manifest-copy"),
                    destination.join("payload.json"),
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            payload::verify(&destination, &manifest, &AtomicBool::new(false)).is_err(),
            "{mutation}"
        );
    }
}

#[test]
fn payload_destinations_reject_repository_overlap_and_symlink_ancestors() {
    let f = Fixture::new();
    symlink(&f.root, f.root.join("alias")).unwrap();
    for destination in [
        f.repo.join("payload"),
        f.root.join("alias/payload"),
        PathBuf::from("relative"),
    ] {
        assert!(payload::stage(
            &f.repo,
            &f.config,
            &f.sources(),
            &f.artifacts,
            Suite::Gallery,
            &destination,
            &AtomicBool::new(false)
        )
        .is_err());
    }
    assert!(!f.repo.join("payload").exists());
    assert!(!f.root.join("payload").exists());
}
