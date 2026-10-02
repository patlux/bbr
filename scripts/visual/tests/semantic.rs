//! Protocol tests execute only this inert consumer fixture, never Gallery.
use bbr_visual::{
    payload::{Entry, Payload},
    semantic::{Record, Requirements, PLAN},
};
use cargo_gpui_visual::{process, store};
use serde_json::json;
use std::{fs, path::PathBuf, process::Command, sync::atomic::AtomicBool};

struct Fixture {
    _temp: tempfile::TempDir,
    payload: Payload,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().canonicalize().unwrap();
        fs::create_dir_all(repo.join("scripts/visual/catalogs")).unwrap();
        store::write_json(
            &repo.join(PLAN),
            &json!({"schema":1,"tests":[
                "e2e::fixture::first", "e2e::fixture::second"
            ]}),
        )
        .unwrap();
        let payload = Payload {
            schema: 1,
            suite: "gallery-semantic".into(),
            repo: repo.clone(),
            sources: [(PLAN.into(), store::digest(&repo.join(PLAN)).unwrap())].into(),
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
        }
    }
    fn plan(&self) -> Requirements {
        Requirements::load(&self.payload).unwrap()
    }
}
fn listing() -> String {
    "e2e::fixture::first: test\ne2e::fixture::second: test\n\n2 tests, 0 benchmarks\n".into()
}
fn execution() -> String {
    "\nrunning 2 tests\ntest e2e::fixture::first ... ok\ntest e2e::fixture::second ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.01s\n\n".into()
}

#[test]
fn exact_inventory_and_results_are_required_not_just_exit_zero() {
    let f = Fixture::new();
    let plan = f.plan();
    plan.validate_listing(&listing()).unwrap();
    plan.validate_execution(&execution()).unwrap();
    for text in [
        String::new(),
        listing().replace("second", "third"),
        listing().replace("second", "first"),
        listing().replace("2 tests", "0 tests"),
        listing().replace(": test", ": benchmark"),
        format!("{}extra: test\n", listing()),
        listing().replace("\n\n2", "\n2 tests, 0 benchmarks\n\n2"),
    ] {
        assert!(plan.validate_listing(&text).is_err(), "{text:?}");
    }
    for text in [
        String::new(),
        execution().replace("second", "first"),
        execution().replace("second", "third"),
        execution().replace("test e2e::fixture::second ... ok\n", ""),
        execution().replace("second ... ok", "second ... ignored"),
        execution().replace("first ... ok", "first ... FAILED"),
        execution().replace("running 2 tests", "running 0 tests"),
        execution().replace("0 failed", "1 failed"),
        execution().replace("0 ignored", "1 ignored"),
        execution().replace("0 measured", "1 measured"),
        execution().replace("2 passed", "0 passed"),
        execution().replace("0.01s", "NaNs"),
        execution().replace("0.01s", "-1s"),
        execution().replace("test result: ok.", "test result: FAILED."),
        execution().replace("test result:", "truncated result:"),
        format!("{}{}", execution(), execution()),
        format!("{}test unknown ... ok\n", execution()),
        execution().replace(" ... ok", " ... \u{1b}[32mok\u{1b}[0m"),
    ] {
        assert!(plan.validate_execution(&text).is_err(), "{text:?}");
    }
}

#[test]
fn result_binds_original_payload_requirement_executable_and_raw_stdout() {
    let f = Fixture::new();
    let digest = "a".repeat(64);
    let record = Record::new(&f.payload, &digest, listing(), execution()).unwrap();
    record.validate(&f.payload, &digest).unwrap();
    for field in [
        "schema",
        "origin",
        "payload_sha256",
        "plan_sha256",
        "executable_sha256",
        "executable",
        "listing",
        "execution",
        "extra",
    ] {
        let mut raw = serde_json::to_value(&record).unwrap();
        raw[field] = if field == "schema" {
            json!(2)
        } else {
            json!("unexpected")
        };
        let result =
            serde_json::from_value::<Record>(raw).map(|r| r.validate(&f.payload, &digest).is_ok());
        assert!(!result.unwrap_or(false), "{field}");
    }
    let mut changed = f.payload.clone();
    changed.files[0].sha256 = "c".repeat(64);
    assert!(record.validate(&changed, &digest).is_err());
    assert!(record.validate(&f.payload, &"c".repeat(64)).is_err());
    fs::write(f.payload.repo.join(PLAN), "changed").unwrap();
    assert!(record.validate(&f.payload, &digest).is_err());
    assert!(Requirements::load(&f.payload).is_err());
}

