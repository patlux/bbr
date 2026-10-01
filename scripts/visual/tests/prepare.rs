//! Host preparation rejects bad destinations before probes, builds or output.
use bbr_visual::{prepare, Suite};
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

#[test]
fn invalid_payload_destinations_fail_before_host_preparation() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    for destination in [
        PathBuf::from("relative"),
        repo.join("reports/visual/payload"),
        root.clone(),
        root.join("missing-parent/payload"),
        root.join("../traversal"),
    ] {
        assert!(
            prepare::prepare(&repo, Suite::Gallery, &destination, &AtomicBool::new(false)).is_err()
        );
    }
    assert!(!root.join("missing-parent").exists());
    assert!(prepare::prepare(
        &repo,
        Suite::Gallery,
        &root.join("cancelled"),
        &AtomicBool::new(true)
    )
    .is_err());
    assert!(!root.join("cancelled").exists());
}
