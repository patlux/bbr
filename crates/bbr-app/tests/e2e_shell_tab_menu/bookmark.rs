//! B2 bookmark parity scenarios (task packet `tab-favorite-context-menus`;
//! packet `saved-pages-actions-command`).
//!
//! Proves that `Bookmark Page` / `Remove Bookmark` route through the one
//! app-owned Saved Page command against the bound source's current URL —
//! without selecting the source, without creating a duplicate canonical
//! URL, and without retaining the `TabId` in the Saved Page (the command
//! stores only the canonical URL + captured title). The tab/Favorite/
//! session identity is byte-stable across the whole save/remove cycle.

use crate::e2e_common::*;
use crate::support::*;

/// Bookmark Page captures the bound source's URL through B2 without
/// selecting it; the row flips to Remove Bookmark on the next open (the
/// canonical-URL dedupe proves the menu consumed the source's URL, not a
/// second record); removing restores the empty store; the source
/// tab/session/selection never change.
#[test]
#[ignore]
fn bookmark_save_and_remove_parity_never_select_or_duplicate() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-bookmark", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first native view");
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let neighbor_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));

        let before = client.snapshot().expect("before snapshot");
        assert_eq!(before.store.saved_pages_count, 0, "starts empty");
        assert_eq!(before.active_tab_index, Some(1), "the neighbor is selected");
        let source_tab = active_space_tab_id(&before, 0);
        let before_identity = identity(&before);

        // The menu shows Bookmark Page (B2 has no record for the source's
        // canonical URL yet) and the row is enabled.
        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let bookmark_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-bookmark")
            .expect("bookmark row");
        assert_eq!(open.tab_menu.rows[bookmark_row].label, "Bookmark Page");
        assert!(open.tab_menu.rows[bookmark_row].enabled);

        // Activate: exactly one Saved Page record appears through the one
        // app-owned command; the source is never selected; the snapshot
        // identity is byte-identical apart from the store count.
        menu_activate(&mut client, bookmark_row);
        let saved = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            saved.store.saved_pages_count, 1,
            "Bookmark Page created exactly one record"
        );
        assert_eq!(
            saved.active_tab_index,
            Some(1),
            "the source is never selected"
        );
        assert_eq!(
            saved.active_web_view,
            Some(neighbor_view),
            "the neighbor stays presented"
        );
        assert_eq!(
            saved
                .all_tabs
                .iter()
                .find(|tab| tab.id == source_tab)
                .and_then(|tab| tab.web_view),
            before
                .all_tabs
                .iter()
                .find(|tab| tab.id == source_tab)
                .and_then(|tab| tab.web_view),
            "the source session is untouched"
        );

        // Reopen the menu on the same source: the canonical-URL dedupe
        // flips the row to Remove Bookmark — the menu consumed the source's
        // URL and would never create a duplicate record.
        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let bookmark_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-bookmark")
            .expect("bookmark row");
        assert_eq!(
            open.tab_menu.rows[bookmark_row].label, "Remove Bookmark",
            "no duplicate canonical URL: the record for the source's URL exists"
        );
        assert!(open.tab_menu.rows[bookmark_row].enabled);

        // Remove through the same row: the store returns to empty and the
        // source identity is stable across the whole cycle.
        menu_activate(&mut client, bookmark_row);
        let removed = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            removed.store.saved_pages_count, 0,
            "Remove Bookmark removed exactly the one record"
        );
        assert_eq!(removed.active_tab_index, Some(1));
        let source_now = removed
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .expect("source survives");
        assert_eq!(source_now.url, page_one, "the source tab is unchanged");
        assert_eq!(source_now.space, before.active_space);
        let (tabs, _, sessions, membership) = identity(&removed);
        assert_eq!(tabs, before_identity.0, "tab order unchanged");
        assert_eq!(sessions, before_identity.2, "session population unchanged");
        assert_eq!(
            membership, before_identity.3,
            "session membership unchanged"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}
