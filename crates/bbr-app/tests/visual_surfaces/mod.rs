//! Additional stills use actual application actions and windows, never Gallery.
use super::e2e_common::{deadline, Client, Instant};
use bbr_persistence::{CommandHistoryEntry, Repository};
use bbr_testkit::protocol::{HistoryInput, Request, SettingsInput, Snapshot};
use std::{fs, path::Path};

const PANES: [&str; 7] = [
    "general",
    "profiles",
    "search",
    "privacy",
    "downloads",
    "appearance",
    "shortcuts",
];

pub fn seed(profile: &Path) {
    let repository = Repository::new(profile.join("bbr.sqlite"));
    repository
        .initialize_and_restore()
        .expect("initialize fixture store");
    repository
        .store_command_history(&[
            CommandHistoryEntry {
                id: 1,
                title: "GPUI documentation".into(),
                url: "https://example.test/gpui".into(),
                visit_count: 3,
                last_visited: 7,
            },
            CommandHistoryEntry {
                id: 2,
                title: "Browser architecture notes".into(),
                url: "https://example.test/browser".into(),
                visit_count: 2,
                last_visited: 8,
            },
            CommandHistoryEntry {
                id: 3,
                title: "Weekend reading list".into(),
                url: "https://example.test/reading".into(),
                visit_count: 1,
                last_visited: 9,
            },
        ])
        .expect("seed fixed undated history without navigating external URLs");
}

fn wait(client: &mut Client, ready: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        client
            .send_ok(&Request::FlushFrame)
            .expect("flush surface frame");
        let snapshot = client.snapshot().expect("surface snapshot");
        if ready(&snapshot) && snapshot.motion.moments.is_empty() {
            return snapshot;
        }
        assert!(Instant::now() < until, "surface readiness deadline");
        std::thread::yield_now();
    }
}

fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("production keystroke");
}

fn save(
    client: &mut Client,
    output: &Path,
    case: &str,
    appearance: &str,
    settings: bool,
    before: Snapshot,
) {
    let temporary = format!(
        "/tmp/bbr-surface-{case}-{appearance}-{}.png",
        std::process::id()
    );
    assert!(!Path::new(&temporary).exists(), "fresh temporary readback");
    let request = if settings {
        Request::CaptureSettings {
            path: temporary.clone(),
        }
    } else {
        Request::CaptureChrome {
            path: temporary.clone(),
        }
    };
    client
        .send_ok(&request)
        .expect("actual window renderer readback");
    let image = image::open(&temporary).expect("read surface PNG");
    assert_eq!(
        (image.width(), image.height()),
        if settings { (1560, 1240) } else { (2160, 2814) },
        "exact 2x window; never rescale"
    );
    assert_eq!(image.color(), image::ColorType::Rgba8);
    let after = client.snapshot().expect("post-readback state");
    assert_eq!(after.tabs, before.tabs, "readback preserves tabs");
    assert_eq!(after.engine_sessions, before.engine_sessions);
    assert_eq!(after.active_profile, before.active_profile);
    assert_eq!(after.active_space, before.active_space);
    assert_eq!(
        serde_json::to_value(&after.settings).unwrap(),
        serde_json::to_value(&before.settings).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&after.history).unwrap(),
        serde_json::to_value(&before.history).unwrap()
    );
    assert_eq!(after.library.open, before.library.open);
    assert_eq!(after.shell_modes.theme, appearance);
    let destination = output.join(format!("{case}--{appearance}.png"));
    assert!(!destination.exists(), "never overwrite surface capture");
    fs::rename(temporary, destination).expect("preserve original PNG bytes");
    let metadata = output.join(format!("{case}--{appearance}.snapshot.json"));
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(metadata)
        .expect("fresh metadata");
    serde_json::to_writer_pretty(file, &after).expect("write semantic evidence");
}

pub fn capture(client: &mut Client, output: &Path, appearance: &str) {
    let absent_path = format!("/tmp/bbr-settings-absent-{}.png", std::process::id());
    let absent = client
        .send_ok(&Request::CaptureSettings {
            path: absent_path.clone(),
        })
        .expect_err("closed Settings cannot capture the browser instead");
    assert_eq!(absent.code, "settings_closed");
    assert!(!Path::new(&absent_path).exists());
    key(client, "cmd-y");
    let history = wait(client, |s| s.history.is_some());
    let page = history.history.as_ref().unwrap();
    assert!(page.focused);
    assert_eq!(page.groups.len(), 1);
    assert_eq!(page.groups[0].title, "Earlier");
    assert_eq!(page.groups[0].rows.len(), 3);
    assert!(page.groups[0].rows.iter().all(|r| r.time.is_none()));
    assert!(history.visible_ids.iter().any(|id| id == "bbr-history"));
    assert!(!history
        .visible_ids
        .iter()
        .any(|id| id.starts_with("bbr-native-web-slot")));
    save(client, output, "history", appearance, false, history);

    key(client, "cmd-shift-backspace");
    let sheet = wait(client, |s| {
        s.history.as_ref().is_some_and(|h| h.clear.is_some())
    });
    assert_eq!(
        sheet
            .history
            .as_ref()
            .unwrap()
            .clear
            .as_ref()
            .unwrap()
            .profile,
        "Default"
    );
    save(
        client,
        output,
        "history-clear-data",
        appearance,
        false,
        sheet,
    );
    client
        .send_ok(&Request::HistoryInput {
            input: HistoryInput::ClearCancel,
        })
        .expect("cancel without deleting data");
    key(client, "escape");
    wait(client, |s| s.history.is_none());

    client
        .send_ok(&Request::OpenSavedPagesLibrary)
        .expect("production Library action");
    let library = wait(client, |s| s.library.open);
    assert!(library.history.is_none());
    assert_eq!(library.library.total, 0);
    assert!(library.library.rows.is_empty());
    save(client, output, "library-empty", appearance, false, library);
    client
        .send_ok(&Request::CloseSavedPagesLibrary)
        .expect("close Library");
    wait(client, |s| !s.library.open);

    key(client, "cmd-,");
    wait(client, |s| s.settings.is_some());
    let invalid = client
        .send_ok(&Request::CaptureSettings {
            path: "/tmp/nested/forbidden.png".into(),
        })
        .expect_err("Settings shares the confined path contract");
    assert_eq!(invalid.code, "invalid_capture_path");
    for pane in PANES {
        client
            .send_ok(&Request::SettingsInput {
                input: SettingsInput::Pane { pane: pane.into() },
            })
            .expect("production settings pane");
        let snapshot = wait(client, |s| {
            s.settings.as_ref().is_some_and(|v| v.pane == pane)
        });
        let settings = snapshot.settings.as_ref().unwrap();
        assert!(!settings.rows.is_empty(), "actual pane rows required");
        assert!(settings.open_select.is_none());
        save(
            client,
            output,
            &format!("settings-{pane}"),
            appearance,
            true,
            snapshot,
        );
    }
    client
        .send_ok(&Request::CloseSettings)
        .expect("close actual Settings window");
    wait(client, |s| s.settings.is_none());
}
