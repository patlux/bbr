#![cfg(unix)]
use cargo_gpui_visual::{
    gate::{Gate, Options},
    process::{run, Limits},
    store::digest,
};
use std::{fs, os::fd::AsRawFd, process::Command, sync::atomic::AtomicBool, time::Duration};

fn options(root: &std::path::Path) -> Options {
    Options {
        path: root.join("host.lock"),
        compatibility: vec![],
        wait: Duration::from_millis(80),
    }
}
#[test]
fn serial_lease_and_clean_finish_reuse_the_same_inode() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let o = options(&root);
    let gate = Gate::acquire(&o, &AtomicBool::new(false)).unwrap();
    let before = fs::read(&o.path).unwrap();
    use std::os::unix::fs::MetadataExt;
    let inode = fs::metadata(&o.path).unwrap().ino();
    assert!(Gate::acquire(&o, &AtomicBool::new(false)).is_err());
    assert_eq!(before, fs::read(&o.path).unwrap());
    gate.finish().unwrap();
    assert_eq!(inode, fs::metadata(&o.path).unwrap().ino());
    Gate::acquire(&o, &AtomicBool::new(false))
        .unwrap()
        .finish()
        .unwrap();
}
#[test]
fn dropping_unfinished_lease_preserves_stale_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let o = options(&temp.path().canonicalize().unwrap());
    drop(Gate::acquire(&o, &AtomicBool::new(false)).unwrap());
    let before = fs::read(&o.path).unwrap();
    let error = Gate::acquire(&o, &AtomicBool::new(false)).unwrap_err();
    assert!(error.to_string().contains("unfinished"));
    assert_eq!(before, fs::read(&o.path).unwrap());
    assert!(
        Gate::recover(
            &o,
            &digest(&o.path).unwrap(),
            "reviewed fixture",
            &AtomicBool::new(false)
        )
        .is_err(),
        "live owner cannot be recovered"
    );
}
#[test]
fn same_user_host_identity_is_repository_independent() {
    let first = Options::host().unwrap();
    let second = Options::host().unwrap();
    assert_eq!(first.path, second.path);
    assert_eq!(
        first.path.parent().unwrap(),
        std::fs::canonicalize("/tmp").unwrap()
    );
    assert!(first.compatibility.is_empty());
    // Resolving the path does not open, create or recover the real host gate.
}

#[test]
fn compatibility_lock_is_flock_compatible_and_never_rewritten() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let mut o = options(&root);
    let legacy = root.join("bbr.lock");
    fs::write(&legacy, b"legacy holder metadata").unwrap();
    o.compatibility.push(legacy.clone());
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&legacy)
        .unwrap();
    assert_eq!(
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0
    );
    assert!(Gate::acquire(&o, &AtomicBool::new(false)).is_err());
    assert_eq!(fs::read(&legacy).unwrap(), b"legacy holder metadata");
    assert_eq!(unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_UN) }, 0);
    Gate::acquire(&o, &AtomicBool::new(false))
        .unwrap()
        .finish()
        .unwrap();
    assert_eq!(fs::read(&legacy).unwrap(), b"legacy holder metadata");
}
#[test]
fn invalid_paths_state_and_cancellation_fail_closed() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let o = options(&root);
    assert!(Gate::acquire(&o, &AtomicBool::new(true)).is_err());
    assert!(!o.path.exists());
    fs::write(&o.path, b"malformed").unwrap();
    assert!(Gate::acquire(&o, &AtomicBool::new(false)).is_err());
    assert_eq!(fs::read(&o.path).unwrap(), b"malformed");
    let mut linked = o.clone();
    linked.path = root.join("link.lock");
    symlink(&o.path, &linked.path).unwrap();
    assert!(Gate::acquire(&linked, &AtomicBool::new(false)).is_err());
    fs::hard_link(&o.path, root.join("hard.lock")).unwrap();
    assert!(Gate::acquire(&o, &AtomicBool::new(false)).is_err());
}
#[test]
fn timeout_cleanup_precedes_lease_release() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let o = options(&root);
    let mut gate = Gate::acquire(&o, &AtomicBool::new(false)).unwrap();
    let mut child = Command::new("sh");
    child
        .args(["-c", "trap '' TERM; sleep 1000 & echo $! > child.pid; wait"])
        .current_dir(&root);
    let limits = Limits {
        timeout: Duration::from_millis(150),
        cleanup_timeout: Duration::from_secs(3),
        ..Limits::default()
    };
    assert!(gate
        .execute(&mut child, &limits, &AtomicBool::new(false), true)
        .unwrap_err()
        .to_string()
        .contains("deadline"));
    assert!(Gate::acquire(&o, &AtomicBool::new(false)).is_err());
    let pid: i32 = fs::read_to_string(root.join("child.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    gate.finish().unwrap();
    Gate::acquire(&o, &AtomicBool::new(false))
        .unwrap()
        .finish()
        .unwrap();
}
#[test]
fn replaced_gate_file_poisons_the_lease_and_never_clears_foreign_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let o = options(&root);
    let mut gate = Gate::acquire(&o, &AtomicBool::new(false)).unwrap();
    let mut command = Command::new("sh");
    command
        .args([
            "-c",
            "mv host.lock interrupted.lock; printf foreign > host.lock",
        ])
        .current_dir(&root);
    assert!(gate
        .execute(
            &mut command,
            &Limits::default(),
            &AtomicBool::new(false),
            true
        )
        .is_err());
    assert!(gate.finish().is_err());
    assert_eq!(fs::read(&o.path).unwrap(), b"foreign");
    let interrupted: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("interrupted.lock")).unwrap()).unwrap();
    assert_eq!(interrupted["active"], true);
    assert!(Gate::acquire(&o, &AtomicBool::new(false)).is_err());
}

