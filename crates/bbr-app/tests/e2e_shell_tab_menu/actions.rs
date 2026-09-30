//! Action-row scenarios (task packet `tab-favorite-context-menus`).
//!
//! Proves the supported rows — Pin/Unpin, Suspend, the Favorite toggle,
//! Copy Link (the real pasteboard carries the **source** URL), region
//! ordering, Close, and Reopen — all dispatch exactly one typed command
//! against the bound source `TabId`, never the selected neighbor; the
//! unsupported rows (Duplicate Tab, native Share, split, media Mute,
//! Rename/Icon, Move to New Window, bulk-close scopes) have no typed
//! authority and are omitted entirely rather than rendered as decorative
//! dead rows.

use std::process::Command;

use crate::e2e_common::*;
use crate::support::*;

/// The §15 ids whose typed command authority does not exist yet (task
/// packet non-goals): an unavailable whole group is omitted, never filled
/// with dead rows.
const UNSUPPORTED_ROW_IDS: &[&str] = &[
    "bbr-tab-menu-duplicate",
    "bbr-tab-menu-share",
    "bbr-tab-menu-split",
    "bbr-tab-menu-mute",
    "bbr-tab-menu-rename",
    "bbr-tab-menu-icon",
    "bbr-tab-menu-new-window",
    "bbr-tab-menu-close-window",
    "bbr-tab-menu-close-other",
    "bbr-tab-menu-close-right",
    "bbr-tab-menu-close-all",
];

fn row_ids(snapshot: &bbr_testkit::protocol::Snapshot) -> Vec<String> {
    snapshot
        .tab_menu
        .rows
        .iter()
        .map(|row| row.id.clone())
        .collect()
}

