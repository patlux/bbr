//! Favorite-menu scenarios (task packet `tab-favorite-context-menus`).
//!
//! A Favorite is another presentation of the same concrete tab: the
//! Favorite menu (source key `bbr-favorite-menu`) resolves and mutates the
//! exact same `TabId` as the tab-row presentation. Opening the menu on a
//! tile never activates it, and the Favorite toggle row follows the same
//! shared availability — `Remove from Favorites` when the tile is shown,
//! `Add to Favorites` from the tab row when it is not.

use crate::e2e_common::*;
use crate::support::*;

/// The Favorite tile and the tab row consume one shared command authority:
/// right-clicking the tile opens the menu bound to the tile's exact `TabId`
/// without activating it; `Remove from Favorites` removes the favorite
/// while the source tab/session/selection stay untouched; re-adding from
/// the tab-row menu resolves the same concrete `TabId`.
#[test]
#[ignore]
fn favorite_menu_resolves_and_mutates_the_exact_tab_id() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-favorite", &page_one);

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
        let source_tab = active_space_tab_id(&before, 0);

        // Unmeasured setup: add the hidden source (index 0) to the
        // favorites grid through the production toggle path. The tile
        // resolves the exact same TabId.
        client
            .send_ok(&bbr_testkit::Request::ToggleFavoriteTab { index: 0 })
            .expect("toggle favorite");
        flush_frame(&mut client);
        let favorited = client.snapshot().expect("favorited snapshot");
        assert_eq!(
            favorited.favorites.first().map(|tile| tile.tab),
            Some(source_tab),
            "the tile resolves the source TabId"
        );

        // Right-click the tile: the menu opens under the Favorite source
        // key bound to the exact same TabId, and the tile is never
        // activated (the selected neighbor stays presented).
        right_click_favorite(&mut client, 0, 200.0, 300.0);
        let open = wait_menu_open(&mut client, source_tab, MENU_SOURCE_FAVORITE, deadline());
        assert_eq!(
            open.active_tab_index,
            Some(1),
            "opening on the tile never activates it"
        );
        assert_eq!(open.active_web_view, Some(neighbor_view));
        let favorite_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-favorite")
            .expect("favorite row");
        assert_eq!(
            open.tab_menu.rows[favorite_row].label, "Remove from Favorites",
            "the favorite menu follows the shared availability"
        );

        // Remove through the menu: the favorite is gone, the source tab
        // (same id, same space, same session) and the selection survive.
        menu_activate(&mut client, favorite_row);
        let removed = wait_menu_closed(&mut client, deadline());
        assert!(removed.favorites.is_empty(), "the favorite was removed");
        let source_now = removed
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .expect("the source tab survives");
        assert_eq!(source_now.space, before.active_space);
        assert_eq!(
            source_now.web_view,
            before
                .all_tabs
                .iter()
                .find(|tab| tab.id == source_tab)
                .and_then(|tab| tab.web_view),
            "the favorite toggle never touches the session"
        );
        assert_eq!(
            removed.active_tab_index,
            Some(1),
            "the neighbor stays selected"
        );

        // Re-add from the tab-row presentation: the menu now reads
        // `Add to Favorites`, and the new tile resolves the SAME concrete
        // TabId the tab row bound.
        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let favorite_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-favorite")
            .expect("favorite row");
        assert_eq!(open.tab_menu.rows[favorite_row].label, "Add to Favorites");
        menu_activate(&mut client, favorite_row);
        let re_added = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            re_added.favorites.first().map(|tile| tile.tab),
            Some(source_tab),
            "both presentations resolve the exact same concrete TabId"
        );
        assert_eq!(re_added.active_tab_index, Some(1));
    }));

    finish(result, &mut harness, &socket, &token);
}
