//! Exercise the consumer's actual producer/finish boundary, without a display
//! or the real host lock. All processes and lock inodes belong to this fixture.
#![cfg(unix)]
use bbr_visual::runner;
use cargo_gpui_visual::{
    gate::{Gate, Options},
    process::Limits,
};
use std::{
    fs,
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

fn options(root: &Path) -> Options {
    Options {
        path: root.join("host.lock"),
        compatibility: vec![root.join("bbr.lock")],
        wait: Duration::from_millis(100),
    }
}

#[test]
fn producer_fixture() {
    let Some(root) = std::env::var_os("BBR_VISUAL_LIFECYCLE_FIXTURE") else {
        return;
    };
    let root = PathBuf::from(root);
    let cancelled = Arc::new(AtomicBool::new(false));
    for signal in [libc::SIGTERM, libc::SIGINT] {
        signal_hook::flag::register(signal, cancelled.clone()).unwrap();
    }
    let mode = std::env::var("BBR_VISUAL_LIFECYCLE_MODE").unwrap();
    let script = if mode == "success" {
        "trap '' TERM; sleep 1000 & echo $! > child.tmp; mv child.tmp child.pid; echo $$ > leader.tmp; mv leader.tmp leader.pid; exit 0"
    } else {
        "trap '' TERM; sleep 1000 & echo $! > child.tmp; mv child.tmp child.pid; echo $$ > leader.tmp; mv leader.tmp leader.pid; wait"
    };
    let result = runner::execute_producer(
        &options(&root),
        Command::new("sh").args(["-c", script]).current_dir(&root),
        &Limits {
            timeout: Duration::from_secs(if mode == "timeout" { 2 } else { 10 }),
            cleanup_timeout: Duration::from_secs(3),
            ..Limits::default()
        },
        &cancelled,
    );
    fs::write(
        root.join("result.txt"),
        match result {
            Ok(()) => "success".into(),
            Err(error) => format!("{error:#}"),
        },
    )
    .unwrap();
}

struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            // Signal only this fixture's unreaped supervisor. It owns cleanup
            // of its separate producer group; never kill foreign processes.
            unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) };
            let deadline = Instant::now() + Duration::from_secs(15);
            while self.0.try_wait().ok().flatten().is_none() {
                if Instant::now() >= deadline {
                    // Its own 10s operation + 3s cleanup must already have
                    // expired. Do not pretend an unresponsive owner is clean.
                    panic!("owned fixture supervisor did not finish cleanup");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}
fn read_pid(path: &Path, deadline: Instant, child: &mut Child) -> i32 {
    loop {
        if let Ok(text) = fs::read_to_string(path) {
            return text.trim().parse().unwrap();
        }
        assert!(Instant::now() < deadline, "missing {}", path.display());
        assert!(child.try_wait().unwrap().is_none(), "fixture exited early");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn assert_absent(pid: i32) {
    assert!(pid > 1);
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}
fn run(mode: &str) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let o = options(&root);
    fs::write(&o.compatibility[0], "preserved legacy metadata").unwrap();
    let mut child = Owned(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "producer_fixture", "--nocapture"])
            .env("BBR_VISUAL_LIFECYCLE_FIXTURE", &root)
            .env("BBR_VISUAL_LIFECYCLE_MODE", mode)
            .stdout(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    let descendant = read_pid(&root.join("child.pid"), deadline, &mut child.0);
    let leader = read_pid(&root.join("leader.pid"), deadline, &mut child.0);
    if mode != "success" {
        assert!(Gate::acquire(&o, &AtomicBool::new(false)).is_err());
        let legacy = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&o.compatibility[0])
            .unwrap();
        assert_eq!(
            unsafe { libc::flock(legacy.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            -1
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::EWOULDBLOCK)
        );
        assert_eq!(unsafe { libc::kill(descendant, 0) }, 0);
        if mode == "cancel" {
            assert_eq!(unsafe { libc::kill(child.0.id() as i32, libc::SIGTERM) }, 0);
        }
    }
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    let outcome = fs::read_to_string(root.join("result.txt")).unwrap();
    assert!(
        outcome.contains(match mode {
            "cancel" => "cancelled",
            "timeout" => "deadline",
            _ => "success",
        }),
        "{outcome}"
    );
    assert_absent(descendant);
    assert_absent(leader);
    assert_eq!(unsafe { libc::kill(-leader, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
    assert_eq!(
        fs::read_to_string(&o.compatibility[0]).unwrap(),
        "preserved legacy metadata"
    );
    let record: serde_json::Value = serde_json::from_slice(&fs::read(&o.path).unwrap()).unwrap();
    assert_eq!(record["active"], false);
    assert_eq!(record["pending"], false);
    assert_eq!(record["groups"], serde_json::json!([]));
    Gate::acquire(&o, &AtomicBool::new(false))
        .unwrap()
        .finish()
        .unwrap();
}

#[test]
fn signal_cancellation_cleans_descendants_before_releasing_both_leases() {
    run("cancel");
}
#[test]
fn deadline_cleans_descendants_before_releasing_both_leases() {
    run("timeout");
}
#[test]
fn successful_leader_cannot_leave_a_background_child() {
    run("success");
}

#[test]
fn actual_cli_int_term_and_hup_cancel_probe_and_reap_its_descendants() {
    use std::os::unix::fs::PermissionsExt;
    for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let compiler = root.join("rustc");
        // Per-child PATH only. This is an environment-probe fixture, not an
        // actual compiler or a modification to any installed toolchain.
        fs::write(&compiler, "#!/bin/sh\ncd \"$BBR_VISUAL_PROBE_FIXTURE\" || exit 2\ntrap '' TERM\nsleep 1000 &\necho $! > child.tmp\nmv child.tmp child.pid\necho $$ > leader.tmp\nmv leader.tmp leader.pid\nwait\n").unwrap();
        fs::set_permissions(&compiler, fs::Permissions::from_mode(0o700)).unwrap();
        let path = std::env::join_paths(
            std::iter::once(root.clone())
                .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
        )
        .unwrap();
        let error_log = root.join("stderr.log");
        let mut child = Owned(
            Command::new(env!("CARGO_BIN_EXE_bbr-visual"))
                .args(["environment", "gallery"])
                .env("PATH", path)
                .env("BBR_VISUAL_PROBE_FIXTURE", &root)
                .stdout(Stdio::null())
                .stderr(fs::File::create(&error_log).unwrap())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(15);
        let descendant = read_pid(&root.join("child.pid"), deadline, &mut child.0);
        let leader = read_pid(&root.join("leader.pid"), deadline, &mut child.0);
        assert_eq!(unsafe { libc::kill(child.0.id() as i32, signal) }, 0);
        loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                assert_eq!(status.code(), Some(2));
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(fs::read_to_string(error_log).unwrap().contains("cancelled"));
        assert_absent(descendant);
        assert_absent(leader);
    }
}

#[test]
fn already_cancelled_never_spawns_or_touches_gate_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let o = options(&root);
    assert!(runner::execute_producer(
        &o,
        Command::new("sh")
            .args(["-c", "touch should-not-exist"])
            .current_dir(&root),
        &Limits::default(),
        &AtomicBool::new(true),
    )
    .is_err());
    assert!(!o.path.exists());
    assert!(!root.join("should-not-exist").exists());
}
