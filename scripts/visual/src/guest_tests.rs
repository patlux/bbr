//! Pure fixed-command and installed-byte tests; never executes native producers.
use super::*;
use crate::payload::{Entry, Payload};
use std::{collections::BTreeMap, os::unix::fs::PermissionsExt};

fn fixture(repo: &Path) -> Payload {
    Payload {
        schema: 1,
        suite: "browser".into(),
        repo: repo.into(),
        sources: BTreeMap::new(),
        files: vec![Entry {
            payload: "executables/0".into(),
            guest: repo.join("target/release/deps/e2e_visual_shell_capture-fixture"),
            sha256: "a".repeat(64),
            bytes: 1,
            executable: true,
        }],
    }
}
fn arguments(command: &Command) -> Vec<String> {
    command
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}
#[test]
fn only_fixed_native_commands_are_constructed_without_execution() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let mut payload = fixture(&root);
    let plan = "scripts/visual/catalogs/gallery-plan.json";
    fs::create_dir_all(root.join("scripts/visual/catalogs")).unwrap();
    store::write_json(&root.join(plan), &serde_json::json!({"schema":1,"cases":[{
        "base":"gallery-16-state-00-standard-normal","label":"Checkbox","state":"default","group":"Core controls","ranges":[[0,5]]
    }]})).unwrap();
    payload
        .sources
        .insert(plan.into(), store::digest(&root.join(plan)).unwrap());
    let gallery = command(&payload, Suite::Gallery, "capture", &root).unwrap();
    assert_eq!(
        gallery.get_program(),
        root.join("target/release/bbr-gallery")
    );
    assert_eq!(
        arguments(&gallery),
        vec![
            "--capture-gallery".to_owned(),
            root.join("raw").to_string_lossy().into_owned()
        ]
    );
    assert_eq!(
        arguments(&command(&payload, Suite::Gallery, "inventory", &root).unwrap()),
        ["--capture-inventory"]
    );
    assert_eq!(
        arguments(&command(&payload, Suite::Browser, "capture", &root).unwrap()),
        ["--ignored", "--nocapture", "--test-threads=1"]
    );
    for suite in [Suite::Gallery, Suite::Browser] {
        assert!(command(&payload, suite, "arbitrary-shell", &root).is_err());
    }
    assert!(command(&payload, Suite::Browser, "inventory", &root).is_err());
    assert!(command(&payload, Suite::Gallery, "capture-selected", &root).is_err());
    assert!(action(Suite::Browser, "capture-selected").is_err());
    let mut requirements: serde_json::Value = store::read_json(&root.join(plan)).unwrap();
    requirements["schema"] = serde_json::json!(2);
    for endpoint in [
        "initial-unchecked",
        "checked-after-click",
        "unchecked-after-second-click",
    ] {
        let mut case = requirements["cases"][0].clone();
        case["base"] = serde_json::json!(format!(
            "gallery-16-state-00-standard-normal-endpoint-{endpoint}"
        ));
        case["endpoint"] = serde_json::json!(endpoint);
        requirements["cases"].as_array_mut().unwrap().push(case);
    }
    store::write_json(&root.join(plan), &requirements).unwrap();
    payload
        .sources
        .insert(plan.into(), store::digest(&root.join(plan)).unwrap());
    let error = command(&payload, Suite::Gallery, "capture", &root).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("endpoint execution/evidence is not wired"),
        "{error:#}"
    );
    assert!(!root.join("raw").exists());
    fs::create_dir(root.join("raw")).unwrap();
    assert!(command(&payload, Suite::Gallery, "capture", &root).is_err());
    fs::write(root.join("live-inventory.tsv"), "preserved").unwrap();
    assert!(command(&payload, Suite::Gallery, "inventory", &root).is_err());
}
#[test]
fn manifest_preflight_binds_bounded_regular_bytes_before_parsing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = root.join("payload.json");
    store::write_json(&path, &fixture(&root)).unwrap();
    let digest = store::digest(&path).unwrap();
    let cancelled = AtomicBool::new(false);
    assert!(read_manifest(&path, &digest, &cancelled).is_ok());
    assert!(read_manifest(&path, &"0".repeat(64), &cancelled).is_err());
    assert!(read_manifest(&path, &digest, &AtomicBool::new(true)).is_err());
    let link = root.join("alias");
    fs::hard_link(&path, &link).unwrap();
    assert!(read_manifest(&path, &digest, &cancelled).is_err());
    fs::remove_file(link).unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();
    assert!(read_manifest(&path, &digest, &cancelled).is_err());
    fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(root.join("missing"), &path).unwrap();
    assert!(read_manifest(&path, &digest, &cancelled).is_err());
}

#[test]
fn installed_source_and_executable_bytes_modes_and_links_are_checked() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let mut payload = fixture(&root);
    let path = &payload.files[0].guest;
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "x").unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o500)).unwrap();
    payload.files[0].sha256 = store::digest(&payload.files[0].guest).unwrap();
    let source = root.join("source.rs");
    fs::write(&source, "source").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o400)).unwrap();
    payload.files.push(Entry {
        payload: "source/source.rs".into(),
        guest: source.clone(),
        sha256: store::digest(&source).unwrap(),
        bytes: 6,
        executable: false,
    });
    let cancelled = AtomicBool::new(false);
    verify_installed(&payload, &cancelled).unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&source, "drift!").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(verify_installed(&payload, &cancelled).is_err());
    fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&source, "source").unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(verify_installed(&payload, &AtomicBool::new(true)).is_err());
    let path = &payload.files[0].guest;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(verify_installed(&payload, &cancelled).is_err());
    fs::set_permissions(path, fs::Permissions::from_mode(0o4500)).unwrap();
    assert!(verify_installed(&payload, &cancelled).is_err());
    fs::set_permissions(path, fs::Permissions::from_mode(0o500)).unwrap();
    let link = root.join("unexpected-link");
    fs::hard_link(path, &link).unwrap();
    assert!(verify_installed(&payload, &cancelled).is_err());
    fs::remove_file(link).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(path, "y").unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o500)).unwrap();
    assert!(verify_installed(&payload, &cancelled).is_err());
    fs::remove_file(path).unwrap();
    std::os::unix::fs::symlink(root.join("missing"), path).unwrap();
    assert!(verify_installed(&payload, &cancelled).is_err());
}
