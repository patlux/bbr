//! Menus/sheets stay in the actual Settings window; changes are cancelled.
use super::{save, wait};
use bbr_testkit::{protocol::SettingsInput, Client, Request};
use std::path::Path;

fn input(client: &mut Client, input: SettingsInput) {
    client.send_ok(&Request::SettingsInput { input }).unwrap();
}

pub fn extended(client: &mut Client, output: &Path, appearance: &str) {
    // Shortcuts was the final initial-pane capture. Overlapping real wheel
    // viewports cover the complete scroll extent without resizing the window.
    let observe = |client: &mut Client, delta_y| match client
        .request(&Request::ScrollSettings { delta_y })
        .unwrap()
    {
        bbr_testkit::protocol::Response::SettingsScroll {
            offset,
            max_offset,
            viewport,
        } => (offset, max_offset, viewport),
        other => panic!("Settings scroll measurement: {other:?}"),
    };
    let (mut offset, max, viewport) = observe(client, 0);
    assert_eq!(offset, [0.0, 0.0]);
    assert!(max[1] > 0.0 && viewport[3] > 400.0);
    let mut pages = 0;
    while -offset[1] < max[1] {
        assert!(pages < 3, "independently bounded Shortcuts page plan");
        let previous = offset[1];
        let (next, next_max, next_viewport) = observe(client, -400);
        assert_eq!(next_max, max);
        assert_eq!(next_viewport, viewport);
        assert!(next[1] < previous && previous - next[1] <= viewport[3]);
        offset = next;
        pages += 1;
        let snapshot = wait(client, |s| {
            s.settings.as_ref().is_some_and(|v| v.pane == "shortcuts")
        });
        save(
            client,
            output,
            &format!("settings-shortcuts-scroll-{pages}"),
            appearance,
            true,
            snapshot,
        );
    }
    assert_eq!(
        -offset[1], max[1],
        "last viewport reaches the actual content end"
    );
    assert_eq!(pages, 3, "exact independent page inventory");
    let (top, _, _) = observe(client, 2000);
    assert_eq!(top, [0.0, 0.0]);
    for (pane, row, case) in [
        ("general", "general-external-links", "settings-general-menu"),
        ("search", "search-engine", "settings-search-menu"),
        ("appearance", "appearance-motion", "settings-motion-menu"),
    ] {
        input(client, SettingsInput::Pane { pane: pane.into() });
        input(client, SettingsInput::OpenSelect { row: row.into() });
        let menu = wait(client, |s| {
            s.settings
                .as_ref()
                .is_some_and(|v| v.open_select.as_deref() == Some(row))
        });
        save(client, output, case, appearance, true, menu);
        input(client, SettingsInput::Dismiss);
        wait(client, |s| {
            s.settings.as_ref().is_some_and(|v| v.open_select.is_none())
        });
    }
    input(
        client,
        SettingsInput::Pane {
            pane: "profiles".into(),
        },
    );
    for (row, title, case) in [
        ("profile-new", "New Profile", "settings-profile-create"),
        ("profile-0", "Edit Profile", "settings-profile-edit"),
    ] {
        input(client, SettingsInput::Press { row: row.into() });
        let sheet = wait(client, |s| {
            s.settings.as_ref().is_some_and(|v| v.sheet.is_some())
        });
        assert_eq!(
            sheet
                .settings
                .as_ref()
                .unwrap()
                .sheet
                .as_ref()
                .unwrap()
                .title,
            title
        );
        save(client, output, case, appearance, true, sheet);
        input(client, SettingsInput::SheetCancel);
        wait(client, |s| {
            s.settings.as_ref().is_some_and(|v| v.sheet.is_none())
        });
        assert_eq!(client.snapshot().unwrap().profiles.len(), 1);
    }
}
