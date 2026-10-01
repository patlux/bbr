//! Cross-language transport using actual Rust staging and synthetic executables.
//! The Python installer runs only on fresh temp paths; no Tart/native launch.
use bbr_visual::{payload, Suite};
use cargo_gpui_visual::{config::Config, process, store};
use serde_json::json;
use std::{
    fs, os::unix::fs::PermissionsExt, path::Path, process::Command, sync::atomic::AtomicBool,
};

#[test]
fn rust_staged_suites_install_byte_exactly_at_compiled_paths() {
    let cancelled = AtomicBool::new(false);
    for suite in [Suite::Gallery, Suite::Browser] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let repo = root.join("compiled-repo");
        fs::create_dir_all(repo.join("target/release/deps")).unwrap();
        process::run(
            Command::new("git").args(["init", "-q"]).arg(&repo),
            &process::Limits::default(),
            &cancelled,
            true,
        )
        .unwrap();
        fs::write(repo.join("source.rs"), "// real staged fixture source\n").unwrap();
        let mut artifacts = Vec::new();
        for (name, test) in [
            ("bbr-gallery", false),
            ("bbr", false),
            ("e2e_visual_shell_capture", true),
        ] {
            let path = repo.join(if test {
                format!("target/release/deps/{name}-fixture")
            } else {
                format!("target/release/{name}")
            });
            fs::write(&path, "synthetic artifact, must never execute\n").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o500)).unwrap();
            artifacts
                .push(json!({"target":{"name":name},"profile":{"test":test},"executable":path}));
        }
        let config = Config::from_toml(
            r#"schema=1
capture=["never-executed"]
baselines="baselines"
report="report"
runs="runs"
[source_policy]
scopes=["source.rs"]
suffixes=[".rs"]
"#,
        )
        .unwrap();
        let sources = payload::source_snapshot(&repo, &config, &cancelled).unwrap();
        let share = root.join("share");
        let payload = payload::stage(
            &repo, &config, &sources, &artifacts, suite, &share, &cancelled,
        )
        .unwrap();
        payload::verify(&share, &payload, &cancelled).unwrap();
        // Preserve the fixture build tree while making its compiled destination
        // fresh, exactly as it will be in a new guest. No path rewriting.
        fs::rename(&repo, root.join("build-source")).unwrap();
        let result = process::run(
            Command::new("python3")
                .args([
                    "-I",
                    "-B",
                    "-c",
                    r#"
import json, sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
import vm_install
share, digest, repo, suite = Path(sys.argv[2]), sys.argv[3], Path(sys.argv[4]), sys.argv[5]
installed = vm_install.install(share, digest, repo, suite)
assert installed == vm_install.verify_installed(share, digest, repo, suite)
print(json.dumps(installed))
"#,
                ])
                .arg(Path::new(env!("CARGO_MANIFEST_DIR")))
                .arg(&share)
                .arg(store::digest(&share.join("payload.json")).unwrap())
                .arg(&repo)
                .arg(suite.id()),
            &process::Limits::default(),
            &cancelled,
            true,
        )
        .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stdout)
        );
        let installed: store::Hashes = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(
            installed,
            payload
                .files
                .iter()
                .map(|entry| (
                    entry.guest.to_string_lossy().into_owned(),
                    entry.sha256.clone()
                ))
                .collect()
        );
        assert!(!repo.join(".git").exists());
        assert!(!repo.join("payload.json").exists());
    }
}
