//! Pointer scroll-wheel E2E (B3 packet `saved-pages-library`): the bounded
//! list's production wheel listener path — the wheel clamp math
//! (`scroll_offset_for_wheel` over the tracked offset, total, and viewport)
//! feeding the one production scroll entry (`library_set_scroll`) that the
//! bounded render window derives from. Driven over the seeded 1,000-record
//! provider so the window/anchor math is exact; the wheel path is never a
//! typed-direct `SetSavedPagesLibraryScroll` substitute.

use crate::e2e_common::*;
use crate::open_library_typed;
use crate::support::{seed_pages, wait_library};

/// The production wheel path over 1,000 records: a wheel-down of 500 rows
/// lands the exact [500, 524) window anchored on BookmarkId 502, a wheel
/// past the end clamps to the last full window [976, 1000), and a wheel
/// back up clamps to the first window — the tracked offset, window bounds,
/// and anchor all derive from the one production scroll entry and the one
/// AppStore authority reports the unchanged 1,000-record count.
#[test]
#[ignore]
fn library_pointer_wheel_scrolls_the_production_path() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-ptr-wheel", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        crate::wait_initial_title(&mut client, &mut since);
        seed_pages(&mut client, 1000, "https://");
        open_library_typed(&mut client, &mut since);
        let seeded = wait_library(&mut client, |l| l.total == 1000);
        assert_eq!(seeded.rows.len(), 24, "the viewport is bounded");
        assert_eq!((seeded.window_first, seeded.window_last), (0, 24));
        assert_eq!(seeded.scroll_offset, 0, "the wheel offset starts at 0");

        // Title-ascending makes the window math exact (numeric ascending
        // zero-padded titles; the missing-title row sorts last).
        client
            .send_ok(&bbr_testkit::Request::CycleSavedPagesLibrarySort)
            .expect("cycle sort");
        client
            .send_ok(&bbr_testkit::Request::CycleSavedPagesLibrarySort)
            .expect("cycle sort to title ascending");
        wait_library(&mut client, |l| l.sort == "title_ascending");

        // Wheel down 500 rows: the production clamp math yields the exact
        // [500, 524) window anchored on BookmarkId 502.
        client
            .send_ok(&bbr_testkit::Request::ScrollSavedPagesLibraryWheel { delta_rows: 500 })
            .expect("wheel down 500");
        let scrolled = wait_library(&mut client, |l| l.scroll_offset == 500);
        assert_eq!((scrolled.window_first, scrolled.window_last), (500, 524));
        assert_eq!(scrolled.rows.len(), 24);
        assert_eq!(
            scrolled.rows[0].bookmark, 502,
            "index 500 in the numeric order"
        );
        assert_eq!(scrolled.window_anchor, Some(502));

        // Wheel past the end: the offset clamps to the last full-window
        // start 976 and the window renders [976, 1000).
        client
            .send_ok(&bbr_testkit::Request::ScrollSavedPagesLibraryWheel { delta_rows: 9999 })
            .expect("wheel past end");
        let clamped = wait_library(&mut client, |l| l.scroll_offset == 976);
        assert_eq!((clamped.window_first, clamped.window_last), (976, 1000));
        assert_eq!(clamped.rows.len(), 24);
        assert_eq!(clamped.rows[0].bookmark, 978);
        assert_eq!(clamped.window_anchor, Some(978));

        // Wheel back up past the start: the offset clamps to 0 and the
        // window returns to [0, 24).
        client
            .send_ok(&bbr_testkit::Request::ScrollSavedPagesLibraryWheel { delta_rows: -9999 })
            .expect("wheel past start");
        let home = wait_library(&mut client, |l| l.scroll_offset == 0);
        assert_eq!((home.window_first, home.window_last), (0, 24));
        assert_eq!(home.rows[0].bookmark, 2, "the first titled row");

        // A small negative wheel at the top clamps to 0 (never negative),
        // and the one AppStore authority's count is untouched.
        client
            .send_ok(&bbr_testkit::Request::ScrollSavedPagesLibraryWheel { delta_rows: -5 })
            .expect("small negative wheel");
        let still_home = wait_library(&mut client, |l| l.scroll_offset == 0);
        assert_eq!((still_home.window_first, still_home.window_last), (0, 24));
        let snapshot = client.snapshot().expect("after wheel snapshot");
        assert_eq!(snapshot.store.saved_pages_count, 1000, "one authority");
    }));
    finish(result, &mut harness, &socket, &token);
}
