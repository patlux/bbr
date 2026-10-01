//! Real local process/cancellation tests, private leases, no Tart or GUI.
use bbr_visual::vm_supervision::{self, Policy};
use cargo_gpui_visual::gate::{Gate, Options};
use std::{
    fs,
    path::Path,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

const FIXTURE: &str = r#"
import fcntl, json, os, signal, subprocess, sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
import vm_control
root, mode = Path(sys.argv[2]), sys.argv[3]
control = Path(os.environ['BBR_VISUAL_CONTROL'])
request = os.environ['BBR_VISUAL_REQUEST_SHA256']
if mode in ('hard', 'flood'):
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
    (root/'ready').write_text(str(os.getpid()))
    if mode == 'flood':
        while True: os.write(1, b'x' * 65536)
    while True: signal.pause()
if mode in ('missing', 'stale'):
    if mode == 'stale':
        (control/'terminal.json').write_text(json.dumps({'schema':1,'request_sha256':'f'*64,'outcome':'completed'}))
    sys.exit(0)

def operation():
    # Detached stand-in models Tart's setsid process, owned by Popen only.
    child = subprocess.Popen(['/bin/sleep', '8'], start_new_session=True)
    try:
        (root/'detached').write_text(str(child.pid))
        (root/'ready').write_text(str(os.getpid()))
        if mode == 'invalid-cancel':
            vm_control._publish(control/'cancel.json', {'schema':1,'request_sha256':'f'*64,'reason':'cancelled'})
        if mode in ('cancel', 'timeout', 'invalid-cancel', 'signal-int', 'signal-term', 'signal-hup'):
            while True: signal.pause()
        if mode == 'failed': raise RuntimeError('synthetic producer failure')
    finally:
        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            signal.signal(sig, signal.SIG_IGN)
        # Both gates must still be held while detached resources are cleaned.
        try:
            for name in ('host.lock', 'bbr.lock'):
                with (root/name).open('r+') as lock:
                    try: fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    except BlockingIOError: pass
                    else: raise AssertionError('lease released before external cleanup')
        finally:
            child.terminate()
            child.wait(timeout=2)
        (root/'cleaned').write_text('owned detached child reaped')

def verify():
    assert (root/'cleaned').read_text() == 'owned detached child reaped'
    if mode == 'cleanup-failed': raise RuntimeError('synthetic cleanup verification failure')
try:
    vm_control.run(control, request, operation, verify)
    if mode == 'signal-after-ack': os.kill(os.getpid(), signal.SIGKILL)
except BaseException:
    sys.exit(2)
"#;

fn options(root: &Path) -> Options {
    Options {
        path: root.join("host.lock"),
        compatibility: vec![root.join("bbr.lock")],
        wait: Duration::from_millis(100),
    }
}
fn policy() -> Policy {
    Policy {
        operation: Duration::from_secs(2),
        grace: Duration::from_secs(3),
        process_cleanup: Duration::from_secs(3),
        output_bytes: 65536,
    }
}
fn scenario(mode: &str, active: bool, success: bool) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let control = root.join("control");
    let options = options(&root);
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = match mode {
        "signal-int" => Some(libc::SIGINT),
        "signal-term" => Some(libc::SIGTERM),
        "signal-hup" => Some(libc::SIGHUP),
        _ => None,
    };
    if let Some(signal) = signal {
        // Only used by the isolated supervisor fixture process below.
        signal_hook::flag::register(signal, cancelled.clone()).unwrap();
    }
    let finished = AtomicBool::new(false);
    let result = std::thread::scope(|scope| {
        let cancellation = scope.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(8);
            while !finished.load(Ordering::SeqCst) && (mode == "cancel" || signal.is_some()) {
                if root.join("ready").exists() {
                    if let Some(signal) = signal {
                        assert_eq!(unsafe { libc::kill(std::process::id() as i32, signal) }, 0);
                    } else {
                        cancelled.store(true, Ordering::SeqCst);
                    }
                    return;
                }
                assert!(Instant::now() < deadline, "fixture not ready");
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let result = vm_supervision::run(
            &options,
            Command::new("python3")
                .args(["-I", "-B", "-c", FIXTURE])
                .arg(env!("CARGO_MANIFEST_DIR"))
                .arg(&root)
                .arg(mode),
            &control,
            &"a".repeat(64),
            &policy(),
            &cancelled,
            || {
                anyhow::ensure!(root.join("cleaned").is_file(), "missing resource evidence");
                let pid: i32 = fs::read_to_string(root.join("detached"))?.parse()?;
                anyhow::ensure!(
                    unsafe { libc::kill(pid, 0) } == -1
                        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH),
                    "detached process remains"
                );
                anyhow::ensure!(mode != "host-reject", "host rejects cleanup evidence");
                Ok(())
            },
        );
        finished.store(true, Ordering::SeqCst);
        cancellation.join().unwrap();
        result
    });
    assert_eq!(result.is_ok(), success, "mode={mode} result={result:?}");
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(&options.path).unwrap()).unwrap();
    assert_eq!(record["active"], active, "{mode}: {result:?}");
    let fresh = Gate::acquire(&options, &AtomicBool::new(false));
    assert_eq!(fresh.is_err(), active, "{mode}");
    if let Ok(gate) = fresh {
        gate.finish().unwrap();
    }
    if root.join("detached").exists() {
        let pid: i32 = fs::read_to_string(root.join("detached"))
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(
            unsafe { libc::kill(pid, 0) },
            -1,
            "detached fixture leaked: {pid}"
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
    if mode == "cancel" || mode == "timeout" || signal.is_some() {
        let terminal: serde_json::Value =
            serde_json::from_slice(&fs::read(control.join("terminal.json")).unwrap()).unwrap();
        assert_eq!(terminal["outcome"], "cancelled");
    }
    if mode == "hard" || mode == "flood" {
        let pid: i32 = fs::read_to_string(root.join("ready"))
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        assert!(!control.join("terminal.json").exists());
    }
}

#[test]
fn normal_and_failed_operations_release_only_after_external_cleanup() {
    scenario("success", false, true);
    scenario("failed", false, false);
}
#[test]
fn cancellation_and_deadline_allow_detached_cleanup_before_leases_release() {
    scenario("cancel", false, false);
    scenario("timeout", false, false);
}
#[test]
fn hard_kill_missing_stale_or_rejected_cleanup_keeps_unfinished_gate() {
    for mode in [
        "hard",
        "flood",
        "missing",
        "stale",
        "cleanup-failed",
        "host-reject",
        "invalid-cancel",
        "signal-after-ack",
    ] {
        scenario(mode, true, false);
    }
}
#[test]
fn signal_supervisor_fixture() {
    if let Ok(mode) = std::env::var("BBR_VM_SIGNAL_FIXTURE") {
        scenario(&mode, false, false);
    }
}

#[test]
fn real_int_term_hup_reach_cooperative_cleanup_in_isolated_supervisors() {
    for mode in ["signal-int", "signal-term", "signal-hup"] {
        let output = cargo_gpui_visual::process::run(
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "signal_supervisor_fixture", "--nocapture"])
                .env("BBR_VM_SIGNAL_FIXTURE", mode),
            &cargo_gpui_visual::process::Limits {
                timeout: Duration::from_secs(12),
                cleanup_timeout: Duration::from_secs(3),
                ..Default::default()
            },
            &AtomicBool::new(false),
            true,
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn invalid_or_pre_cancelled_request_never_spawns_or_creates_control() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    for (cancelled, digest, operation) in [
        (true, "a".repeat(64), Duration::from_secs(1)),
        (false, "invalid".into(), Duration::from_secs(1)),
        (false, "a".repeat(64), Duration::ZERO),
    ] {
        let result = vm_supervision::run(
            &options(&root),
            Command::new("sh")
                .arg("-c")
                .arg("touch tripwire")
                .current_dir(&root),
            &root.join("control"),
            &digest,
            &Policy {
                operation,
                ..policy()
            },
            &AtomicBool::new(cancelled),
            || Ok(()),
        );
        assert!(result.is_err());
        assert!(!root.join("tripwire").exists());
        assert!(!root.join("control").exists());
        assert!(!root.join("host.lock").exists());
    }
}
