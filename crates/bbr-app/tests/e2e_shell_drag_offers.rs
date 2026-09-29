//! Real-app drag-offer E2E (task packet `favorites-drag-offers`, DESIGN.md
//! §10.9, §11.2, §11.10) over real WKWebViews.
//!
//! Once a tab drag passes the 4 pt activation distance, the organized area
//! appends the "Pin tab here" ghost row (`bbr-pinned-drop-row`) for an
//! unpinned source, the Favorites region offers its band or drop cell
//! (`bbr-favorites-drop-cell`), and the lifted row follows the pointer as
//! the drag proxy (`bbr-tab-drag-proxy`). Exactly one target is live at a
//! time, a drop on the ghost row pins the exact dragged tab without touching
//! its session, a drop on the drop cell favorites it, and the growth the
//! offers need is committed once at activation — the region never reflows
//! while the pointer moves. Every drag goes through the production
//! request/handler path; synchronization is on typed browser changes and
//! snapshot state after flushed frames — no sleeps, no pixels.

#![cfg(target_os = "macos")]

mod e2e_common;
mod e2e_semantics_common;

use bbr_testkit::protocol::{dsl, BrowserChange, DragOffersSnapshot, Snapshot};
use e2e_common::*;
use e2e_semantics_common::find;

/// §8 normal-density row surface; the press anchors at its center.
const ROW_H: f32 = 36.0;
/// §8 normal-density row pitch (surface + gap) — also the ghost row's
/// layout height, so the growth above an unpinned row is one pitch.
const PITCH: f32 = 40.0;
/// §8 normal-density Favorites gap (the row gap an extra tile row adds).
const FAVORITES_GAP: f32 = 4.0;
const PRESS_X: f32 = 130.0;
const PRESS_Y: f32 = 400.0;
/// The activation move: past the 4 pt threshold.
const ACTIVATION_DY: f32 = 6.0;

