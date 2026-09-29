//! Blueprint D15 real-app E2E (boards L-1, L-2, PR-01): the first run.
//!
//! A launch with a fresh store and no address opens one empty space whose
//! card shows the welcome (not the empty-space pane). Get Started ends it
//! and opens the ⌘T palette. "Import from Safari or Chrome…" leads through
//! the three skippable steps: the import step reads a fixture home (never
//! the user's own Library) with a Chrome profile (Bookmarks JSON, History
//! SQLite, Local State name) and Safari (Bookmarks.plist, History.db), lists
//! both with counts, and Import lands the bookmarks as Saved Pages and the
//! history in History; the default-browser step is skipped (Make Default
//! would ask macOS); the first space is named by typing. The welcome never
//! returns on the next launch.

#![cfg(target_os = "macos")]

mod e2e_common;

use std::path::Path;

use bbr_testkit::protocol::{CommandSurface, Snapshot, WelcomeInput};
use bbr_testkit::Request;
use e2e_common::*;

/// 2026-09-20 12:00:00 UTC in Chrome's microseconds since 1601.
const CHROME_TIME: i64 = (1_789_905_600 + 11_644_473_600) * 1_000_000;
/// 2026-09-10 12:00:00 UTC in Safari's seconds since 2001.
const SAFARI_TIME: f64 = 1_789_041_600.0 - 978_307_200.0;

fn spawn(tag: &str, home: &Path) -> (Harness, PathBuf, String) {
    let pid = std::process::id();
    let root = harness_root(tag, pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = harness_socket_path(tag, pid);
    let token = format!("shell-{tag}-{pid}");
    let harness = Harness::spawn(
        env!("CARGO_BIN_EXE_bbr"),
        &[
            "--bbr-profile-dir",
            profile.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
            "--bbr-hidden",
            "--bbr-import-home",
            home.to_str().unwrap(),
        ],
        &root,
    );
    (harness, socket, token)
}

fn fixture_home(dir: &Path) {
    let chrome = dir.join("Library/Application Support/Google/Chrome");
    let default = chrome.join("Default");
    std::fs::create_dir_all(&default).unwrap();
    std::fs::write(
        chrome.join("Local State"),
        r#"{"profile":{"info_cache":{"Default":{"name":"Patrick"}}}}"#,
    )
    .unwrap();
    std::fs::write(
        default.join("Bookmarks"),
        r#"{"roots":{"bookmark_bar":{"type":"folder","children":[
            {"type":"url","name":"Rust","url":"https://www.rust-lang.org/"},
            {"type":"url","name":"Docs","url":"https://docs.rs/"}]},
            "other":{"type":"folder","children":[]}}}"#,
    )
    .unwrap();
    let history = rusqlite::Connection::open(default.join("History")).unwrap();
    history
        .execute_batch(
            "CREATE TABLE urls(id INTEGER PRIMARY KEY, url TEXT, title TEXT, visit_count INTEGER, \
             last_visit_time INTEGER, hidden INTEGER DEFAULT 0);",
        )
        .unwrap();
    for (url, title, count) in [
        ("https://news.example/", "News", 5),
        ("https://mail.example/inbox", "Inbox", 12),
        ("chrome://settings/", "Settings", 1),
    ] {
        history
            .execute(
                "INSERT INTO urls(url, title, visit_count, last_visit_time) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![url, title, count, CHROME_TIME],
            )
            .unwrap();
    }
    drop(history);
    let safari = dir.join("Library/Safari");
    std::fs::create_dir_all(&safari).unwrap();
    std::fs::write(
        safari.join("Bookmarks.plist"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>WebBookmarkType</key><string>WebBookmarkTypeList</string>
<key>Children</key><array><dict><key>WebBookmarkType</key><string>WebBookmarkTypeLeaf</string>
<key>URLString</key><string>https://webkit.org/</string>
<key>URIDictionary</key><dict><key>title</key><string>WebKit</string></dict></dict></array></dict></plist>"#,
    )
    .unwrap();
    let db = rusqlite::Connection::open(safari.join("History.db")).unwrap();
    db.execute_batch(
        "CREATE TABLE history_items(id INTEGER PRIMARY KEY, url TEXT, visit_count INTEGER);
         CREATE TABLE history_visits(id INTEGER PRIMARY KEY, history_item INTEGER, visit_time REAL, title TEXT);",
    )
    .unwrap();
    db.execute(
        "INSERT INTO history_items(id, url, visit_count) VALUES (1, 'https://news.example/', 3)",
        [],
    )
    .unwrap();
    db.execute(
        "INSERT INTO history_visits(history_item, visit_time, title) VALUES (1, ?1, 'News')",
        rusqlite::params![SAFARI_TIME],
    )
    .unwrap();
}

