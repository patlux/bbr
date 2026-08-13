//! Real-app §11.10 favorites drag evidence over real WKWebViews: dragging a
//! concrete tab into the rendered favorites band targets the grid through
//! the typed during-drag state (`drag_favorites` + the band's recorded
//! `favorites_rect`), a drop toggles the exact pressed `TabId` through the
//! canonical Favorite command — one tile, no copy, no session recreation,
//! the tab stays open and selected — a repeated drop toggles it back off,
//! a pointer over the fixed chrome above the band is rejected whole, and
//! Escape cancels a targeted drag without committing. Every drag runs
//! through the production request/handler path
//! (`BeginTabDrag`/`MoveTabDrag`/`EndTabDrag`/`CancelTabDrag`) — never a
//! direct controller mutation.

use super::*;
use bbr_testkit::protocol::{dsl, BrowserChange, Event};

/// The §8 normal-density row surface the press-anchored drag geometry
/// anchors within (`row() / 2` — the press sits inside the 36 pt surface;
/// slot moves step on the §8 pitch, which the favorites-drag scenarios
/// express in absolute pointer deltas).
const ROW_H: f32 = 36.0;
/// The press anchor for every drag: window x 130 (the old seam's stable
/// anchor), window y 400 (inside the tab list, above the footer strip).
const PRESS_X: f32 = 130.0;
const PRESS_Y: f32 = 400.0;

/// Begin a drag on row `index` at the shared press anchor, cross the
/// drag-activation distance (a plain click never presents the drag — the
/// press arms and the presentation begins only after real pointer travel,
/// so a click never shifts the rail layout), flush a rendered frame (the
/// favorites band — empty grid during the drag — lays out and the
/// production band anchor records its rect), and return the during-drag
/// snapshot plus the recorded band rect.
/// `rest_band` declares whether the favorites region legitimately renders
/// at rest (a tile exists); only then may a band rect be recorded before
/// the drag activates.
fn begin_drag_and_band(
    client: &mut Client,
    index: usize,
    rest_band: bool,
) -> bbr_testkit::protocol::Snapshot {
    client
        .send_ok(&dsl::begin_tab_drag(index, PRESS_X, PRESS_Y, ROW_H / 2.0))
        .expect("begin drag");
    // Regression (layout jump on click): a plain press — no pointer travel
    // yet — presents no drag: no dragged source and, while no tile exists,
    // no favorites drop band, so a click never shifts the rail layout.
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame after the plain press");
    let armed = client.snapshot().expect("armed-press snapshot");
    assert_eq!(
        armed.drag_source, None,
        "a plain press arms without presenting a drag"
    );
    if !rest_band {
        assert!(
            armed.favorites_rect.is_none(),
            "no favorites drop band renders for a plain press (no layout shift)"
        );
    }
    // The activation move: past the 4 pt threshold, still far inside the
    // source row's own slot (no target change).
    client
        .send_ok(&dsl::move_tab_drag(0.0, 6.0))
        .expect("activation move");
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame after begin");
    let during = client.snapshot().expect("during-drag snapshot");
    assert_eq!(
        during.drag_source,
        Some(index),
        "the pressed row is dragged"
    );
    assert!(
        during.favorites_rect.is_some(),
        "the favorites band renders and records its rect during the drag"
    );
    during
}

/// Move the pointer to the exact center of the recorded favorites band.
fn move_into_band(client: &mut Client, band: [f32; 4]) {
    let target_x = band[0] + band[2] / 2.0;
    let target_y = band[1] + band[3] / 2.0;
    client
        .send_ok(&dsl::move_tab_drag(target_x - PRESS_X, target_y - PRESS_Y))
        .expect("move into the favorites band");
}