fn flush(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

/// Press row `index`, cross the activation distance, flush the frame the
/// offers first render in, and return that snapshot.
fn activate_drag(client: &mut Client, index: usize) -> Snapshot {
    client
        .send_ok(&dsl::begin_tab_drag(index, PRESS_X, PRESS_Y, ROW_H / 2.0))
        .expect("begin drag");
    client
        .send_ok(&dsl::move_tab_drag(0.0, ACTIVATION_DY))
        .expect("activation move");
    flush(client);
    client.snapshot().expect("activated snapshot")
}

/// Move the pointer to window `(x, y)`, flush, and return the snapshot.
fn move_to(client: &mut Client, x: f32, y: f32) -> Snapshot {
    client
        .send_ok(&dsl::move_tab_drag(x - PRESS_X, y - PRESS_Y))
        .expect("move drag");
    flush(client);
    client.snapshot().expect("moved snapshot")
}

fn center_y(rect: [f32; 4]) -> f32 {
    rect[1] + rect[3] / 2.0
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

/// The measured Favorites region height equals the pure
/// `favorites_region_height` the drag growth derives from.
fn assert_region_height(snapshot: &Snapshot) {
    let rect = snapshot.favorites_rect.expect("favorites rect");
    assert!(
        (rect[3] - snapshot.favorites_region_height).abs() < 0.5,
        "rendered region {} vs favorites_region_height {}",
        rect[3],
        snapshot.favorites_region_height
    );
}

fn has(snapshot: &Snapshot, id: &str) -> bool {
    snapshot.visible_ids.iter().any(|visible| visible == id)
}

fn assert_no_offers(snapshot: &Snapshot) {
    assert_eq!(
        snapshot.drag_offers,
        DragOffersSnapshot::default(),
        "no offer renders without an active drag"
    );
    for id in [
        "bbr-pinned-drop-row",
        "bbr-tab-drag-proxy",
        "bbr-favorites-drop-cell",
    ] {
        assert!(!has(snapshot, id), "{id} renders only during a drag");
    }
}

/// The ghost row pins the dragged tab; the Favorites band and the ghost row
/// are offered together but only one is the live target; the proxy follows
/// the pointer; a pinned source is never offered pinning.
#[test]
#[ignore]
fn ghost_row_pins_the_dragged_tab_with_one_live_target() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("drag-offers-pin", &page_one);

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
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let dragged_view = open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));
        flush(&mut client);
        let rest = client.snapshot().expect("rest snapshot");
        assert_eq!(rest.pinned_count, 0);
        assert_eq!(rest.tabs.len(), 3);
        assert_no_offers(&rest);
        let sessions = rest.engine_sessions;
        let dragged_id = id_at(&rest, 2);
        assert_region_height(&rest);

        // Regression: releasing right after the activation move is a no-op.
        // The ghost row grows the rail one pitch above the source in the
        // next frame, so resolving the activation event against the
        // pre-growth geometry used to target the slot above and reorder.
        let micro = activate_drag(&mut client, 2);
        assert_eq!(micro.drag_slot, Some(2), "activation keeps the source slot");
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("release micro drag");
        flush(&mut client);
        let unmoved = client.snapshot().expect("micro-drag snapshot");
        assert_eq!(id_at(&unmoved, 2), dragged_id, "no reorder");
        assert_eq!(unmoved.pinned_count, 0, "no pin");
        assert_eq!(unmoved.engine_sessions, sessions);
        assert_no_offers(&unmoved);

        // Activation: the ghost row, the band and the proxy appear together
        // in one frame, and the growth above the unpinned row is exactly
        // the ghost row's one pitch.
        let active = activate_drag(&mut client, 2);
        assert_eq!(active.drag_source, Some(2));
        assert_ids(
            &active,
            &[
                "bbr-pinned-drop-row",
                "bbr-tab-drag-proxy",
                "bbr-favorites-empty",
            ],
        );
        assert_eq!(active.drag_offers.growth, Some(PITCH));
        let ghost = active
            .drag_offers
            .pin_offer_rect
            .expect("the ghost row records its rendered rect");
        assert_eq!(ghost[3], ROW_H, "the ghost row is one row surface tall");
        let band = active.favorites_rect.expect("band rect");
        assert!(
            band[1] + band[3] <= ghost[1],
            "the ghost row sits below the Favorites region"
        );
        assert_eq!(
            active.favorites_rect, rest.favorites_rect,
            "the zero-favorites band keeps its geometry during the drag"
        );
        assert!(!active.drag_offers.pin_offer_targeted);
        assert!(!active.drag_favorites);
        assert_region_height(&active);

        // The proxy keeps the press point under the pointer and follows it.
        let proxy = active.drag_offers.proxy_rect.expect("proxy rect");
        let pointer_y = PRESS_Y + ACTIVATION_DY;
        assert!(
            (proxy[1] - (pointer_y - ROW_H / 2.0)).abs() < 0.5,
            "proxy top {} follows the pointer {pointer_y}",
            proxy[1]
        );
        assert_eq!(proxy[3], ROW_H);

        // Pointer over the ghost row: it is the live target, the band stays
        // offered but quiet, and no slot tick renders.
        let over_ghost = move_to(&mut client, PRESS_X, center_y(ghost));
        assert!(over_ghost.drag_offers.pin_offer_targeted);
        assert!(!over_ghost.drag_favorites, "one live target at a time");
        assert_eq!(over_ghost.drag_slot, None);
        assert!(over_ghost.drag_valid, "pinning an unpinned tab is legal");
        assert!(
            has(&over_ghost, "bbr-favorites-empty"),
            "the band stays offered"
        );
        let moved_proxy = over_ghost.drag_offers.proxy_rect.expect("proxy rect");
        assert!(
            (moved_proxy[1] - (center_y(ghost) - ROW_H / 2.0)).abs() < 0.5,
            "the proxy follows the pointer"
        );
        assert_eq!(
            over_ghost.drag_offers.pin_offer_rect,
            Some(ghost),
            "the ghost row never moves while the pointer does"
        );

        // Pointer over the band: the band is live, the ghost row quiet.
        let over_band = move_to(&mut client, PRESS_X, center_y(band));
        assert!(over_band.drag_favorites);
        assert!(!over_band.drag_offers.pin_offer_targeted);

        // Back over the ghost row and drop: the exact dragged tab is pinned
        // at the end of the (empty) pinned block with its live session.
        move_to(&mut client, PRESS_X, center_y(ghost));
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("drop on ghost row");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        flush(&mut client);
        let pinned = client.snapshot().expect("pinned snapshot");
        assert_eq!(pinned.pinned_count, 1);
        assert_eq!(id_at(&pinned, 0), dragged_id, "the exact dragged tab");
        assert_eq!(pinned.tabs[0].web_view, Some(dragged_view));
        assert_eq!(pinned.engine_sessions, sessions, "no session recreated");
        assert!(pinned.favorites.is_empty(), "a pin never favorites");
        assert_no_offers(&pinned);

        // A pinned source is never offered pinning: no ghost row, and
        // nothing grows above it (it sits above every offer).
        let pinned_drag = activate_drag(&mut client, 0);
        assert_eq!(pinned_drag.drag_source, Some(0));
        assert!(!has(&pinned_drag, "bbr-pinned-drop-row"));
        assert_eq!(pinned_drag.drag_offers.pin_offer_rect, None);
        assert_eq!(pinned_drag.drag_offers.growth, Some(0.0));
        assert!(has(&pinned_drag, "bbr-tab-drag-proxy"));
        client.send_ok(&dsl::cancel_tab_drag()).expect("cancel");
        flush(&mut client);
        let cancelled = client.snapshot().expect("cancelled snapshot");
        assert_no_offers(&cancelled);
        assert_eq!(cancelled.pinned_count, 1, "cancel commits nothing");
        let _ = since;

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// A populated grid offers the drop cell in place of its rest slot; with a
/// full last row the cell opens one more row exactly once, at activation,
/// and the region never reflows while the pointer moves. A drop on it
/// favorites the exact dragged tab without a copy or a session change.
#[test]
#[ignore]
fn drop_cell_grows_once_at_activation_and_favorites_the_tab() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("drag-offers-cell", &page_one);

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
        // Four favorites fill the first grid row exactly.
        for index in 0..4 {
            client
                .send_ok(&dsl::toggle_favorite_tab(index))
                .expect("favorite a tab");
        }
        flush(&mut client);
        let rest = client.snapshot().expect("full-row snapshot");
        assert_eq!(rest.favorites.len(), 4);
        assert!(
            !has(&rest, "bbr-favorites-empty"),
            "a full row omits the rest slot"
        );
        assert_no_offers(&rest);
        let rest_band = rest.favorites_rect.expect("band rect at rest");
        assert_region_height(&rest);
        let sessions = rest.engine_sessions;
        let dragged_id = id_at(&rest, 4);

        // Activation commits the drop cell's extra row and the ghost row in
        // one frame.
        let active = activate_drag(&mut client, 4);
        assert_ids(&active, &["bbr-favorites-drop-cell", "bbr-pinned-drop-row"]);
        let tile_row = active.favorites_tile_size[1] + FAVORITES_GAP;
        assert_eq!(active.drag_offers.growth, Some(PITCH + tile_row));
        let grown = active.favorites_rect.expect("band rect during the drag");
        assert_region_height(&active);
        assert!(
            (grown[3] - (rest_band[3] + tile_row)).abs() < 0.5,
            "the region grows by one tile row: {} → {}",
            rest_band[3],
            grown[3]
        );

        // Pointer travel over the list, the ghost row and back never
        // reflows the region again.
        let ghost = active.drag_offers.pin_offer_rect.expect("ghost rect");
        for y in [PRESS_Y + 2.0 * PITCH, center_y(ghost), PRESS_Y + PITCH] {
            let moved = move_to(&mut client, PRESS_X, y);
            assert_eq!(moved.favorites_rect, Some(grown), "no reflow at y {y}");
            assert_eq!(moved.drag_offers.pin_offer_rect, Some(ghost));
        }

        // Over the grid the drop cell is the live target; drop it.
        let over_cell = move_to(&mut client, PRESS_X, center_y(grown));
        assert!(over_cell.drag_favorites);
        assert_eq!(
            over_cell.favorites_rect,
            Some(grown),
            "targeting the grid never outlines (and reflows) the region"
        );
        assert!(!over_cell.drag_offers.pin_offer_targeted);
        assert!(over_cell.drag_valid);
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("drop on the cell");
        flush(&mut client);
        let dropped = client.snapshot().expect("dropped snapshot");
        assert_eq!(dropped.favorites.len(), 5, "one new tile");
        assert_eq!(
            dropped.favorites[4].tab, dragged_id,
            "the exact dragged tab"
        );
        assert_eq!(dropped.tabs.len(), 5, "a drop never copies a tab");
        assert_eq!(dropped.engine_sessions, sessions, "no session change");
        assert_eq!(dropped.pinned_count, 0, "a favorite drop never pins");
        assert_no_offers(&dropped);
        // At rest again: five tiles wrap into two rows with the glyph slot
        // in the next free cell — the height the drag already reserved.
        assert!(has(&dropped, "bbr-favorites-empty"));
        assert_eq!(dropped.favorites_rect, Some(grown));
        assert_region_height(&dropped);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Blueprint D02 (§10.9, §11.11): a folder row is a drop target in two
