//! Real-app folder-membership invariant E2E (task packet
//! `folder-pinned-invariant`): a folder is the grouped form of pinned tabs.
//!
//! Grouping through the production Tab-menu paths (pointer submenu commit
//! and keyboard submenu commit) pins a regular tab in the same transition
//! and reports the pin as an ordinary `TabPinned` browser change; ungrouping
//! leaves a loose pinned tab; unpinning a member through the keyboard
//! command (`Cmd-Shift-P`) or the Tab menu's pointer path ends its membership;
//! rejected groupings (unknown folder, a tab of another space) change
//! neither the pin nor the membership. Every step keeps the exact `TabId`,
//! the live `WebViewId`, the selection, and the owning space/profile.
//! Synchronization is on typed browser changes and snapshot state under a
//! deadline — no sleeps.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{Event, GlobalTabSnapshot, Snapshot};
use e2e_common::*;

const MENU_SOURCE_TAB: &str = "bbr-tab-menu";

fn flush_frame(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

fn tab(snapshot: &Snapshot, id: u64) -> &GlobalTabSnapshot {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.id == id)
        .expect("tab identity survives")
}

fn id_at(snapshot: &Snapshot, index: usize) -> u64 {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.space == snapshot.active_space && tab.index == index)
        .expect("active-space tab exists")
        .id
}

/// Poll until the menu is open on `tab` and return the open snapshot.
fn wait_menu_open(client: &mut Client, tab: u64) -> Snapshot {
    let deadline = deadline();
    loop {
        let snapshot = client.snapshot().expect("menu snapshot");
        if snapshot.tab_menu.open
            && snapshot.tab_menu.bound_tab == Some(tab)
            && snapshot.tab_menu.source_key == MENU_SOURCE_TAB
        {
            return snapshot;
        }
        assert!(Instant::now() < deadline, "tab menu never opened on {tab}");
        std::thread::yield_now();
    }
}

fn wait_menu_closed(client: &mut Client) {
    let deadline = deadline();
    while client.snapshot().expect("menu snapshot").tab_menu.open {
        assert!(Instant::now() < deadline, "tab menu never closed");
        std::thread::yield_now();
    }
}

/// Right-click the row at `index` (production pointer path) and return the
/// position of the `Move to Folder` parent row.
fn open_folder_menu(client: &mut Client, index: usize, tab: u64) -> usize {
    client
        .send_ok(&bbr_testkit::Request::RightClickRow {
            index,
            x: 140.0,
            y: 240.0,
        })
        .expect("right click row");
    flush_frame(client);
    wait_menu_open(client, tab)
        .tab_menu
        .rows
        .iter()
        .position(|row| row.id == "bbr-tab-menu-move-folder")
        .expect("the folder submenu parent row")
}

/// Folder members, pins, order, selection, and sessions after a grouping
/// change: `members` are the folder's exact identities; `order` the active
/// space's tab ids; `pinned` its pinned boundary.
fn assert_organized(snapshot: &Snapshot, members: &[u64], order: &[u64], pinned: usize) {
    assert_eq!(snapshot.folders[0].members, members, "exact folder members");
    assert_eq!(snapshot.folders[0].tab_count, members.len());
    let got: Vec<u64> = (0..snapshot.tabs.len())
        .map(|index| id_at(snapshot, index))
        .collect();
    assert_eq!(got, order, "active-space tab order");
    assert_eq!(snapshot.pinned_count, pinned, "pinned boundary");
    for member in members {
        assert!(tab(snapshot, *member).pinned, "every member is pinned");
    }
}

