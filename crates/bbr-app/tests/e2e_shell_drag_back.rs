//! Dragging back out of the pinned area (task packet
//! `drag-back-out-of-pins`, DESIGN.md §10.9, §11.2, §11.10) over real
//! WKWebViews.
//!
//! A Favorite tile is draggable: its press activates the tile and arms a
//! drag of that tab. Dropped among the regular tabs it is unpinned at the
//! slot under the pointer (which ends the favorite); dropped on a loose
//! pinned row or the ghost row it stays pinned and leaves Favorites. Its
//! own grid is no target, it offers no drop cell, and a plain press stays
//! an activation. Every drag goes through the production press/move/release
//! handlers at the rects the rail drew; synchronization is on typed browser
//! changes and snapshots after flushed frames — no sleeps, no pixels.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{dsl, Snapshot};
use e2e_common::real_input::{center, rect_of};
use e2e_common::*;

/// §8 normal-density row pitch: the ghost row's height.
const PITCH: f32 = 40.0;
/// The activation move: past the 4 pt threshold.
const ACTIVATION_DY: f32 = 6.0;

fn drawn(client: &mut Client) -> Snapshot {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
    client.snapshot().expect("snapshot")
}

fn has(snapshot: &Snapshot, id: &str) -> bool {
    snapshot.visible_ids.iter().any(|visible| visible == id)
}

/// The persistent `TabId` of the active space's row `index`.
fn id_at(snapshot: &Snapshot, index: usize) -> u64 {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.space == snapshot.active_space && tab.index == index)
        .expect("active-space tab exists")
        .id
}

fn index_of(snapshot: &Snapshot, id: u64) -> usize {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.id == id)
        .expect("tab exists")
        .index
}

/// Press Favorite tile `index` where it is drawn and cross the activation
/// distance; returns the press point and the activated snapshot.
fn grab_tile(client: &mut Client, index: usize) -> ((f32, f32), Snapshot) {
    let rest = drawn(client);
    let tile = rect_of(&rest, &format!("bbr-fav-{index}")).expect("tile drawn");
    let (x, y) = center(tile);
    let press = (x as f32, y as f32);
    client
        .send_ok(&dsl::begin_favorite_drag(index, press.0, press.1))
        .expect("press tile");
    client
        .send_ok(&dsl::move_tab_drag(0.0, ACTIVATION_DY))
        .expect("activation move");
    (press, drawn(client))
}

/// Move the pointer to window y `y` (x stays at the press) and return the
/// drawn snapshot.
fn move_to_y(client: &mut Client, press: (f32, f32), y: f32) -> Snapshot {
    client
        .send_ok(&dsl::move_tab_drag(0.0, y - press.1))
        .expect("move drag");
    drawn(client)
}

