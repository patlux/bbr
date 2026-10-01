//! Recovery must not reactivate the legacy host producer while Tart is unwired.
use bbr_visual::{process, runner, Suite};
use std::{path::Path, process::Command, sync::atomic::AtomicBool};

#[test]
fn unwired_native_entrypoints_fail_before_artifacts_or_builds() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("must-not-exist");
    for suite in [Suite::Gallery, Suite::Browser] {
        let error = runner::produce(&repo, suite, &output, &AtomicBool::new(false)).unwrap_err();
        assert!(error
            .to_string()
            .contains("native host capture is forbidden"));
        assert!(!output.exists());
    }
    for args in [
        vec!["check"],
        vec!["produce", "gallery"],
        vec!["produce", "browser"],
    ] {
        let result = cargo_gpui_visual::process::run(
            Command::new(env!("CARGO_BIN_EXE_bbr-visual"))
                .args(args)
                .env("GPUI_VISUAL_OUTPUT", &output),
            &Default::default(),
            &AtomicBool::new(false),
            true,
        )
        .unwrap();
        assert_eq!(result.status.code(), Some(2));
        assert!(!output.exists());
    }
}

#[test]
fn portable_doctor_does_not_need_capture_permission() {
    assert!(process::output(
        Command::new(env!("CARGO_BIN_EXE_bbr-visual")).arg("doctor"),
        &AtomicBool::new(false),
    )
    .unwrap()
    .contains("sealed source inputs"));
}