#[test]
#[ignore]
fn folder_membership_pins_tabs_through_the_production_paths() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, page_two) = fixtures();
    let page_three = fixtures.url("/no-favicon.html");
    let (mut harness, socket, token) = spawn_harness("folder-pins", &page_one);

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
        open_tab(&mut client, &mut since, &page_three, None);
        let base = client.snapshot().expect("three-tab snapshot");
        let (a, b, c) = (id_at(&base, 0), id_at(&base, 1), id_at(&base, 2));
        let views: Vec<_> = [a, b, c]
            .iter()
            .map(|id| tab(&base, *id).web_view)
            .collect();
        let profile = base.active_profile;
        assert_eq!(base.active_tab_index, Some(2));

        // A folder through the production plus-menu commit.
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("commit New Folder");
        flush_frame(&mut client);
        let created = client.snapshot().expect("folder snapshot");
        assert_eq!(created.folders.len(), 1);
        let folder = created.folders[0].id;

        // Pointer grouping: the submenu parent click opens the submenu and
        // the row press commits. The regular tab is pinned in the same
        // transition, reported as an ordinary pin.
        let folder_row = open_folder_menu(&mut client, 0, a);
        client
            .send_ok(&bbr_testkit::Request::TabMenuActivate { index: folder_row })
            .expect("open folder submenu");
        flush_frame(&mut client);
        client
            .send_ok(&bbr_testkit::Request::TabMenuSubmenuActivate { index: 0 })
            .expect("commit folder row");
        wait_menu_closed(&mut client);
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        flush_frame(&mut client);
        let pointer = client.snapshot().expect("pointer grouping snapshot");
        assert_organized(&pointer, &[a], &[a, b, c], 1);
        assert_eq!(pointer.active_tab_index, Some(2), "selection unchanged");
        assert_eq!(tab(&pointer, a).web_view, views[0], "same live session");
        assert_eq!(pointer.engine_sessions, 3, "no session churn");
        assert!(pointer.chrome_focused, "focus returns to the chrome domain");

        // Keyboard grouping: Right opens the submenu, Enter commits.
        open_folder_menu(&mut client, 1, b);
        key(&mut client, "right");
        flush_frame(&mut client);
        key(&mut client, "enter");
        wait_menu_closed(&mut client);
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 1 }, since);
        flush_frame(&mut client);
        let keyboard = client.snapshot().expect("keyboard grouping snapshot");
        assert_organized(&keyboard, &[a, b], &[a, b, c], 2);
        assert_eq!(tab(&keyboard, b).web_view, views[1]);
        assert!(
            keyboard.chrome_focused,
            "focus returns to the chrome domain"
        );

        // Ungroup: the member stays a loose pinned tab.
        client
            .send_ok(&dsl::remove_tab_from_folder(1, folder))
            .expect("ungroup");
        flush_frame(&mut client);
        let ungrouped = client.snapshot().expect("ungrouped snapshot");
        assert_organized(&ungrouped, &[a], &[a, b, c], 2);
        assert!(tab(&ungrouped, b).pinned, "ungroup keeps the pin");

        // Keyboard unpin of a member (`Cmd-Shift-P` on the selected member)
        // ends its membership; the tab moves to the regular region's end.
        client.send_ok(&dsl::click_tab(0)).expect("select member");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        key(&mut client, "cmd-shift-p");
        since = wait_browser_change(&mut client, BrowserChange::TabUnpinned { index: 2 }, since);
        flush_frame(&mut client);
        let unpinned = client.snapshot().expect("keyboard unpin snapshot");
        assert_organized(&unpinned, &[], &[b, c, a], 1);
        assert!(!tab(&unpinned, a).pinned, "the unpin stands");
        assert_eq!(
            unpinned.active_web_view, views[0],
            "the member stays selected"
        );

        // Re-group, then unpin through the Tab menu's pointer path.
        client
            .send_ok(&dsl::move_tab_into_folder(2, folder))
            .expect("re-group");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 1 }, since);
        flush_frame(&mut client);
        assert_organized(
            &client.snapshot().expect("regrouped snapshot"),
            &[a],
            &[b, a, c],
            2,
        );
        toggle_pin_via_tab_menu(&mut client, 1);
        since = wait_browser_change(&mut client, BrowserChange::TabUnpinned { index: 2 }, since);
        flush_frame(&mut client);
        let pointer_unpin = client.snapshot().expect("pointer unpin snapshot");
        assert_organized(&pointer_unpin, &[], &[b, c, a], 1);

        // Illegal boundaries are rejected whole: an unknown folder pins
        // nothing.
        client
            .send_ok(&dsl::move_tab_into_folder(2, 9_999))
            .expect("unknown folder");
        flush_frame(&mut client);
        let rejected = client.snapshot().expect("unknown folder snapshot");
        assert_organized(&rejected, &[], &[b, c, a], 1);

        // A tab of another space — in the same profile or in an isolated
        // profile — cannot join this space's folder and is not pinned by the
        // attempt.
        client
            .send_ok(&dsl::create_space(profile, "Second"))
            .expect("create second space");
        let (next, change) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = next;
        let BrowserChange::SpaceCreated { space: second, .. } = change else {
            unreachable!("matched SpaceCreated");
        };
        client
            .send_ok(&bbr_testkit::Request::CreateProfile {
                profile_title: "Private".into(),
                space_title: "Private Space".into(),
            })
            .expect("create isolated profile");
        let (next, change) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = next;
        let BrowserChange::ProfileCreated { space: private, .. } = change else {
            unreachable!("matched ProfileCreated");
        };
        let mut foreign = Vec::new();
        for space in [second, private] {
            client
                .send_ok(&dsl::activate_space(space))
                .expect("activate foreign space");
            since =
                wait_browser_change(&mut client, BrowserChange::SpaceActivated { space }, since);
            open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
            client
                .send_ok(&dsl::move_tab_into_folder(0, folder))
                .expect("foreign grouping attempt");
            flush_frame(&mut client);
            let attempt = client.snapshot().expect("foreign attempt snapshot");
            assert!(attempt.folders[0].members.is_empty(), "rejected whole");
            let id = id_at(&attempt, 0);
            assert!(!tab(&attempt, id).pinned, "a rejected grouping never pins");
            assert_eq!(tab(&attempt, id).space, space);
            foreign.push(id);
        }
        let cross = client.snapshot().expect("cross-boundary snapshot");
        assert_ne!(
            tab(&cross, foreign[1]).profile,
            profile.0,
            "the isolated profile's tab keeps its own profile"
        );

        // Identity, sessions, and profile/space ownership survived every
        // transition.
        for (id, view) in [a, b, c].iter().zip(&views) {
            let state = tab(&cross, *id);
            assert_eq!(state.web_view, *view, "exact live session for {id}");
            assert_eq!(state.profile, profile.0, "profile unchanged for {id}");
            assert_eq!(state.space, base.active_space, "space unchanged for {id}");
        }
        assert_eq!(cross.engine_sessions, 5, "one session per tab, no churn");

        // Grouping a favorite whose tab lives in another space (the
        // Favorite menu groups into the tab's own space) pins that tab, but
        // `TabPinned` carries an active-space index: the active tab list
        // does not change, so no pin may be reported for it.
        let default_space = base.active_space;
        client
            .send_ok(&dsl::activate_space(default_space))
            .expect("back to the default space");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let home = client.snapshot().expect("default space snapshot");
        let c_index = home
            .tabs
            .iter()
            .position(|row| row.web_view == views[2])
            .expect("c is in the default space");
        client
            .send_ok(&dsl::toggle_favorite_tab(c_index))
            .expect("favorite c");
        client
            .send_ok(&dsl::activate_space(second))
            .expect("activate the second space");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: second },
            since,
        );
        flush_frame(&mut client);
        client
            .send_ok(&bbr_testkit::Request::RightClickFavorite {
                index: 0,
                x: 140.0,
                y: 120.0,
            })
            .expect("right click the favorite");
        flush_frame(&mut client);
        let menu_deadline = deadline();
        let menu = loop {
            let snapshot = client.snapshot().expect("favorite menu snapshot");
            if snapshot.tab_menu.open && snapshot.tab_menu.bound_tab == Some(c) {
                break snapshot;
            }
            assert!(Instant::now() < menu_deadline, "favorite menu never opened");
            std::thread::yield_now();
        };
        let folder_row = menu
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id.ends_with("move-folder"))
            .expect("the favorite's own-space folders are offered");
        client
            .send_ok(&bbr_testkit::Request::TabMenuActivate { index: folder_row })
            .expect("open folder submenu");
        flush_frame(&mut client);
        client
            .send_ok(&bbr_testkit::Request::TabMenuSubmenuActivate { index: 0 })
            .expect("commit the folder row");
        wait_menu_closed(&mut client);
        let group_deadline = deadline();
        let grouped = loop {
            flush_frame(&mut client);
            let snapshot = client.snapshot().expect("cross-space grouping snapshot");
            if snapshot.folders[0].members == [c] {
                break snapshot;
            }
            assert!(
                Instant::now() < group_deadline,
                "the favorite was never grouped"
            );
            std::thread::yield_now();
        };
        assert!(tab(&grouped, c).pinned, "grouping pins in the owning space");
        assert_eq!(tab(&grouped, c).space, default_space);
        assert_eq!(grouped.active_space, second, "no space switch");
        let reported = client
            .attach(Some(since))
            .expect("replay after the cross-space grouping")
            .into_iter()
            .filter(|(_, event)| {
                matches!(
                    event,
                    Event::BrowserChanged {
                        kind: BrowserChange::TabPinned { .. }
                    }
                )
            })
            .count();
        assert_eq!(reported, 0, "no active-space pin is reported");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
