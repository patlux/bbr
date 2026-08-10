//! 1k bounded-render E2E (B3 packet `saved-pages-library`): the surface
//! renders a fixed viewport window over 1,000 records — never the full
//! list — with deterministic scroll offsets and a stable anchor, selection
//! inside the window, the mid-ellipsized long-URL fallback, the
//! missing-title hostname fallback, and no second Saved Page authority.

use crate::e2e_common::*;
use crate::support::{row_by_bookmark, seed_pages, wait_library};
use crate::{open_library_typed, wait_initial_title};

/// The 1k bounded-render evidence: the surface renders a fixed viewport
/// window over 1,000 records — never the full list — with deterministic
/// scroll offsets and a stable anchor, selection inside the window, the
/// mid-ellipsized long-URL fallback, the missing-title hostname fallback,
/// and no second Saved Page authority (the typed counts agree).
#[test]
#[ignore]
fn library_one_thousand_records_stay_bounded() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-1k", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        wait_initial_title(&mut client, &mut since);
        // Seed 1,000 records through the one save authority (see the seed
        // contract in `support`): 998 titled rows, the missing-title row,
        // and the over-budget-URL row.
        seed_pages(&mut client, 1000, "https://");
        open_library_typed(&mut client, &mut since);
        let seeded = wait_library(&mut client, |l| l.total == 1000);
        assert_eq!(seeded.rows.len(), 24, "the viewport is bounded");
        assert_eq!(
            (seeded.window_first, seeded.window_last),
            (0, 24),
            "the initial window"
        );
        assert_eq!(seeded.window_anchor, Some(seeded.rows[0].bookmark));
        let snapshot = client.snapshot().expect("seeded snapshot");
        assert_eq!(
            snapshot.store.saved_pages_count, 1000,
            "one provider authority"
        );

        // Deterministic sort ladder: the title-ascending order is numeric
        // ascending (zero-padded titles), so the window math is exact.
        client
            .send_ok(&bbr_testkit::Request::CycleSavedPagesLibrarySort)
            .expect("cycle sort");
        client
            .send_ok(&bbr_testkit::Request::CycleSavedPagesLibrarySort)
            .expect("cycle sort to title ascending");
        wait_library(&mut client, |l| l.sort == "title_ascending");

        // Scrolled window: offset 500 windows [500, 524) anchored on the
        // first visible row; the anchor is the stable BookmarkId.
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibraryScroll { offset: 500 })
            .expect("set scroll 500");
        let scrolled = wait_library(&mut client, |l| l.window_first == 500);
        assert_eq!((scrolled.window_first, scrolled.window_last), (500, 524));
        assert_eq!(scrolled.rows.len(), 24);
        assert_eq!(
            scrolled.rows[0].bookmark, 502,
            "index 500 in the numeric order"
        );
        assert_eq!(scrolled.window_anchor, Some(502));

        // Clamped window: an offset past the end renders the last complete
        // viewport [976, 1000).
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibraryScroll { offset: 9999 })
            .expect("set scroll past end");
        let clamped = wait_library(&mut client, |l| l.window_first == 976);
        assert_eq!((clamped.window_first, clamped.window_last), (976, 1000));
        assert_eq!(clamped.rows.len(), 24);
        assert_eq!(
            clamped.rows[0].bookmark, 978,
            "the last window starts at 976"
        );
        assert_eq!(clamped.window_anchor, Some(978));

        // Selection inside the windowed collection.
        let anchor = clamped.rows[0].bookmark;
        client
            .send_ok(&bbr_testkit::Request::ToggleSavedPagesLibrarySelect { id: anchor })
            .expect("toggle select anchor");
        let selected = wait_library(&mut client, |l| l.selected == vec![anchor]);
        assert!(row_by_bookmark(&selected, anchor).selected);
        client
            .send_ok(&bbr_testkit::Request::ClearSavedPagesLibrarySelection)
            .expect("clear selection");
        wait_library(&mut client, |l| l.selected.is_empty());

        // The long-URL fallback: the over-budget URL is mid-ellipsized to
        // the 48-char budget with the ellipsis mark.
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibrarySearch {
                text: "Page 999".into(),
            })
            .expect("search long row");
        let long = wait_library(&mut client, |l| l.total == 1);
        let long_row = &long.rows[0];
        assert_eq!(long_row.bookmark, 1000);
        // The budget is a char budget (`mid_ellipsis`); the ellipsis mark
        // itself is multibyte UTF-8, so assert the char count.
        assert_eq!(long_row.url.chars().count(), 48, "the URL fits the budget");
        assert!(long_row.url.contains('…'), "the ellipsis is present");

        // The missing-title fallback: the empty captured title falls back
        // to the hostname on the display ladder.
        client
            .send_ok(&bbr_testkit::Request::SetSavedPagesLibrarySearch {
                text: "missing-000".into(),
            })
            .expect("search missing-title row");
        let missing = wait_library(&mut client, |l| l.total == 1);
        assert_eq!(missing.rows[0].bookmark, 1);
        assert_eq!(
            missing.rows[0].title, "missing-000.example.test",
            "the hostname is the missing-title fallback"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}
