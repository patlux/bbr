//! Real Find portal with a deterministic WKWebView document; no page compositing.
use super::{key, wait, Client};
use crate::e2e_common::{
    catch_unwind, connect, deadline, harness_root, harness_socket_path, AssertUnwindSafe, Harness,
};
use bbr_testkit::protocol::{dsl, Request, Snapshot, ThemeModeOverride};
use std::{
    fs,
    path::{Path, PathBuf},
};

const PAGE: &str =
    "data:text/html,<title>BBR%20Visual%20Find</title><p>alice%20bob%20alice%20alice</p>";

pub fn capture() {
    let _serial = crate::e2e_common::serial_gui();
    for (appearance, mode) in [
        ("light", ThemeModeOverride::Light),
        ("dark", ThemeModeOverride::Dark),
    ] {
        let tag = format!("visual-find-{appearance}");
        let root = harness_root(&tag, std::process::id());
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        let socket = harness_socket_path(&tag, std::process::id());
        let token = format!("visual-find-{}", std::process::id());
        let output = PathBuf::from(std::env::var_os("BBR_VISUAL_CAPTURE_DIR").unwrap());
        fs::create_dir_all(&output).unwrap();
        let bin = std::env::var("BBR_VISUAL_CAPTURE_BIN").unwrap();
        let mut harness = Harness::spawn(
            &bin,
            &[
                "--bbr-initial-url",
                PAGE,
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
            client.send_ok(&Request::FocusWeb).unwrap();
            let initial = wait(&mut client, |s| {
                !s.loading && s.title.as_deref() == Some("BBR Visual Find")
            });
            assert_eq!(initial.engine_kind, "native-wkwebview");
            assert!(initial.web_view_hosted);
            assert_eq!(initial.tabs.len(), 1);
            let absent = format!("/tmp/bbr-find-absent-{}.png", std::process::id());
            refused(&mut client, &absent, "find_closed");
            key(&mut client, "cmd-f");
            let opened = wait(&mut client, |s| {
                s.find.open && s.find.focused && s.portal.frame_synced
            });
            assert_find(&opened, appearance);
            assert_eq!(opened.find.query, "");
            assert_eq!(opened.find.match_count, None);
            assert_eq!(opened.find.current_match, None);
            assert_eq!(opened.tabs, initial.tabs);
            refused(
                &mut client,
                "/tmp/nested/forbidden.png",
                "invalid_capture_path",
            );
            save(&mut client, &output, "find-open", appearance, opened);
            for (case, query, count, ordinal) in [
                ("find-matches", "alice", 3, Some(1)),
                ("find-no-matches", "xyzzy", 0, None),
            ] {
                // The existing input seam enters find_set_query, shared with
                // production typing; the real native adapter supplies results.
                client.send_ok(&dsl::find_set_query(query)).unwrap();
                let reported = wait(&mut client, |s| {
                    s.find.query == query
                        && !s.find.pending
                        && !s.find.stale
                        && s.find.match_count == Some(count)
                        && s.find.current_match == ordinal
                });
                assert_find(&reported, appearance);
                assert_eq!(reported.tabs, initial.tabs);
                assert_eq!(reported.active_web_view, initial.active_web_view);
                assert_eq!(reported.engine_sessions, initial.engine_sessions);
                save(&mut client, &output, case, appearance, reported);
            }
            // Formerly withheld (2026-10-12): stepping to the next match.
            client.send_ok(&dsl::find_set_query("alice")).unwrap();
            wait(&mut client, |s| {
                s.find.query == "alice"
                    && !s.find.pending
                    && !s.find.stale
                    && s.find.current_match == Some(1)
            });
            client.send_ok(&dsl::find_next()).unwrap();
            let next = wait(&mut client, |s| {
                !s.find.pending
                    && !s.find.stale
                    && s.find.match_count == Some(3)
                    && s.find.current_match == Some(2)
            });
            assert_find(&next, appearance);
            save(&mut client, &output, "find-next", appearance, next);
            key(&mut client, "escape");
            let closed = wait(&mut client, |s| !s.find.open && !s.portal.visible);
            assert_eq!(closed.tabs, initial.tabs);
            assert_eq!(closed.engine_sessions, initial.engine_sessions);
            assert_eq!(closed.chrome_focused, initial.chrome_focused);
            assert_eq!(closed.active_web_view, initial.active_web_view);
            key(&mut client, "cmd-t");
            wait(&mut client, |s| s.portal.presentation == Some(0xC1));
            refused(&mut client, &absent, "find_closed");
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

fn refused(client: &mut Client, path: &str, code: &str) {
    assert!(!Path::new(path).exists());
    assert_eq!(
        client
            .send_ok(&Request::CaptureFindBar { path: path.into() })
            .unwrap_err()
            .code,
        code
    );
    assert!(!Path::new(path).exists());
}

fn assert_find(s: &Snapshot, appearance: &str) {
    assert!(s.find.open && s.find.focused);
    assert!(!s.find.pending && !s.find.stale);
    assert_eq!(s.portal.presentation, Some(0xC3));
    assert_eq!(s.portal.host.as_deref(), Some("native-above-web"));
    assert!(s.portal.visible && s.portal.frame_synced);
    assert!(
        !s.portal.parent_visible && !s.portal.native_visible,
        "hidden renderer evidence only"
    );
    assert_eq!(s.portal.portal_scale_factor, 2.0);
    assert!(s.portal.portal_window_number > 0);
    assert_ne!(s.portal.portal_window_number, s.portal.parent_window_number);
    assert_eq!(s.window_size, [1080.0, 1407.0]);
    assert_eq!(s.portal.rect, Some([712.0, 8.0, 360.0, 36.0]));
    assert_eq!(s.shell_modes.theme, appearance);
    assert!(s.shell_modes.reduced_motion && s.motion.moments.is_empty());
    for id in ["bar", "field", "text", "count", "previous", "next", "close"] {
        assert!(s.portal.ids.contains(&format!("bbr-find-{id}")));
    }
}

fn save(client: &mut Client, output: &Path, case: &str, appearance: &str, before: Snapshot) {
    let temporary = format!("/tmp/bbr-{case}-{appearance}-{}.png", std::process::id());
    assert!(!Path::new(&temporary).exists());
    client
        .send_ok(&Request::CaptureFindBar {
            path: temporary.clone(),
        })
        .unwrap();
    let image = image::open(&temporary).unwrap();
    assert_eq!((image.width(), image.height()), (2160, 2814));
    assert_eq!(image.color(), image::ColorType::Rgba8);
    let rgba = image.as_rgba8().unwrap();
    assert_eq!(rgba.get_pixel(0, 0).0[3], 0);
    assert!(rgba.pixels().any(|pixel| pixel.0[3] == 255));
    let after = client.snapshot().unwrap();
    assert_find(&after, appearance);
    assert_eq!(after.tabs, before.tabs);
    assert_eq!(after.engine_sessions, before.engine_sessions);
    assert_eq!(after.active_web_view, before.active_web_view);
    assert_eq!(after.active_profile, before.active_profile);
    assert_eq!(after.active_space, before.active_space);
    let before_json = serde_json::to_value(&before).unwrap();
    let after_json = serde_json::to_value(&after).unwrap();
    for field in [
        "find",
        "portal",
        "chrome_focused",
        "command_surface",
        "command_bar_text",
    ] {
        assert!(before_json.get(field).is_some());
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
