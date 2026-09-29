//! Real-app menu bar and Dock menu E2E (blueprint D19, DESIGN §11.13):
//! BBR installs the menu bar (BBR, File, View, History, Spaces, Tab,
//! Window) and a Dock menu that list the spaces; every item dispatches the
//! same action as its shortcut. Items are invoked through the action
//! dispatch an `NSMenuItem` click takes; the menus follow a new space and
//! the active space; Quit BBR exits cleanly. Every step waits on typed
//! snapshot state or browser events under a deadline.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{BrowserChange, CommandBarMode, MenuItemSnapshot, Snapshot};
use e2e_common::*;

fn invoke(client: &mut Client, menu: &str, item: &str) {
    client
        .send_ok(&bbr_testkit::Request::InvokeMenuItem {
            menu: menu.into(),
            item: item.into(),
        })
        .unwrap_or_else(|error| panic!("{menu} › {item}: {error}"));
}

fn settle(client: &mut Client, what: &str, ready: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush");
        let snapshot = client.snapshot().expect("menu snapshot");
        if ready(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "{what} never settled");
    }
}

fn checked_spaces(snapshot: &Snapshot) -> Vec<(String, bool)> {
    let spaces = snapshot
        .menu_bar
        .iter()
        .find(|menu| menu.name == "Spaces")
        .expect("Spaces menu");
    spaces
        .items
        .iter()
        .filter(|item| item.action.as_deref() == Some("bbr::ActivateSpace"))
        .map(|item| (item.name.clone().unwrap_or_default(), item.checked))
        .collect()
}

#[test]
#[ignore]
fn menu_bar_and_dock_menu_dispatch_the_shortcut_actions_and_follow_the_spaces() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("menus", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let initial = settle(&mut client, "menus", |s| !s.menu_bar.is_empty());
        let names: Vec<&str> = initial.menu_bar.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(
            names,
            ["BBR", "File", "View", "History", "Spaces", "Tab", "Window"]
        );
        let default_space = initial.active_space;
        let default_title = checked_spaces(&initial)
            .into_iter()
            .find(|(_, checked)| *checked)
            .expect("the active space is checked")
            .0;
        // The Tab menu names match the ⌘T actions and their actions.
        let tab = &initial.menu_bar[5].items;
        assert!(tab.contains(&MenuItemSnapshot {
            name: Some("Move Tab to Space…".into()),
            action: Some("bbr::MoveTab".into()),
            checked: false,
        }));

        // A new space joins the Spaces and Dock menus.
        client
            .send_ok(&bbr_testkit::protocol::dsl::create_space(
                initial.active_profile,
                "Work",
            ))
            .expect("create Work");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let work = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        let listed = settle(&mut client, "Work listed", |s| {
            checked_spaces(s).iter().any(|(name, _)| name == "Work")
        });
        assert!(listed
            .dock_menu
            .iter()
            .any(|item| item.name.as_deref() == Some("Work")));

        // Spaces › Work activates it; the check follows.
        invoke(&mut client, "Spaces", "Work");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: work },
            since,
        );
        settle(&mut client, "Work checked", |s| {
            checked_spaces(s).contains(&("Work".to_string(), true))
        });
        // The Dock menu switches back.
        invoke(&mut client, "Dock", &default_title);
        wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );

        // View › Toggle Sidebar is ⌘S.
        invoke(&mut client, "View", "Toggle Sidebar");
        settle(&mut client, "sidebar hidden", |s| !s.sidebar_visible);
        invoke(&mut client, "View", "Toggle Sidebar");
        settle(&mut client, "sidebar back", |s| s.sidebar_visible);
        // Tab › New Tab is ⌘T.
        invoke(&mut client, "Tab", "New Tab");
        settle(&mut client, "palette", |s| {
            s.command_bar_mode == CommandBarMode::NewTab
        });

        // BBR › About BBR opens the About window with the version.
        invoke(&mut client, "BBR", "About BBR");
        let about = settle(&mut client, "About BBR", |s| s.about.is_some());
        assert_eq!(
            about.about.as_deref(),
            Some(concat!("Version ", env!("CARGO_PKG_VERSION")))
        );

        // BBR › Quit BBR flushes and exits cleanly.
        invoke(&mut client, "BBR", "Quit BBR");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
