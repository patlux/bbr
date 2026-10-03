//! Pure synthetic return evidence: no VM, native process or performance claim.
use bbr_visual::{
    payload::{Entry, Payload},
    semantic,
    vm_environment::{Guest, HostBuild, Receipt},
    vm_semantic,
};
use cargo_gpui_visual::store;
use serde_json::{json, Value};
use std::{fs, path::PathBuf, sync::atomic::AtomicBool};

struct Fixture {
    _temp: tempfile::TempDir,
    workspace: PathBuf,
    payload: Payload,
    host: HostBuild,
    evidence: Value,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().canonicalize().unwrap();
        let repo = workspace.join("repo");
        fs::create_dir_all(repo.join("scripts/visual/catalogs")).unwrap();
        store::write_json(
            &repo.join(semantic::PLAN),
            &json!({"schema":1,"tests":["e2e::fixture::one","e2e::fixture::two"]}),
        )
        .unwrap();
        let plan_hash = store::digest(&repo.join(semantic::PLAN)).unwrap();
        let payload = Payload {
            schema: 1,
            suite: "gallery-semantic".into(),
            repo: repo.clone(),
            sources: [(semantic::PLAN.into(), plan_hash.clone())].into(),
            files: vec![
                Entry {
                    payload: format!("source/{}", semantic::PLAN),
                    guest: repo.join(semantic::PLAN),
                    sha256: plan_hash,
                    bytes: fs::metadata(repo.join(semantic::PLAN)).unwrap().len(),
                    executable: false,
                },
                Entry {
                    payload: "executables/0".into(),
                    guest: repo.join("target/release/deps/bbr_gallery-fixture"),
                    sha256: "b".repeat(64),
                    bytes: 1,
                    executable: true,
                },
                Entry {
                    payload: "executables/1".into(),
                    guest: repo.join("scripts/visual/target/release/bbr-visual"),
                    sha256: "c".repeat(64),
                    bytes: 1,
                    executable: true,
                },
            ],
        };
        let host = HostBuild {
            os: "host".into(),
            os_build: "host-build".into(),
            arch: "arm64".into(),
            rustc: "fixture".into(),
            xcode: "fixture".into(),
            sdk: "fixture".into(),
            gpui_source: "fixture".into(),
            tool_revision: "fixture".into(),
            executable_hashes: payload
                .files
                .iter()
                .filter(|e| e.executable)
                .map(|e| (e.guest.to_string_lossy().into_owned(), e.sha256.clone()))
                .collect(),
        };
        let guest = Guest {
            schema: 1,
            origin: "tart-guest".into(),
            os: "guest".into(),
            os_build: "guest-build".into(),
            arch: "arm64".into(),
            model: "VirtualMac-fixture".into(),
            displays: json!([{"origin":"guest-coregraphics-metal","screens":[{"id":1,"width_px":6400,"height_px":3600,"width_pt":3200,"height_pt":1800}],"metal":{"name":"Synthetic Metal","registry_id":100}}]),
            fonts: [("/System/Library/Fonts/fixture.ttf".into(), "c".repeat(64))].into(),
            console_user: "admin".into(),
            on_console: true,
            screen_locked: false,
            window_server: true,
            timezone: "UTC".into(),
            locale: "C".into(),
        };
        let receipt = Receipt {
            schema: 1,
            payload_sha256: "d".repeat(64),
            sources: payload.sources.clone(),
            installed: payload
                .files
                .iter()
                .map(|e| (e.guest.to_string_lossy().into_owned(), e.sha256.clone()))
                .collect(),
            environment_before: guest.clone(),
            environment_after: guest,
        };
        let raw = workspace.join("raw");
        fs::create_dir(&raw).unwrap();
        store::write_json(&raw.join("guest-receipt.json"), &receipt).unwrap();
        let record=semantic::Record::new(&payload,&"d".repeat(64),"e2e::fixture::one: test\ne2e::fixture::two: test\n2 tests, 0 benchmarks\n".into(),"running 2 tests\ntest e2e::fixture::one ... ok\ntest e2e::fixture::two ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s\n".into()).unwrap();
        store::write_json(&raw.join(semantic::RESULT), &record).unwrap();
        for suite in ["every-story-renders", "mode-state-switching"] {
            store::write_json(&raw.join(format!("semantic-{suite}.json")),&json!({"suite":suite,"sanity_only":true,"note":"report-only frame-time observations; never a product budget","observations_ms":{"fixture":0.25}})).unwrap();
        }
        let mut result = Self {
            _temp: temp,
            workspace,
            payload,
            host,
            evidence: json!(null),
        };
        result.refresh();
        result
    }
    fn refresh(&mut self) {
        let raw = self.workspace.join("raw");
        let mut files = serde_json::Map::new();
        for entry in fs::read_dir(&raw).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap();
            if name == "return.json" {
                continue;
            }
            files.insert(name.into(),json!({"bytes":path.metadata().unwrap().len(),"sha256":store::digest(&path).unwrap()}));
        }
        let marker = json!({"schema":1,"payload_sha256":"d".repeat(64),"files":files});
        store::write_json(&raw.join("return.json"), &marker).unwrap();
        self.evidence = marker;
        self.evidence["raw"] = json!(raw);
    }
    fn check(&self) -> anyhow::Result<vm_semantic::Verified> {
        vm_semantic::validate_return(
            &serde_json::to_vec(&self.evidence)?,
            &self.workspace,
            &self.payload,
            &"d".repeat(64),
            &self.host,
            &AtomicBool::new(false),
        )
    }
}
#[test]
fn semantic_receipt_retains_original_identity_and_never_becomes_capture_evidence() {
    let f = Fixture::new();
    let result = f.check().unwrap();
    assert_eq!(result.raw(), f.workspace.join("raw"));
    result
        .recheck(
            &f.payload,
            &"d".repeat(64),
            &f.host,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(!result.raw().join("capture.json").exists());
    let mut changed_payload = f.payload.clone();
    changed_payload.files[1].sha256 = "a".repeat(64);
    assert!(result
        .recheck(
            &changed_payload,
            &"d".repeat(64),
            &f.host,
            &AtomicBool::new(false)
        )
        .is_err());
    let mut changed_host = f.host.clone();
    changed_host.sdk = "different SDK".into();
    assert!(result
        .recheck(
            &f.payload,
            &"d".repeat(64),
            &changed_host,
            &AtomicBool::new(false)
        )
        .is_err());
    assert!(result
        .recheck(
            &f.payload,
            &"a".repeat(64),
            &f.host,
            &AtomicBool::new(false)
        )
        .is_err());
    assert!(bbr_visual::vm_transport::validate_return(
        &serde_json::to_vec(&f.evidence).unwrap(),
        &f.workspace,
        bbr_visual::vm_transport::ExpectedReturn {
            suite: bbr_visual::Suite::Gallery,
            staged: &f.payload,
            digest: &"d".repeat(64),
            host: &f.host,
            selected: None
        },
        &AtomicBool::new(false)
    )
    .is_err());
}
#[test]
fn invalid_or_rebound_semantic_returns_fail_closed() {
    for mutation in [
        "purpose",
        "source",
        "missing",
        "extra",
        "image",
        "changed",
        "hardlink",
        "symlink",
        "marker",
        "payload",
        "run-failed",
        "plan-hash",
        "binary-hash",
        "result-payload",
        "diagnostic",
        "diagnostic-extra",
        "guest",
        "host",
        "cancelled",
    ] {
        let mut f = Fixture::new();
        let raw = f.workspace.join("raw");
        let path = raw.join(semantic::RESULT);
        match mutation {
            "purpose" => f.payload.suite = "gallery".into(),
            "source" => fs::write(f.payload.repo.join(semantic::PLAN), "{}").unwrap(),
            "missing" => {
                fs::remove_file(&path).unwrap();
                f.refresh();
            }
            "extra" | "image" => {
                fs::write(
                    raw.join(if mutation == "extra" {
                        "selection.json"
                    } else {
                        "fixture--light.png"
                    }),
                    "{}",
                )
                .unwrap();
                f.refresh();
            }
            "changed" => fs::write(&path, "{}").unwrap(),
            "hardlink" => fs::hard_link(&path, f.workspace.join("link")).unwrap(),
            "symlink" => {
                fs::rename(&path, f.workspace.join("copy")).unwrap();
                std::os::unix::fs::symlink(f.workspace.join("copy"), &path).unwrap();
            }
            "marker" => fs::write(raw.join("return.json"), "{}").unwrap(),
            "payload" => f.evidence["payload_sha256"] = json!("a".repeat(64)),
            "host" => f.host.executable_hashes.clear(),
            "cancelled" => {}
            _ => {
                let target = if mutation == "guest" {
                    raw.join("guest-receipt.json")
                } else if mutation.starts_with("diagnostic") {
                    raw.join("semantic-every-story-renders.json")
                } else {
                    path
                };
                let mut value: Value = store::read_json(&target).unwrap();
                match mutation {
                    "run-failed" => {
                        value["execution"] = json!(value["execution"]
                            .as_str()
                            .unwrap()
                            .replace(" ... ok", " ... FAILED"))
                    }
                    "plan-hash" => value["plan_sha256"] = json!("a".repeat(64)),
                    "binary-hash" => value["executable_sha256"] = json!("a".repeat(64)),
                    "result-payload" => value["payload_sha256"] = json!("a".repeat(64)),
                    "diagnostic" => value["sanity_only"] = json!(false),
                    "diagnostic-extra" => value["accepted"] = json!(true),
                    "guest" => value["environment_after"]["os_build"] = json!("changed"),
                    _ => unreachable!(),
                }
                store::write_json(&target, &value).unwrap();
                f.refresh();
            }
        }
        let result = vm_semantic::validate_return(
            &serde_json::to_vec(&f.evidence).unwrap(),
            &f.workspace,
            &f.payload,
            &"d".repeat(64),
            &f.host,
            &AtomicBool::new(mutation == "cancelled"),
        );
        assert!(result.is_err(), "{mutation}");
        assert!(!raw.join("capture.json").exists());
    }
}
#[test]
fn wrong_purpose_stale_sources_and_cancelled_inputs_cannot_start_transport() {
    for mode in ["purpose", "source", "cancelled"] {
        let mut f = Fixture::new();
        if mode == "purpose" {
            f.payload.suite = "gallery".into();
        }
        if mode == "source" {
            fs::write(f.payload.repo.join(semantic::PLAN), "{}").unwrap();
        }
        let workspace = f.workspace.join("must-not-exist");
        let options = cargo_gpui_visual::gate::Options {
            path: f.workspace.join("must-not-lock"),
            compatibility: vec![],
            wait: std::time::Duration::from_millis(50),
        };
        assert!(vm_semantic::run(
            vm_semantic::Input {
                repo: &f.payload.repo,
                share: &f.workspace.join("no-share"),
                staged: &f.payload,
                host: &f.host,
                workspace: &workspace,
            },
            &options,
            &AtomicBool::new(mode == "cancelled")
        )
        .is_err());
        assert!(!workspace.exists() && !options.path.exists());
    }
}

#[test]
fn recheck_does_not_adopt_a_rewritten_transport_manifest() {
    let mut f = Fixture::new();
    let verified = f.check().unwrap();
    let path = f.workspace.join("raw/semantic-every-story-renders.json");
    let mut value: Value = store::read_json(&path).unwrap();
    value["observations_ms"]["fixture"] = json!(0.5);
    store::write_json(&path, &value).unwrap();
    f.refresh();
    assert!(f.check().is_ok()); // New independent evidence describes valid different bytes.
    assert!(verified
        .recheck(
            &f.payload,
            &"d".repeat(64),
            &f.host,
            &AtomicBool::new(false)
        )
        .is_err());
}
