//! Payload verification cannot relabel omitted sources or executables as complete.
use bbr_visual::{
    payload::{self, Entry, Payload},
    Suite,
};
use cargo_gpui_visual::{config::Config, store};
use std::{
    collections::BTreeMap, fs, os::unix::fs::PermissionsExt, path::PathBuf,
    sync::atomic::AtomicBool,
};

#[test]
fn current_source_policies_are_narrow_transfer_inputs_without_reading_build_outputs() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    for suite in [Suite::Gallery, Suite::Browser] {
        let config = Config::load(&repo.join(suite.config())).unwrap();
        let sources = payload::source_snapshot(&repo, &config, &AtomicBool::new(false)).unwrap();
        assert!(sources.contains_key("Cargo.lock"));
        assert!(sources.contains_key("scripts/visual/tool-source.json"));
        assert!(!sources.keys().any(|name| name.contains("/target/")
            || name.starts_with("reports/")
            || name.starts_with("tests/visual/")));
    }
}

#[test]
fn forged_completion_cannot_omit_sources_change_guest_destinations_or_change_suite() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    fs::create_dir(root.join("source")).unwrap();
    fs::create_dir(root.join("executables")).unwrap();
    fs::write(root.join("source/source.rs"), "source").unwrap();
    fs::write(root.join("executables/0"), "binary").unwrap();
    fs::set_permissions(
        root.join("executables/0"),
        fs::Permissions::from_mode(0o500),
    )
    .unwrap();
    let guest_repo = PathBuf::from("/Users/fixture/repo");
    let hash = store::digest(&root.join("source/source.rs")).unwrap();
    let original = Payload {
        schema: 1,
        suite: "gallery".into(),
        repo: guest_repo.clone(),
        sources: BTreeMap::from([("source.rs".into(), hash.clone())]),
        files: vec![
            Entry {
                payload: "source/source.rs".into(),
                guest: guest_repo.join("source.rs"),
                sha256: hash,
                bytes: 6,
                executable: false,
            },
            Entry {
                payload: "executables/0".into(),
                guest: guest_repo.join("target/release/bbr-gallery"),
                sha256: store::digest(&root.join("executables/0")).unwrap(),
                bytes: 6,
                executable: true,
            },
        ],
    };
    let mut original = original;
    fs::write(root.join("executables/1"), "consumer").unwrap();
    fs::set_permissions(
        root.join("executables/1"),
        fs::Permissions::from_mode(0o500),
    )
    .unwrap();
    original.files.push(Entry {
        payload: "executables/1".into(),
        guest: guest_repo.join("scripts/visual/target/release/bbr-visual"),
        sha256: store::digest(&root.join("executables/1")).unwrap(),
        bytes: 8,
        executable: true,
    });
    store::write_json(&root.join("payload.json"), &original).unwrap();
    payload::verify(&root, &original, &AtomicBool::new(false)).unwrap();
    for mutation in [
        "source",
        "destination",
        "suite",
        "role",
        "duplicate",
        "traversal",
        "hash",
        "absolute",
    ] {
        let mut forged = original.clone();
        match mutation {
            "source" => {
                forged.sources.insert("missing.rs".into(), "f".repeat(64));
            }
            "destination" => forged.files[0].guest = guest_repo.join("different.rs"),
            "suite" => forged.suite = "browser".into(),
            "role" => forged.files[1].guest = guest_repo.join("target/release/foreign"),
            "duplicate" => forged.files.push(forged.files[0].clone()),
            "traversal" => forged.files[0].guest = guest_repo.join("../outside"),
            "hash" => forged.files[0].sha256 = "f".into(),
            "absolute" => forged.repo = PathBuf::from("relative"),
            _ => unreachable!(),
        }
        store::write_json(&root.join("payload.json"), &forged).unwrap();
        assert!(
            payload::verify(&root, &forged, &AtomicBool::new(false)).is_err(),
            "{mutation}"
        );
    }
}
