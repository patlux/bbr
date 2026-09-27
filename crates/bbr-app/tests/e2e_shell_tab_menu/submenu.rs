//! Folder/space submenu scenarios (task packet `tab-favorite-context-menus`).
//!
//! Proves the submenu contract end to end: keyboard traversal preserves the
//! exact source `TabId` across Right/Left entry and exit, a committed
//! `Move to Folder` routes through the organization authority keyed by the
//! source id, and a committed `Move to Space` follows the shared move
//! authority — same-profile destinations keep the live `WebViewId`,
//! cross-profile destinations recreate the session exactly once in the
//! destination profile's isolated store. The active space and the selected
//! neighbor never change, and the complete snapshot identity stays stable
//! except for the one committed move.

use crate::e2e_common::*;
use crate::support::*;

/// The `Move to Folder` submenu is only built when the source space has
/// folders. Create one through the production plus-menu path (the footer
/// `[+]` → New Folder commit), then open the menu on an inactive tab and
/// prove the folder submenu's keyboard traversal preserves the source
/// `TabId`, commits through the id-keyed organization command, and leaves
/// the selection/session untouched.
#[test]
#[ignore]
fn folder_submenu_keyboard_traversal_preserves_source_and_commits() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-folder", &page_one);

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

        // Unmeasured setup: one folder in the source space through the
        // production plus-menu commit (footer `[+]` → New Folder).
        client
            .send_ok(&bbr_testkit::Request::ClickFooterPlus)
            .expect("open plus menu");
        client
            .send_ok(&bbr_testkit::Request::ClickPlusMenuItem { index: 1 })
            .expect("commit New Folder");
        wait_for_folder_count(&mut client, 1, deadline());

        let before = client.snapshot().expect("before snapshot");
        assert_eq!(before.active_tab_index, Some(1), "the neighbor is selected");
        let source_tab = active_space_tab_id(&before, 0);
        let before_identity = identity(&before);

        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let folder_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-move-folder")
            .expect("the folder submenu parent row");
        assert_eq!(
            open.tab_menu.selected,
            Some(folder_row),
            "first enabled row"
        );

        // Right opens the folder submenu; the source TabId never changes.
        key(&mut client, "right");
        flush_frame(&mut client);
        let sub = client.snapshot().expect("folder submenu snapshot");
        assert_eq!(sub.tab_menu.open_submenu, Some(folder_row));
        assert_eq!(sub.tab_menu.submenu_selected, Some(0));
        assert_eq!(
            sub.tab_menu.bound_tab,
            Some(source_tab),
            "submenu traversal preserves the source TabId"
        );
        assert_eq!(
            identity(&sub),
            before_identity,
            "no mutation on submenu open"
        );
        let folder_row_ids: Vec<&str> = sub
            .tab_menu
            .rows
            .iter()
            .flat_map(|row| row.submenu.iter())
            .map(|s| s.id.as_str())
            .collect();
        assert_eq!(folder_row_ids, vec!["bbr-tab-menu-folder-0"]);

        // Left closes the submenu; Right reopens it; the source survives.
        key(&mut client, "left");
        flush_frame(&mut client);
        let back = client.snapshot().expect("back snapshot");
        assert_eq!(back.tab_menu.open_submenu, None);
        key(&mut client, "right");
        flush_frame(&mut client);
        let reopen = client.snapshot().expect("reopen submenu snapshot");
        assert_eq!(reopen.tab_menu.open_submenu, Some(folder_row));
        assert_eq!(reopen.tab_menu.bound_tab, Some(source_tab));

        // Enter commits the folder move through the id-keyed organization
        // authority: the source joins the folder, its tab identity/space/
        // order/session stay untouched, and the neighbor stays selected.
        key(&mut client, "enter");
        let committed = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            committed.folders[0].tab_count, 1,
            "the source tab joined the folder"
        );
        let moved = committed
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .expect("the source tab survives");
        assert_eq!(
            moved.space, before.active_space,
            "space membership unchanged"
        );
        assert_eq!(moved.index, 0, "order unchanged");
        assert!(
            moved.pinned,
            "a folder is the grouped form of pinned tabs: grouping pins"
        );
        assert_eq!(committed.folders[0].members, vec![source_tab]);
        assert_eq!(
            moved.web_view,
            before
                .all_tabs
                .iter()
                .find(|tab| tab.id == source_tab)
                .and_then(|tab| tab.web_view),
            "the folder move never touches the live session"
        );
        assert_eq!(
            committed.active_tab_index,
            Some(1),
            "the neighbor stays selected"
        );
        assert_eq!(committed.active_space, before.active_space);
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Enter or a pointer press on a **closed** submenu parent opens its
/// submenu card and never dispatches a command — the parent row carries no
/// action (no placeholder Reload). The bound source TabId, the snapshot
/// identity, and the live sessions stay byte-identical.
#[test]
#[ignore]
fn enter_and_click_on_a_closed_submenu_parent_open_it_without_dispatching() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-parent-open", &page_one);

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
        create_second_space(&mut client, deadline());

        let before = client.snapshot().expect("before snapshot");
        let source_tab = active_space_tab_id(&before, 0);
        let before_identity = identity(&before);
        let view_before = before
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .and_then(|tab| tab.web_view)
            .expect("the source owns a live session");

        // Enter on the closed Move to Space parent: the submenu opens, the
        // menu stays open, the source/identity/session never change, and no
        // Reload/action is dispatched (a dispatched command would dismiss
        // the menu or mutate the session). The cursor lands on the parent as
        // the first enabled row — no hover opens it first.
        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let space_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-move-space")
            .expect("move-space row");
        assert_eq!(
            open.tab_menu.selected,
            Some(space_row),
            "the closed submenu parent is the first enabled row"
        );
        assert_eq!(open.tab_menu.open_submenu, None, "submenu starts closed");
        key(&mut client, "enter");
        flush_frame(&mut client);
        let opened = client.snapshot().expect("enter-opened submenu snapshot");
        assert_eq!(
            opened.tab_menu.open_submenu,
            Some(space_row),
            "Enter opens the closed submenu parent"
        );
        assert_eq!(opened.tab_menu.submenu_selected, Some(0));
        assert!(
            opened.tab_menu.open,
            "the menu stays open after opening the submenu"
        );
        assert_eq!(opened.tab_menu.bound_tab, Some(source_tab));
        assert_eq!(identity(&opened), before_identity, "no mutation");
        let source_now = opened
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .and_then(|tab| tab.web_view);
        assert_eq!(
            source_now,
            Some(view_before),
            "no session churn on parent activation"
        );
        assert!(
            opened
                .visible_ids
                .iter()
                .any(|id| id == "bbr-tab-menu-submenu"),
            "the destination card id is visible after Enter"
        );
        key(&mut client, "escape");
        let closed = wait_menu_closed(&mut client, deadline());
        assert_eq!(identity(&closed), before_identity);

        // A pointer press on the closed parent does the same: the submenu
        // opens, nothing is dispatched, nothing mutates. The press lands
        // directly on the parent (no hover opens it first).
        let _open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        menu_activate(&mut client, space_row);
        flush_frame(&mut client);
        let clicked = client.snapshot().expect("click-opened submenu snapshot");
        assert_eq!(
            clicked.tab_menu.open_submenu,
            Some(space_row),
            "a press on the closed parent opens the submenu"
        );
        assert!(clicked.tab_menu.open, "the menu stays open");
        assert_eq!(identity(&clicked), before_identity, "no mutation");
        assert_eq!(
            clicked
                .all_tabs
                .iter()
                .find(|tab| tab.id == source_tab)
                .and_then(|tab| tab.web_view),
            Some(view_before),
            "no session churn on parent click"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}

