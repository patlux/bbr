//! Regression (2026-09-30): a launch without `--bbr-profile-dir` ran
//! without a store. The default was a per-process temporary directory that
//! nothing created, so the store failed to open, the session stayed in
//! memory and every launch started empty; every test passed the flag and
//! never saw it. Launched the way a user launches it — no flag, only an
//! isolated `HOME` so no test touches the real directory — BBR keeps its
//! profile in `~/Library/Application Support/<bundle id>` and the next
//! launch restores the tab that was open.

use std::path::{Path, PathBuf};

use crate::*;

/// The unbundled harness binary has no bundle identifier; it takes the id
/// of its build (`profile_location`): dev for a debug build.
fn expected_profile(home: &Path) -> PathBuf {
    let id = if cfg!(debug_assertions) {
        "dev.patwoz.bbr.dev"
    } else {
        "dev.patwoz.bbr"
    };
    home.join("Library").join("Application Support").join(id)
}

fn launch(home: &Path, root: &Path, page_one: &str, token: &str) -> Harness {
    Harness::spawn_with_env(
        env!("CARGO_BIN_EXE_bbr"),
        &[
            "--bbr-initial-url",
            page_one,
            "--bbr-test-token",
            token,
            "--bbr-ids=sequential",
            "--bbr-hidden",
        ],
        &[("HOME", home)],
        root,
    )
}

fn quit(client: &mut Client, harness: &mut Harness) {
    client.quit().expect("graceful quit");
    let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
    assert!(status.success(), "app exit: {status}");
}

#[test]
#[ignore]
fn a_launch_without_a_profile_flag_keeps_its_profile_in_application_support() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let root = harness_root("dflt", std::process::id());
    let _ = std::fs::remove_dir_all(&root);
    let home = root.join("h");
    std::fs::create_dir_all(&home).unwrap();
    let profile = expected_profile(&home);
    let socket = profile.join("bbr-test.sock");
    let token = format!("dflt-{}", std::process::id());

    let mut harness = launch(&home, &root, &page_one, &token);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let first = client.snapshot().expect("first snapshot");
        assert_eq!(
            first.store.store_load,
            StoreLoadSnapshot::Loaded,
            "the store opened in the default profile directory"
        );
        client
            .send_ok(&bbr_testkit::Request::OpenTab {
                url: page_two.clone(),
            })
            .expect("open tab");
        quit(&mut client, &mut harness);
        assert!(profile.join("bbr.sqlite").is_file(), "the store is on disk");
    }));
    match result {
        // Keep the root for the relaunch.
        Ok(()) => harness.success = false,
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    drop(harness);

    let mut harness = launch(&home, &root, &page_one, &token);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("restart snapshot");
        assert_eq!(restored.store.store_load, StoreLoadSnapshot::Loaded);
        assert!(
            matches!(restored.restore, RestoreDisposition::Restored { .. }),
            "the second launch restores: {:?}",
            restored.restore
        );
        assert!(
            restored.all_tabs.iter().any(|tab| tab.url == page_two),
            "the tab opened before quitting came back"
        );
        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}
