//! Closed pinned tabs and Favorites that leave the list (DESIGN.md §10.12,
//! §11.2, §11.10; task packet `closed-pins-and-favorites`).
//!
//! A pinned tab is a kept place: a real ⌘W closes it in place — the row
//! stays, greyed, without a close control, owns no native session and shows
//! its pinned page — and a real click on the row opens the pinned URL in a
//! fresh WKWebView. A tab moved into Favorites leaves the tab list (the tile
//! *is* the tab); the tile carries the selection, greys out when the tab is
//! closed and reopens it on a real click; Remove from Favorites returns a
//! loose pinned row. A real ⇧⌘T after closing a pinned row brings back the
//! page it closed on in the same row.
//!
//! The window is shown and key, and the user-facing steps are real AppKit
//! events aimed at the rects the controls drew at. Every step synchronizes
//! on typed browser changes or drawn snapshots under a deadline — no sleeps.

#![cfg(target_os = "macos")]

mod e2e_common;
mod e2e_semantics_common;

use bbr_testkit::protocol::Snapshot;
use e2e_common::real_input::*;
use e2e_common::*;
use e2e_semantics_common::find;

/// Connect, settle on page one and make the window key.
fn start(client: &mut Client, token: &str) -> (WebViewId, u64) {
    client.hello(token).expect("hello");
    let mut since = client
        .attach(None)
        .expect("attach")
        .iter()
        .map(|(seq, _)| *seq)
        .max()
        .unwrap_or(0);
    let settled = wait_for_window_settled(client, deadline());
    let view = settled.active_web_view.expect("first native view");
    if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
        since = wait_title(client, view, PAGE_ONE_TITLE, since);
    }
    make_key(client);
    (view, since)
}

fn quit(client: &mut Client, harness: &mut Harness) {
    client.quit().expect("quit");
    let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
    assert!(status.success(), "app exit: {status}");
}

fn has_id(snapshot: &Snapshot, id: &str) -> bool {
    snapshot.visible_ids.iter().any(|present| present == id)
}

/// The persistent id of the active space's tab at `index`.
fn id_at(snapshot: &Snapshot, index: usize) -> u64 {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.space == snapshot.active_space && tab.index == index)
        .expect("active-space tab exists")
        .id
}