#[test]
fn requirement_schema_names_order_and_purpose_fail_closed() {
    for value in [
        json!({"schema":2,"tests":["e2e::fixture::first"]}),
        json!({"schema":1,"tests":[]}),
        json!({"schema":1,"tests":["e2e::fixture::second","e2e::fixture::first"]}),
        json!({"schema":1,"tests":["e2e::fixture::first","e2e::fixture::first"]}),
        json!({"schema":1,"tests":["e2e::fixture::first extra"]}),
        json!({"schema":1,"tests":["unit::fixture::first"]}),
        json!({"schema":1,"tests":["e2e::fixture::first"],"extra":true}),
    ] {
        let mut f = Fixture::new();
        store::write_json(&f.payload.repo.join(PLAN), &value).unwrap();
        f.payload.sources.insert(
            PLAN.into(),
            store::digest(&f.payload.repo.join(PLAN)).unwrap(),
        );
        assert!(Requirements::load(&f.payload).is_err());
    }
    let mut f = Fixture::new();
    f.payload.suite = "gallery".into();
    assert!(Requirements::load(&f.payload).is_err());
    f.payload.suite = "gallery-semantic".into();
    f.payload.sources.clear();
    assert!(Requirements::load(&f.payload).is_err());
}

#[test]
fn actual_stable_libtest_protocol_matches_without_running_gallery() {
    let f = Fixture::new();
    let plan = f.plan();
    let binary = std::env::current_exe().unwrap();
    let run = |args: &[&str]| {
        process::run(
            Command::new(&binary).args(args),
            &process::Limits::default(),
            &AtomicBool::new(false),
            true,
        )
        .unwrap()
    };
    let listed = run(&[
        "--ignored",
        "--list",
        "--format",
        "pretty",
        "--color",
        "never",
    ]);
    assert!(listed.status.success());
    let listed = String::from_utf8(listed.stdout).unwrap();
    plan.validate_listing(&listed).unwrap();
    let executed = run(&[
        "--ignored",
        "--nocapture",
        "--test-threads=1",
        "--format",
        "pretty",
        "--color",
        "never",
    ]);
    assert!(executed.status.success());
    let executed = String::from_utf8(executed.stdout).unwrap();
    plan.validate_execution(&executed).unwrap();
    Record::new(&f.payload, &"a".repeat(64), listed, executed).unwrap();
}

#[test]
fn authored_current_inventory_has_the_pending_observer_and_all_source_scenarios() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let raw: serde_json::Value = store::read_json(&repo.join(PLAN)).unwrap();
    let tests = raw["tests"].as_array().unwrap();
    assert_eq!(tests.len(), 49);
    assert!(tests.iter().any(|t| t
        .as_str()
        .unwrap()
        .contains("capture_target::checkbox_capture_observes")));
    // Independent authored inventory is checked against source, never generated at runtime.
    let mut source_tests = Vec::new();
    for entry in fs::read_dir(repo.join("crates/bbr-gallery/src/e2e")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let mut ignored = false;
        for line in source.lines().map(str::trim) {
            if line == "#[ignore]" {
                ignored = true;
            } else if ignored && line.starts_with("fn ") {
                source_tests.push(format!(
                    "e2e::{}::{}",
                    path.file_stem().unwrap().to_str().unwrap(),
                    line.strip_prefix("fn ").unwrap().split('(').next().unwrap()
                ));
                ignored = false;
            }
        }
    }
    source_tests.sort();
    assert_eq!(
        source_tests,
        tests
            .iter()
            .map(|t| t.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    );
}

mod e2e {
    mod fixture {
        #[test]
        #[ignore]
        fn first() {
            assert_eq!(2 + 2, 4);
        }
        #[test]
        #[ignore]
        fn second() {
            assert!(std::env::current_exe().unwrap().is_file());
        }
    }
}
