//! Row-activation E2E (B3 packet `saved-pages-library`, B2 command
//! authority): opening a Library row revalidates the record and creates
//! exactly one fresh regular tab in the currently active Space/Profile —
//! never a silent selection of the existing matching tab; a stale identity
//! rejects truthfully. The Library stays open and keeps hiding the fresh
//! native slot.

use crate::e2e_common::*;
use crate::support::wait_library;
use crate::{open_library_typed, save, tab_evidence};

/// Opening a Library row revalidates the record and creates exactly one
/// fresh regular tab in the currently active Space/Profile through the B2
/// command authority — never a silent selection of the existing matching
/// tab; a stale identity rejects truthfully. The Library stays open and
/// keeps hiding the fresh native slot.
#[test]
#[ignore]
fn library_activation_opens_exactly_one_fresh_tab() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-open-row", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        save(&mut client, &page_one, PAGE_ONE_TITLE);
        save(&mut client, "https://open-b.test/", "Open B");
        open_library_typed(&mut client, &mut since);
        wait_library(&mut client, |l| l.total == 2);
        let before = client.snapshot().expect("before activation snapshot");
        let (tabs, _all, sessions) = tab_evidence(&before);
        let before_view = before.active_web_view.expect("source view");
        let active_space = before.active_space;
        let source_tab = before
            .all_tabs
            .iter()
            .find(|tab| tab.url == page_one)
            .expect("source tab")
            .id;

        // Activate the saved page row: exactly one fresh regular tab and
        // one live session in the same active space.
        client
            .send_ok(&bbr_testkit::Request::ActivateSavedPagesLibraryRow { id: 1 })
            .expect("activate row");
        let opened = wait_library(&mut client, |l| l.total == 2);
        assert!(opened.open, "the Library stays open over the fresh view");
        assert!(opened.native_view_hidden, "the fresh slot stays hidden");
        assert_eq!(opened.saved_pages_count, 2);
        let fresh = client.snapshot().expect("fresh tab snapshot");
        assert_eq!(fresh.tabs.len(), tabs + 1, "exactly one new regular tab");
        assert_eq!(fresh.engine_sessions, sessions + 1, "one new live session");
        let new_view = fresh.active_web_view.expect("fresh view");
        assert_ne!(new_view, before_view, "the fresh view is not the source");
        assert_eq!(fresh.library.total, 2);
        let added = fresh
            .all_tabs
            .iter()
            .find(|tab| tab.url == page_one && tab.id != source_tab)
            .expect("the fresh regular tab");
        assert!(!added.pinned, "never pinned");
        assert_eq!(added.space, active_space, "the currently active space");
        assert_eq!(
            added.web_view,
            Some(new_view),
            "the fresh view backs the row"
        );
        // The destination is the record's canonical URL in the address text.
        assert_eq!(fresh.command_bar_text, page_one);
        if fresh.title.as_deref() != Some(PAGE_ONE_TITLE) {
            wait_title(&mut client, new_view, PAGE_ONE_TITLE, since);
        }

        // A stale identity rejects truthfully and changes nothing.
        let stale = client
            .request(&bbr_testkit::Request::ActivateSavedPagesLibraryRow { id: 9999 })
            .expect("stale activation");
        match stale {
            bbr_testkit::protocol::Response::Error(error) => {
                assert_eq!(error.code, "saved_page_missing")
            }
            other => panic!("expected a stale-id rejection, got {other:?}"),
        }
        let after = client.snapshot().expect("after stale snapshot");
        assert_eq!(after.tabs.len(), tabs + 1, "the stale id opens nothing");
    }));
    finish(result, &mut harness, &socket, &token);
}
