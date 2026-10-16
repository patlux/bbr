//! Additional Library stills exercise its existing production state authority.
use super::{save, wait, PAGE};
use bbr_testkit::{protocol::SavedPagesLibraryCollectionOp, Client, Request};
use std::path::Path;

fn capture(client: &mut Client, output: &Path, appearance: &str, case: &str) {
    let snapshot = wait(client, |s| s.library.open);
    assert_eq!(snapshot.store.saved_pages_count, 3);
    assert_eq!(snapshot.library.total, 3);
    save(client, output, case, appearance, PAGE, snapshot);
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
        PAGE,
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
        PAGE,
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
        PAGE,
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
        PAGE,
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
        PAGE,
        create,
    );
    client
        .send_ok(&Request::CancelSavedPagesLibraryCollection)
        .unwrap();
    wait(client, |s| !s.library.collection_create_staged);
    remaining(client, output, appearance);
}

/// The formerly withheld Saved Pages states (2026-10-12): search with and
/// without a match, a title sort, the inline row rename, the staged
/// collection rename and removal, and a long scrolled list. Every staged
/// change is cancelled; only the long list adds records, last.
fn remaining(client: &mut Client, output: &Path, appearance: &str) {
    let search = |client: &mut Client, text: &str| {
        client
            .send_ok(&Request::SetSavedPagesLibrarySearch { text: text.into() })
            .unwrap();
    };
    search(client, "architecture");
    let found = wait(client, |s| {
        s.library.search == "architecture" && s.library.total == 1
    });
    assert_eq!(found.library.rows[0].title, "Browser architecture notes");
    save(client, output, "library-search", appearance, PAGE, found);
    search(client, "no-match");
    let none = wait(client, |s| {
        s.library.search == "no-match" && s.library.total == 0
    });
    assert!(none.library.rows.is_empty());
    save(
        client,
        output,
        "library-search-empty",
        appearance,
        PAGE,
        none,
    );
    search(client, "");
    let rest = wait(client, |s| {
        s.library.search.is_empty() && s.library.total == 3
    });

    let initial_sort = rest.library.sort.clone();
    let mut cycles = 0;
    let sorted = loop {
        client
            .send_ok(&Request::CycleSavedPagesLibrarySort)
            .unwrap();
        cycles += 1;
        let snapshot = client.snapshot().unwrap();
        if snapshot.library.sort == "title_ascending" {
            break wait(client, |s| s.library.sort == "title_ascending");
        }
        assert!(cycles < 4, "four deterministic sort orders");
    };
    let titles: Vec<_> = sorted
        .library
        .rows
        .iter()
        .map(|r| r.title.as_str())
        .collect();
    assert_eq!(
        titles,
        [
            "Browser architecture notes",
            "GPUI documentation",
            "Weekend reading list"
        ]
    );
    save(
        client,
        output,
        "library-sort-title",
        appearance,
        PAGE,
        sorted,
    );
    while client.snapshot().unwrap().library.sort != initial_sort {
        client
            .send_ok(&Request::CycleSavedPagesLibrarySort)
            .unwrap();
        cycles += 1;
        assert!(cycles < 8, "the sort cycle returns to its start");
    }

    client
        .send_ok(&Request::BeginSavedPagesLibraryRowRename { id: 2 })
        .unwrap();
    client
        .send_ok(&Request::SetSavedPagesLibraryRenameDraft {
            text: "Architecture notes".into(),
        })
        .unwrap();
    let renaming = wait(client, |s| {
        s.library.editing_row == Some(2) && s.library.rename_draft == "Architecture notes"
    });
    save(
        client,
        output,
        "library-row-rename",
        appearance,
        PAGE,
        renaming,
    );
    client
        .send_ok(&Request::CancelSavedPagesLibraryRename)
        .unwrap();
    wait(client, |s| s.library.editing_row.is_none());

    let collection = rest.library.collections[0].id;
    client
        .send_ok(&Request::ClickSavedPagesLibraryCollectionRow { id: collection })
        .unwrap();
    client
        .send_ok(&Request::SetSavedPagesLibraryCollectionDraft {
            text: "Weekend reading".into(),
        })
        .unwrap();
    let rename = wait(client, |s| {
        s.library.collection_action == Some((collection, "rename".into()))
            && s.library.collection_draft == "Weekend reading"
    });
    save(
        client,
        output,
        "library-collection-rename",
        appearance,
        PAGE,
        rename,
    );
    client
        .send_ok(&Request::ClickSavedPagesLibraryCollectionRow { id: collection })
        .unwrap();
    let remove = wait(client, |s| {
        s.library.collection_action == Some((collection, "remove".into()))
    });
    save(
        client,
        output,
        "library-collection-remove",
        appearance,
        PAGE,
        remove,
    );
    client
        .send_ok(&Request::CancelSavedPagesLibraryCollection)
        .unwrap();
    let kept = wait(client, |s| s.library.collection_action.is_none());
    assert_eq!(
        kept.library.collections[0].title, "Reading",
        "nothing committed"
    );

    // 50 records behind the 24-row bounded window, scrolled 12 rows in.
    for index in 1..=47 {
        client
            .send_ok(&Request::SaveSavedPage {
                url: format!("https://list.example.test/page-{index:02}"),
                title: format!("Reading list item {index:02}"),
            })
            .expect("seed through production Saved Pages authority without navigation");
    }
    wait(client, |s| s.library.total == 50);
    client
        .send_ok(&Request::SetSavedPagesLibraryScroll { offset: 12 })
        .unwrap();
    let scrolled = wait(client, |s| {
        s.library.total == 50 && s.library.scroll_offset == 12 && s.library.window_first == 12
    });
    save(
        client,
        output,
        "library-long-list",
        appearance,
        PAGE,
        scrolled,
    );
    client
        .send_ok(&Request::SetSavedPagesLibraryScroll { offset: 0 })
        .unwrap();
    wait(client, |s| s.library.scroll_offset == 0);
}
