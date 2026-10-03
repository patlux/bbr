//! Additional Library stills exercise its existing production state authority.
use super::{save, wait};
use bbr_testkit::{protocol::SavedPagesLibraryCollectionOp, Client, Request};
use std::path::Path;

fn capture(client: &mut Client, output: &Path, appearance: &str, case: &str) {
    let snapshot = wait(client, |s| s.library.open);
    assert_eq!(snapshot.store.saved_pages_count, 3);
    assert_eq!(snapshot.library.total, 3);
    save(client, output, case, appearance, false, snapshot);
}

pub fn extended(client: &mut Client, output: &Path, appearance: &str) {
    client
        .send_ok(&Request::SavedPagesLibraryCollection {
            op: SavedPagesLibraryCollectionOp::Create {
                title: "Reading".into(),
            },
        })
        .expect("create through production collection authority");
    for id in [2, 3] {
        client
            .send_ok(&Request::ToggleSavedPagesLibrarySelect { id })
            .unwrap();
    }
    client
        .send_ok(&Request::MoveSavedPagesLibrarySelection {
            collection: Some(1),
        })
        .unwrap();
    let collections = wait(client, |s| {
        s.library.selected.is_empty()
            && s.library
                .collections
                .first()
                .is_some_and(|c| c.member_count == 2)
    });
    assert_eq!(collections.library.collections[0].title, "Reading");
    assert_eq!(
        collections
            .library
            .rows
            .iter()
            .filter(|r| r.collection.as_deref() == Some("Reading"))
            .count(),
        2
    );
    save(
        client,
        output,
        "library-collections",
        appearance,
        false,
        collections,
    );

    client
        .send_ok(&Request::ToggleSavedPagesLibrarySelect { id: 2 })
        .unwrap();
    let single = wait(client, |s| s.library.selected == [2]);
    assert_eq!(single.library.rows.iter().filter(|r| r.selected).count(), 1);
    save(
        client,
        output,
        "library-selection-single",
        appearance,
        false,
        single,
    );
    client
        .send_ok(&Request::ToggleSavedPagesLibrarySelect { id: 3 })
        .unwrap();
    let multi = wait(client, |s| s.library.selected == [2, 3]);
    assert_eq!(multi.library.rows.iter().filter(|r| r.selected).count(), 2);
    assert!(multi
        .visible_ids
        .iter()
        .any(|id| id == "bbr-saved-pages-selection"));
    save(
        client,
        output,
        "library-selection-multiple",
        appearance,
        false,
        multi,
    );

    client
        .send_ok(&Request::ClickSavedPagesLibrarySelectionMove)
        .unwrap();
    wait(client, |s| s.library.move_sheet_open);
    capture(client, output, appearance, "library-move-sheet");
    client
        .send_ok(&Request::ClickSavedPagesLibrarySelectionMove)
        .unwrap();
    wait(client, |s| !s.library.move_sheet_open);
    client
        .send_ok(&Request::ClickSavedPagesLibrarySelectionRemove)
        .unwrap();
    let remove = wait(client, |s| s.library.remove_confirm);
    assert_eq!(remove.library.remove_targets, [2, 3]);
    save(
        client,
        output,
        "library-remove-sheet",
        appearance,
        false,
        remove,
    );
    client
        .send_ok(&Request::CancelSavedPagesLibraryRemove)
        .unwrap();
    client
        .send_ok(&Request::ClickSavedPagesLibrarySelectionClear)
        .unwrap();
    wait(client, |s| {
        !s.library.remove_confirm && s.library.selected.is_empty()
    });

    client
        .send_ok(&Request::ClickSavedPagesLibraryCollectionCreate)
        .unwrap();
    client
        .send_ok(&Request::SetSavedPagesLibraryCollectionDraft {
            text: "Research".into(),
        })
        .unwrap();
    let create = wait(client, |s| {
        s.library.collection_create_staged && s.library.collection_draft == "Research"
    });
    save(
        client,
        output,
        "library-collection-create",
        appearance,
        false,
        create,
    );
    client
        .send_ok(&Request::CancelSavedPagesLibraryCollection)
        .unwrap();
    wait(client, |s| !s.library.collection_create_staged);
}
