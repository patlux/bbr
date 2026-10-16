//! Additional stills use actual application actions and windows, never Gallery.
pub mod command;
pub mod downloads;
pub mod find;
pub mod footer;
mod history;
pub mod history_dated;
mod library;
pub mod portal;
mod settings;
pub mod welcome;
use super::e2e_common::{deadline, Client, Instant};
use bbr_persistence::{CommandHistoryEntry, Repository};
use bbr_testkit::protocol::{HistoryInput, Request, Response, SettingsInput, Snapshot};
use std::{fs, path::Path};

/// What one still reads back (Patrick, 2026-10-12): only its subject, so a
/// change elsewhere in the window never touches its reference. Elements are
/// read back as the union of their drawn rects, grown by
/// [`SUBJECT_MARGIN_PT`] for edge rings and near shadows; Settings is its
/// own window.
#[derive(Clone, Copy, Debug)]
pub enum Still {
    Settings,
    /// The About BBR window (340 x 380 pt).
    About,
    Elements(&'static [&'static str]),
}

/// A History or Saved Pages still: the content card the page draws in.
pub const PAGE: Still = Still::Elements(&["bbr-content-card"]);

/// Rings (1 pt), the lift and the near part of `elev:2`; the long shadow
/// tail would reach into neighbouring chrome.
pub const SUBJECT_MARGIN_PT: f32 = 4.0;

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
    still: Still,
    before: Snapshot,
) {
    let settings = matches!(still, Still::Settings);
    let scroll_before = settings.then(|| {
        client
            .request(&Request::ScrollSettings { delta_y: 0 })
            .expect("Settings scroll observation")
    });
    let temporary = format!(
        "/tmp/bbr-surface-{case}-{appearance}-{}.png",
        std::process::id()
    );
    assert!(!Path::new(&temporary).exists(), "fresh temporary readback");
    let region = match still {
        Still::Settings => {
            client
                .send_ok(&Request::CaptureSettings {
                    path: temporary.clone(),
                })
                .expect("actual Settings window renderer readback");
            None
        }
        Still::About => {
            client
                .send_ok(&Request::CaptureAbout {
                    path: temporary.clone(),
                })
                .expect("actual About window renderer readback");
            None
        }
        Still::Elements(ids) => {
            let reply = client
                .request(&Request::CaptureElements {
                    path: temporary.clone(),
                    ids: ids.iter().map(|id| id.to_string()).collect(),
                    margin: SUBJECT_MARGIN_PT,
                })
                .expect("element readback request");
            let Response::CapturedRegion { pixels } = reply else {
                panic!("{case}: element readback of {ids:?} failed: {reply:?}");
            };
            Some((ids, pixels))
        }
    };
    let image = image::open(&temporary).expect("read surface PNG");
    match region {
        None => assert_eq!(
            (image.width(), image.height()),
            if settings { (1560, 1240) } else { (680, 760) },
            "exact 2x Settings or About window; never rescale"
        ),
        Some((_, [x, y, width, height])) => {
            assert_eq!(
                (image.width(), image.height()),
                (width, height),
                "exactly the reported element region; never rescale"
            );
            assert!(
                x + width <= 2160 && y + height <= 2814,
                "inside the 2x window"
            );
            assert!(
                width < 2160 || height < 2814,
                "{case}: the subject, never the whole window"
            );
        }
    }
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
    assert_eq!(
        serde_json::to_value(&after.library).unwrap(),
        serde_json::to_value(&before.library).unwrap(),
        "readback preserves exact Library state"
    );
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
    let mut metadata = serde_json::to_value(&after).unwrap();
    if let Some((ids, pixels)) = region {
        metadata["visual_region"] = serde_json::json!({
            "ids": ids,
            "margin_pt": SUBJECT_MARGIN_PT,
            "pixels": pixels,
        });
    }
    if let Some(scroll) = scroll_before {
        assert!(matches!(
            scroll,
            bbr_testkit::protocol::Response::SettingsScroll { .. }
        ));
        assert_eq!(
            client
                .request(&Request::ScrollSettings { delta_y: 0 })
                .unwrap(),
            scroll,
            "readback preserves measured scroll state"
        );
        metadata["visual_settings_scroll"] = serde_json::to_value(scroll).unwrap();
    }
    serde_json::to_writer_pretty(file, &metadata).expect("write semantic evidence");
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
    assert_eq!(
        client
            .send_ok(&Request::ScrollSettings { delta_y: 0 })
            .unwrap_err()
            .code,
        "settings_closed"
    );
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
    save(client, output, "history", appearance, PAGE, history);

    history::filters(client, output, appearance);
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
        PAGE,
        sheet,
    );
    client
        .send_ok(&Request::HistoryInput {
            input: HistoryInput::ClearRangeMenu,
        })
        .unwrap();
    let range = wait(client, |s| {
        s.history
            .as_ref()
            .and_then(|h| h.clear.as_ref())
            .is_some_and(|c| c.range_open)
    });
    save(
        client,
        output,
        "history-clear-range-menu",
        appearance,
        PAGE,
        range,
    );
    key(client, "escape");
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
    save(client, output, "library-empty", appearance, PAGE, library);
    for (url, title) in [
        ("https://docs.example.test/gpui", "GPUI documentation"),
        (
            "https://reading.example.test/browser",
            "Browser architecture notes",
        ),
        (
            "https://weekend.example.test/reading",
            "Weekend reading list",
        ),
    ] {
        client
            .send_ok(&Request::SaveSavedPage {
                url: url.into(),
                title: title.into(),
            })
            .expect("seed through production Saved Pages authority without navigation");
    }
    let populated = wait(client, |s| {
        s.library.total == 3 && s.library.rows.len() == 3
    });
    assert_eq!(populated.store.saved_pages_count, 3);
    assert!(populated.library.selected.is_empty());
    assert!(populated.library.collections.is_empty());
    save(
        client,
        output,
        "library-populated",
        appearance,
        PAGE,
        populated,
    );
    library::extended(client, output, appearance);
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
    for delta_y in [i32::MIN, -2001, 2001, i32::MAX] {
        assert_eq!(
            client
                .send_ok(&Request::ScrollSettings { delta_y })
                .unwrap_err()
                .code,
            "invalid_scroll"
        );
    }
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
            Still::Settings,
            snapshot,
        );
    }
    settings::extended(client, output, appearance);
    client
        .send_ok(&Request::CloseSettings)
        .expect("close actual Settings window");
    wait(client, |s| s.settings.is_none());
    history::closed(client, output, appearance);
    // Creating a profile activates its first space: always last.
    settings::profiles(client, output, appearance);
}

