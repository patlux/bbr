//! Shared Saved Pages Library E2E helpers (B3 packet `saved-pages-library`):
//! the deterministic record seeding, the deadline-bounded snapshot polling,
//! and the row/collection lookups the suite's tests share. Split from the
//! test binary so no file outgrows the ~500-line budget.
//!
//! Every helper synchronizes on typed snapshot state under an explicit
//! deadline — no sleeps and no pixel comparisons.

use crate::e2e_common::*;
use bbr_testkit::protocol::{SavedPagesCollectionSnapshot, SavedPagesLibrarySnapshot};

/// Poll the typed Library snapshot until `predicate` holds; panics under
/// the shared explicit deadline with the last observed state.
pub fn wait_library(
    client: &mut Client,
    predicate: impl Fn(&SavedPagesLibrarySnapshot) -> bool,
) -> SavedPagesLibrarySnapshot {
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("library snapshot");
        let library = &snapshot.library;
        if predicate(library) {
            return library.clone();
        }
        assert!(
            Instant::now() < until,
            "Library state never satisfied the predicate; last {:?}",
            library
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The single row snapshot for `bookmark` in a Library snapshot.
pub fn row_by_bookmark(
    library: &SavedPagesLibrarySnapshot,
    bookmark: u64,
) -> &bbr_testkit::protocol::SavedPagesLibraryRowSnapshot {
    library
        .rows
        .iter()
        .find(|row| row.bookmark == bookmark)
        .unwrap_or_else(|| panic!("Library row {bookmark} missing; rows {:?}", library.rows))
}

/// The collection snapshot for `id` in a Library snapshot.
pub fn collection_by_id(
    library: &SavedPagesLibrarySnapshot,
    id: u64,
) -> &SavedPagesCollectionSnapshot {
    library
        .collections
        .iter()
        .find(|collection| collection.id == id)
        .unwrap_or_else(|| {
            panic!(
                "Library collection {id} missing; collections {:?}",
                library.collections
            )
        })
}

/// The bookmark ids of the rendered rows, in render order.
pub fn row_bookmarks(library: &SavedPagesLibrarySnapshot) -> Vec<u64> {
    library.rows.iter().map(|row| row.bookmark).collect()
}

/// Seed `count` Saved Page records through the one B2 save authority (the
/// typed `SaveSavedPage` seam Cmd-D shares — pure in-memory provider work,
/// no navigation). The seed contract for the 1k bounded-window evidence:
///
/// - `i == 0` carries an empty captured title and the distinct
///   `missing-000` host, so its row title falls back to the hostname (the
///   §11.16 missing-title fallback) and its title-sort key lands last
///   (`P` < `m` bytewise);
/// - `i == count - 1` carries an over-budget URL (the mid-ellipsis
///   evidence) and the title `Page {count-1}`;
/// - every other record carries the zero-padded title `Page {i:03}`, so
///   the title-ascending order is exactly numeric ascending and the window
///   math in the E2E assertions is fully deterministic.
pub fn seed_pages(client: &mut Client, count: usize, base_url: &str) {
    for i in 0..count {
        let (url, title) = if i == 0 {
            (
                format!("{base_url}missing-000.example.test/"),
                String::new(),
            )
        } else if i == count - 1 {
            (
                format!(
                    "{base_url}very-long-host-{i}.example.test/some/deeply/nested/path/segment/{i}/and/another/long/segment"
                ),
                format!("Page {i}"),
            )
        } else {
            (
                format!("{base_url}page-{i}.example.test/"),
                format!("Page {i:03}"),
            )
        };
        client
            .send_ok(&bbr_testkit::Request::SaveSavedPage {
                url: url.clone(),
                title: title.clone(),
            })
            .expect("seed save");
    }
}
