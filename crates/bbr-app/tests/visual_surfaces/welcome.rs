//! First-run and About stills (formerly withheld, 2026-10-12): a fresh
//! profile whose import home holds fixed Safari and Chrome data, the real
//! welcome steps through `WelcomeInput`, and the About window opened through
//! its production menu item. Make Default is skipped (it would ask macOS).
use super::{save, wait, Client, Still, PAGE};
use crate::e2e_common::{
    catch_unwind, connect, deadline, harness_root, harness_socket_path, AssertUnwindSafe, Harness,
};
use bbr_testkit::protocol::{dsl, Request, ThemeModeOverride, WelcomeInput};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// 2026-09-20 12:00 UTC in Chrome's microseconds since 1601.
const CHROME_TIME: i64 = (1_789_905_600 + 11_644_473_600) * 1_000_000;
/// 2026-09-10 12:00 UTC in Safari's seconds since 2001.
const SAFARI_TIME: f64 = 1_789_041_600.0 - 978_307_200.0;

/// The import home: one Chrome profile and Safari, as the first-run E2E.
fn import_home(dir: &Path) {
    let chrome = dir.join("Library/Application Support/Google/Chrome");
    let default = chrome.join("Default");
    fs::create_dir_all(&default).unwrap();
    fs::write(
        chrome.join("Local State"),
        r#"{"profile":{"info_cache":{"Default":{"name":"Patrick"}}}}"#,
    )
    .unwrap();
    fs::write(
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
    fs::create_dir_all(&safari).unwrap();
    fs::write(
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

fn welcome(client: &mut Client, input: WelcomeInput) {
    client
        .send_ok(&Request::WelcomeInput { input })
        .expect("welcome input");
}

pub fn capture() {
    let _serial = crate::e2e_common::serial_gui();
    for (appearance, mode) in [
        ("light", ThemeModeOverride::Light),
        ("dark", ThemeModeOverride::Dark),
    ] {
        let tag = format!("visual-welcome-{appearance}");
        let root = harness_root(&tag, std::process::id());
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        let home = root.join("import-home");
        import_home(&home);
        let socket = harness_socket_path(&tag, std::process::id());
        let token = format!("visual-welcome-{}", std::process::id());
        let output = PathBuf::from(std::env::var_os("BBR_VISUAL_CAPTURE_DIR").unwrap());
        fs::create_dir_all(&output).unwrap();
        let bin = std::env::var("BBR_VISUAL_CAPTURE_BIN").unwrap();
        let mut harness = Harness::spawn(
            &bin,
            &[
                "--bbr-profile-dir",
                profile.to_str().unwrap(),
                "--bbr-test-token",
                &token,
                "--bbr-ids=sequential",
                "--bbr-hidden",
                "--bbr-window-size=1080x1407",
                "--bbr-import-home",
                home.to_str().unwrap(),
            ],
            &root,
        );
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut client = connect(&socket);
            client.hello(&token).unwrap();
            client.attach(None).unwrap();
            client
                .send_ok(&dsl::set_appearance_override(Some(mode)))
                .unwrap();
            client
                .send_ok(&dsl::set_display_accessibility(
                    Some(false),
                    Some(true),
                    Some(false),
                ))
                .unwrap();
            client
                .send_ok(&Request::SetWindowSize {
                    width: 1080.0,
                    height: 1407.0,
                })
                .unwrap();
            crate::e2e_common::wait_for_window_settled(&mut client, deadline());
            first_run(&mut client, &output, appearance);
            about(&mut client, &output, appearance);
            client.quit().unwrap();
            crate::wait_for_process_exit(&mut harness, deadline());
        }));
        match result {
            Ok(()) => harness.success = true,
            Err(panic) => {
                harness.diagnostics(&socket, &token);
                std::panic::resume_unwind(panic);
            }
        }
    }
}

fn first_run(client: &mut Client, output: &Path, appearance: &str) {
    let step = |s: &bbr_testkit::protocol::Snapshot, name: &str| {
        s.onboarding.as_ref().is_some_and(|o| o.step == name)
    };
    let open = wait(client, |s| step(s, "welcome"));
    assert!(open.tabs.is_empty(), "one empty space");
    save(client, output, "welcome", appearance, PAGE, open);

    welcome(client, WelcomeInput::StartImport);
    let sources = wait(client, |s| {
        s.onboarding
            .as_ref()
            .and_then(|o| o.sources.as_ref())
            .is_some_and(|sources| sources.len() == 2)
    });
    let found = sources
        .onboarding
        .as_ref()
        .unwrap()
        .sources
        .clone()
        .unwrap();
    assert_eq!(found[0].0, "Safari");
    assert_eq!(found[1].0, "Google Chrome · Patrick");
    save(client, output, "welcome-import", appearance, PAGE, sources);

    welcome(client, WelcomeInput::Import);
    // The import toast leaves on its own timer; the still waits for it so
    // no capture races the toast.
    let imported = wait(client, |s| step(s, "default-browser") && s.toast.is_none());
    assert_eq!(imported.store.saved_pages_count, 3);
    save(
        client,
        output,
        "welcome-default-browser",
        appearance,
        PAGE,
        imported,
    );

    welcome(client, WelcomeInput::Skip);
    wait(client, |s| step(s, "first-space"));
    welcome(client, WelcomeInput::FocusName);
    for ch in "Home".chars() {
        super::key(client, &format!("{}->{ch}", ch.to_ascii_lowercase()));
    }
    let named = wait(client, |s| {
        s.onboarding
            .as_ref()
            .is_some_and(|o| o.step == "first-space" && o.name == "Home")
    });
    save(
        client,
        output,
        "welcome-first-space",
        appearance,
        PAGE,
        named,
    );
    super::key(client, "enter");
    wait(client, |s| s.onboarding.is_none());
}

fn about(client: &mut Client, output: &Path, appearance: &str) {
    client
        .send_ok(&Request::InvokeMenuItem {
            menu: "BBR".into(),
            item: "About BBR".into(),
        })
        .expect("BBR › About BBR");
    let open = wait(client, |s| s.about.is_some());
    assert_eq!(
        open.about.as_deref(),
        Some(concat!("Version ", env!("CARGO_PKG_VERSION")))
    );
    save(client, output, "about", appearance, Still::About, open);
}
