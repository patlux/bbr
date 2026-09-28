//! Real-app folder menu E2E (blueprint D25, DESIGN §11.11/§11.13): a
//! secondary click on a folder row opens the folder menu bound to that
//! exact folder without toggling it; Rename… opens the D20 inline rename;
//! New Tab in Folder opens ⌘T and the committed tab joins the folder
//! (pinned); Delete Folder — keep its tabs unpacks the folder and closes no
//! tab. Every step synchronizes on typed snapshot state or semantic events
//! under a deadline.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{CommandBarMode, Snapshot};
use e2e_common::*;

fn flush(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush");
}

fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("keystroke");
}

fn open_menu(client: &mut Client, folder: u64) -> Snapshot {
    client
        .send_ok(&bbr_testkit::Request::RightClickFolderRow {
            folder,
            x: 120.0,
            y: 300.0,
        })
        .expect("right-click the folder");
    flush(client);
    let snapshot = client.snapshot().expect("menu snapshot");
    assert!(snapshot.tab_menu.open);
    assert_eq!(snapshot.tab_menu.source_key, "bbr-folder-menu");
    assert_eq!(snapshot.tab_menu.bound_folder, Some(folder));
    snapshot
}

/// The folder facts a menu open/dismiss must leave unchanged.
fn folder_state(snapshot: &Snapshot) -> Vec<(u64, String, bool, Vec<u64>)> {
    snapshot
        .folders
        .iter()
        .map(|f| (f.id, f.title.clone(), f.expanded, f.members.clone()))
        .collect()
}

fn activate(client: &mut Client, index: usize) {
    client
        .send_ok(&bbr_testkit::Request::TabMenuActivate { index })
        .expect("activate row");
    flush(client);
}

#[test]
#[ignore]
fn folder_menu_renames_adds_a_tab_and_unpacks_without_closing_tabs() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("folder-menu", &page_one);

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

        // A folder holding the first tab.
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("New Folder");
        key(&mut client, "enter");
        flush(&mut client);
        let folder = client.snapshot().expect("folder").folders[0].id;
        client
            .send_ok(&dsl::move_tab_into_folder(0, folder))
            .expect("group the first tab");
        flush(&mut client);

        // Opening never toggles; Escape dismisses as a no-op.
        let before = client.snapshot().expect("before the menu");
        let opened = open_menu(&mut client, folder);
        assert_eq!(
            folder_state(&opened),
            folder_state(&before),
            "opening never toggles"
        );
        key(&mut client, "escape");
        flush(&mut client);
        let dismissed = client.snapshot().expect("dismissed");
        assert!(!dismissed.tab_menu.open);
        assert_eq!(folder_state(&dismissed), folder_state(&before));
        assert_eq!(dismissed.active_tab_index, before.active_tab_index);
        assert_eq!(dismissed.tabs.len(), before.tabs.len());

        // The rows follow the board.
        let opened = open_menu(&mut client, folder);
        let ids: Vec<&str> = opened.tab_menu.rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "bbr-folder-menu-rename",
                "bbr-folder-menu-new-tab",
                "bbr-folder-menu-unpack"
            ]
        );

        // Rename… opens the inline rename with the title selected.
        activate(&mut client, 0);
        let renaming = client.snapshot().expect("rename snapshot");
        assert!(!renaming.tab_menu.open);
        assert_eq!(
            renaming.folders[0].rename_draft.as_deref(),
            Some("New Folder")
        );
        assert!(renaming.folders[0].rename_selected);
        key(&mut client, "r->r");
        key(&mut client, "enter");
        flush(&mut client);
        assert_eq!(client.snapshot().expect("renamed").folders[0].title, "r");

        // New Tab in Folder: ⌘T opens, the committed tab joins the folder.
        open_menu(&mut client, folder);
        activate(&mut client, 1);
        let staged = client.snapshot().expect("staged");
        assert_eq!(staged.command_bar_mode, CommandBarMode::NewTab);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_two.clone(),
            })
            .expect("type the destination");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("rank");
        let before = client.snapshot().expect("before commit").tabs.len();
        key(&mut client, "enter");
        flush(&mut client);
        let joined = client.snapshot().expect("joined snapshot");
        assert_eq!(joined.tabs.len(), before + 1, "one new tab");
        assert_eq!(joined.folders[0].members.len(), 2, "the new tab joined");
        assert_eq!(joined.pinned_count, 2, "folder members are pinned");

        // Delete Folder — keep its tabs: the folder goes, no tab closes.
        let sessions = joined.engine_sessions;
        open_menu(&mut client, folder);
        activate(&mut client, 2);
        let unpacked = client.snapshot().expect("unpacked snapshot");
        assert!(unpacked.folders.is_empty(), "the folder is gone");
        assert_eq!(unpacked.tabs.len(), joined.tabs.len(), "no tab closed");
        assert_eq!(unpacked.pinned_count, 2, "members stay loose pinned tabs");
        assert_eq!(unpacked.engine_sessions, sessions);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
