#![cfg(unix)]
use cargo_gpui_visual::{
    capture::{capture_guarded, capture_with, Execution, Paths},
    config::Config,
    gate::{Gate, Options},
    process::Limits,
};
use std::{
    fs,
    process::{Command, Stdio},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

const CONFIG: &str = r#"
schema = 1
capture = ["sh", "produce.sh"]
environment_command = ["sh", "environment.sh"]
baselines = "references"
report = "review"
runs = "runs"
[source_policy]
scopes = ["renderer.rs"]
suffixes = [".rs"]
"#;
fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    assert!(Command::new("git")
        .args(["init", "-q"])
        .arg(temp.path())
        .status()
        .unwrap()
        .success());
    fs::write(temp.path().join("renderer.rs"), "fixture").unwrap();
    fs::write(
        temp.path().join("environment.sh"),
        "printf '{\"layer\":\"gpui\"}'",
    )
    .unwrap();
    fs::write(
        temp.path().join("produce.sh"),
        "trap '' TERM; sleep 1000 & echo $! > child.pid.tmp; mv child.pid.tmp child.pid; wait",
    )
    .unwrap();
    fs::write(temp.path().join("visual.toml"), CONFIG).unwrap();
    temp
}
fn no_acceptance(temp: &tempfile::TempDir) {
    for path in ["runs/a/capture.json", "review", "references"] {
        assert!(!temp.path().join(path).exists());
    }
    let pid: i32 = fs::read_to_string(temp.path().join("child.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    // Safety: only observe the owned test child, no signal delivered.
    assert!(unsafe { libc::kill(pid, 0) } == -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}
#[test]
fn capture_and_probe_deadlines_refuse_seals_and_clean_their_children() {
    for probe in [false, true] {
        let temp = fixture();
        if probe {
            fs::copy(
                temp.path().join("produce.sh"),
                temp.path().join("environment.sh"),
            )
            .unwrap();
        }
        let limits = Limits {
            timeout: Duration::from_millis(250),
            cleanup_timeout: Duration::from_secs(3),
            max_output_bytes: 4096,
        };
        let execution = Execution {
            producer: limits.clone(),
            environment: limits,
        };
        let result = capture_with(
            temp.path(),
            &Config::from_toml(CONFIG).unwrap(),
            "runs/a".as_ref(),
            "references".as_ref(),
            "review".as_ref(),
            &execution,
            &AtomicBool::new(false),
        );
        assert!(result.unwrap_err().to_string().contains("deadline"));
        no_acceptance(&temp);
    }
}
#[test]
fn guarded_probe_and_capture_timeout_release_only_after_cleanup() {
    for probe in [false, true] {
        let temp = fixture();
        let root = temp.path().canonicalize().unwrap();
        if probe {
            fs::copy(root.join("produce.sh"), root.join("environment.sh")).unwrap();
        }
        let options = Options {
            path: root.join("host.lock"),
            compatibility: vec![],
            wait: Duration::from_secs(1),
        };
        let limits = Limits {
            timeout: Duration::from_millis(250),
            cleanup_timeout: Duration::from_secs(3),
            max_output_bytes: 4096,
        };
        let execution = Execution {
            producer: limits.clone(),
            environment: limits,
        };
        let result = capture_guarded(
            Paths {
                repo: &root,
                run: "runs/a".as_ref(),
                baselines: "references".as_ref(),
                report: "review".as_ref(),
            },
            &Config::from_toml(CONFIG).unwrap(),
            &execution,
            &AtomicBool::new(false),
            &options,
        );
        assert!(result.unwrap_err().to_string().contains("deadline"));
        no_acceptance(&temp);
        Gate::acquire(&options, &AtomicBool::new(false))
            .unwrap()
            .finish()
            .unwrap();
    }
}

#[test]
fn guarded_capture_refuses_stale_gate_before_probes_or_production() {
    let temp = fixture();
    let root = temp.path().canonicalize().unwrap();
    let options = Options {
        path: root.join("host.lock"),
        compatibility: vec![],
        wait: Duration::from_secs(1),
    };
    drop(Gate::acquire(&options, &AtomicBool::new(false)).unwrap());
    let before = fs::read(&options.path).unwrap();
    fs::write(root.join("environment.sh"), "touch probe-executed").unwrap();
    let result = capture_guarded(
        Paths {
            repo: &root,
            run: "runs/a".as_ref(),
            baselines: "references".as_ref(),
            report: "review".as_ref(),
        },
        &Config::from_toml(CONFIG).unwrap(),
        &Execution::default(),
        &AtomicBool::new(false),
        &options,
    );
    assert!(result.unwrap_err().to_string().contains("unfinished"));
    assert!(!root.join("probe-executed").exists());
    assert!(!root.join("child.pid").exists());
    assert!(!root.join("runs/a/capture.json").exists());
    assert_eq!(before, fs::read(&options.path).unwrap());
}

#[test]
fn cli_signals_wait_for_group_cleanup_before_exiting_without_a_seal() {
    for signal in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP] {
        let temp = fixture();
        let mut child = Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"))
            .args(["check", "--repo"])
            .arg(temp.path())
            .args(["--run", "runs/a"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !temp.path().join("child.pid").exists() {
            assert!(Instant::now() < deadline, "producer did not start");
            assert!(
                child.try_wait().unwrap().is_none(),
                "CLI exited before fixture started"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        // Safety: signal exactly the CLI process owned by this test.
        assert_eq!(unsafe { libc::kill(child.id() as i32, signal) }, 0);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(2));
                break;
            }
            assert!(Instant::now() < deadline, "CLI did not handle interruption");
            std::thread::sleep(Duration::from_millis(5));
        }
        no_acceptance(&temp);
    }
}