#[test]
fn crashed_holder_fixture() {
    let Ok(root) = std::env::var("VISUAL_GATE_FIXTURE") else {
        return;
    };
    let o = options(root.as_ref());
    let mut gate = Gate::acquire(&o, &AtomicBool::new(false)).unwrap();
    if std::env::var_os("VISUAL_GATE_EXECUTE").is_some() {
        // Parent failures before PID handoff must ask the real runner to clean
        // up rather than SIGKILLing an as-yet-unidentified producer.
        let cancelled = std::sync::Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(libc::SIGTERM, cancelled.clone()).unwrap();
        let mut command = Command::new("sh");
        command
            .args([
                "-c",
                "echo $$ > producer.tmp; mv producer.tmp producer.pid; exec sleep 1000",
            ])
            .current_dir(&root);
        let _ = gate.execute(
            &mut command,
            &Limits {
                timeout: Duration::from_secs(10),
                cleanup_timeout: Duration::from_secs(3),
                ..Limits::default()
            },
            &cancelled,
            false,
        );
    }
    // Deliberately bypass Drop, representing an interrupted lease.
    std::process::exit(0);
}
#[test]
fn killed_supervisor_does_not_release_a_surviving_producers_lease() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let o = options(&root);
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "crashed_holder_fixture"])
        .env("VISUAL_GATE_FIXTURE", &root)
        .env("VISUAL_GATE_EXECUTE", "1")
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    // Own cleanup even if an assertion fails. This fixture intentionally kills
    // its supervisor before the descendant, unlike the production runner.
    struct Owned {
        child: std::process::Child,
        group: Option<i32>,
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            // Retain the supervisor until its runner has cleaned any producer
            // whose PGID the parent has not received yet. Intentional SIGKILL
            // below happens only after group ownership has been recorded here.
            if self.child.try_wait().ok().flatten().is_none() {
                unsafe {
                    libc::kill(self.child.id() as i32, libc::SIGTERM);
                }
                let deadline = std::time::Instant::now() + Duration::from_secs(15);
                while self.child.try_wait().ok().flatten().is_none() {
                    if std::time::Instant::now() >= deadline {
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
            if let Some(group) = self.group.take() {
                unsafe {
                    libc::kill(-group, libc::SIGKILL);
                }
            }
        }
    }
    let mut owned = Owned { child, group: None };
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !root.join("producer.pid").exists() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    let group: i32 = fs::read_to_string(root.join("producer.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    owned.group = Some(group);
    // The intent marker is durable even if supervisor dies between spawn and
    // persisting PGID. Wait for PGID here to also exercise live-group recovery.
    loop {
        // This observer does not hold flock while the supervisor persists its
        // record. A partial read is not a readiness event; acquisition/recovery
        // themselves still parse only under flock and fail closed.
        if let Ok(record) = serde_json::from_slice::<serde_json::Value>(&fs::read(&o.path).unwrap())
        {
            if record["groups"]
                .as_array()
                .is_some_and(|groups| groups.contains(&serde_json::json!(group)))
            {
                break;
            }
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    owned.child.kill().unwrap();
    owned.child.wait().unwrap();
    assert_eq!(unsafe { libc::kill(group, 0) }, 0);
    let before = digest(&o.path).unwrap();
    assert!(Gate::acquire(&o, &AtomicBool::new(false)).is_err());
    assert!(Gate::recover(
        &o,
        &before,
        "owner dead but producer still alive",
        &AtomicBool::new(false)
    )
    .is_err());
    assert_eq!(before, digest(&o.path).unwrap());
    assert_eq!(unsafe { libc::kill(-group, libc::SIGKILL) }, 0);
    owned.group = None;
    while unsafe { libc::kill(-group, 0) } == 0 {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
    Gate::recover(
        &o,
        &before,
        "owned fixture producer killed and group confirmed absent",
        &AtomicBool::new(false),
    )
    .unwrap();
    Gate::acquire(&o, &AtomicBool::new(false))
        .unwrap()
        .finish()
        .unwrap();
}

#[test]
fn recovery_requires_exact_evidence_and_dead_owner() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let o = options(&root);
    let mut child = Command::new(std::env::current_exe().unwrap());
    child
        .args(["--exact", "crashed_holder_fixture"])
        .env("VISUAL_GATE_FIXTURE", &root);
    assert!(run(
        &mut child,
        &Limits::default(),
        &AtomicBool::new(false),
        true
    )
    .unwrap()
    .status
    .success());
    assert!(Gate::acquire(&o, &AtomicBool::new(false)).is_err());
    let hash = digest(&o.path).unwrap();
    let mut wrong_compatibility = o.clone();
    wrong_compatibility
        .compatibility
        .push(root.join("not-recorded.lock"));
    assert!(Gate::recover(
        &wrong_compatibility,
        &hash,
        "wrong gate set",
        &AtomicBool::new(false)
    )
    .is_err());
    assert!(Gate::recover(
        &o,
        "wrong",
        "verified fixture has no descendants",
        &AtomicBool::new(false)
    )
    .is_err());
    assert!(Gate::recover(&o, &hash, "", &AtomicBool::new(false)).is_err());
    assert_eq!(hash, digest(&o.path).unwrap());
    Gate::recover(
        &o,
        &hash,
        "verified fixture never spawned a producer",
        &AtomicBool::new(false),
    )
    .unwrap();
    Gate::acquire(&o, &AtomicBool::new(false))
        .unwrap()
        .finish()
        .unwrap();
}