#[test]
#[ignore]
fn a_favorite_dragged_back_unpins_or_stays_a_loose_pinned_row() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("drag-back-favorites", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first view");
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        for step in 0..4 {
            let (url, title) = if step % 2 == 0 {
                (&page_two, PAGE_TWO_TITLE)
            } else {
                (&page_one, PAGE_ONE_TITLE)
            };
            open_tab(&mut client, &mut since, url, Some(title));
        }
        // [t0 | t1 t2 t3 t4] → [t0, t1★, t2★ | t3, t4]: one loose pinned
        // row, two Favorites, two regular rows.
        client.send_ok(&dsl::pin_tab(0)).expect("pin t0");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        client
            .send_ok(&dsl::toggle_favorite_tab(1))
            .expect("favorite t1");
        client
            .send_ok(&dsl::toggle_favorite_tab(2))
            .expect("favorite t2");
        let rest = drawn(&mut client);
        assert_eq!(rest.pinned_count, 3);
        assert_eq!(rest.favorites.len(), 2);
        let [t0, t1, t2, t3, t4] = [0, 1, 2, 3, 4].map(|index| id_at(&rest, index));
        assert_eq!(rest.favorites[0].tab, t1);
        let sessions = rest.engine_sessions;
        let t1_view = rest.tabs[1].web_view;

        // A plain press on a tile is an activation, never a drag.
        let tile = rect_of(&rest, "bbr-fav-0").expect("tile drawn");
        let (x, y) = center(tile);
        client
            .send_ok(&dsl::begin_favorite_drag(0, x as f32, y as f32))
            .expect("press tile");
        client.send_ok(&dsl::end_tab_drag()).expect("release");
        let pressed = drawn(&mut client);
        assert_eq!(pressed.active_tab_index, Some(1), "the press selected t1");
        assert_eq!(pressed.drag_source, None);
        assert_eq!(pressed.favorites.len(), 2, "nothing moved");
        assert_eq!(pressed.pinned_count, 3);

        // Activation: the proxy and the ghost row, but no drop cell in the
        // tile's own grid.
        let (press, active) = grab_tile(&mut client, 0);
        assert!(active.drag_from_favorite);
        assert_eq!(active.drag_source, Some(1));
        assert!(has(&active, "bbr-tab-drag-proxy"));
        assert!(has(&active, "bbr-pinned-drop-row"));
        assert!(
            !has(&active, "bbr-favorites-drop-cell"),
            "a Favorite is not offered its own grid"
        );
        assert_eq!(active.drag_offers.growth, Some(PITCH));
        assert_eq!(active.drag_slot, None, "no slot before the pointer moves");
        assert!(!active.drag_valid);

        // Over its own tile: no target.
        let own = move_to_y(&mut client, press, press.1);
        assert!(!own.drag_valid);
        assert!(!own.drag_favorites);
        assert_eq!(own.drag_unpin_slot, None);

        // Over the upper quarter of the regular row t4: unpin before it.
        let t4_row = tab_row_rect(&mut client, 4);
        let over_t4 = move_to_y(&mut client, press, t4_row[1] + t4_row[3] / 4.0);
        assert_eq!(over_t4.drag_unpin_slot, Some(4));
        assert!(over_t4.drag_valid);
        assert!(has(&over_t4, "bbr-drag-tick-4"));
        client.send_ok(&dsl::end_tab_drag()).expect("drop");
        // [t0, t2★ | t3, t1, t4]
        since = wait_browser_change(&mut client, BrowserChange::TabUnpinned { index: 3 }, since);
        let unpinned = drawn(&mut client);
        assert_eq!(unpinned.pinned_count, 2);
        assert_eq!(
            [0, 1, 2, 3, 4].map(|index| id_at(&unpinned, index)),
            [t0, t2, t3, t1, t4]
        );
        assert!(!unpinned.tabs[3].pinned);
        assert_eq!(unpinned.tabs[3].web_view, t1_view, "the session stays");
        assert_eq!(unpinned.engine_sessions, sessions);
        assert_eq!(
            unpinned.favorites.iter().map(|f| f.tab).collect::<Vec<_>>(),
            [t2],
            "unpinning ended the favorite"
        );
        assert!(!has(&unpinned, "bbr-tab-drag-proxy"));

        // t2's tile onto the upper quarter of the loose pinned row t0: it
        // stays pinned, before t0, as a loose row — no longer a Favorite.
        let (press, _) = grab_tile(&mut client, 0);
        let t0_row = tab_row_rect(&mut client, 0);
        let over_t0 = move_to_y(&mut client, press, t0_row[1] + t0_row[3] / 4.0);
        assert_eq!(over_t0.drag_slot, Some(0));
        assert!(over_t0.drag_valid);
        assert!(has(&over_t0, "bbr-drag-tick-0"));
        client.send_ok(&dsl::end_tab_drag()).expect("drop on t0");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabReordered { from: 1, to: 0 },
            since,
        );
        let loose = wait_snapshot(&mut client, "t2 leaves Favorites", |snapshot| {
            snapshot.favorites.is_empty()
        });
        assert_eq!(loose.pinned_count, 2);
        assert_eq!(id_at(&loose, 0), t2);
        assert_eq!(id_at(&loose, 1), t0);
        assert!(
            loose.tab_row_rects.iter().any(|(row, _)| *row == 0),
            "t2 renders as a loose pinned row again"
        );
        assert_eq!(loose.engine_sessions, sessions);

        // Favorite t2 again and drop its tile on the ghost row: a loose
        // pinned row at the end of the pinned block.
        client
            .send_ok(&dsl::toggle_favorite_tab(0))
            .expect("favorite t2 again");
        let (press, active) = grab_tile(&mut client, 0);
        let ghost = active
            .drag_offers
            .pin_offer_rect
            .expect("the ghost row is offered to a Favorite");
        let over_ghost = move_to_y(&mut client, press, ghost[1] + ghost[3] / 2.0);
        assert!(over_ghost.drag_offers.pin_offer_targeted);
        client.send_ok(&dsl::end_tab_drag()).expect("drop on ghost");
        let ghosted = wait_snapshot(&mut client, "t2 leaves Favorites", |snapshot| {
            snapshot.favorites.is_empty() && snapshot.drag_source.is_none()
        });
        assert_eq!(ghosted.pinned_count, 2);
        assert_eq!(id_at(&ghosted, 1), t2, "the end of the pinned block");
        assert_eq!(index_of(&ghosted, t1), 3);

        // Escape cancels a tile drag: nothing changes.
        client
            .send_ok(&dsl::toggle_favorite_tab(1))
            .expect("favorite t2 once more");
        let (press, _) = grab_tile(&mut client, 0);
        let t3_row = tab_row_rect(&mut client, 2);
        move_to_y(&mut client, press, t3_row[1] + t3_row[3] / 4.0);
        client.send_ok(&dsl::cancel_tab_drag()).expect("cancel");
        let cancelled = drawn(&mut client);
        assert_eq!(cancelled.favorites.len(), 1);
        assert_eq!(cancelled.pinned_count, 2);
        assert_eq!(cancelled.drag_source, None);
        assert!(!cancelled.drag_from_favorite);
        let _ = since;

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Poll drawn snapshots until `ready` holds.
fn wait_snapshot(client: &mut Client, what: &str, ready: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = drawn(client);
        if ready(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "timed out waiting for {what}");
        std::thread::yield_now();
    }
}
