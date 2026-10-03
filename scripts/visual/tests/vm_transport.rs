//! Actual return staging/receipt checks with explicitly synthetic non-PNG data.
use bbr_visual::{
    payload::{Entry, Payload},
    vm_environment::{Guest, HostBuild, Receipt},
    vm_transport, Suite,
};
use cargo_gpui_visual::store;
use serde_json::{json, Value};
use std::{fs, path::Path, sync::atomic::AtomicBool};

fn fixture(workspace: &Path) -> (Payload, HostBuild, Value) {
    let raw = workspace.join("raw");
    fs::create_dir(&raw).unwrap();
    let repo = workspace.join("compiled");
    let mut sources: store::Hashes = [("fixture.rs".into(), "a".repeat(64))].into();
    sources.extend(
        bbr_visual::vm_rendering::TOOL_INPUTS
            .iter()
            .map(|p| (format!("vendor/cargo-gpui-visual/{p}"), "f".repeat(64))),
    );
    let files = vec![
        Entry {
            payload: "source/fixture.rs".into(),
            guest: repo.join("fixture.rs"),
            sha256: "a".repeat(64),
            bytes: 7,
            executable: false,
        },
        Entry {
            payload: "executables/0".into(),
            guest: repo.join("binary"),
            sha256: "b".repeat(64),
            bytes: 7,
            executable: true,
        },
    ];
    let payload = Payload {
        schema: 1,
        suite: "gallery".into(),
        repo,
        sources,
        files,
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
        executable_hashes: [(
            payload.files[1].guest.to_string_lossy().into_owned(),
            "b".repeat(64),
        )]
        .into(),
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
    fs::write(
        raw.join("guest-receipt.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    fs::write(raw.join("fixture--light.png"), b"fixture bytes, NOT PNG").unwrap();
    let mut files = serde_json::Map::new();
    for name in ["guest-receipt.json", "fixture--light.png"] {
        let path = raw.join(name);
        files.insert(
            name.into(),
            json!({"bytes":path.metadata().unwrap().len(),"sha256":store::digest(&path).unwrap()}),
        );
    }
    let marker = json!({"schema":1,"payload_sha256":"d".repeat(64),"files":files});
    fs::write(
        raw.join("return.json"),
        serde_json::to_vec(&marker).unwrap(),
    )
    .unwrap();
    let mut evidence = marker;
    evidence["raw"] = json!(raw);
    (payload, host, evidence)
}

#[test]
fn returned_bytes_receipt_and_host_build_bind_without_claiming_pixels_or_coverage() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let (payload, host, evidence) = fixture(&root);
    let captured = vm_transport::validate_return(
        &serde_json::to_vec(&evidence).unwrap(),
        &root,
        vm_transport::ExpectedReturn {
            suite: Suite::Gallery,
            staged: &payload,
            digest: &"d".repeat(64),
            host: &host,
            selected: None,
        },
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(captured.raw, root.join("raw"));
    assert_eq!(
        captured.environment["rendering"]["host_build"]["os_build"],
        "host-build"
    );
    assert_eq!(
        captured.environment["rendering"]["guest_rendering"]["os_build"],
        "guest-build"
    );
    assert!(!captured.raw.join("capture.json").exists());
}

#[test]
fn changed_missing_extra_linked_or_swapped_return_evidence_fails_closed() {
    for mode in [
        "payload",
        "raw",
        "missing",
        "extra",
        "changed",
        "symlink",
        "hardlink",
        "marker",
        "receipt",
        "cancelled",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let (payload, mut host, mut evidence) = fixture(&root);
        let path = root.join("raw/fixture--light.png");
        match mode {
            "payload" => evidence["payload_sha256"] = json!("f".repeat(64)),
            "raw" => evidence["raw"] = json!(root.join("other")),
            "missing" => fs::remove_file(&path).unwrap(),
            "extra" => fs::write(root.join("raw/extra.png"), "fixture").unwrap(),
            "changed" => fs::write(&path, "changed").unwrap(),
            "symlink" => {
                fs::rename(&path, root.join("original")).unwrap();
                std::os::unix::fs::symlink(root.join("original"), &path).unwrap();
            }
            "hardlink" => fs::hard_link(&path, root.join("hardlink")).unwrap(),
            "marker" => fs::write(root.join("raw/return.json"), "{}").unwrap(),
            "receipt" => host.executable_hashes.clear(),
            "cancelled" => {}
            _ => unreachable!(),
        }
        assert!(
            vm_transport::validate_return(
                &serde_json::to_vec(&evidence).unwrap(),
                &root,
                vm_transport::ExpectedReturn {
                    suite: Suite::Gallery,
                    staged: &payload,
                    digest: &"d".repeat(64),
                    host: &host,
                    selected: None
                },
                &AtomicBool::new(mode == "cancelled")
            )
            .is_err(),
            "{mode}"
        );
    }
}