fn snapshot(client: &mut Client) -> Snapshot {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
    client.snapshot().expect("snapshot")
}

fn welcome(client: &mut Client, input: WelcomeInput) -> Snapshot {
    client
        .send_ok(&Request::WelcomeInput { input })
        .expect("welcome input");
    snapshot(client)
}

fn wait(client: &mut Client, what: &str, done: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        let s = snapshot(client);
        if done(&s) {
            return s;
        }
        assert!(Instant::now() < until, "{what} never happened");
    }
}

#[test]
#[ignore]
fn the_first_run_welcomes_imports_and_names_the_first_space() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let home = std::env::temp_dir().join(format!("bbr-first-run-home-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    fixture_home(&home);
    let (mut harness, socket, token) = spawn("first-run", &home);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        let open = wait(&mut client, "the welcome", |s| s.onboarding.is_some());
        assert!(open.tabs.is_empty(), "one empty space");
        assert_eq!(open.onboarding.as_ref().unwrap().step, "welcome");
        for id in [
            "bbr-onboarding",
            "bbr-welcome",
            "bbr-welcome-start",
            "bbr-welcome-import",
        ] {
            assert!(open.visible_ids.iter().any(|v| v == id), "{id}");
        }
        assert!(!open.visible_ids.iter().any(|v| v == "bbr-empty-new-tab"));

        // Import: both browsers are found in the fixture home.
        welcome(&mut client, WelcomeInput::StartImport);
        let found = wait(&mut client, "the sources", |s| {
            s.onboarding.as_ref().is_some_and(|o| o.sources.is_some())
        });
        let sources = found.onboarding.unwrap().sources.unwrap();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].0, "Safari");
        assert_eq!(sources[0].1, "1 bookmark · less than 2 weeks of history");
        assert_eq!(sources[1].0, "Google Chrome · Patrick");
        assert_eq!(sources[1].1, "2 bookmarks · less than 2 weeks of history");
        assert!(sources
            .iter()
            .all(|(_, _, checked, enabled)| *checked && *enabled));
        let before = found.store.history_count;

        // Import lands 3 Saved Pages and the 2 web visits (news merges).
        let imported = welcome(&mut client, WelcomeInput::Import);
        assert_eq!(imported.store.saved_pages_count, 3);
        assert_eq!(imported.store.history_count, before + 2);
        assert_eq!(
            imported.toast.as_ref().map(|t| t.message.as_str()),
            Some("Imported 3 Saved Pages and 2 history entries")
        );
        assert_eq!(imported.onboarding.unwrap().step, "default-browser");

        // Skip the default browser (Make Default would ask macOS).
        let space_step = welcome(&mut client, WelcomeInput::Skip);
        assert_eq!(space_step.onboarding.unwrap().step, "first-space");

        // Type the first space's name; Return finishes.
        welcome(&mut client, WelcomeInput::FocusName);
        for ch in "Home".chars() {
            client
                .send_ok(&Request::DispatchKeystroke {
                    keystroke: format!("{}->{ch}", ch.to_ascii_lowercase()),
                })
                .expect("type");
        }
        assert_eq!(snapshot(&mut client).onboarding.unwrap().name, "Home");
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "enter".into(),
            })
            .expect("enter");
        let done = wait(&mut client, "the finished first run", |s| {
            s.onboarding.is_none()
        });
        assert_eq!(done.spaces[0].title, "Home");
        assert!(done.visible_ids.iter().any(|v| v == "bbr-empty-new-tab"));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);

    // The welcome never returns.
    let (mut again, socket, token) = spawn("first-run", &home);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        let settled = wait_for_window_settled(&mut client, deadline());
        let later = snapshot(&mut client);
        assert!(later.onboarding.is_none(), "{:?}", later.onboarding);
        assert_eq!(settled.spaces[0].title, "Home");
        client.quit().expect("quit");
        let status = wait_for_exit(&mut again.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut again, &socket, &token);
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
#[ignore]
fn get_started_opens_the_palette() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let home = std::env::temp_dir().join(format!("bbr-first-run-empty-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    let (mut harness, socket, token) = spawn("first-run-start", &home);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        wait(&mut client, "the welcome", |s| s.onboarding.is_some());
        let started = welcome(&mut client, WelcomeInput::GetStarted);
        assert!(started.onboarding.is_none());
        assert_eq!(started.command_surface, CommandSurface::Palette);
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
    let _ = std::fs::remove_dir_all(&home);
}