/// A portal-child still: the transparent scene of whatever surface the
/// portal presents (prompts, sheets, Peek's bar), never the parent or page.
pub fn save_portal(
    client: &mut Client,
    output: &Path,
    case: &str,
    appearance: &str,
    before: Snapshot,
) {
    // Presentations and the transient overlays share the synced child frame.
    assert!(before.portal.frame_synced, "{case}");
    let temporary = format!(
        "/tmp/bbr-portal-{case}-{appearance}-{}.png",
        std::process::id()
    );
    assert!(!Path::new(&temporary).exists());
    client
        .send_ok(&Request::CapturePortal {
            path: temporary.clone(),
        })
        .expect("portal child readback");
    let image = image::open(&temporary).expect("read portal PNG");
    assert_eq!((image.width(), image.height()), (2160, 2814));
    assert_eq!(image.color(), image::ColorType::Rgba8);
    let rgba = image.as_rgba8().unwrap();
    assert_eq!(
        rgba.get_pixel(0, 0).0[3],
        0,
        "transparent outside the surface"
    );
    assert!(
        rgba.pixels().any(|pixel| pixel.0[3] == 255),
        "{case}: drawn surface"
    );
    let after = client.snapshot().expect("post-readback state");
    assert_eq!(after.portal.presentation, before.portal.presentation);
    assert_eq!(after.portal.ids, before.portal.ids);
    assert_eq!(after.tabs, before.tabs);
    assert_eq!(after.engine_sessions, before.engine_sessions);
    assert_eq!(after.active_profile, before.active_profile);
    assert_eq!(after.active_space, before.active_space);
    assert_eq!(after.shell_modes.theme, appearance);
    let destination = output.join(format!("{case}--{appearance}.png"));
    assert!(!destination.exists(), "never overwrite surface capture");
    fs::rename(temporary, destination).expect("preserve original PNG bytes");
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join(format!("{case}--{appearance}.snapshot.json")))
        .expect("fresh metadata");
    serde_json::to_writer_pretty(file, &after).expect("write semantic evidence");
}