/// A same-profile `Move to Space` through the keyboard path keeps the live
/// `WebViewId`: the submenu row commits the exact bound source into the
/// destination space of the same profile, never the selected neighbor, and
/// the active space/selection stay put.
#[test]
#[ignore]
fn space_submenu_same_profile_move_keeps_the_live_web_view() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-space-same", &page_one);

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
        create_second_space(&mut client, deadline());

        let before = client.snapshot().expect("before snapshot");
        assert_eq!(before.active_tab_index, Some(1));
        let source_tab = active_space_tab_id(&before, 0);
        let view_before = before
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .and_then(|tab| tab.web_view)
            .expect("the source owns a live session");
        let sessions_before = before.engine_sessions;

        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let space_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-move-space")
            .expect("move-space row");
        menu_select(&mut client, space_row);
        key(&mut client, "right");
        flush_frame(&mut client);
        let sub = client.snapshot().expect("space submenu snapshot");
        assert_eq!(sub.tab_menu.open_submenu, Some(space_row));
        assert_eq!(sub.tab_menu.bound_tab, Some(source_tab));
        let destination = &sub.tab_menu.rows[space_row].submenu[0];
        assert_eq!(destination.id, "bbr-tab-menu-space-0");
        assert!(
            !destination.label.contains('·'),
            "same-profile destination carries no profile identity"
        );

        // Enter commits the same-profile move: the live WebViewId is
        // preserved, the source now lives in the second space, the active
        // space and the selected neighbor never change, and the session
        // population is untouched.
        key(&mut client, "enter");
        let committed = wait_menu_closed(&mut client, deadline());
        let second_space = committed
            .spaces
            .iter()
            .find(|space| space.title == "Second")
            .expect("second space")
            .space;
        let moved = committed
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .expect("the source survives");
        assert_eq!(moved.space, second_space, "the source moved to Second");
        assert_eq!(
            moved.web_view,
            Some(view_before),
            "same-profile move keeps the live WebViewId"
        );
        assert_eq!(
            committed.active_space, before.active_space,
            "active space unchanged"
        );
        assert_eq!(
            committed.active_tab_index,
            Some(0),
            "the neighbor stays selected (its index shifted to 0 when the
             source left the space)"
        );
        assert_eq!(
            committed.engine_sessions, sessions_before,
            "no session churn"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}