/// A tab drag into the empty favorites band creates one tile for the exact
/// pressed TabId through the canonical Favorite toggle — no copy, no
/// session recreation, the tab stays open — and a repeated drop toggles the
/// tile back off without closing the tab. Rejected chrome drops and Escape
/// leave state unchanged.
#[test]
#[ignore]
fn drag_tab_into_favorites_band_toggles_exact_tab_without_copy_or_session() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-u2-favorites-drag", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = initial.active_space;
        let default_profile = initial.active_profile;
        // Two warm tabs; the active tab is row 0, so the dragged row (1) is
        // explicitly not the active tab — the pressed identity is the
        // evidence target.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client.send_ok(&dsl::click_tab(0)).expect("select row 0");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        let two = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two.active_tab_index, Some(0));
        assert_eq!(two.engine_sessions, 2);
        assert!(two.favorites.is_empty(), "no favorites before the drag");
        assert!(!two.visible_ids.iter().any(|id| id == "bbr-favorites"));

        // Begin a drag on row 1 (the non-active second tab): the empty grid
        // renders as the compact drop band and the production band anchor
        // records its rect. The pointer has not entered it yet.
        let during = begin_drag_and_band(&mut client, 1, false);
        let band = during.favorites_rect.expect("band rect recorded");
        assert!(band[3] > 0.0, "the band has a positive height");
        assert!(
            !during.drag_favorites,
            "not targeted before entering the band"
        );
        // The band's ids mirror the render during the drag (§15).
        assert_ids(&during, &["bbr-section-favorites", "bbr-favorites"]);

        // Enter the band: the typed target flips to favorites.
        move_into_band(&mut client, band);
        let targeted = client.snapshot().expect("targeted snapshot");
        assert!(
            targeted.drag_favorites,
            "the drag targets the favorites grid"
        );
        assert_eq!(targeted.drag_slot, None, "no slot tick for the band");
        assert!(targeted.drag_valid, "the band is a legal drop");
        // The semantic tree reports the section as the drag target (§15).
        let section = e2e_semantics_common::find(&targeted.semantics, "bbr-section-favorites")
            .expect("favorites section in the tree");
        assert!(section.drag_target, "the section reports the drag target");

        // Drop: one tile for the exact pressed TabId, no copy, no session
        // recreation, the tab stays open and selected.
        let sessions_before = targeted.engine_sessions;
        let tabs_before = targeted.all_tabs.len();
        client.send_ok(&dsl::end_tab_drag()).expect("end drag");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after drop");
        let dropped = client.snapshot().expect("dropped snapshot");
        assert_eq!(dropped.favorites.len(), 1, "one tile after the drop");
        let tile = &dropped.favorites[0];
        assert_eq!(tile.index, 0, "insertion order index");
        let pressed = dropped
            .all_tabs
            .iter()
            .find(|tab| tab.space == default_space && tab.index == 1)
            .expect("pressed row in the global model");
        assert_eq!(tile.tab, pressed.id, "the exact pressed TabId");
        assert_eq!(tile.space, default_space.0, "canonical owning space");
        assert_eq!(tile.profile, default_profile.0, "profile from the space");
        assert!(!tile.letter.is_empty(), "§7.2 monogram fallback letter");
        assert_eq!(
            dropped.all_tabs.len(),
            tabs_before,
            "a drop never copies or moves a tab"
        );
        assert_eq!(
            dropped.engine_sessions, sessions_before,
            "a drop never recreates a session"
        );
        assert_eq!(dropped.active_tab_index, Some(0), "selection untouched");
        assert_eq!(
            dropped.active_web_view,
            Some(first_view),
            "the selected session is untouched"
        );
        assert_eq!(dropped.tabs.len(), 2, "both tabs still open");
        assert_eq!(dropped.tabs[1].web_view, Some(second_view));
        assert_ids(
            &dropped,
            &["bbr-section-favorites", "bbr-favorites", "bbr-fav-0"],
        );

        // Repeated drop on the same tab: the canonical Favorite command
        // toggles the tile back off; the tab stays open with its session.
        let during_again = begin_drag_and_band(&mut client, 1, true);
        let band_again = during_again.favorites_rect.expect("band rect recorded");
        move_into_band(&mut client, band_again);
        let targeted_again = client.snapshot().expect("second targeted snapshot");
        assert!(targeted_again.drag_favorites, "targeted again");
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("end second drag");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after second drop");
        let toggled_off = client.snapshot().expect("toggled-off snapshot");
        assert!(
            toggled_off.favorites.is_empty(),
            "repeated drop toggles the tile off"
        );
        assert_eq!(toggled_off.tabs.len(), 2, "the tab is never closed");
        assert_eq!(
            toggled_off.tabs[1].web_view,
            Some(second_view),
            "the tab's live session is untouched"
        );
        assert_eq!(toggled_off.all_tabs.len(), tabs_before, "no copy or move");
        assert_eq!(
            toggled_off.engine_sessions, sessions_before,
            "no session recreation"
        );

        // Rejected target: a pointer over the fixed chrome above the band
        // (the command field / brand row) is no target — the drop commits
        // nothing.
        let during_rej = begin_drag_and_band(&mut client, 1, false);
        let band_rej = during_rej.favorites_rect.expect("band rect recorded");
        let chrome_y = band_rej[1] - 12.0;
        assert!(
            chrome_y >= 0.0,
            "chrome above the band is inside the window"
        );
        client
            .send_ok(&dsl::move_tab_drag(PRESS_X - PRESS_X, chrome_y - PRESS_Y))
            .expect("move above the band");
        let rejected = client.snapshot().expect("rejected snapshot");
        assert!(
            !rejected.drag_favorites,
            "chrome above the band is no target"
        );
        assert!(!rejected.drag_valid, "the drop is rejected whole");
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("end rejected drag");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after rejected drop");
        let after_rejected = client.snapshot().expect("after-rejected snapshot");
        assert!(
            after_rejected.favorites.is_empty(),
            "a rejected drop commits nothing"
        );
        assert_eq!(after_rejected.tabs.len(), 2, "no tab change");
        assert_eq!(
            after_rejected.engine_sessions, sessions_before,
            "no session change"
        );

        // Escape cancels a targeted favorites drag without committing.
        let during_cancel = begin_drag_and_band(&mut client, 1, false);
        let band_cancel = during_cancel.favorites_rect.expect("band rect recorded");
        move_into_band(&mut client, band_cancel);
        let targeted_cancel = client.snapshot().expect("cancel-targeted snapshot");
        assert!(targeted_cancel.drag_favorites, "targeted before cancel");
        client
            .send_ok(&dsl::cancel_tab_drag())
            .expect("cancel the drag");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after cancel");
        let cancelled = client.snapshot().expect("cancelled snapshot");
        assert_eq!(cancelled.drag_source, None, "the drag is cleared");
        assert!(!cancelled.drag_favorites, "no target after cancel");
        assert!(
            cancelled.favorites.is_empty(),
            "Escape never commits a favorite"
        );
        assert_eq!(cancelled.tabs.len(), 2, "no tab change");
        assert_eq!(
            cancelled.engine_sessions, sessions_before,
            "no session change"
        );

        // No spurious organize event left the app across the whole scenario.
        let replay = client
            .attach(Some(since))
            .expect("replay after the scenario");
        for (_, event) in replay {
            if let Event::BrowserChanged {
                kind: BrowserChange::TabMoved { .. } | BrowserChange::TabReordered { .. },
            } = event
            {
                panic!("no move/reorder may leave the app: {event:?}");
            }
        }

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
