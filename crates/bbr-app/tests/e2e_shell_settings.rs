//! Blueprint D16 real-app E2E (boards L-3…L-9): the Settings window.
//!
//! ⌘, opens it on General through the production action. Every row applies
//! at once through the method its control calls: the appearance, contrast
//! and density choices re-present the browser (typed shell modes), the
//! search engine changes the effective settings, the Peek toggle turns a
//! link that asks for a new window into an ordinary tab, and the shortcut
//! list shows the bound keys. The choices land in the preferences file and
//! a relaunch restores them. Typed snapshots under deadlines; no sleeps.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{SettingsInput, Snapshot};
use bbr_testkit::Request;
use e2e_common::*;

fn input(client: &mut Client, input: SettingsInput) -> Snapshot {
    client
        .send_ok(&Request::SettingsInput { input })
        .expect("settings input");
    client.snapshot().expect("settings snapshot")
}

fn pane(client: &mut Client, pane: &str) -> Snapshot {
    input(client, SettingsInput::Pane { pane: pane.into() })
}

fn choose(client: &mut Client, row: &str, index: usize) -> Snapshot {
    input(
        client,
        SettingsInput::Choose {
            row: row.into(),
            index,
        },
    )
}

fn row<'a>(snapshot: &'a Snapshot, id: &str) -> &'a bbr_testkit::protocol::SettingsRowSnapshot {
    snapshot
        .settings
        .as_ref()
        .expect("settings open")
        .rows
        .iter()
        .find(|row| row.id == id)
        .unwrap_or_else(|| panic!("no settings row {id}"))
}

fn wait_settings(client: &mut Client) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("snapshot");
        if snapshot.settings.is_some() {
            return snapshot;
        }
        assert!(Instant::now() < until, "Settings never opened");
    }
}

#[test]
#[ignore]
fn settings_apply_at_once_and_survive_a_relaunch() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("settings", &page_one);
    let profile_dir = harness.root.join("profile");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first view");
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // ⌘, opens Settings on General.
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "cmd-,".into(),
            })
            .expect("cmd-,");
        let general = wait_settings(&mut client);
        assert_eq!(general.settings.as_ref().unwrap().pane, "general");
        assert_eq!(row(&general, "general-peek").value, "on");
        let default_row = row(&general, "general-default-browser");
        assert!(
            default_row.value == "Make Default" || default_row.value == "Default",
            "{default_row:?}"
        );

        // Appearance: Dark, increased contrast, a pinned density.
        pane(&mut client, "appearance");
        let dark = choose(&mut client, "appearance-theme", 2);
        assert_eq!(dark.shell_modes.theme, "dark");
        assert_eq!(row(&dark, "appearance-theme").value, "Dark");
        let contrast = choose(&mut client, "appearance-contrast", 1);
        assert_eq!(contrast.shell_modes.contrast, "increased");
        let small = choose(&mut client, "appearance-density", 0);
        assert_eq!(small.shell_modes.density, "small");
        assert_eq!(row(&small, "appearance-density").value, "small");
        // The same card again returns to the automatic size.
        let automatic = choose(&mut client, "appearance-density", 0);
        assert_eq!(automatic.shell_modes.density, "normal");
        assert_eq!(row(&automatic, "appearance-density").value, "automatic");
        choose(&mut client, "appearance-paper", 2);

        // The pop-up choice opens and a press outside closes it.
        let open = input(
            &mut client,
            SettingsInput::OpenSelect {
                row: "appearance-motion".into(),
            },
        );
        assert_eq!(
            open.settings.as_ref().unwrap().open_select.as_deref(),
            Some("appearance-motion")
        );
        let closed = input(&mut client, SettingsInput::Dismiss);
        assert_eq!(closed.settings.as_ref().unwrap().open_select, None);

        // Search: the engine is the effective search setting.
        pane(&mut client, "search");
        let google = choose(&mut client, "search-engine", 1);
        assert_eq!(google.store.search_engine, "google");
        assert_eq!(row(&google, "search-engine").value, "Google");

        // Shortcuts list the bound keys.
        let shortcuts = pane(&mut client, "shortcuts");
        assert_eq!(row(&shortcuts, "shortcut-cmd-t").value, "⌘ T");
        assert_eq!(row(&shortcuts, "shortcut-alt-cmd-s").value, "⌥ ⌘ S");

        // General: Peek off — a link asking for a new window opens a tab.
        pane(&mut client, "general");
        let off = input(
            &mut client,
            SettingsInput::Toggle {
                row: "general-peek".into(),
            },
        );
        assert_eq!(row(&off, "general-peek").value, "off");
        let before = off.tabs.len();
        client
            .send_ok(&Request::NewWindow {
                source: first_view,
                source_profile: off.active_profile,
                url: page_two.clone(),
                user_initiated: true,
                intent: bbr_web::NewWindowIntent::Default,
                input: bbr_web::new_window::NewWindowInput::Primary,
            })
            .expect("new-window request");
        let opened = client.snapshot().expect("opened");
        assert!(!opened.peek_presented, "no Peek with the toggle off");
        assert_eq!(opened.tabs.len(), before + 1, "one ordinary tab instead");

        // The choices are in the preferences file.
        let file = std::fs::read_to_string(profile_dir.join("preferences.txt"))
            .expect("preferences written");
        for line in [
            "appearance=dark",
            "increase_contrast=on",
            "density=automatic",
            "default_paper=rich",
            "peek_new_windows=false",
        ] {
            assert!(file.contains(line), "{line} in {file}");
        }

        client
            .send_ok(&Request::CloseSettings)
            .expect("close settings");
        let until = deadline();
        while client.snapshot().expect("closed").settings.is_some() {
            assert!(Instant::now() < until, "Settings never closed");
        }
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);

    // A relaunch on the same profile restores the choices.
    let (mut harness, socket, token) = spawn_harness("settings", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        let restored = wait_for_window_settled(&mut client, deadline());
        assert_eq!(restored.shell_modes.theme, "dark");
        assert_eq!(restored.shell_modes.contrast, "increased");
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
