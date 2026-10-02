//! Inject only the process boundary: no Gallery/native execution on host.
use super::*;
use crate::payload::Entry;
use serde_json::json;
use std::{os::unix::process::ExitStatusExt, sync::atomic::Ordering};

struct Fixture {
    _temp: tempfile::TempDir,
    payload: Payload,
    work: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let repo = root.join("repo");
        let work = root.join("work");
        fs::create_dir(&work).unwrap();
        fs::create_dir_all(repo.join("scripts/visual/catalogs")).unwrap();
        store::write_json(
            &repo.join(semantic::PLAN),
            &json!({"schema":1,"tests":["e2e::fixture::one","e2e::fixture::two"]}),
        )
        .unwrap();
        let payload = Payload {
            schema: 1,
            suite: "gallery-semantic".into(),
            repo: repo.clone(),
            sources: [(
                semantic::PLAN.into(),
                store::digest(&repo.join(semantic::PLAN)).unwrap(),
            )]
            .into(),
            files: vec![Entry {
                payload: "executables/0".into(),
                guest: repo.join("target/release/deps/bbr_gallery-fixture"),
                sha256: "b".repeat(64),
                bytes: 1,
                executable: true,
            }],
        };
        Self {
            _temp: temp,
            payload,
            work,
        }
    }
}
fn reports(repo: &Path) {
    let dir = repo.join("target/bbr-gallery-perf");
    fs::create_dir_all(&dir).unwrap();
    for suite in ["every-story-renders", "mode-state-switching"] {
        store::write_json(
            &dir.join(format!("{suite}.json")),
            &json!({
                "suite": suite, "sanity_only": true,
                "note": "report-only frame-time observations; never a product budget",
                "observations_ms": {"fixture": 0.25}
            }),
        )
        .unwrap();
    }
}
fn output(list: bool) -> Output {
    Output {
        status: std::process::ExitStatus::from_raw(0),
        stdout: if list {
            b"e2e::fixture::one: test\ne2e::fixture::two: test\n\n2 tests, 0 benchmarks\n".to_vec()
        } else {
            b"\nrunning 2 tests\ntest e2e::fixture::one ... ok\ntest e2e::fixture::two ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s\n\n".to_vec()
        },
    }
}
#[test]
fn fixed_listing_precedes_serial_complete_run_and_publishes_exact_record() {
    let f = Fixture::new();
    let cancel = AtomicBool::new(false);
    let mut calls = 0;
    execute(
        &f.payload,
        &f.work,
        &"a".repeat(64),
        &cancel,
        |cmd, limits| {
            let args: Vec<_> = cmd.get_args().map(|a| a.to_str().unwrap()).collect();
            assert_eq!(cmd.get_program(), f.payload.files[0].guest);
            assert_eq!(cmd.get_current_dir(), Some(f.payload.repo.as_path()));
            assert!(limits.timeout <= Duration::from_secs(600) && !limits.timeout.is_zero());
            assert_eq!(limits.max_output_bytes, semantic::OUTPUT_LIMIT);
            if calls == 0 {
                assert_eq!(
                    args,
                    [
                        "--ignored",
                        "--list",
                        "--format",
                        "pretty",
                        "--color",
                        "never"
                    ]
                );
            } else {
                assert_eq!(
                    args,
                    [
                        "--ignored",
                        "--nocapture",
                        "--test-threads=1",
                        "--format",
                        "pretty",
                        "--color",
                        "never"
                    ]
                );
            }
            let env: std::collections::BTreeMap<_, _> = cmd.get_envs().collect();
            assert_eq!(
                env.get(std::ffi::OsStr::new("HOME")).unwrap().unwrap(),
                "/Users/admin"
            );
            assert_eq!(env.get(std::ffi::OsStr::new("SEED")).unwrap().unwrap(), "0");
            let out = output(calls == 0);
            if calls == 1 {
                reports(&f.payload.repo);
            }
            calls += 1;
            Ok(out)
        },
    )
    .unwrap();
    assert_eq!(calls, 2);
    let record: semantic::Record =
        store::read_json(&f.work.join("raw").join(semantic::RESULT)).unwrap();
    record.validate(&f.payload, &"a".repeat(64)).unwrap();
    assert_eq!(fs::read_dir(f.work.join("raw")).unwrap().count(), 3);
    assert!(!f.payload.repo.join("target/bbr-gallery-perf").exists());
    assert!(!f.work.join("raw/capture.json").exists());
}
#[test]
fn symlinked_requirements_fail_without_invocation_or_success() {
    let f = Fixture::new();
    let path = f.payload.repo.join(semantic::PLAN);
    let copy = f.work.join("plan-copy.json");
    fs::rename(&path, &copy).unwrap();
    std::os::unix::fs::symlink(&copy, &path).unwrap();
    assert!(execute(
        &f.payload,
        &f.work,
        &"a".repeat(64),
        &AtomicBool::new(false),
        |_, _| {
            panic!("linked requirements must not execute");
        }
    )
    .is_err());
    assert!(!f.work.join("raw").exists());
}

