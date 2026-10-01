//! Host runtime receipt binding only; no native execution or VM/network probes.
use bbr_visual::{payload, vm_runtime};
use cargo_gpui_visual::store;
use serde_json::json;
use std::fs;

#[test]
fn runtime_receipt_requires_original_lock_payload_and_every_executable() {
    let temp = tempfile::tempdir().unwrap();
    let share = temp.path().canonicalize().unwrap();
    let lock_path = share.join("source/scripts/visual/guest-runtime.json");
    fs::create_dir_all(lock_path.parent().unwrap()).unwrap();
    let python = json!({"path":"/fixture/python","version":"fixture","sha256":"a".repeat(64)});
    fs::write(
        &lock_path,
        json!({"python":python,"paths":{"fixture":{"narSize":17}}}).to_string(),
    )
    .unwrap();
    fs::write(
        share.join("payload.json"),
        "synthetic payload identity, not a manifest",
    )
    .unwrap();
    let lock_hash = store::digest(&lock_path).unwrap();
    let staged = payload::Payload {
        schema: 1,
        suite: "gallery".into(),
        repo: share.join("compiled"),
        sources: [(
            "scripts/visual/guest-runtime.json".into(),
            lock_hash.clone(),
        )]
        .into(),
        files: vec![payload::Entry {
            payload: "executables/0".into(),
            guest: share.join("compiled/consumer"),
            sha256: "b".repeat(64),
            bytes: 1,
            executable: true,
        }],
    };
    let good = json!({"schema":1,"origin":"host-runtime-prerequisite",
        "payload_sha256":store::digest(&share.join("payload.json")).unwrap(),
        "runtime_lock_sha256":lock_hash,"python":python,
        "dependencies":{staged.files[0].guest.to_string_lossy().into_owned():[]},
        "closure_paths":1,"closure_nar_bytes":17});
    vm_runtime::validate(&serde_json::to_vec(&good).unwrap(), &share, &staged).unwrap();
    for (field, value) in [
        ("schema", json!(2)),
        ("origin", json!("tart-guest")),
        ("payload_sha256", json!("f".repeat(64))),
        ("runtime_lock_sha256", json!("f".repeat(64))),
        (
            "python",
            json!({"path":"/other","version":"fixture","sha256":"a".repeat(64)}),
        ),
        ("closure_paths", json!(2)),
        ("closure_nar_bytes", json!(18)),
        ("dependencies", json!({})),
    ] {
        let mut bad = good.clone();
        bad[field] = value;
        assert!(
            vm_runtime::validate(&serde_json::to_vec(&bad).unwrap(), &share, &staged).is_err(),
            "{field}"
        );
    }
    let mut bad = good.clone();
    bad["dependencies"][staged.files[0].guest.to_string_lossy().into_owned()] =
        json!(["/unlocked/library"]);
    assert!(vm_runtime::validate(&serde_json::to_vec(&bad).unwrap(), &share, &staged).is_err());
    fs::write(&lock_path, "changed").unwrap();
    assert!(vm_runtime::validate(&serde_json::to_vec(&good).unwrap(), &share, &staged).is_err());
}

#[test]
fn checked_in_runtime_lock_is_in_both_narrow_source_policies() {
    use bbr_visual::Suite;
    use cargo_gpui_visual::config::Config;
    use std::{path::Path, sync::atomic::AtomicBool};
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    for suite in [Suite::Gallery, Suite::Browser] {
        let config = Config::load(&repo.join(suite.config())).unwrap();
        let sources = payload::source_snapshot(&repo, &config, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            sources["scripts/visual/guest-runtime.json"],
            store::digest(&repo.join("scripts/visual/guest-runtime.json")).unwrap()
        );
    }
}
