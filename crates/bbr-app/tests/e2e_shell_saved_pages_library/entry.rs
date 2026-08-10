//! Production entry E2E (B3 packet `saved-pages-library`): Cmd-Shift-L
//! opens the Library through the integrated production entry, the surface
//! covers the content area, the active view's native slot is hidden (never
//! destroyed), the command surface is dismissed, and Escape restores the
//! exact view with focus back on the page.

use crate::e2e_common::*;
use crate::{close_library_escape, open_library, tab_evidence};
use bbr_testkit::protocol::CommandBarMode;

/// Production open through Cmd-Shift-L (the integrated entry): the surface
/// covers the content area, the active view's native slot is hidden (never
/// destroyed), the command surface is dismissed, and Escape restores the
/// exact view with focus back on the page.
#[test]
#[ignore]
fn library_production_open_hides_native_view_and_escape_restores() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("library-open", &page_one);
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
        let before = tab_evidence(&client.snapshot().expect("before snapshot"));
        let store = client.snapshot().expect("store snapshot");
        assert_eq!(store.store.saved_pages_count, 0, "starts empty");

        // Cmd-Shift-L: the Library opens ready, hides the active native
        // slot, dismisses any command surface, and exposes the empty
        // decided state with the surface ids.
        open_library(&mut client, &mut since);
        let opened = client.snapshot().expect("opened snapshot");
        assert_eq!(opened.library.stage, "ready");
        assert_eq!(opened.library.total, 0);
        assert!(opened.library.rows.is_empty());
        assert!(
            opened.library.native_view_hidden,
            "the active slot is hidden"
        );
        assert_eq!(opened.command_bar_mode, CommandBarMode::Inactive);
        assert_ids(
            &opened,
            &[
                "bbr-saved-pages-library",
                "bbr-saved-pages-search",
                "bbr-saved-pages-sort",
                "bbr-saved-pages-close",
                "bbr-saved-pages-empty",
            ],
        );
        assert_eq!(tab_evidence(&opened), before, "opening never touches tabs");

        // A repeated open is a no-op (the surface is already open).
        client
            .send_ok(&bbr_testkit::Request::OpenSavedPagesLibrary)
            .expect("repeat open request");
        let again = client.snapshot().expect("repeat open snapshot");
        assert!(again.library.open);
        assert_eq!(again.library.total, 0);

        // Escape: the exact native view is restored and web focus returns.
        close_library_escape(&mut client, &mut since);
        let closed = client.snapshot().expect("closed snapshot");
        assert!(!closed.library.native_view_hidden);
        assert_eq!(
            closed.active_web_view,
            Some(first_view),
            "the same native view returns"
        );
        assert_eq!(tab_evidence(&closed), before, "closing never touches tabs");
    }));
    finish(result, &mut harness, &socket, &token);
}
