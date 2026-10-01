//! Real-app startup recovery E2E (blueprint D17, DESIGN §11.8): a store
//! written by a newer BBR is left byte-for-byte unchanged while BBR runs a
//! fresh in-memory session and says so; an unreadable store is kept aside
//! before the discard and the empty space explains it. Profile data is never
//! touched. Every step waits on typed snapshot state under a deadline.

#![cfg(target_os = "macos")]

mod e2e_common;

use std::path::Path;

use e2e_common::*;

/// Spawn BBR on a prepared profile directory (no initial tab URL: the
/// recovery start opens none).
fn spawn_on(
    tag: &str,
    prepare: impl FnOnce(&Path),
) -> (Harness, std::path::PathBuf, String, std::path::PathBuf) {
    let pid = std::process::id();
    let root = harness_root(tag, pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).expect("profile dir");
    prepare(&profile);
    let socket = harness_socket_path(tag, pid);
    let token = format!("shell-{tag}-{pid}");
    let harness = Harness::spawn(
        env!("CARGO_BIN_EXE_bbr"),
        &[
            "--bbr-profile-dir",
            profile.to_str().expect("profile path"),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
            "--bbr-hidden",
        ],
        &root,
    );
    (harness, socket, token, profile)
}

fn store_with(profile: &Path, sql: &str) {
    let connection = rusqlite::Connection::open(profile.join("bbr.sqlite")).expect("store");
    connection.execute_batch(sql).expect("prepare store");
}

fn wait_recovery(client: &mut Client, expected: &str) -> bbr_testkit::protocol::Snapshot {
    let until = deadline();
    loop {
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush");
        let snapshot = client.snapshot().expect("recovery snapshot");
        if snapshot.recovery.as_deref() == Some(expected) {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "recovery pane {expected} never showed: {:?}",
            snapshot.recovery
        );
    }
}

#[test]
#[ignore]
fn a_newer_store_is_left_unchanged_and_the_session_runs_fresh() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (mut harness, socket, token, profile) = spawn_on("recovery-newer", |profile| {
        store_with(
            profile,
            "CREATE TABLE future_rows (id INTEGER PRIMARY KEY, value TEXT);
             INSERT INTO future_rows (value) VALUES ('kept');
             PRAGMA user_version = 99;",
        );
    });
    let before = std::fs::read(profile.join("bbr.sqlite")).expect("store bytes");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        let shown = wait_recovery(&mut client, "newer");
        assert!(shown.tabs.is_empty(), "a fresh, empty session");
        // Continue shows the ordinary empty space.
        client
            .send_ok(&bbr_testkit::Request::RecoveryAction)
            .expect("continue");
        let until = deadline();
        loop {
            client
                .send_ok(&bbr_testkit::Request::FlushFrame)
                .expect("flush");
            if client
                .snapshot()
                .expect("after continue")
                .recovery
                .is_none()
            {
                break;
            }
            assert!(Instant::now() < until, "Continue never dismissed the pane");
        }
        // Opening and closing tabs writes nothing to the newer store.
        client
            .send_ok(&bbr_testkit::protocol::dsl::open_tab("about:blank"))
            .expect("open a tab");
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
    let after = std::fs::read(profile.join("bbr.sqlite")).expect("store bytes");
    assert_eq!(after, before, "the newer store is byte-for-byte unchanged");
}

#[test]
#[ignore]
fn an_unreadable_store_is_kept_aside_and_the_empty_space_says_so() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (mut harness, socket, token, profile) = spawn_on("recovery-unreadable", |profile| {
        // The current stamp over a structure BBR cannot read.
        store_with(
            profile,
            "CREATE TABLE unrelated (id INTEGER PRIMARY KEY);
             INSERT INTO unrelated DEFAULT VALUES;
             PRAGMA user_version = 5;",
        );
    });
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        let shown = wait_recovery(&mut client, "unreadable-kept");
        assert!(shown.tabs.is_empty(), "BBR started with an empty space");
        let kept: Vec<String> = std::fs::read_dir(&profile)
            .expect("profile dir")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".unreadable-"))
            .collect();
        assert_eq!(kept.len(), 1, "exactly one kept copy: {kept:?}");
        let copy = rusqlite::Connection::open(profile.join(&kept[0])).expect("kept copy");
        let rows: i64 = copy
            .query_row("SELECT count(*) FROM unrelated", [], |row| row.get(0))
            .expect("the kept copy is the old store");
        assert_eq!(rows, 1);
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