/// Open the menu on an inactive tab in a one-space, no-folder harness and
/// assert the supported row model exactly: the owned rows render in the
/// packet's group order with their genuine enablement, and every
/// unsupported row id is absent.
#[test]
#[ignore]
fn owned_rows_render_and_unsupported_rows_are_omitted() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-rows", &page_one);

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
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let before = client.snapshot().expect("before snapshot");
        let source_tab = active_space_tab_id(&before, 0);

        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        // The owned rows, in the packet's group order (no folders, one
        // space → the placement submenu groups are omitted whole; Reload
        // and Reopen are disabled for a session-less/no-recent source; the
        // B2 bookmark row is present because the deterministic fixture URL
        // is navigable for a durable Saved Page).
        let ids = row_ids(&open);
        assert_eq!(
            ids,
            vec![
                "bbr-tab-menu-reload",
                "bbr-tab-menu-favorite",
                "bbr-tab-menu-bookmark",
                "bbr-tab-menu-pin",
                "bbr-tab-menu-suspend",
                "bbr-tab-menu-copy-link",
                "bbr-tab-menu-move-start",
                "bbr-tab-menu-move-end",
                "bbr-tab-menu-close-tab",
                "bbr-tab-menu-reopen",
            ]
        );
        for id in UNSUPPORTED_ROW_IDS {
            assert!(
                !ids.iter().any(|present| present == id),
                "unsupported row {id} must be omitted, not a dead row"
            );
        }
        // Genuine enablement on the wire: Reload enabled (the deterministic
        // fixture source owns a live session), Suspend + Copy Link +
        // Move to End enabled for the hidden warm middle source, Move to
        // Start disabled at the region edge, Reopen disabled (no
        // recently-closed tab yet).
        let by_id = |id: &str| {
            open.tab_menu
                .rows
                .iter()
                .find(|row| row.id == id)
                .unwrap_or_else(|| panic!("missing row {id}"))
        };
        assert!(by_id("bbr-tab-menu-reload").enabled);
        assert!(by_id("bbr-tab-menu-suspend").enabled);
        assert!(by_id("bbr-tab-menu-copy-link").enabled);
        assert!(
            !by_id("bbr-tab-menu-move-start").enabled,
            "first row has no Move to Start"
        );
        assert!(by_id("bbr-tab-menu-move-end").enabled);
        assert!(
            !by_id("bbr-tab-menu-reopen").enabled,
            "no recently-closed tab yet"
        );
        assert!(
            by_id("bbr-tab-menu-close-tab").danger,
            "Close Tab is the danger row"
        );
        // The visible §15 ids mirror the open rows exactly.
        for id in &ids {
            assert!(
                open.visible_ids.iter().any(|present| present == id),
                "visible id {id} missing"
            );
        }
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Pin, suspend, and the Favorite toggle target the bound source rather
/// than the selected neighbor; the live neighbor session and the selection
/// never change.
#[test]
#[ignore]
fn pin_suspend_and_favorite_target_the_source_tab() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-actions", &page_one);

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
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));

        let before = client.snapshot().expect("before snapshot");
        assert_eq!(before.active_tab_index, Some(2), "the neighbor is selected");
        let source_tab = active_space_tab_id(&before, 0);
        let neighbor = active_space_tab_id(&before, 1);

        // Pin the hidden source through the menu: the source row flips to
        // the pinned region, the neighbor row stays unpinned, and the
        // selected neighbor (index 2) keeps the selection.
        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let pin_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-pin")
            .expect("pin row");
        assert_eq!(open.tab_menu.rows[pin_row].label, "Pin Tab");
        menu_activate(&mut client, pin_row);
        let pinned = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            pinned.pinned_count, 1,
            "the source moved into the pinned region"
        );
        let source_now = pinned
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .expect("source survives");
        assert!(source_now.pinned, "the source is pinned");
        let neighbor_now = pinned
            .all_tabs
            .iter()
            .find(|tab| tab.id == neighbor)
            .expect("neighbor survives");
        assert!(!neighbor_now.pinned, "the neighbor is not pinned");
        assert_eq!(
            pinned.active_tab_index,
            Some(2),
            "the neighbor stays selected"
        );
        assert!(
            pinned.all_tabs.iter().any(|tab| tab.id == neighbor),
            "the neighbor session survives"
        );

        // Unpin through the same row (the label follows the pinned state).
        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let pin_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-pin")
            .expect("unpin row");
        assert_eq!(open.tab_menu.rows[pin_row].label, "Unpin Tab");
        menu_activate(&mut client, pin_row);
        let unpinned = wait_menu_closed(&mut client, deadline());
        assert_eq!(unpinned.pinned_count, 0);

        // Suspend the hidden warm source: the row is enabled and the
        // session disappears while the neighbor's live session stays. The
        // source moved to the regular-region end when it was unpinned, so
        // the menu opens on its current row (re-resolved from the
        // snapshot).
        let neighbor_session = unpinned
            .all_tabs
            .iter()
            .find(|tab| tab.id == neighbor)
            .and_then(|tab| tab.web_view);
        let source_row = active_space_row_of(&unpinned, source_tab);
        assert_eq!(source_row, 2, "unpin moved the source to the region end");
        let open = open_menu(
            &mut client,
            source_row,
            source_tab,
            MENU_SOURCE_TAB,
            140.0,
            240.0,
        );
        let suspend_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-suspend")
            .expect("suspend row");
        assert!(
            open.tab_menu.rows[suspend_row].enabled,
            "a hidden warm tab can suspend"
        );
        menu_activate(&mut client, suspend_row);
        let suspended = wait_menu_closed(&mut client, deadline());
        let source_now = suspended
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .expect("source survives");
        assert_eq!(source_now.lifecycle, "suspended");
        assert_eq!(
            source_now.web_view, None,
            "the suspended source owns no session"
        );
        let neighbor_now = suspended
            .all_tabs
            .iter()
            .find(|tab| tab.id == neighbor)
            .expect("neighbor survives");
        assert_eq!(
            neighbor_now.web_view, neighbor_session,
            "the neighbor's live session is untouched"
        );
        assert_eq!(
            suspended.active_tab_index,
            Some(1),
            "the neighbor stays selected (its index shifted when the source
             unpinned to the region end)"
        );

        // Favorite toggle through the menu: the source becomes a favorite
        // (the grid tile resolves the exact same TabId); a Favorite is
        // pinned, so the source moves to the pinned region and leaves the
        // tab list (DESIGN §11.10). The suspended source's Copy Link row is disabled:
        // a session-less tab cannot dispatch the pasteboard command.
        let source_row = active_space_row_of(&suspended, source_tab);
        let selected_tab = active_space_tab_id(
            &suspended,
            suspended.active_tab_index.expect("a selected tab"),
        );
        let open = open_menu(
            &mut client,
            source_row,
            source_tab,
            MENU_SOURCE_TAB,
            140.0,
            240.0,
        );
        let copy_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-copy-link")
            .expect("copy link row");
        assert!(
            !open.tab_menu.rows[copy_row].enabled,
            "Copy Link is disabled for the suspended source"
        );
        let favorite_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-favorite")
            .expect("favorite row");
        assert_eq!(open.tab_menu.rows[favorite_row].label, "Add to Favorites");
        menu_activate(&mut client, favorite_row);
        let favorited = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            favorited.favorites.first().map(|tile| tile.tab),
            Some(source_tab),
            "the favorite tile resolves the exact source TabId"
        );
        assert_eq!(
            active_space_row_of(&favorited, source_tab),
            0,
            "favoriting pinned the source to the front"
        );
        assert_eq!(favorited.pinned_count, 1);
        assert_eq!(
            favorited.active_tab_index,
            Some(active_space_row_of(&favorited, selected_tab)),
            "the selected tab stays selected (its index shifted with the pin)"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Copy Link writes the **source** tab's URL to the real system pasteboard
/// (read back via `pbpaste`) without selecting the source, and the region
/// ordering rows move the source within its own region.
#[test]
#[ignore]
fn copy_link_writes_the_source_url_and_ordering_moves_the_source() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-copy", &page_one);

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
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        // A third distinct fixture: the source (page two, index 1) differs
        // from the selected neighbor (long title, index 2), so the paste
        // content can only come from the bound source.
        let long_title = fixtures.url("/long-title.html");
        let long_view = open_tab(&mut client, &mut since, &long_title, Some(LONG_TITLE));

        let before = client.snapshot().expect("before snapshot");
        assert_eq!(before.active_tab_index, Some(2), "the neighbor is selected");
        let source_tab = active_space_tab_id(&before, 1);
        assert_eq!(
            before
                .all_tabs
                .iter()
                .find(|t| t.id == source_tab)
                .map(|t| t.url.as_str()),
            Some(page_two.as_str()),
            "the source is the page-two tab"
        );

        // Copy Link: the real pasteboard carries the source's URL, the
        // source is never selected, and the snapshot identity is unchanged.
        let open = open_menu(&mut client, 1, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let copy_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-copy-link")
            .expect("copy link row");
        menu_activate(&mut client, copy_row);
        let copied = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            read_pasteboard().as_deref(),
            Some(page_two.as_str()),
            "Copy Link wrote the source URL (not the neighbor's)"
        );
        assert_eq!(
            copied.active_tab_index,
            Some(2),
            "the source is never selected"
        );
        assert_eq!(
            copied.active_web_view,
            Some(long_view),
            "the neighbor stays presented"
        );

        // Ordering: Move to End moves the middle source to its region end;
        // the neighbor selection is untouched.
        let open = open_menu(&mut client, 1, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let end_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-move-end")
            .expect("move-end row");
        assert!(
            open.tab_menu.rows[end_row].enabled,
            "a middle row can move to the end"
        );
        menu_activate(&mut client, end_row);
        let reordered = wait_menu_closed(&mut client, deadline());
        let moved = reordered
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .expect("source survives");
        assert_eq!(moved.index, 2, "the source moved to the region end");
        assert_eq!(
            reordered.active_tab_index,
            Some(1),
            "the selected neighbor stays selected (its index shifted from 2
             to 1 when the middle source moved to the end)"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Close and Reopen target the source's own space: closing the bound source
/// leaves the selected neighbor presented, and the reopen row (legal only
/// after the close) restores the closed identity in the same space.
#[test]
#[ignore]
fn close_and_reopen_target_the_sources_own_space() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-close", &page_one);

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
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));

        let before = client.snapshot().expect("before snapshot");
        assert_eq!(before.active_tab_index, Some(1), "the neighbor is selected");
        let source_tab = active_space_tab_id(&before, 0);
        let neighbor_view = before
            .all_tabs
            .iter()
            .find(|tab| tab.id == active_space_tab_id(&before, 1))
            .and_then(|tab| tab.web_view)
            .expect("the neighbor owns a live session");

        // Close the bound source: it disappears; the selected neighbor
        // (page two) keeps its session and stays active.
        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let close_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-close-tab")
            .expect("close row");
        menu_activate(&mut client, close_row);
        let closed = wait_menu_closed(&mut client, deadline());
        assert!(
            !closed.all_tabs.iter().any(|tab| tab.id == source_tab),
            "the bound source is closed"
        );
        assert_eq!(closed.all_tabs.len(), 1, "exactly the source is gone");
        assert_eq!(
            closed.active_tab_index,
            Some(0),
            "the neighbor stays selected (its index shifted to 0 when the
             source closed)"
        );
        assert_eq!(
            closed.active_web_view,
            Some(neighbor_view),
            "the neighbor stays presented"
        );

        // Reopen: the menu on the remaining row now shows the legal Reopen
        // row; committing restores the closed identity in the same space.
        let survivor = active_space_tab_id(&closed, 0);
        let open = open_menu(&mut client, 0, survivor, MENU_SOURCE_TAB, 140.0, 240.0);
        let reopen_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-reopen")
            .expect("reopen row");
        assert!(
            open.tab_menu.rows[reopen_row].enabled,
            "reopen is legal after a close"
        );
        menu_activate(&mut client, reopen_row);
        let reopened = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            reopened.all_tabs.len(),
            2,
            "the closed identity is restored"
        );
        let restored = reopened
            .all_tabs
            .iter()
            .find(|tab| tab.url == page_one)
            .expect("the closed page is restored in the same space");
        assert_eq!(restored.space, reopened.active_space);
        assert_eq!(
            reopened.active_tab_index,
            Some(0),
            "the selection never jumps"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Read the system general pasteboard through `pbpaste` (the macOS CLI).
fn read_pasteboard() -> Option<String> {
    let output = Command::new("pbpaste").output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
    )
}
