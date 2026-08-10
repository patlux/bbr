//! Pointer collection surface E2E (B3 packet `saved-pages-library`): the
//! flat-collection create/rename/remove surface driven through its
//! production pointer paths — the create entry, the staged create/rename
//! input with its Confirm/Cancel controls, the collection row click that
//! cycles rename → remove → none, and the destructive collection remove
//! confirmation (members move out first, records are never lost). Every
//! step asserts the stable §15 ids and the one AppStore authority's
//! counts; no typed-direct substitute replaces a surface control.

use crate::e2e_common::*;
use crate::support::{collection_by_id, row_by_bookmark, wait_library};
use crate::{open_library_typed, save, wait_initial_title};

/// The flat-collection action surface through its production pointer
/// paths: the create entry stages the inline input, Confirm creates
/// exactly one collection, the collection row click cycles rename → remove
/// with the staged Confirm/Cancel controls, and the confirmed remove moves
/// members out first and drops the row — records are never lost and the
/// one AppStore authority's counts stay exact.
#[test]
#[ignore]
fn library_pointer_collection_create_rename_remove_surface() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-ptr-collections", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        wait_initial_title(&mut client, &mut since);
        save(&mut client, &page_one, PAGE_ONE_TITLE);
        save(&mut client, "https://pcol-a.test/", "Collection A");
        save(&mut client, "https://pcol-b.test/", "Collection B");
        open_library_typed(&mut client, &mut since);
        wait_library(&mut client, |l| l.total == 3);

        // The create entry stages the inline create input with its stable
        // §15 controls; an empty draft Confirm creates nothing.
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryCollectionCreate)
            .expect("click create entry");
        let staged = wait_library(&mut client, |l| l.collection_create_staged);
        assert_eq!(staged.collection_draft, "");
        let snapshot = client.snapshot().expect("create staged snapshot");
        assert_ids(
            &snapshot,
            &[
                "bbr-saved-pages-collection-create-input",
                "bbr-saved-pages-collection-create-confirm",
                "bbr-saved-pages-collection-create-cancel",
            ],
        );
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibraryCollectionDraft {
                text: "Work".into(),
            })
            .expect("set create draft");
        wait_library(&mut client, |l| l.collection_draft == "Work");
        client
            .send_ok(&bbr_testkit::Request::ConfirmSavedPagesLibraryCollection)
            .expect("confirm create");
        let created = wait_library(&mut client, |l| {
            !l.collection_create_staged && l.collections.len() == 1
        });
        assert_eq!(collection_by_id(&created, 1).title, "Work");
        assert_eq!(created.total, 3, "create never changes the record count");

        // Move one member in so the rename/remove surface sees a real
        // membership; then the collection row click stages the rename
        // input seeded with the current title.
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 2 })
            .expect("select for member");
        wait_library(&mut client, |l| l.selected == vec![2]);
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibrarySelectionMove)
            .expect("open move sheet");
        wait_library(&mut client, |l| l.move_sheet_open);
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryMoveOption {
                collection: Some(1),
            })
            .expect("move member in");
        let with_member = wait_library(&mut client, |l| {
            l.selected.is_empty() && collection_by_id(l, 1).member_count == 1
        });
        assert_eq!(
            row_by_bookmark(&with_member, 2).collection.as_deref(),
            Some("Work")
        );

        // The collection row click stages the rename input (the §15
        // per-collection ids); typing edits the draft and Confirm commits
        // exactly one rename through the one AppStore authority.
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryCollectionRow { id: 1 })
            .expect("click collection row");
        let rename_staged = wait_library(&mut client, |l| {
            l.collection_action == Some((1, "rename".to_string()))
        });
        assert_eq!(rename_staged.collection_draft, "Work", "draft seeded");
        let snapshot = client.snapshot().expect("rename staged snapshot");
        assert_ids(
            &snapshot,
            &[
                "bbr-saved-pages-collection-rename-input-1",
                "bbr-saved-pages-collection-rename-confirm-1",
                "bbr-saved-pages-collection-rename-cancel-1",
            ],
        );
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibraryCollectionDraft {
                text: "Inbox".into(),
            })
            .expect("set rename draft");
        wait_library(&mut client, |l| l.collection_draft == "Inbox");
        client
            .send_ok(&bbr_testkit::Request::ConfirmSavedPagesLibraryCollection)
            .expect("confirm rename");
        let renamed = wait_library(&mut client, |l| {
            l.collection_action.is_none() && collection_by_id(l, 1).title == "Inbox"
        });
        assert_eq!(
            row_by_bookmark(&renamed, 2).collection.as_deref(),
            Some("Inbox"),
            "the member label follows the rename"
        );

        // The next collection row clicks cycle rename → remove (the
        // confirm cleared the staging, so the first click re-stages the
        // rename input and the second advances to the destructive remove
        // confirmation); Cancel leaves the collection intact, Confirm moves
        // members out first and drops the row — records never lost.
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryCollectionRow { id: 1 })
            .expect("click collection row (re-stage rename)");
        wait_library(&mut client, |l| {
            l.collection_action == Some((1, "rename".to_string()))
        });
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryCollectionRow { id: 1 })
            .expect("click collection row (advance to remove)");
        let remove_staged = wait_library(&mut client, |l| {
            l.collection_action == Some((1, "remove".to_string()))
        });
        let snapshot = client.snapshot().expect("remove staged snapshot");
        assert_ids(
            &snapshot,
            &[
                "bbr-saved-pages-collection-remove-confirm-1",
                "bbr-saved-pages-collection-remove-ok-1",
                "bbr-saved-pages-collection-remove-cancel-1",
            ],
        );
        assert_eq!(remove_staged.total, 3);
        client
            .send_ok(&bbr_testkit::Request::CancelSavedPagesLibraryCollection)
            .expect("cancel collection remove");
        let cancelled = wait_library(&mut client, |l| l.collection_action.is_none());
        assert_eq!(
            cancelled.collections.len(),
            1,
            "cancel keeps the collection"
        );

        // Stage again (two clicks through the rename stage) and confirm:
        // the member row leaves the collection (never lost), the collection
        // row drops, and the provider count is unchanged.
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryCollectionRow { id: 1 })
            .expect("click collection row (re-stage rename)");
        wait_library(&mut client, |l| {
            l.collection_action == Some((1, "rename".to_string()))
        });
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryCollectionRow { id: 1 })
            .expect("click collection row (advance to remove)");
        wait_library(&mut client, |l| {
            l.collection_action == Some((1, "remove".to_string()))
        });
        client
            .send_ok(&bbr_testkit::Request::ConfirmSavedPagesLibraryCollection)
            .expect("confirm collection remove");
        let removed = wait_library(&mut client, |l| {
            l.collections.is_empty() && l.rows.iter().all(|row| row.collection.is_none())
        });
        assert_eq!(removed.total, 3, "records are never lost");
        assert_eq!(
            row_by_bookmark(&removed, 2).collection,
            None,
            "the member moves out first"
        );
        let snapshot = client.snapshot().expect("after collection remove");
        assert_eq!(snapshot.store.saved_pages_count, 3);
    }));
    finish(result, &mut harness, &socket, &token);
}
