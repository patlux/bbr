//! Pointer selection-toolbar E2E (B3 packet `saved-pages-library`): the
//! multi-select toolbar controls driven through their production pointer
//! paths — the Clear control (empties the whole set), the Remove control
//! (stages the destructive confirmation for exactly the selection), the
//! Move control (toggles the move-to sheet), and the sheet options (move
//! the selection into / out of a flat collection). Every step asserts the
//! stable §15 ids and the one AppStore authority's counts; no typed-direct
//! substitute replaces a toolbar control.

use crate::e2e_common::*;
use crate::support::{collection_by_id, row_by_bookmark, wait_library};
use crate::{open_library_typed, save, tab_evidence, wait_initial_title};

/// The multi-select toolbar through its production pointer paths: Clear
/// empties the set, Move opens the sheet whose options move the selection
/// into and out of a flat collection, and Remove stages the destructive
/// confirmation for exactly the selection — every control persists through
/// the one AppStore authority and open tabs stay untouched.
#[test]
#[ignore]
fn library_pointer_selection_toolbar_clear_remove_move() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-ptr-toolbar", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        wait_initial_title(&mut client, &mut since);
        save(&mut client, &page_one, PAGE_ONE_TITLE);
        save(&mut client, "https://ptr-a.test/", "Pointer A");
        save(&mut client, "https://ptr-b.test/", "Pointer B");
        open_library_typed(&mut client, &mut since);
        wait_library(&mut client, |l| l.total == 3);
        let before = tab_evidence(&client.snapshot().expect("before toolbar"));

        // The toolbar renders only while something is selected, with its
        // three stable §15 controls; the Clear control empties the whole
        // set through the pointer path (`library_clear_selection`).
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 2 })
            .expect("select 2");
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 3 })
            .expect("select 3");
        let selected = wait_library(&mut client, |l| l.selected == vec![2, 3]);
        assert!(!selected.move_sheet_open);
        let snapshot = client.snapshot().expect("toolbar snapshot");
        assert_ids(
            &snapshot,
            &[
                "bbr-saved-pages-selection",
                "bbr-saved-pages-selection-clear",
                "bbr-saved-pages-selection-remove",
                "bbr-saved-pages-selection-move",
            ],
        );
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibrarySelectionClear)
            .expect("clear selection");
        let cleared = wait_library(&mut client, |l| l.selected.is_empty());
        assert_eq!(cleared.total, 3, "clear never removes a record");
        let snapshot = client.snapshot().expect("cleared snapshot");
        assert!(
            !snapshot
                .visible_ids
                .iter()
                .any(|id| id == "bbr-saved-pages-selection"),
            "the toolbar hides when the selection empties"
        );

        // A flat collection exists for the move-in: create it through the
        // pointer create entry (the create surface's own production path).
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryCollectionCreate)
            .expect("create entry");
        wait_library(&mut client, |l| l.collection_create_staged);
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibraryCollectionDraft {
                text: "Work".into(),
            })
            .expect("create draft");
        client
            .send_ok(&bbr_testkit::Request::ConfirmSavedPagesLibraryCollection)
            .expect("confirm create");
        let created = wait_library(&mut client, |l| l.collections.len() == 1);
        assert_eq!(collection_by_id(&created, 1).title, "Work");

        // Move the multi-select into the collection through the toolbar
        // Move control + the sheet option; membership labels the rows and
        // the selection is consumed exactly once.
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 2 })
            .expect("select 2 again");
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 3 })
            .expect("select 3 again");
        wait_library(&mut client, |l| l.selected == vec![2, 3]);
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibrarySelectionMove)
            .expect("open move sheet");
        let sheet = wait_library(&mut client, |l| l.move_sheet_open);
        let snapshot = client.snapshot().expect("move sheet snapshot");
        assert_ids(
            &snapshot,
            &["bbr-saved-pages-move-1", "bbr-saved-pages-move-none"],
        );
        assert_eq!(sheet.selected, vec![2, 3]);
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryMoveOption {
                collection: Some(1),
            })
            .expect("move into collection");
        let moved = wait_library(&mut client, |l| l.selected.is_empty() && !l.move_sheet_open);
        assert_eq!(collection_by_id(&moved, 1).member_count, 2);
        assert_eq!(
            row_by_bookmark(&moved, 2).collection.as_deref(),
            Some("Work")
        );
        assert_eq!(
            row_by_bookmark(&moved, 3).collection.as_deref(),
            Some("Work")
        );
        assert_eq!(row_by_bookmark(&moved, 1).collection, None);

        // Move one member back out through the sheet's "No collection"
        // option: the membership clears and the member count updates.
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 2 })
            .expect("select 2 for move-out");
        wait_library(&mut client, |l| l.selected == vec![2]);
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibrarySelectionMove)
            .expect("open move sheet again");
        wait_library(&mut client, |l| l.move_sheet_open);
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibraryMoveOption { collection: None })
            .expect("move out of collection");
        let moved_out = wait_library(&mut client, |l| l.selected.is_empty() && !l.move_sheet_open);
        assert_eq!(collection_by_id(&moved_out, 1).member_count, 1);
        assert_eq!(row_by_bookmark(&moved_out, 2).collection, None);
        assert_eq!(
            row_by_bookmark(&moved_out, 3).collection.as_deref(),
            Some("Work")
        );

        // The toolbar Remove control stages the destructive confirmation
        // for exactly the current selection; cancel leaves every record.
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 2 })
            .expect("select 2 for remove");
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 3 })
            .expect("select 3 for remove");
        wait_library(&mut client, |l| l.selected == vec![2, 3]);
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibrarySelectionRemove)
            .expect("stage remove via toolbar");
        let staged = wait_library(&mut client, |l| l.remove_confirm);
        assert_eq!(staged.remove_targets, vec![2, 3], "exactly the selection");
        let snapshot = client.snapshot().expect("staged remove snapshot");
        assert_ids(
            &snapshot,
            &[
                "bbr-saved-pages-remove-confirm",
                "bbr-saved-pages-remove-confirm-ok",
                "bbr-saved-pages-remove-confirm-cancel",
            ],
        );
        client
            .send_ok(&bbr_testkit::Request::CancelSavedPagesLibraryRemove)
            .expect("cancel remove");
        let cancelled = wait_library(&mut client, |l| !l.remove_confirm);
        assert_eq!(cancelled.total, 3, "cancel never removes");

        // Confirm: exactly the two targets drop through the one AppStore
        // authority; tabs/sessions stay untouched and the collection
        // member (row 3) leaves the collection with the record.
        client
            .send_ok(&bbr_testkit::Request::ClickSavedPagesLibrarySelectionRemove)
            .expect("stage remove again");
        wait_library(&mut client, |l| l.remove_confirm);
        client
            .send_ok(&bbr_testkit::Request::ConfirmSavedPagesLibraryRemove)
            .expect("confirm remove");
        let removed = wait_library(&mut client, |l| l.total == 1);
        assert_eq!(removed.rows[0].bookmark, 1, "the source record survives");
        assert!(
            removed.selected.is_empty(),
            "removed ids leave the selection"
        );
        let snapshot = client.snapshot().expect("after toolbar remove");
        assert_eq!(
            snapshot.store.saved_pages_count, 1,
            "one provider authority"
        );
        assert_eq!(
            tab_evidence(&snapshot),
            before,
            "removing records never touches tabs"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}
