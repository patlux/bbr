//! Inline pointer rename E2E (B3 packet `saved-pages-library`): staging
//! seeds the draft with the record's current label, typing edits the staged
//! draft, Cancel leaves the input without committing, and Confirm commits
//! exactly the non-empty draft through the one rename authority.

use crate::e2e_common::*;
use crate::support::{row_by_bookmark, wait_library};
use crate::{open_library_typed, save, wait_initial_title};

/// The inline pointer rename entry: staging seeds the draft with the
/// record's current label, typing edits the staged draft, Cancel leaves
/// the input without committing, and Confirm commits exactly the non-empty
/// draft through the one rename authority — the row displays the new label
/// and the provider count is untouched.
#[test]
#[ignore]
fn library_inline_rename_draft_confirm_and_cancel() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-rename", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        wait_initial_title(&mut client, &mut since);
        save(&mut client, &page_one, PAGE_ONE_TITLE);
        save(&mut client, "https://rename-b.test/", "Rename B");
        open_library_typed(&mut client, &mut since);
        wait_library(&mut client, |l| l.total == 2);

        // Stage the pointer rename for row 1: the draft seeds empty (no
        // label yet) and the editing row exposes the inline input ids.
        client
            .send_ok(&bbr_testkit::Request::BeginSavedPagesLibraryRowRename { id: 1 })
            .expect("begin rename");
        let staged = wait_library(&mut client, |l| l.editing_row == Some(1));
        assert_eq!(
            staged.rename_draft, "",
            "the draft seeds with the current label"
        );
        let snapshot = client.snapshot().expect("staged rename snapshot");
        assert_ids(
            &snapshot,
            &[
                "bbr-saved-pages-rename-input-1",
                "bbr-saved-pages-rename-confirm-1",
                "bbr-saved-pages-rename-cancel-1",
            ],
        );

        // Type into the staged draft, then cancel: the row and the record
        // are untouched.
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibraryRenameDraft {
                text: "Discarded".into(),
            })
            .expect("set draft");
        wait_library(&mut client, |l| l.rename_draft == "Discarded");
        client
            .send_ok(&bbr_testkit::Request::CancelSavedPagesLibraryRename)
            .expect("cancel rename");
        let cancelled = wait_library(&mut client, |l| l.editing_row.is_none());
        assert!(cancelled.rename_draft.is_empty());
        assert_eq!(
            row_by_bookmark(&cancelled, 1).title,
            PAGE_ONE_TITLE,
            "cancel never commits"
        );

        // Stage again, type a real label, confirm: the row displays the
        // new label through the one rename authority.
        client
            .send_ok(&bbr_testkit::Request::BeginSavedPagesLibraryRowRename { id: 1 })
            .expect("begin rename again");
        wait_library(&mut client, |l| l.editing_row == Some(1));
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibraryRenameDraft {
                text: "Labeled One".into(),
            })
            .expect("set draft again");
        wait_library(&mut client, |l| l.rename_draft == "Labeled One");
        client
            .send_ok(&bbr_testkit::Request::ConfirmSavedPagesLibraryRename)
            .expect("confirm rename");
        let confirmed = wait_library(&mut client, |l| l.editing_row.is_none());
        assert!(confirmed.rename_draft.is_empty());
        assert_eq!(
            row_by_bookmark(&confirmed, 1).title,
            "Labeled One",
            "the label is the display title"
        );
        assert_eq!(confirmed.total, 2, "rename never changes the count");

        // An empty draft confirm is a no-op: the label stays.
        client
            .send_ok(&bbr_testkit::Request::BeginSavedPagesLibraryRowRename { id: 1 })
            .expect("begin rename third");
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibraryRenameDraft {
                text: String::new(),
            })
            .expect("empty draft");
        client
            .send_ok(&bbr_testkit::Request::ConfirmSavedPagesLibraryRename)
            .expect("confirm empty draft");
        let empty = wait_library(&mut client, |l| l.editing_row.is_none());
        assert_eq!(
            row_by_bookmark(&empty, 1).title,
            "Labeled One",
            "an empty draft never commits"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}
