//! Blueprint D06 real-app E2E (boards P-6, L-4, PR-08): create, edit and
//! delete profiles.
//!
//! The command bar's New Profile… opens Settings › Profiles with the New
//! Profile sheet. Typing names the profile and (until edited) its first
//! space; the colour swatch picks the hue; Return creates the profile and
//! activates its empty first space. Edit renames and recolours; Delete names
//! what goes, removes the profile with its spaces and tabs, and returns to
//! a remaining profile; the last profile offers no Delete. Every step goes
//! through the method the Settings controls call; typed snapshots under
//! deadlines.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{SettingsInput, Snapshot};
use bbr_testkit::Request;
use e2e_common::*;

fn send(client: &mut Client, input: SettingsInput) -> Snapshot {
    client
        .send_ok(&Request::SettingsInput { input })
        .expect("settings input");
    client.snapshot().expect("snapshot")
}

fn typed(client: &mut Client, text: &str) {
    for ch in text.chars() {
        send(
            client,
            SettingsInput::Key {
                key: ch.to_string(),
                text: Some(ch.to_string()),
            },
        );
    }
}

fn key(client: &mut Client, key: &str, times: usize) {
    for _ in 0..times {
        send(
            client,
            SettingsInput::Key {
                key: key.into(),
                text: None,
            },
        );
    }
}

fn wait(client: &mut Client, what: &str, done: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("snapshot");
        if done(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "{what} never happened");
    }
}

fn has_row(snapshot: &Snapshot, id: &str) -> bool {
    snapshot
        .settings
        .as_ref()
        .is_some_and(|settings| settings.rows.iter().any(|row| row.id == id))
}

#[test]
#[ignore]
fn profiles_are_created_edited_and_deleted_from_settings() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("profiles-manage", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        let settled = wait_for_window_settled(&mut client, deadline());
        let default_space = settled.active_space;
        assert_eq!(settled.profiles.len(), 1);

        // ⌘T › New Profile… opens Settings › Profiles with the sheet.
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "cmd-t".into(),
            })
            .expect("cmd-t");
        client
            .send_ok(&Request::SetCommandBarText {
                text: "> new profile".into(),
            })
            .expect("query");
        client
            .send_ok(&Request::FlushCommandRanking)
            .expect("flush");
        let rows = client.snapshot().expect("rows");
        let first = rows.command_results.first().expect("an action row");
        assert_eq!(first.key, "action:15", "{:?}", rows.command_results);
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "enter".into(),
            })
            .expect("enter");
        let sheet = wait(&mut client, "the New Profile sheet", |s| {
            s.settings
                .as_ref()
                .is_some_and(|settings| settings.sheet.is_some())
        });
        let settings = sheet.settings.as_ref().unwrap();
        assert_eq!(settings.pane, "profiles");
        let draft = settings.sheet.as_ref().unwrap();
        assert_eq!(draft.title, "New Profile");
        assert_eq!(draft.swatch, Some(1), "the next hue round-robin");
        assert!(!draft.confirm_enabled, "a profile needs a name");
        assert_sheet_centered(
            &mut client,
            "bbr-settings-sheet",
            "bbr-settings-sheet-veil",
            480.0,
        );

        // The first space follows the name until it is typed.
        typed(&mut client, "Client work");
        let named = client.snapshot().expect("named");
        let draft = named.settings.as_ref().unwrap().sheet.as_ref().unwrap();
        assert_eq!(draft.fields, ["Client work", "Client work"]);
        send(&mut client, SettingsInput::SheetSwatch { index: 4 });
        key(&mut client, "tab", 1);
        key(&mut client, "backspace", "Client work".len());
        typed(&mut client, "Customers");
        key(&mut client, "enter", 1);

        // The profile exists with its hue; its empty first space is active.
        let created = wait(&mut client, "the new profile's space", |s| {
            s.profiles.len() == 2 && s.active_space != default_space
        });
        let profile = created
            .profiles
            .iter()
            .find(|p| p.title == "Client work")
            .expect("the new profile");
        assert_eq!(profile.hue, "terracotta");
        assert_eq!(created.active_profile, profile.id);
        let space = created
            .spaces
            .iter()
            .find(|space| space.space == created.active_space)
            .expect("active space");
        assert_eq!(space.title, "Customers");
        assert!(created.tabs.is_empty(), "the first space is empty");
        assert!(created.settings.as_ref().unwrap().sheet.is_none());
        assert!(has_row(&created, "profile-delete"));

        // Edit the Default profile: rename and recolour.
        send(
            &mut client,
            SettingsInput::Press {
                row: "profile-0".into(),
            },
        );
        key(&mut client, "backspace", "Default".len());
        typed(&mut client, "Personal");
        send(&mut client, SettingsInput::SheetSwatch { index: 2 });
        let edited = send(&mut client, SettingsInput::SheetConfirm);
        let personal = edited.profiles.iter().find(|p| p.index == 0).unwrap();
        assert_eq!(personal.title, "Personal");
        assert_eq!(personal.hue, "fuchsia");

        // Delete the new profile: the sheet names what goes.
        send(
            &mut client,
            SettingsInput::Select {
                row: "profile-1".into(),
            },
        );
        let asking = send(
            &mut client,
            SettingsInput::Press {
                row: "profile-delete".into(),
            },
        );
        let confirm = asking.settings.as_ref().unwrap().sheet.as_ref().unwrap();
        assert_eq!(confirm.title, "Delete “Client work”?");
        assert_sheet_centered(
            &mut client,
            "bbr-settings-sheet",
            "bbr-settings-sheet-veil",
            220.0,
        );
        assert!(
            confirm
                .message
                .as_deref()
                .is_some_and(|m| m.contains("1 space") && m.contains("0 tabs")),
            "{confirm:?}"
        );
        send(&mut client, SettingsInput::SheetConfirm);
        let deleted = wait(&mut client, "the fallback space", |s| {
            s.profiles.len() == 1 && s.active_space == default_space
        });
        assert!(deleted
            .spaces
            .iter()
            .all(|space| space.space != created.active_space));
        assert!(
            !has_row(&deleted, "profile-delete"),
            "the last profile stays"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
