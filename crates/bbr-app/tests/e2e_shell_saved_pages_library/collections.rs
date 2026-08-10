//! Flat collection lifecycle E2E (B3 packet `saved-pages-library`):
//! create (with invalid-title rejection), move the multi-select into and
//! out of the collection, rename, and remove — removal moves members out
//! first, then drops the row; records are never lost.

use crate::e2e_common::*;
use crate::support::{collection_by_id, row_by_bookmark, wait_library};
use crate::{open_library_typed, save, wait_initial_title};

/// The flat collection lifecycle through the one AppStore authority:
/// create (with invalid-title rejection), move the multi-select into and
/// out of the collection, rename, and remove — removal moves members out
/// first, then drops the row; records are never lost.
#[test]
#[ignore]
fn library_flat_collections_create_rename_move_remove() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-collections", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        wait_initial_title(&mut client, &mut since);
        save(&mut client, &page_one, PAGE_ONE_TITLE);
        save(&mut client, "https://collection-b.test/", "Collection B");
        save(&mut client, "https://collection-c.test/", "Collection C");
        open_library_typed(&mut client, &mut since);
        wait_library(&mut client, |l| l.total == 3);

        // Create the flat collection; an invalid title is rejected and
        // creates nothing.
        client
            .send_ok(&bbr_testkit::Request::SavedPagesLibraryCollection {
                op: bbr_testkit::protocol::SavedPagesLibraryCollectionOp::Create {
                    title: "Work".into(),
                },
            })
            .expect("create collection");
        let created = wait_library(&mut client, |l| l.collections.len() == 1);
        let collection = collection_by_id(&created, 1);
        assert_eq!(collection.title, "Work");
        assert_eq!(collection.member_count, 0);
        let rejected = client
            .request(&bbr_testkit::Request::SavedPagesLibraryCollection {
                op: bbr_testkit::protocol::SavedPagesLibraryCollectionOp::Create {
                    title: String::new(),
                },
            })
            .expect("invalid create");
        match rejected {
            bbr_testkit::protocol::Response::Error(error) => {
                assert_eq!(error.code, "library_collection_rejected")
            }
            other => panic!("expected rejection, got {other:?}"),
        }
        let still = client.snapshot().expect("after rejection snapshot");
        assert_eq!(still.library.collections.len(), 1, "nothing was created");

        // Move the multi-select into the collection: membership labels the
        // rows and the member count updates; the selection is consumed.
        for id in [2, 3] {
            client
                .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id })
                .expect("select for move");
        }
        wait_library(&mut client, |l| l.selected == vec![2, 3]);
        client
            .send_ok(&bbr_testkit::Request::MoveSavedPagesLibrarySelection {
                collection: Some(1),
            })
            .expect("move into collection");
        let moved = wait_library(&mut client, |l| l.selected.is_empty());
        assert_eq!(collection_by_id(&moved, 1).member_count, 2);
        assert_eq!(
            row_by_bookmark(&moved, 2).collection.as_deref(),
            Some("Work"),
            "the row carries the collection label"
        );
        assert_eq!(row_by_bookmark(&moved, 1).collection, None);

        // Rename the collection; an invalid title is a no-op.
        client
            .send_ok(&bbr_testkit::Request::SavedPagesLibraryCollection {
                op: bbr_testkit::protocol::SavedPagesLibraryCollectionOp::Rename {
                    id: 1,
                    title: "Inbox".into(),
                },
            })
            .expect("rename collection");
        let renamed = wait_library(&mut client, |l| {
            l.collections.first().is_some_and(|c| c.title == "Inbox")
        });
        assert_eq!(
            row_by_bookmark(&renamed, 2).collection.as_deref(),
            Some("Inbox")
        );
        client
            .send_ok(&bbr_testkit::Request::SavedPagesLibraryCollection {
                op: bbr_testkit::protocol::SavedPagesLibraryCollectionOp::Rename {
                    id: 1,
                    title: String::new(),
                },
            })
            .expect("invalid rename");
        wait_library(&mut client, |l| l.collections[0].title == "Inbox");

        // Move one member out (None clears membership).
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 2 })
            .expect("select for move out");
        wait_library(&mut client, |l| l.selected == vec![2]);
        client
            .send_ok(&bbr_testkit::Request::MoveSavedPagesLibrarySelection { collection: None })
            .expect("move out of collection");
        let moved_out = wait_library(&mut client, |l| {
            l.collections[0].member_count == 1 && row_by_bookmark(l, 2).collection.is_none()
        });
        assert_eq!(collection_by_id(&moved_out, 1).member_count, 1);
        assert_eq!(
            row_by_bookmark(&moved_out, 3).collection.as_deref(),
            Some("Inbox")
        );

        // Remove the collection: members move out first, then the row
        // drops; every record survives.
        client
            .send_ok(&bbr_testkit::Request::SavedPagesLibraryCollection {
                op: bbr_testkit::protocol::SavedPagesLibraryCollectionOp::Remove { id: 1 },
            })
            .expect("remove collection");
        let removed = wait_library(&mut client, |l| l.collections.is_empty());
        assert!(removed.rows.iter().all(|row| row.collection.is_none()));
        assert_eq!(removed.total, 3, "records are never lost");
        let store = client.snapshot().expect("store after collection remove");
        assert_eq!(store.store.saved_pages_count, 3);
    }));
    finish(result, &mut harness, &socket, &token);
}