/// A cross-profile `Move to Space` recreates the session exactly once in
/// the destination profile's isolated store. The destination submenu row
/// carries the visible profile identity (DESIGN.md §11.3), the old session
/// is destroyed, exactly one fresh session exists for the moved tab, and
/// the active space/selection never change.
#[test]
#[ignore]
fn space_submenu_cross_profile_move_recreates_once_and_preserves_isolation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-space-cross", &page_one);

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

        // Unmeasured setup: an isolated profile with one initial space.
        client
            .send_ok(&bbr_testkit::Request::CreateProfile {
                profile_title: "Private".into(),
                space_title: "Private Space".into(),
            })
            .expect("create profile");
        wait_for_space_count(&mut client, 2, deadline());

        let before = client.snapshot().expect("before snapshot");
        assert_eq!(before.active_tab_index, Some(1));
        let source_tab = active_space_tab_id(&before, 0);
        let view_before = before
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .and_then(|tab| tab.web_view)
            .expect("the source owns a live session");
        let sessions_before = before.engine_sessions;

        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        let space_row = open
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-move-space")
            .expect("move-space row");
        menu_select(&mut client, space_row);
        key(&mut client, "right");
        flush_frame(&mut client);
        let sub = client.snapshot().expect("space submenu snapshot");
        let destination = &sub.tab_menu.rows[space_row].submenu[0];
        assert_eq!(destination.id, "bbr-tab-menu-space-0");
        assert!(
            destination.label.contains("Private Space") && destination.label.contains("Private"),
            "cross-profile destination carries the visible profile identity (got {:?})",
            destination.label
        );

        key(&mut client, "enter");
        let committed = wait_menu_closed(&mut client, deadline());
        let private_space = committed
            .spaces
            .iter()
            .find(|space| space.title == "Private Space")
            .expect("private space");
        let moved = committed
            .all_tabs
            .iter()
            .find(|tab| tab.id == source_tab)
            .expect("the source survives");
        assert_eq!(moved.space, private_space.space, "the source moved across");
        assert_eq!(
            moved.profile, private_space.profile.0,
            "the moved tab lives in the destination profile's store"
        );
        assert_ne!(
            moved.web_view,
            Some(view_before),
            "cross-profile move recreates the session"
        );
        assert_eq!(
            committed.engine_sessions, sessions_before,
            "exactly one fresh session: the old is destroyed, the new replaces it"
        );
        assert_eq!(
            committed.active_space, before.active_space,
            "active space unchanged"
        );
        assert_eq!(
            committed.active_tab_index,
            Some(0),
            "the neighbor stays selected (its index shifted to 0 when the
             source left the space)"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}
