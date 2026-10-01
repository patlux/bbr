#![cfg(unix)]
//! OS/process contracts only. No native UI, network or product data.
use cargo_gpui_visual::process::{run, Limits};
use std::{
    fs,
    process::Command,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

fn limits(ms: u64) -> Limits {
    Limits {
        timeout: Duration::from_millis(ms),
        cleanup_timeout: Duration::from_secs(3),
        max_output_bytes: 1024,
    }
}
fn command(script: &str) -> Command {
    let mut c = Command::new("sh");
    c.args(["-c", script]);
    c
}
fn gone(pid: i32) -> bool {
    // Safety: signal 0 only observes the owned test child; never signals it.
    unsafe {
        libc::kill(pid, 0) == -1
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
    }
}
fn owns_child(script: &str, budget: u64, expected: &str) {
    let temp = tempfile::tempdir().unwrap();
    let mut c = command(script);
    c.current_dir(temp.path());
    let start = Instant::now();
    let result = run(&mut c, &limits(budget), &AtomicBool::new(false), true);
    match expected {
        "success" => assert!(result.unwrap().status.success()),
        "exit9" => assert_eq!(result.unwrap().status.code(), Some(9)),
        _ => assert!(format!("{:#}", result.unwrap_err()).contains(expected)),
    }
    assert!(start.elapsed() < Duration::from_secs(8));
    let pid: i32 = fs::read_to_string(temp.path().join("child.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(gone(pid), "child {pid} survived return");
}

#[test]
fn deadline_kills_a_term_ignoring_child_and_reaps_before_return() {
    owns_child(
        "trap '' TERM; sleep 1000 & echo $! > child.pid; wait",
        250,
        "deadline",
    );
}

#[test]
fn deadline_cleans_grandchildren_in_the_owned_group() {
    owns_child(
        "sh -c 'sleep 1000 & echo $! > child.pid; wait' & wait",
        250,
        "deadline",
    );
}

#[test]
fn successful_parent_cannot_return_before_descendants_are_gone() {
    owns_child("sleep 1000 & echo $! > child.pid; exit 0", 2000, "success");
}

#[test]
fn failed_parent_cleans_its_children_before_return() {
    owns_child("sleep 1000 & echo $! > child.pid; exit 9", 2000, "exit9");
}

#[test]
fn cancellation_before_spawn_has_no_side_effects() {
    let temp = tempfile::tempdir().unwrap();
    let mut c = command("touch executed");
    c.current_dir(temp.path());
    let error = run(&mut c, &limits(1000), &AtomicBool::new(true), false).unwrap_err();
    assert!(error.to_string().contains("cancelled"));
    assert!(!temp.path().join("executed").exists());
}

#[test]
fn cancellation_during_execution_cleans_the_child_group() {
    let temp = tempfile::tempdir().unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancel = cancelled.clone();
    let repo = temp.path().to_owned();
    let worker = std::thread::spawn(move || {
        let mut c = command("trap '' TERM; sleep 1000 & echo $! > child.pid; wait");
        c.current_dir(repo);
        run(&mut c, &limits(5000), &cancel, false)
    });
    let deadline = Instant::now() + Duration::from_secs(3);
    while !temp.path().join("child.pid").exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
    assert!(worker
        .join()
        .unwrap()
        .unwrap_err()
        .to_string()
        .contains("cancelled"));
    let pid = fs::read_to_string(temp.path().join("child.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(gone(pid));
}

#[test]
fn stdout_is_captured_without_a_pipe_and_nonzero_exit_is_preserved() {
    let result = run(
        &mut command("printf '{\"ok\":true}'; exit 7"),
        &limits(2000),
        &AtomicBool::new(false),
        true,
    )
    .unwrap();
    assert_eq!(result.status.code(), Some(7));
    assert_eq!(result.stdout, b"{\"ok\":true}");
}

#[test]
fn output_limit_fails_closed_and_cleans_children() {
    owns_child("sleep 1000 & echo $! > child.pid; while :; do printf '123456789012345678901234567890'; done",2000,"output limit");
}

#[test]
fn invalid_limits_and_missing_executables_fail_without_spawning() {
    let huge = Limits {
        timeout: Duration::MAX,
        ..limits(1000)
    };
    assert!(run(
        &mut command("exit 0"),
        &huge,
        &AtomicBool::new(false),
        false
    )
    .is_err());
    assert!(run(
        &mut command("exit 0"),
        &limits(0),
        &AtomicBool::new(false),
        false
    )
    .is_err());
    assert!(run(
        &mut Command::new("/definitely/not/a/visual-producer"),
        &limits(1000),
        &AtomicBool::new(false),
        false
    )
    .is_err());
}