#[test]
fn exhausted_overall_deadline_cannot_invoke_or_publish() {
    let f = Fixture::new();
    let error = execute_until(
        &f.payload,
        &f.work,
        &"a".repeat(64),
        &AtomicBool::new(false),
        Instant::now(),
        |_, _| {
            panic!("expired action must not execute");
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("deadline"));
    assert!(!f.work.join("raw").exists());
}

#[test]
fn failed_incomplete_drifted_or_cancelled_invocations_never_publish_success() {
    for mutation in [
        "pre-cancel",
        "freshness",
        "wrong-purpose",
        "missing-harness",
        "duplicate-harness",
        "bad-digest",
        "listing-exit",
        "listing-missing",
        "listing-extra",
        "run-exit",
        "run-missing",
        "run-failure",
        "overflow",
        "source-after-list",
        "source-after-run",
        "cancel-after-list",
        "cancel-after-run",
        "invocation-error",
        "missing-reports",
        "invalid-reports",
        "extra-reports",
        "stale-reports",
        "reports-after-list",
    ] {
        let mut f = Fixture::new();
        let cancel = AtomicBool::new(mutation == "pre-cancel");
        let mut calls = 0;
        if mutation == "stale-reports" {
            reports(&f.payload.repo);
        }
        if mutation == "freshness" {
            fs::create_dir(f.work.join("raw")).unwrap();
        }
        if mutation == "wrong-purpose" {
            f.payload.suite = "gallery".into();
        }
        if mutation == "missing-harness" {
            f.payload.files.clear();
        }
        if mutation == "duplicate-harness" {
            f.payload.files.push(f.payload.files[0].clone());
        }
        let digest = if mutation == "bad-digest" {
            "invalid".into()
        } else {
            "a".repeat(64)
        };
        let result = execute(&f.payload, &f.work, &digest, &cancel, |_, _| {
            let list = calls == 0;
            calls += 1;
            let mut out = output(list);
            if list && mutation == "reports-after-list" {
                reports(&f.payload.repo);
            }
            if !list && mutation != "missing-reports" {
                reports(&f.payload.repo);
                if mutation == "invalid-reports" {
                    fs::write(
                        f.payload
                            .repo
                            .join("target/bbr-gallery-perf/every-story-renders.json"),
                        "{}",
                    )
                    .unwrap();
                }
                if mutation == "extra-reports" {
                    fs::write(
                        f.payload.repo.join("target/bbr-gallery-perf/extra.json"),
                        "{}",
                    )
                    .unwrap();
                }
            }
            if mutation == "invocation-error" {
                anyhow::bail!("injected cleanup failure");
            }
            if (list && mutation == "listing-exit") || (!list && mutation == "run-exit") {
                out.status = std::process::ExitStatus::from_raw(256);
            }
            if (list && mutation == "listing-missing") || (!list && mutation == "run-missing") {
                out.stdout.clear();
            }
            if list && mutation == "listing-extra" {
                out.stdout.extend(b"e2e::fixture::extra: test\n");
            }
            if !list && mutation == "run-failure" {
                out.stdout = String::from_utf8(out.stdout)
                    .unwrap()
                    .replace(" ... ok", " ... FAILED")
                    .into_bytes();
            }
            if mutation == "overflow" {
                out.stdout = vec![b'x'; semantic::OUTPUT_LIMIT as usize + 1];
            }
            if (list && mutation == "source-after-list")
                || (!list && mutation == "source-after-run")
            {
                fs::write(f.payload.repo.join(semantic::PLAN), "drift").unwrap();
            }
            if (list && mutation == "cancel-after-list")
                || (!list && mutation == "cancel-after-run")
            {
                cancel.store(true, Ordering::SeqCst);
            }
            Ok(out)
        });
        assert!(result.is_err(), "{mutation}");
        assert!(
            !f.work.join("raw").join(semantic::RESULT).exists(),
            "{mutation}"
        );
        if [
            "pre-cancel",
            "freshness",
            "wrong-purpose",
            "missing-harness",
            "duplicate-harness",
            "bad-digest",
            "stale-reports",
        ]
        .contains(&mutation)
        {
            assert_eq!(calls, 0, "{mutation}");
        }
        if [
            "listing-exit",
            "listing-missing",
            "listing-extra",
            "overflow",
            "source-after-list",
            "cancel-after-list",
            "invocation-error",
            "reports-after-list",
        ]
        .contains(&mutation)
        {
            assert_eq!(calls, 1, "{mutation}");
        }
    }
}
