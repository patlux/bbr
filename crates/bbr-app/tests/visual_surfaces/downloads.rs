//! Actual sidebar Downloads panel; native transfer states, never Gallery DTOs.
use super::{key, wait, Client};
use crate::e2e_common::{
    catch_unwind, connect, deadline, harness_root, harness_socket_path, AssertUnwindSafe, Harness,
};
use bbr_testkit::protocol::{dsl, Request, Snapshot, ThemeModeOverride};
use std::{fs, path::PathBuf};

pub fn capture() {
    let _serial = crate::e2e_common::serial_gui();
    for (appearance, mode) in [
        ("light", ThemeModeOverride::Light),
        ("dark", ThemeModeOverride::Dark),
    ] {
        let tag = format!("visual-downloads-{appearance}");
        let root = harness_root(&tag, std::process::id());
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        let socket = harness_socket_path(&tag, std::process::id());
        let token = format!("visual-downloads-{}", std::process::id());
        let output = PathBuf::from(std::env::var_os("BBR_VISUAL_CAPTURE_DIR").unwrap());
        fs::create_dir_all(&output).unwrap();
        let bin = std::env::var("BBR_VISUAL_CAPTURE_BIN").unwrap();
        let mut harness = Harness::spawn(
            &bin,
            &[
                "--bbr-initial-url",
                "data:text/html,<title>BBR%20Visual%20Downloads</title><p>Download%20fixture</p>",
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
            client
                .send_ok(&Request::SetWindowSize {
                    width: 1080.0,
                    height: 1407.0,
                })
                .unwrap();
            crate::e2e_common::wait_for_window_settled(&mut client, deadline());
            client.send_ok(&Request::FocusWeb).unwrap();
            let initial = wait(&mut client, |s| {
                !s.loading
                    && s.title.as_deref() == Some("BBR Visual Downloads")
                    && s.tabs.iter().all(|tab| !tab.unresponsive && !tab.crashed)
            });
            assert!(!initial.downloads.open && initial.download_rows.is_empty());
            client.send_ok(&Request::ClickFooterDownloads).unwrap();
            let empty = wait(&mut client, |s| s.downloads.open);
            assert_panel(&empty, &initial, appearance);
            assert!(empty.download_rows.is_empty());
            assert_eq!(empty.downloads.total_rows, 0);
            assert_eq!(empty.downloads.footer_count, 0);
            assert!(empty.visible_ids.contains(&"bbr-downloads-empty".into()));
            for id in ["active", "history", "clear"] {
                assert!(!empty.visible_ids.contains(&format!("bbr-downloads-{id}")));
            }
            let empty_state = empty.clone();
            let empty_frame = save(&mut client, &output, "downloads-empty", appearance, empty);
            key(&mut client, "escape");
            let closed = wait(&mut client, |s| {
                !s.downloads.open && !s.visible_ids.contains(&"bbr-downloads-panel".into())
            });
            assert_eq!(closed.tabs, initial.tabs);
            assert_eq!(closed.active_web_view, initial.active_web_view);
            assert_eq!(closed.engine_sessions, initial.engine_sessions);
            assert_eq!(closed.active_profile, initial.active_profile);
            assert_eq!(closed.active_space, initial.active_space);
            assert!(closed.chrome_focused, "Escape returns focus to the footer");
            // IDs alone disappear at logical close. The guarded production
            // readback additionally proves the fade reached alpha=0/goal=0.
            settled_frame(&mut client, appearance);
            client.send_ok(&Request::ClickFooterDownloads).unwrap();
            let reopened = wait(&mut client, |s| s.downloads.open);
            assert_panel(&reopened, &initial, appearance);
            assert_eq!(reopened.downloads, empty_state.downloads);
            assert_eq!(reopened.download_rows, empty_state.download_rows);
            // The still holds only the panel, so the whole settled window of
            // the first opening is the reference: byte-identical as before.
            let reopened_png = settled_frame(&mut client, appearance);
            assert_eq!(
                reopened_png, empty_frame,
                "reopening reaches the identical actual panel, not the closed cache"
            );
            capture_history(&mut client, &output, &profile, appearance, &initial);
            key(&mut client, "escape");
            wait(&mut client, |s| !s.downloads.open);
            settled_frame(&mut client, appearance);
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

fn capture_history(
    client: &mut Client,
    output: &std::path::Path,
    profile: &std::path::Path,
    appearance: &str,
    initial: &Snapshot,
) {
    use bbr_testkit::protocol::{
        DownloadGroup, DownloadMachineState as State, DownloadRowAction as Action,
    };
    let (fixtures, _, _) = crate::e2e_common::fixtures();
    let filename = format!("bbr-visual-{appearance}.bin");
    let expected = profile.join("downloads").join(&filename);
    assert!(
        !expected.exists(),
        "never overwrite a pre-existing download"
    );
    client
        .send_ok(&Request::DownloadStart {
            url: fixtures.url("/__fx/dl/known.bin"),
            destination: filename.clone(),
        })
        .unwrap();
    let succeeded = wait(client, |s| {
        s.download_rows
            .iter()
            .any(|r| r.filename == filename && r.machine_state == State::Succeeded)
    });
    let row = succeeded
        .download_rows
        .iter()
        .find(|r| r.filename == filename)
        .unwrap();
    let success_id = row.id;
    assert_eq!(row.resolved_path.as_deref(), expected.to_str());
    let expected_bytes: Vec<u8> = (0u8..255).cycle().take(8192).collect::<Vec<_>>().repeat(8);
    assert_eq!(fs::read(&expected).unwrap(), expected_bytes);
    client
        .send_ok(&Request::DownloadStart {
            url: fixtures.url("/__fx/dl/fail.bin"),
            destination: format!("bbr-visual-{appearance}-failed.bin"),
        })
        .unwrap();
    let history = wait(client, |s| {
        s.download_rows.len() == 2
            && s.download_rows
                .iter()
                .any(|r| r.machine_state == State::Failed)
            && s.motion.moments.is_empty()
    });
    assert_panel(&history, initial, appearance);
    assert!(history
        .download_rows
        .iter()
        .all(|r| r.group == DownloadGroup::History));
    assert_eq!(history.download_rows[0].machine_state, State::Failed);
    assert_eq!(history.download_rows[0].action, Action::Retry);
    assert!(history.download_rows[0].action_enabled);
    assert_eq!(history.download_rows[1].id, success_id);
    assert_eq!(history.download_rows[1].action, Action::Reveal);
    assert!(history.download_rows[1].action_enabled);
    save(client, output, "downloads-history", appearance, history);
    // Only this fresh, byte-verified fixture result is removed, inside the guest.
    assert_eq!(fs::read(&expected).unwrap(), expected_bytes);
    fs::remove_file(&expected).unwrap();
    key(client, "escape");
    wait(client, |s| !s.downloads.open);
    settled_frame(client, appearance);
    client.send_ok(&Request::ClickFooterDownloads).unwrap();
    let missing = wait(client, |s| {
        s.downloads.open
            && s.download_rows.iter().any(|r| {
                r.id == success_id
                    && !r.action_enabled
                    && r.action_reason.as_deref() == Some("File no longer exists")
            })
    });
    assert_eq!(missing.download_rows.len(), 2);
    assert_eq!(missing.download_rows[1].machine_state, State::Succeeded);
    save(
        client,
        output,
        "downloads-missing-file",
        appearance,
        missing,
    );
}

fn assert_panel(s: &Snapshot, initial: &Snapshot, appearance: &str) {
    assert!(s.downloads.open && s.chrome_focused);
    assert!(s.tabs.iter().all(|tab| !tab.unresponsive && !tab.crashed));
    assert!(!s.portal.visible, "Downloads is ordinary sidebar chrome");
    assert_eq!(s.engine_kind, "native-wkwebview");
    assert!(s.web_view_hosted && s.sidebar_visible);
    assert_eq!(s.window_size, [1080.0, 1407.0]);
    assert_eq!(s.shell_modes.theme, appearance);
    assert!(s.shell_modes.reduced_motion && s.motion.moments.is_empty());
    assert!(s.visible_ids.contains(&"bbr-downloads-panel".into()));
    assert_eq!(s.tabs, initial.tabs);
    assert_eq!(s.active_web_view, initial.active_web_view);
    assert_eq!(s.engine_sessions, initial.engine_sessions);
    assert_eq!(s.active_profile, initial.active_profile);
    assert_eq!(s.active_space, initial.active_space);
}

fn save(
    client: &mut Client,
    output: &std::path::Path,
    case: &str,
    appearance: &str,
    before: Snapshot,
) -> Vec<u8> {
    let frame = settled_frame(client, appearance);
    // Only the panel: the footer and the content card have their own stills.
    let panel = super::Still::Elements(&["bbr-downloads-panel"]);
    super::save(client, output, case, appearance, panel, before.clone());
    let after = client.snapshot().unwrap();
    assert_panel(&after, &before, appearance);
    assert_eq!(after.downloads, before.downloads);
    assert_eq!(after.download_rows, before.download_rows);
    assert_eq!(after.chrome_focused, before.chrome_focused);
    // The settled whole-window frame the still was cut from.
    frame
}

fn settled_frame(client: &mut Client, appearance: &str) -> Vec<u8> {
    let probe = format!(
        "/tmp/bbr-downloads-ready-{}-{appearance}.png",
        std::process::id()
    );
    assert!(!std::path::Path::new(&probe).exists());
    let until = deadline();
    let started = std::time::Instant::now();
    let mut attempts = 0;
    loop {
        attempts += 1;
        match client.send_ok(&Request::CaptureChrome {
            path: probe.clone(),
        }) {
            Ok(()) => {
                let png = fs::read(&probe).expect("read the actual settled production frame");
                fs::remove_file(&probe).expect("remove only this successful probe readback");
                return png;
            }
            Err(error) => {
                assert!(
                    !std::path::Path::new(&probe).exists(),
                    "refused readback writes nothing"
                );
                assert_eq!(error.code, "capture_failed");
                assert!(
                    error.message.starts_with("downloads opacity not ready:"),
                    "{error:?}"
                );
                if attempts == 1 {
                    eprintln!("Downloads readback first observation: {error:?}");
                }
                assert!(
                    std::time::Instant::now() < until,
                    "Downloads readback never reached its rendered endpoint after {attempts} draws / {:?}: {error:?}",
                    started.elapsed()
                );
                std::thread::yield_now();
            }
        }
    }
}
