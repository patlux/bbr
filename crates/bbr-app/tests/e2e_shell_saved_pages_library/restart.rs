//! Restart persistence E2E (B3 packet `saved-pages-library`): records,
//! labels, and the flat collection with its membership commit through the
//! deterministic flush and restore on a same-profile restart — the
//! Library's open reset leaves search and sort at their defaults.

use crate::e2e_common::*;
use crate::support::{collection_by_id, row_by_bookmark, wait_library};
use crate::{open_library_typed, save};

/// Restart persistence: records, labels, and the flat collection with its
/// membership commit through the deterministic flush and restore on a
/// same-profile restart — the Library's open reset leaves search and sort
/// at their defaults.
#[test]
#[ignore]
fn library_persists_rows_labels_and_collections_across_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("library-restart", pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = harness_socket_path("library-restart", pid);
    let token = format!("shell-library-restart-{pid}");
    let bin = env!("CARGO_BIN_EXE_bbr");
    let spawn = |root: &PathBuf| {
        Harness::spawn(
            bin,
            &[
                "--bbr-initial-url",
                &page_one,
                "--bbr-profile-dir",
                profile.to_str().unwrap(),
                "--bbr-test-token",
                &token,
                "--bbr-ids=sequential",
                "--bbr-hidden",
            ],
            root,
        )
    };

    let mut harness = spawn(&root);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        if let Some(view) = first.active_web_view {
            if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
                let _ = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
            }
        }
        // Commit: two records, one labeled, one moved into a collection.
        save(&mut client, &page_one, PAGE_ONE_TITLE);
        save(&mut client, "https://persist-b.test/", "Persist B");
        client
            .send_ok(&bbr_testkit::Request::SavedPagesLibraryCollection {
                op: bbr_testkit::protocol::SavedPagesLibraryCollectionOp::Create {
                    title: "Work".into(),
                },
            })
            .expect("create collection");
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: 2 })
            .expect("select for persistence");
        client
            .send_ok(&bbr_testkit::Request::MoveSavedPagesLibrarySelection {
                collection: Some(1),
            })
            .expect("move into collection");
        client
            .send_ok(&bbr_testkit::Request::RenameSavedPagesLibraryRow {
                id: 1,
                label: "Labeled One".into(),
            })
            .expect("rename row");
        client
            .send_ok(&bbr_testkit::Request::FlushSavedPages)
            .expect("flush saved pages");
    }));
    // Keep the profile for the restart: a successful finish would let the
    // harness Drop remove the root and destroy the store we reload.
    match result {
        Ok(()) => {
            harness.success = false;
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    drop(harness);

    let mut harness2 = spawn(&root);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let store = client.snapshot().expect("restart store snapshot");
        assert_eq!(
            store.store.saved_pages_count, 2,
            "the committed records restored"
        );
        if store.title.as_deref() != Some(PAGE_ONE_TITLE) {
            if let Some(view) = store.active_web_view {
                since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
            }
        }
        open_library_typed(&mut client, &mut since);
        let restored = wait_library(&mut client, |l| l.total == 2);
        assert_eq!(restored.search, "", "open resets the search");
        assert_eq!(restored.sort, "updated_newest", "open resets the sort");
        assert_eq!(
            row_by_bookmark(&restored, 1).title,
            "Labeled One",
            "the label restored"
        );
        assert_eq!(
            row_by_bookmark(&restored, 2).collection.as_deref(),
            Some("Work"),
            "the collection membership restored"
        );
        assert_eq!(restored.collections.len(), 1);
        assert_eq!(collection_by_id(&restored, 1).title, "Work");
        assert_eq!(collection_by_id(&restored, 1).member_count, 1);
    }));
    finish(result2, &mut harness2, &socket, &token);
}
