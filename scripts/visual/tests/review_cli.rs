//! Actual stored-review CLI, no producer/build/probe or native execution.
use cargo_gpui_visual::store;
use serde_json::Value;
use std::{
    fs, os::unix::fs::PermissionsExt, path::Path, process::Command, sync::atomic::AtomicBool,
};

fn cli(root: &Path, args: &[&str]) -> (i32, Value) {
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    for name in ["mbx", "cargo", "rustc", "python3", "tart", "xcrun"] {
        let file = bin.join(name);
        fs::write(
            &file,
            "#!/bin/sh\nprintf invoked > \"$TRIPWIRE\"\nexit 99\n",
        )
        .unwrap();
        fs::set_permissions(file, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let path = std::env::join_paths(
        std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let out = cargo_gpui_visual::process::run(
        Command::new(env!("CARGO_BIN_EXE_bbr-visual"))
            .arg("compare")
            .args(args)
            .arg("--json")
            .env("PATH", path)
            .env("TRIPWIRE", root.join("invoked"))
            .env("GPUI_VISUAL_HOST_GATE", root.join("host.lock")),
        &Default::default(),
        &AtomicBool::new(false),
        true,
    )
    .unwrap();
    assert!(!root.join("invoked").exists());
    assert!(!root.join("host.lock").exists());
    (
        out.status.code().unwrap(),
        serde_json::from_slice(&out.stdout).unwrap(),
    )
}

#[test]
fn actual_cli_never_builds_probes_captures_or_chooses_latest() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let report = root.join("review");
    let missing = root.join("missing");
    let (code, result) = cli(
        &root,
        &[
            "--gallery-run",
            missing.to_str().unwrap(),
            "--report",
            report.to_str().unwrap(),
        ],
    );
    assert_eq!(code, 1);
    assert_eq!(result["data"]["scope"], "collection");
    assert_eq!(result["data"]["producer_executed"], false);
    assert_eq!(result["data"]["environment_probed"], false);
    assert_eq!(result["data"]["baseline_writes"], false);
    assert!(!missing.exists());
    let hash = store::digest(&report.join("collection.json")).unwrap();
    for args in [
        vec![],
        vec!["--latest"],
        vec![
            "--gallery-run",
            missing.to_str().unwrap(),
            "--report",
            report.to_str().unwrap(),
        ],
    ] {
        let (code, response) = cli(&root, &args);
        assert_eq!(code, 2);
        assert!(response["error"].is_object());
    }
    assert_eq!(
        store::digest(&report.join("collection.json")).unwrap(),
        hash
    );
}
