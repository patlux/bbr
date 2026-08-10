//! Multi-select and destructive remove E2E (B3 packet
//! `saved-pages-library`): toggles flip the typed selection, the remove
//! confirmation stages and cancels without touching a record, and the
//! confirmed removal drops exactly the targeted records — never an open tab
//! or session; closing tabs never resurrects or removes a record.

use crate::e2e_common::*;
use crate::support::{row_by_bookmark, wait_library};
use crate::{key, open_library_typed, save, tab_evidence, wait_initial_title};

/// Multi-select and the destructive remove contract: toggles flip the typed
/// selection, the remove confirmation stages and cancels without touching a
/// record, and the confirmed removal drops exactly the targeted records —
/// never an open tab or session; closing tabs never resurrects or removes a
/// record.
#[test]
#[ignore]
fn library_select_and_destructive_remove_leave_tabs_untouched() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-remove", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        wait_initial_title(&mut client, &mut since);
        save(&mut client, &page_one, PAGE_ONE_TITLE);
        save(&mut client, "https://synthetic-a.test/", "Synthetic A");
        save(&mut client, "https://synthetic-b.test/", "Synthetic B");
        open_library_typed(&mut client, &mut since);
        wait_library(&mut client, |l| l.total == 3);
        let before = tab_evidence(&client.snapshot().expect("before remove"));

        // Multi-select toggles in and out; row states mirror the set.
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 2 })
            .expect("select 2");
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 3 })
            .expect("select 3");
        let selected = wait_library(&mut client, |l| l.selected == vec![2, 3]);
        assert!(row_by_bookmark(&selected, 2).selected);
        assert!(row_by_bookmark(&selected, 3).selected);
        assert!(!row_by_bookmark(&selected, 1).selected);
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 2 })
            .expect("deselect 2");
        wait_library(&mut client, |l| l.selected == vec![3]);

        // Stage the destructive remove for the two synthetic records; the
        // confirmation cancels without touching any record.
        client
            .send_ok(&bbr_testkit::Request::BeginSavedPagesLibraryRemove { ids: vec![2, 3] })
            .expect("begin remove");
        let staged = wait_library(&mut client, |l| l.remove_confirm);
        assert_eq!(staged.remove_targets, vec![2, 3]);
        let snapshot = client.snapshot().expect("staged snapshot");
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
        assert!(cancelled.remove_targets.is_empty());
        assert_eq!(cancelled.total, 3, "cancel never removes a record");

        // Confirm the staged removal: exactly the two targets drop; the
        // selection is cleared of removed ids; tabs/sessions untouched.
        client
            .send_ok(&bbr_testkit::Request::BeginSavedPagesLibraryRemove { ids: vec![2, 3] })
            .expect("begin remove again");
        wait_library(&mut client, |l| l.remove_confirm);
        client
            .send_ok(&bbr_testkit::Request::ConfirmSavedPagesLibraryRemove)
            .expect("confirm remove");
        let removed = wait_library(&mut client, |l| l.total == 1);
        assert!(!removed.remove_confirm);
        assert!(
            removed.selected.is_empty(),
            "removed ids leave the selection"
        );
        assert_eq!(removed.rows[0].bookmark, 1, "the source record survives");
        let after = client.snapshot().expect("after remove snapshot");
        assert_eq!(after.store.saved_pages_count, 1, "one provider authority");
        assert_eq!(
            tab_evidence(&after),
            before,
            "removing records never touches tabs"
        );

        // Close the source tab: the Library record survives the close.
        client
            .send_ok(&bbr_testkit::Request::SelectTab { index: 0 })
            .expect("select source tab");
        key(&mut client, "cmd-w");
        let closed = client.snapshot().expect("closed tab snapshot");
        assert!(closed.tabs.is_empty(), "the source tab is closed");
        assert_eq!(
            closed.library.total, 1,
            "closing tabs never removes a record"
        );
        let store = client.snapshot().expect("store after close");
        assert_eq!(store.store.saved_pages_count, 1);
    }));
    finish(result, &mut harness, &socket, &token);
}
