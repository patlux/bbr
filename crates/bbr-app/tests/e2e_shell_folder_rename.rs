//! Real-app inline folder rename E2E (blueprint D20, DESIGN.md §11.11).
//!
//! A folder created through the production plus-menu commit opens in
//! inline rename (`bbr-folder-rename`) with its default title selected and
//! the folders region owning chrome focus. Real keystrokes (`key->char`
//! syntax so GPUI's `dispatch_keystroke` carries a `key_char`, exactly like
//! a real keyboard) edit the draft: the first printable character replaces
//! the selection, Backspace deletes. Return commits through the
//! organization's rename command; Escape keeps the committed title; an
//! emptied draft keeps it too; a press on bare chrome and focus leaving the
//! region (Cmd-T) commit like Return. Every step synchronizes on typed
//! snapshot state — no sleeps.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{CommandBarMode, Snapshot};
use e2e_common::*;

const RENAME_ID: &str = "bbr-folder-rename";

fn flush_frame(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

/// Type lowercase ASCII through the production keyboard seam.
fn type_text(client: &mut Client, text: &str) {
    for ch in text.chars() {
        key(client, &format!("{ch}->{ch}"));
    }
}

fn renaming(snapshot: &Snapshot) -> bool {
    snapshot.visible_ids.iter().any(|id| id == RENAME_ID)
}

/// Create a folder through the production plus-menu commit and return the
/// snapshot with the new folder (the last row) in rename.
fn new_folder(client: &mut Client) -> Snapshot {
    client
        .send_ok(&dsl::click_footer_plus())
        .expect("open plus menu");
    client
        .send_ok(&dsl::click_plus_menu_item(1))
        .expect("commit New Folder");
    flush_frame(client);
    let snapshot = client.snapshot().expect("new folder snapshot");
    let folder = snapshot.folders.last().expect("folder created");
    assert_eq!(folder.title, "New Folder", "the committed default title");
    assert_eq!(
        folder.rename_draft.as_deref(),
        Some("New Folder"),
        "a new folder opens in rename with its default title"
    );
    assert!(folder.rename_selected, "the default title is selected");
    assert!(renaming(&snapshot), "the rename field is in the tree");
    assert!(snapshot.chrome_focused, "the rename owns chrome focus");
    assert!(
        snapshot
            .folders
            .iter()
            .filter(|f| f.rename_draft.is_some())
            .count()
            == 1,
        "exactly one folder is in rename"
    );
    snapshot
}

fn assert_committed(snapshot: &Snapshot, index: usize, title: &str) {
    let folder = &snapshot.folders[index];
    assert_eq!(folder.title, title, "committed title of folder {index}");
    assert_eq!(folder.rename_draft, None, "the rename closed");
    assert!(!folder.rename_selected);
    assert!(!renaming(snapshot), "the rename field left the tree");
}

#[test]
#[ignore]
fn new_folders_open_in_inline_rename_and_commit_or_keep_the_title() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("folder-rename", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        wait_for_window_settled(&mut client, deadline());

        // Return commits: the first character replaces the selected
        // default title, Backspace deletes, and the rename command lands
        // the trimmed draft on the exact folder.
        new_folder(&mut client);
        type_text(&mut client, "reviex");
        key(&mut client, "backspace");
        type_text(&mut client, "w");
        flush_frame(&mut client);
        let draft = client.snapshot().expect("draft snapshot");
        assert_eq!(draft.folders[0].rename_draft.as_deref(), Some("review"));
        assert!(
            !draft.folders[0].rename_selected,
            "typing collapses the selection"
        );
        assert_eq!(
            draft.folders[0].title, "New Folder",
            "the draft is not committed"
        );
        key(&mut client, "enter");
        flush_frame(&mut client);
        let committed = client.snapshot().expect("committed snapshot");
        assert_committed(&committed, 0, "review");
        assert!(
            !committed.folders[0].expanded,
            "Return never toggles the folder"
        );
        assert!(committed.chrome_focused, "focus stays on the folder row");

        // Escape keeps the committed title.
        new_folder(&mut client);
        type_text(&mut client, "gone");
        key(&mut client, "escape");
        flush_frame(&mut client);
        let cancelled = client.snapshot().expect("cancelled snapshot");
        assert_committed(&cancelled, 1, "New Folder");
        assert_eq!(
            cancelled.folders[0].title, "review",
            "other folders untouched"
        );

        // An emptied draft keeps the committed title.
        new_folder(&mut client);
        key(&mut client, "backspace");
        flush_frame(&mut client);
        let emptied = client.snapshot().expect("emptied snapshot");
        assert_eq!(emptied.folders[2].rename_draft.as_deref(), Some(""));
        key(&mut client, "enter");
        flush_frame(&mut client);
        assert_committed(&client.snapshot().expect("empty commit"), 2, "New Folder");

        // A press on bare chrome commits like Return.
        new_folder(&mut client);
        type_text(&mut client, "haus");
        client
            .send_ok(&dsl::chrome_background_press())
            .expect("press outside the field");
        flush_frame(&mut client);
        assert_committed(&client.snapshot().expect("outside press"), 3, "haus");

        // Focus leaving the folders region (Cmd-T opens the command bar)
        // commits too; the command bar then cancels normally.
        new_folder(&mut client);
        type_text(&mut client, "ai");
        key(&mut client, "cmd-t");
        flush_frame(&mut client);
        flush_frame(&mut client);
        let blurred = client.snapshot().expect("focus-out snapshot");
        assert_eq!(blurred.command_bar_mode, CommandBarMode::NewTab);
        assert_committed(&blurred, 4, "ai");
        key(&mut client, "escape");
        flush_frame(&mut client);
        let closed = client.snapshot().expect("command bar closed");
        assert_eq!(closed.command_bar_mode, CommandBarMode::Inactive);
        let titles: Vec<&str> = closed.folders.iter().map(|f| f.title.as_str()).collect();
        assert_eq!(titles, ["review", "New Folder", "New Folder", "haus", "ai"]);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
