//! Search/sort E2E (B3 packet `saved-pages-library`): the typed search
//! filters the matching window (a no-match query presents the decided
//! no-results state, never the empty state), and each deterministic sort
//! cycle applies the settled order with the machine name on the wire.

use crate::e2e_common::*;
use crate::support::{row_bookmarks, wait_library};
use crate::{open_library_typed, save, wait_initial_title};

/// Search and the deterministic sort ladder: the typed search filters the
/// matching window, and each sort cycle applies the settled order — the
/// updated orders follow the provider key, the title orders the display
/// ladder — with the machine names exposed on the wire.
#[test]
#[ignore]
fn library_search_and_sort_are_deterministic() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-sort", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        wait_initial_title(&mut client, &mut since);
        // Five records, distinct titles: Alpha/Beta/Gamma/Delta/Epsilon.
        for (i, (title, host)) in [
            ("Alpha", "alpha"),
            ("Beta", "beta"),
            ("Gamma", "gamma"),
            ("Delta", "delta"),
            ("Epsilon", "epsilon"),
        ]
        .iter()
        .enumerate()
        {
            save(&mut client, &format!("https://{host}.test/a/{i}"), title);
        }
        open_library_typed(&mut client, &mut since);
        let opened = client.snapshot().expect("opened snapshot");
        assert_eq!(opened.library.total, 5);
        assert_eq!(opened.library.sort, "updated_newest");
        assert_eq!(opened.store.saved_pages_count, 5, "one provider authority");
        // UpdatedNewest is the provider presentation order; assert the set
        // (the same-second tie-break is id order, but only the provider key
        // decides — the exact order is proven by the title sorts below).
        let mut bookmarks = row_bookmarks(&opened.library);
        bookmarks.sort_unstable();
        assert_eq!(bookmarks, vec![1, 2, 3, 4, 5]);

        // Search filters to exactly the Gamma row (title match, caseless).
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibrarySearch {
                text: "gaMmA".into(),
            })
            .expect("set search");
        let filtered = wait_library(&mut client, |l| l.total == 1);
        assert_eq!(filtered.rows.len(), 1);
        assert_eq!(filtered.rows[0].bookmark, 3);
        assert_eq!(filtered.rows[0].title, "Gamma");
        assert_eq!(filtered.window_first, 0, "search resets the scroll");

        // Search back to the unfiltered state.
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibrarySearch {
                text: String::new(),
            })
            .expect("clear search");
        wait_library(&mut client, |l| l.total == 5);

        // A search that matches nothing presents the decided no-results
        // state (records exist, none match) — never the empty state.
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibrarySearch {
                text: "zzz-no-match".into(),
            })
            .expect("no-match search");
        let no_results = wait_library(&mut client, |l| l.total == 0);
        assert!(no_results.rows.is_empty());
        let snapshot = client.snapshot().expect("no-results snapshot");
        assert_ids(&snapshot, &["bbr-saved-pages-no-results"]);

        // Clear the search and walk the deterministic sort cycle.
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibrarySearch {
                text: String::new(),
            })
            .expect("clear search");
        wait_library(&mut client, |l| l.total == 5);
        let cycle = [
            ("updated_oldest", vec![1, 2, 3, 4, 5]),
            ("title_ascending", vec![1, 2, 4, 5, 3]),
            ("title_descending", vec![3, 5, 4, 2, 1]),
            ("updated_newest", vec![]), // provider order, set-asserted below
        ];
        // Each iteration cycles the sort once more from the default state:
        // the `next()` ladder is cumulative, so the `i`-th iteration needs
        // exactly `i + 1` total cycles.
        let mut cycles = 0;
        for (i, (machine, expected)) in cycle.iter().enumerate() {
            let target = i + 1;
            for _ in cycles..target {
                client
                    .send_ok(&bbr_testkit::Request::CycleSavedPagesLibrarySort)
                    .expect("cycle sort");
            }
            cycles = target;
            let sorted = wait_library(&mut client, |l| l.sort == *machine);
            assert_eq!(sorted.sort, *machine, "machine name on the wire");
            if expected.is_empty() {
                let mut ids = row_bookmarks(&sorted);
                ids.sort_unstable();
                assert_eq!(ids, vec![1, 2, 3, 4, 5]);
            } else {
                assert_eq!(row_bookmarks(&sorted), *expected, "order for {machine}");
            }
        }
    }));
    finish(result, &mut harness, &socket, &token);
}
