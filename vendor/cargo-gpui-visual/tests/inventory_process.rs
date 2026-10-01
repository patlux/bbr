#![cfg(unix)]
//! Fake Git exists only on a spawned test process's PATH, never the parent PATH.
use cargo_gpui_visual::{
    process::{run, Limits},
    store::{source_hashes_with, Policy},
};
use std::{
    fs, os::unix::fs::PermissionsExt, process::Command, sync::atomic::AtomicBool, time::Duration,
};

#[test]
fn inventory_child() {
    let Ok(root) = std::env::var("VISUAL_INVENTORY_FIXTURE") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let policy = Policy {
        scopes: vec![".".into()],
        suffixes: vec![".rs".into()],
        artifacts: vec![],
    };
    let limits = Limits {
        timeout: Duration::from_millis(250),
        cleanup_timeout: Duration::from_secs(3),
        max_output_bytes: 4096,
    };
    let result = source_hashes_with(root, &policy, None, &limits, &AtomicBool::new(false));
    let message = format!("{:#}", result.unwrap_err());
    assert!(message.contains("deadline"), "{message}");
    let pid: i32 = fs::read_to_string(root.join("child.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    // Safety: signal 0 observes only the owned fixture child.
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}

#[test]
fn all_inventory_git_operations_have_deadlines_and_descendant_cleanup() {
    for hanging in ["ls-files", "toplevel", "head"] {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::create_dir(root.join("bin")).unwrap();
        fs::create_dir(root.join("sdk")).unwrap();
        let script = format!(
            r#"#!/bin/sh
set -eu
kind=$1
if [ "$1" = rev-parse ]; then
  if [ "$2" = --show-toplevel ]; then kind=toplevel; else kind=head; fi
fi
if [ "$kind" = '{hanging}' ]; then
  /bin/sleep 1000 & echo $! > "$VISUAL_INVENTORY_FIXTURE/child.pid"; wait
fi
case "$kind" in
  ls-files) printf 'sdk\0';;
  toplevel) printf '%s\n' "$VISUAL_INVENTORY_FIXTURE/sdk";;
  head) printf '0000000000000000000000000000000000000000\n';;
esac
"#
        );
        let git = root.join("bin/git");
        fs::write(&git, script).unwrap();
        fs::set_permissions(&git, fs::Permissions::from_mode(0o700)).unwrap();
        // macOS may spend the entire 250ms on the first execution security
        // check of a newly written script, before its descendant exists. Warm
        // only fixture executable startup with a non-hanging command under the
        // same bounded runner; the tested operation/cleanup deadline is unchanged.
        let ready = run(
            Command::new(&git).arg("--version"),
            &Limits {
                timeout: Duration::from_secs(8),
                ..Limits::default()
            },
            &AtomicBool::new(false),
            true,
        )
        .unwrap();
        assert!(ready.status.success());
        let mut child = Command::new(std::env::current_exe().unwrap());
        child
            .args(["--exact", "inventory_child", "--nocapture"])
            .env("VISUAL_INVENTORY_FIXTURE", &root)
            .env("PATH", root.join("bin"));
        let limit = Limits {
            timeout: Duration::from_secs(8),
            ..Limits::default()
        };
        let result = run(&mut child, &limit, &AtomicBool::new(false), true).unwrap();
        assert!(
            result.status.success(),
            "{hanging}: {}",
            String::from_utf8_lossy(&result.stdout)
        );
    }
}

#[test]
fn inventory_cancellation_fails_before_git_discovery() {
    let temp = tempfile::tempdir().unwrap();
    let policy = Policy {
        scopes: vec![".".into()],
        suffixes: vec![".rs".into()],
        artifacts: vec![],
    };
    let error = source_hashes_with(
        temp.path(),
        &policy,
        None,
        &Limits::default(),
        &AtomicBool::new(true),
    )
    .unwrap_err();
    assert!(error.to_string().contains("cancelled"));
}