/// dimensions — the row under the pointer takes the drag-target state, the
/// page beside the rail at the same height is no target, and a drop moves
/// the exact dragged tab into the folder (pinned, same live session).
#[test]
#[ignore]
fn a_tab_dropped_on_a_folder_row_joins_the_folder() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("drag-offers-folder", &page_one);

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
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));

        // An empty folder, committed with its default name.
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("New Folder");
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "enter".into(),
            })
            .expect("commit the name");
        flush(&mut client);
        let rest = client.snapshot().expect("rest snapshot");
        assert_eq!(rest.folders.len(), 1);
        let folder = rest.folders[0].id;
        assert!(rest.folders[0].members.is_empty());
        let sessions = rest.engine_sessions;
        let dragged = id_at(&rest, 2);

        let active = activate_drag(&mut client, 2);
        let row = active.folders[0]
            .rect
            .expect("the folder row records its rect");
        let (row_x, row_y) = (row[0] + row[2] / 2.0, center_y(row));

        // Over the folder row: the folder is the one live target.
        let over = move_to(&mut client, row_x, row_y);
        assert!(over.folders[0].drag_target, "the folder row is the target");
        assert!(over.drag_valid);
        assert_eq!(over.drag_slot, None);
        assert!(!over.drag_favorites);
        assert!(!over.drag_offers.pin_offer_targeted);
        let node = find(&over.semantics, "bbr-folder-row-0").expect("folder node");
        assert!(node.drag_target, "the semantic row carries drag-target");

        // Beside the rail at the same height: the page is no target.
        let beside = move_to(&mut client, over.sidebar_width + 40.0, row_y);
        assert!(!beside.folders[0].drag_target);
        assert_eq!(beside.drag_slot, None);
        assert!(!beside.drag_valid, "the page beside the rail drops nothing");

        // Back over the row and drop: the exact tab joins the folder.
        move_to(&mut client, row_x, row_y);
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("drop on the folder");
        flush(&mut client);
        let dropped = client.snapshot().expect("dropped snapshot");
        assert_eq!(dropped.folders[0].id, folder);
        assert_eq!(dropped.folders[0].members, vec![dragged]);
        assert!(!dropped.folders[0].drag_target, "the target clears on drop");
        assert_eq!(dropped.pinned_count, 1, "a folder member is pinned");
        assert_eq!(dropped.tabs.len(), 3, "a drop never copies a tab");
        assert_eq!(dropped.engine_sessions, sessions, "same live session");
        assert_no_offers(&dropped);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
