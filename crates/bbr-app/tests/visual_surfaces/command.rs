//! Actual portal-child renderer evidence, not a reconstructed/composited palette.
use super::{key, wait};
use crate::e2e_common::{
    catch_unwind, connect, deadline, harness_root, harness_socket_path, AssertUnwindSafe, Harness,
};
use bbr_testkit::protocol::{
    dsl, CommandBarMode, CommandResultState, CommandSurface, Request, Snapshot, ThemeModeOverride,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn capture() {
    let _serial = crate::e2e_common::serial_gui();
    for (appearance, mode) in [
        ("light", ThemeModeOverride::Light),
        ("dark", ThemeModeOverride::Dark),
    ] {
        let tag = format!("visual-command-{appearance}");
        let root = harness_root(&tag, std::process::id());
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        let socket = harness_socket_path(&tag, std::process::id());
        let token = format!("visual-command-{}", std::process::id());
        let output = PathBuf::from(std::env::var_os("BBR_VISUAL_CAPTURE_DIR").unwrap());
        fs::create_dir_all(&output).unwrap();
        let bin = std::env::var("BBR_VISUAL_CAPTURE_BIN").unwrap();
        // No random HTTP port or real-user corpus can enter palette text/recents.
        let mut harness = Harness::spawn(
            &bin,
            &[
                "--bbr-initial-url",
                "about:blank",
                "--bbr-profile-dir",
                profile.to_str().unwrap(),
                "--bbr-test-token",
                &token,
                "--bbr-ids=sequential",
                "--bbr-hidden",
                "--bbr-window-size=1080x1407",
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
                .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
                .unwrap();
            // AppKit rounds the startup frame; use the same production resize
            // and layout-settled boundary as the existing browser captures.
            client
                .send_ok(&Request::SetWindowSize {
                    width: 1080.0,
                    height: 1407.0,
                })
                .unwrap();
            crate::e2e_common::wait_for_window_settled(&mut client, deadline());
            client.send_ok(&Request::FocusWeb).unwrap();
            let initial = wait(&mut client, |s| !s.loading && s.tabs.len() == 1);
            let temporary = format!("/tmp/bbr-command-absent-{}.png", std::process::id());
            assert!(!Path::new(&temporary).exists());
            assert_eq!(
                client
                    .send_ok(&Request::CaptureCommandPalette {
                        path: temporary.clone()
                    })
                    .unwrap_err()
                    .code,
                "palette_closed"
            );
            assert!(!Path::new(&temporary).exists());
            key(&mut client, "cmd-t");
            let opened = wait(&mut client, |s| {
                s.command_surface == CommandSurface::Palette
                    && s.command_result_state == CommandResultState::Empty
                    && s.portal.frame_synced
            });
            assert_palette(&opened, appearance);
            assert_eq!(opened.tabs, initial.tabs, "Cmd-T only stages a destination");
            assert!(opened.command_results.is_empty());
            assert_eq!(opened.command_bar_text, "");
            assert_eq!(
                client
                    .send_ok(&Request::CaptureCommandPalette {
                        path: "/tmp/nested/forbidden.png".into()
                    })
                    .unwrap_err()
                    .code,
                "invalid_capture_path"
            );
            save(
                &mut client,
                &output,
                "command-palette-open",
                appearance,
                opened,
            );
            searches(&mut client, &output, appearance);
            key(&mut client, "escape");
            let closed = wait(&mut client, |s| {
                s.command_surface == CommandSurface::None && !s.portal.visible
            });
            assert_eq!(closed.tabs, initial.tabs);
            assert_eq!(closed.chrome_focused, initial.chrome_focused);
            assert_eq!(closed.engine_sessions, initial.engine_sessions);
            assert_eq!(closed.command_bar_mode, initial.command_bar_mode);
            assert_eq!(closed.command_bar_text, initial.command_bar_text);
            key(&mut client, "cmd-l");
            wait(&mut client, |s| {
                s.command_surface == CommandSurface::Overlay
            });
            assert_eq!(
                client
                    .send_ok(&Request::CaptureCommandPalette {
                        path: temporary.clone()
                    })
                    .unwrap_err()
                    .code,
                "palette_closed",
                "other portal presentations cannot masquerade as the palette"
            );
            assert!(!Path::new(&temporary).exists());
            key(&mut client, "escape");
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

fn searches(client: &mut super::Client, output: &Path, appearance: &str) {
    for (case, query, state) in [
        (
            "command-palette-actions-match",
            "> tab",
            CommandResultState::Results,
        ),
        (
            "command-palette-actions-empty",
            "> qqqzzz",
            CommandResultState::Empty,
        ),
    ] {
        client
            .send_ok(&Request::SetCommandBarText { text: query.into() })
            .unwrap();
        client.send_ok(&Request::FlushCommandRanking).unwrap();
        let snapshot = wait(client, |s| {
            s.command_bar_text == query && s.command_result_state == state
        });
        assert_palette(&snapshot, appearance);
        assert!(snapshot
            .command_results
            .iter()
            .all(|row| row.group == "action"));
        if state == CommandResultState::Results {
            assert!(snapshot.command_results.len() >= 2, "real filtered actions");
            assert!(snapshot
                .command_results
                .iter()
                .all(|row| row.title.to_lowercase().contains("tab")));
            assert!(snapshot
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-palette-result-")));
        } else {
            assert!(snapshot.command_results.is_empty());
            assert_eq!(snapshot.command_result_total, 0);
            assert!(!snapshot
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-palette-result-")));
        }
        save(client, output, case, appearance, snapshot);
    }
    // Formerly withheld (2026-10-12): a typed address and a plain search
    // query, each with its real result rows and the keyboard row.
    for (case, query) in [
        ("command-palette-url", "docs.example.test/guide"),
        ("command-palette-search", "gpui layout"),
    ] {
        client
            .send_ok(&Request::SetCommandBarText { text: query.into() })
            .unwrap();
        client.send_ok(&Request::FlushCommandRanking).unwrap();
        let snapshot = wait(client, |s| {
            s.command_bar_text == query
                && s.command_result_state == CommandResultState::Results
                && !s.command_results.is_empty()
        });
        assert_palette(&snapshot, appearance);
        assert!(
            snapshot.command_results.iter().any(|row| row.selected),
            "{case}: one keyboard row"
        );
        save(client, output, case, appearance, snapshot);
    }
}

fn assert_palette(snapshot: &Snapshot, appearance: &str) {
    assert_eq!(snapshot.command_surface, CommandSurface::Palette);
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::NewTab);
    assert!(snapshot.chrome_focused);
    assert_eq!(snapshot.portal.presentation, Some(0xC1));
    assert!(
        snapshot.portal.visible,
        "logical presentation, not native visibility"
    );
    assert_eq!(snapshot.portal.host.as_deref(), Some("native-above-web"));
    assert!(snapshot.portal.frame_synced);
    assert_eq!(snapshot.portal.portal_scale_factor, 2.0);
    assert!(!snapshot.portal.parent_visible, "hidden renderer fixture");
    assert!(
        !snapshot.portal.native_visible,
        "not on-screen native proof"
    );
    assert!(snapshot.portal.portal_window_number > 0);
    assert_ne!(
        snapshot.portal.portal_window_number,
        snapshot.portal.parent_window_number
    );
    assert_eq!(snapshot.window_size, [1080.0, 1407.0]);
    assert_eq!(snapshot.shell_modes.theme, appearance);
    assert!(snapshot.shell_modes.reduced_motion);
    assert!(
        snapshot.motion.moments.is_empty(),
        "palette capture must be settled; active moments {:?}, tab states {:?}, loading {}",
        snapshot.motion.moments,
        snapshot.tab_states,
        snapshot.loading
    );
    assert!(snapshot.visible_ids.iter().any(|id| id == "bbr-palette"));
}

fn save(client: &mut super::Client, output: &Path, case: &str, appearance: &str, before: Snapshot) {
    let temporary = format!(
        "/tmp/bbr-command-{case}-{appearance}-{}.png",
        std::process::id()
    );
    assert!(!Path::new(&temporary).exists());
    client
        .send_ok(&Request::CaptureCommandPalette {
            path: temporary.clone(),
        })
        .unwrap();
    let image = image::open(&temporary).unwrap();
    assert_eq!((image.width(), image.height()), (2160, 2814));
    assert_eq!(image.color(), image::ColorType::Rgba8);
    let rgba = image.as_rgba8().unwrap();
    assert_eq!(
        rgba.get_pixel(0, 0).0[3],
        0,
        "transparent child, not opaque browser frame"
    );
    assert!(
        rgba.pixels().any(|pixel| pixel.0[3] == 255),
        "actual card pixels, not an empty child frame"
    );
    let after = client.snapshot().unwrap();
    assert_palette(&after, appearance);
    assert_eq!(after.tabs, before.tabs);
    assert_eq!(after.engine_sessions, before.engine_sessions);
    assert_eq!(after.active_profile, before.active_profile);
    assert_eq!(after.active_space, before.active_space);
    let before_json = serde_json::to_value(&before).unwrap();
    let after_json = serde_json::to_value(&after).unwrap();
    for field in [
        "command_surface",
        "command_bar_mode",
        "command_bar_text",
        "chrome_focused",
        "command_result_state",
        "command_results",
        "command_result_total",
        "command_cursor",
        "command_selection",
        "portal",
    ] {
        assert!(
            before_json.get(field).is_some(),
            "typed field {field} must exist"
        );
        assert_eq!(
            after_json[field], before_json[field],
            "readback preserves {field}"
        );
    }
    let destination = output.join(format!("{case}--{appearance}.png"));
    assert!(!destination.exists());
    fs::rename(temporary, destination).unwrap();
    let metadata = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join(format!("{case}--{appearance}.snapshot.json")))
        .unwrap();
    serde_json::to_writer_pretty(metadata, &after).unwrap();
}