#[test]
#[ignore]
fn a_closed_pinned_row_stays_greyed_and_reopens_its_pinned_url() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("closed-pin", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (first_view, mut since) = start(&mut client, &token);
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));

        // Pin the first tab (a real click selects it, a real ⇧⌘P pins it).
        real_click(&mut client, "bbr-tab-row-0");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        real_key(&mut client, "cmd-shift-p");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);

        // The pinned tab navigates on; its pinned page stays page one.
        client
            .send_ok(&dsl::navigate(page_two.clone()))
            .expect("navigate the pinned tab");
        since = wait_title(&mut client, first_view, PAGE_TWO_TITLE, since);

        // A real ⌘W closes it in place.
        real_key(&mut client, "cmd-w");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let closed = wait_for(&mut client, "the closed row drawn", |s| {
            s.tabs.first().is_some_and(|tab| tab.lifecycle == "closed")
        });
        assert_eq!(closed.tabs.len(), 2, "the pinned row stays");
        assert_eq!(closed.pinned_count, 1);
        assert_eq!(closed.tabs[0].url, page_one, "reset to the pinned page");
        assert_eq!(closed.tabs[0].web_view, None, "no native session");
        assert_eq!(closed.engine_sessions, 1);
        assert_eq!(closed.lifecycle_counts.closed, 1);
        assert_eq!(
            closed.active_tab_index,
            Some(1),
            "the next open tab is selected"
        );
        assert!(has_id(&closed, "bbr-tab-row-0"));
        assert!(
            !has_id(&closed, "bbr-tab-close-0"),
            "a closed row offers no close control"
        );
        let row = find(&closed.semantics, "bbr-tab-row-0").expect("closed row node");
        assert_eq!(row.value.as_deref(), Some("closed"));
        assert!(!row.children.iter().any(|c| c.id == "bbr-tab-close-0"));

        // A real click on the closed row opens the pinned URL in a fresh
        // session.
        real_click(&mut client, "bbr-tab-row-0");
        let reopened = wait_for(&mut client, "the row reopened", |s| {
            s.active_tab_index == Some(0)
                && s.tabs[0].lifecycle == "active"
                && s.tabs[0].web_view.is_some()
        });
        let fresh = reopened.tabs[0].web_view.expect("a fresh session");
        assert_ne!(fresh, first_view, "the session was recreated");
        assert_eq!(reopened.engine_sessions, 2);
        wait_title(&mut client, fresh, PAGE_ONE_TITLE, since);
        let loaded = drawn(&mut client);
        assert_eq!(loaded.tabs[0].url, page_one);
        assert!(has_id(&loaded, "bbr-tab-close-0"));
        assert_eq!(
            find(&loaded.semantics, "bbr-tab-row-0")
                .expect("row node")
                .value,
            None
        );

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn a_favorite_leaves_the_list_greys_when_closed_and_reopens_from_its_tile() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("closed-favorite", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (_first_view, mut since) = start(&mut client, &token);
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let base = drawn(&mut client);
        let (first, second) = (id_at(&base, 0), id_at(&base, 1));
        assert_eq!(base.pinned_count, 0);

        // Favoriting the regular tab pins it by the same transition and
        // takes it out of the tab list: the tile is the tab.
        client
            .send_ok(&dsl::toggle_favorite_tab(1))
            .expect("favorite the second tab");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        let favored = wait_for(&mut client, "the tile drawn", |s| s.favorites.len() == 1);
        assert_eq!(favored.favorites[0].tab, second);
        assert_eq!(favored.pinned_count, 1);
        assert_eq!(id_at(&favored, 0), second, "pinned at the region end");
        assert!(has_id(&favored, "bbr-fav-0"));
        assert!(
            !has_id(&favored, "bbr-tab-row-0"),
            "the favorite leaves the tab list"
        );
        assert!(has_id(&favored, "bbr-tab-row-1"), "the other tab stays");
        assert!(
            favored.favorites[0].selected,
            "the tile carries the selection"
        );
        assert!(
            find(&favored.semantics, "bbr-fav-0")
                .expect("tile node")
                .selected
        );

        // Select the other tab, then activate the tile with a real click.
        real_click(&mut client, "bbr-tab-row-1");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        assert!(!drawn(&mut client).favorites[0].selected);
        real_click(&mut client, "bbr-fav-0");
        let active = wait_for(&mut client, "the tile's tab selected", |s| {
            s.active_tab_index == Some(0) && s.favorites[0].selected
        });
        let tile_view = active.tabs[0].web_view.expect("the favorite is live");

        // A real ⌘W closes the favorite in place: the tile stays, greyed.
        real_key(&mut client, "cmd-w");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let closed = wait_for(&mut client, "the closed tile drawn", |s| {
            s.favorites.first().is_some_and(|tile| tile.closed)
        });
        assert_eq!(closed.favorites.len(), 1, "the tile stays");
        assert!(!closed.favorites[0].selected);
        assert_eq!(closed.tabs[0].lifecycle, "closed");
        assert_eq!(closed.tabs[0].web_view, None);
        assert_eq!(closed.active_tab_index, Some(1));
        assert_eq!(closed.engine_sessions, 1);
        assert!(!has_id(&closed, "bbr-tab-row-0"), "still not in the list");
        assert_eq!(
            find(&closed.semantics, "bbr-fav-0")
                .expect("tile node")
                .value
                .as_deref(),
            Some("closed")
        );

        // A real click on the closed tile opens its page again.
        real_click(&mut client, "bbr-fav-0");
        let reopened = wait_for(&mut client, "the favorite reopened", |s| {
            s.active_tab_index == Some(0)
                && !s.favorites[0].closed
                && s.favorites[0].selected
                && s.tabs[0].web_view.is_some()
        });
        let fresh = reopened.tabs[0].web_view.expect("a fresh session");
        assert_ne!(fresh, tile_view);
        wait_title(&mut client, fresh, PAGE_TWO_TITLE, since);
        assert_eq!(drawn(&mut client).tabs[0].url, page_two);

        // Remove from Favorites returns a loose pinned row.
        client
            .send_ok(&dsl::toggle_favorite_tab(0))
            .expect("remove from favorites");
        let removed = wait_for(&mut client, "the row back in the list", |s| {
            s.favorites.is_empty() && s.visible_ids.iter().any(|id| id == "bbr-tab-row-0")
        });
        assert_eq!(removed.pinned_count, 1, "it stays pinned");
        assert_eq!(id_at(&removed, 0), second);
        assert_eq!(id_at(&removed, 1), first);

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn cmd_shift_t_reopens_a_closed_pinned_row_at_the_page_it_closed_on() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("closed-pin-reopen", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (first_view, mut since) = start(&mut client, &token);
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));

        // Pin the first tab at page one; it navigates on to page two.
        real_click(&mut client, "bbr-tab-row-0");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        real_key(&mut client, "cmd-shift-p");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        client
            .send_ok(&dsl::navigate(page_two.clone()))
            .expect("navigate the pinned tab");
        since = wait_title(&mut client, first_view, PAGE_TWO_TITLE, since);

        // A real ⌘W closes it in place at its pinned page.
        real_key(&mut client, "cmd-w");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let closed = wait_for(&mut client, "the closed row drawn", |s| {
            s.tabs.first().is_some_and(|tab| tab.lifecycle == "closed")
        });
        assert_eq!(closed.tabs[0].url, page_one);
        assert_eq!(closed.active_tab_index, Some(1));

        // A real ⇧⌘T brings back the page it closed on, in the same row.
        real_key(&mut client, "cmd-shift-t");
        since = wait_browser_change(&mut client, BrowserChange::TabReopened { index: 0 }, since);
        let reopened = wait_for(&mut client, "the row reopened", |s| {
            s.active_tab_index == Some(0)
                && s.tabs[0].lifecycle == "active"
                && s.tabs[0].web_view.is_some()
        });
        assert_eq!(reopened.tabs.len(), 2, "no row was added");
        assert_eq!(reopened.pinned_count, 1);
        assert_eq!(reopened.tabs[0].url, page_two, "the page it closed on");
        assert_eq!(reopened.lifecycle_counts.closed, 0);
        let fresh = reopened.tabs[0].web_view.expect("a fresh session");
        assert_ne!(fresh, first_view, "the session was recreated");
        wait_title(&mut client, fresh, PAGE_TWO_TITLE, since);
        assert!(has_id(&drawn(&mut client), "bbr-tab-close-0"));

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}
