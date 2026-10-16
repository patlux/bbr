//! Real-app space menu and delete confirmation E2E (blueprint D05, DESIGN
//! §11.3/§11.13): a secondary click on a space chip opens the space menu
//! bound to that exact space without activating it; Move Left/Right
//! reorder the strip within the profile group; Edit Space… opens the
//! configuration surface; New Tab in Space activates the space and opens
//! ⌘T; Delete Space… asks once in a blocking sheet over the content card
//! (Return on the default Cancel, Tab to the danger button, Escape) and a
//! confirmed delete closes the space's tabs and sessions, activates the
//! fallback space, and leaves Reopen Closed Tab as the undo. The last space
//! of a profile offers no delete. Every step synchronizes on typed browser
//! changes or snapshot state — no sleeps.

use bbr_testkit::protocol::{Snapshot, TestSpaceId};

use super::*;

const MENU: &str = "bbr-space-menu";
const EDIT: usize = 0;
const NEW_TAB: usize = 1;
const MOVE_LEFT: usize = 2;
const DELETE: usize = 4;

fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

fn flush(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

fn create_space(client: &mut Client, since: &mut u64, title: &str) -> TestSpaceId {
    let profile = client.snapshot().expect("profile snapshot").active_profile;
    client
        .send_ok(&dsl::create_space(profile, title))
        .expect("create space");
    let (seq, change) = wait_browser_change_matching(
        client,
        |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
        *since,
    );
    *since = seq;
    match change {
        BrowserChange::SpaceCreated { space, .. } => space,
        other => panic!("unexpected change {other:?}"),
    }
}

/// Open the space menu on `space`'s chip and return the snapshot.
fn open_menu(client: &mut Client, space: TestSpaceId) -> Snapshot {
    client
        .send_ok(&bbr_testkit::Request::RightClickSpaceChip {
            space,
            x: 120.0,
            y: 600.0,
        })
        .expect("right-click the chip");
    flush(client);
    let snapshot = client.snapshot().expect("menu snapshot");
    let menu = &snapshot.tab_menu;
    assert!(menu.open, "the space menu opens");
    assert_eq!(menu.source_key, MENU);
    assert_eq!(menu.bound_space, Some(space.0), "bound to the exact space");
    assert_eq!(menu.bound_tab, None);
    assert!(snapshot.visible_ids.iter().any(|id| id == MENU));
    snapshot
}

fn activate_row(client: &mut Client, index: usize) {
    client
        .send_ok(&bbr_testkit::Request::TabMenuActivate { index })
        .expect("activate the menu row");
    flush(client);
}

fn order(snapshot: &Snapshot) -> Vec<TestSpaceId> {
    snapshot.spaces.iter().map(|space| space.space).collect()
}

#[test]
#[ignore]
fn space_menu_moves_edits_and_opens_new_tabs_without_activating_on_open() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("space-menu", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let initial = wait_for_window_settled(&mut client, deadline());
        let home = initial.active_space;
        let reading = create_space(&mut client, &mut since, "Reading");
        let work = create_space(&mut client, &mut since, "Work");

        // Opening never activates; the rows follow the board order and the
        // edges disable the matching move.
        let opened = open_menu(&mut client, reading);
        assert_eq!(opened.active_space, home, "opening never activates");
        let ids: Vec<&str> = opened.tab_menu.rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "bbr-space-menu-edit",
                "bbr-space-menu-new-tab",
                "bbr-space-menu-move-left",
                "bbr-space-menu-move-right",
                "bbr-space-menu-delete",
            ]
        );
        assert!(opened.tab_menu.rows.iter().all(|row| row.enabled));
        assert!(opened.tab_menu.rows[DELETE].danger);

        // Move Left swaps Reading with Home in the strip.
        activate_row(&mut client, MOVE_LEFT);
        let (seq, moved) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceMoved { .. }),
            since,
        );
        since = seq;
        assert!(matches!(
            moved,
            BrowserChange::SpaceMoved { space, to: 0 } if space == reading
        ));
        let after_move = client.snapshot().expect("moved snapshot");
        assert!(!after_move.tab_menu.open, "the commit closes the menu");
        assert_eq!(order(&after_move), vec![reading, home, work]);
        assert_eq!(after_move.active_space, home, "moves never activate");
        let again = open_menu(&mut client, reading);
        assert!(!again.tab_menu.rows[MOVE_LEFT].enabled, "first chip");
        client
            .send_ok(&bbr_testkit::Request::TabMenuClose)
            .expect("dismiss");

        // Edit Space… opens the configuration surface for the exact space.
        open_menu(&mut client, work);
        activate_row(&mut client, EDIT);
        let editing = client.snapshot().expect("edit snapshot");
        let config = editing.space_config.as_ref().expect("config open");
        assert_eq!(config.space, work);
        assert!(!editing.tab_menu.open);
        client
            .send_ok(&dsl::space_config_cancel())
            .expect("cancel config");

        // New Tab in Space activates the space and opens ⌘T.
        open_menu(&mut client, work);
        activate_row(&mut client, NEW_TAB);
        let new_tab = client.snapshot().expect("new tab snapshot");
        assert_eq!(new_tab.active_space, work);
        assert_eq!(new_tab.command_bar_mode, CommandBarMode::NewTab);
        key(&mut client, "escape");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn delete_space_asks_once_and_closes_its_tabs_into_the_fallback() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, page_two) = fixtures();
    let page_three = fixtures.url("/no-favicon.html");
    let (mut harness, socket, token) = spawn_visible_harness("space-delete", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let initial = wait_for_window_settled(&mut client, deadline());
        let home = initial.active_space;
        let doomed = create_space(&mut client, &mut since, "Doomed");
        client
            .send_ok(&dsl::click_space(doomed))
            .expect("activate the doomed space");
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        open_tab(&mut client, &mut since, &page_three, None);
        let before = client.snapshot().expect("populated snapshot");
        assert_eq!(before.active_space, doomed);
        let doomed_tabs: Vec<u64> = before
            .all_tabs
            .iter()
            .filter(|tab| tab.space == doomed)
            .map(|tab| tab.id)
            .collect();
        assert_eq!(doomed_tabs.len(), 2);
        let sessions = before.engine_sessions;

        // Delete Space… opens the blocking sheet over the card.
        open_menu(&mut client, doomed);
        activate_row(&mut client, DELETE);
        // The portal orders in on a deferred AppKit turn: wait for the
        // native evidence under the deadline instead of racing it.
        let until = deadline();
        let asking = loop {
            client
                .send_ok(&bbr_testkit::Request::FlushFrame)
                .expect("flush");
            let snapshot = client.snapshot().expect("sheet snapshot");
            if snapshot.portal.native_visible || Instant::now() >= until {
                break snapshot;
            }
        };
        assert!(!asking.tab_menu.open, "the menu hands off to the sheet");
        let sheet = asking.space_delete.as_ref().expect("sheet open");
        assert_eq!(sheet.space, doomed.0);
        assert_eq!(sheet.title, "Delete “Doomed”?");
        assert_eq!(
            sheet.body,
            "Its 2 tabs close. Reopen Closed Tab brings tabs back. \
             History and site data stay in the profile."
        );
        assert!(!sheet.delete_focused, "Cancel owns the keyboard");
        assert!(
            sheet.card[2] > 0.0 && sheet.card[3] > 0.0,
            "veil covers the card"
        );
        // The sheet renders in the native portal above the web view.
        let portal = &asking.portal;
        assert!(
            portal.visible && portal.native_visible,
            "portal shows the sheet"
        );
        for id in [
            "bbr-space-delete-veil",
            "bbr-space-delete-sheet",
            "bbr-space-delete-cancel",
            "bbr-space-delete-confirm",
        ] {
            assert!(portal.ids.iter().any(|v| v == id), "missing {id}");
        }
        // Patrick (2026-10-12): the sheet sits centred on the card's veil.
        let rect = portal.rect.expect("sheet rect");
        assert!(
            (rect[0] + rect[2] / 2.0 - (sheet.card[0] + sheet.card[2] / 2.0)).abs() < 0.5,
            "the sheet frame is centred on the card: {rect:?} card {:?}",
            sheet.card
        );
        assert_sheet_centered(
            &mut client,
            "bbr-space-delete-sheet",
            "bbr-space-delete-veil",
            240.0,
        );

        // Return triggers the default Cancel: nothing is deleted.
        key(&mut client, "enter");
        flush(&mut client);
        let kept = client.snapshot().expect("cancelled snapshot");
        assert!(kept.space_delete.is_none());
        assert!(kept.spaces.iter().any(|s| s.space == doomed));
        assert_eq!(kept.engine_sessions, sessions);

        // Tab moves to the danger button; Escape still cancels.
        open_menu(&mut client, doomed);
        activate_row(&mut client, DELETE);
        key(&mut client, "tab");
        flush(&mut client);
        let armed = client.snapshot().expect("armed snapshot");
        assert!(armed.space_delete.as_ref().expect("sheet").delete_focused);
        key(&mut client, "escape");
        flush(&mut client);
        let escaped = client.snapshot().expect("escaped snapshot");
        assert!(escaped.space_delete.is_none());
        assert!(escaped.spaces.iter().any(|s| s.space == doomed));

        // The danger button deletes: tabs and sessions close, the fallback
        // (the previous space of the profile) becomes active.
        open_menu(&mut client, doomed);
        activate_row(&mut client, DELETE);
        client
            .send_ok(&bbr_testkit::Request::SpaceDeleteConfirm)
            .expect("confirm delete");
        let (seq, deleted) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceDeleted { .. }),
            since,
        );
        since = seq;
        assert_eq!(
            deleted,
            BrowserChange::SpaceDeleted {
                space: doomed,
                fallback: home,
                closed_tabs: 2,
                activated: true,
            }
        );
        flush(&mut client);
        let after = client.snapshot().expect("deleted snapshot");
        assert!(after.space_delete.is_none());
        assert!(after.spaces.iter().all(|s| s.space != doomed));
        assert_eq!(after.active_space, home);
        assert!(after
            .all_tabs
            .iter()
            .all(|tab| !doomed_tabs.contains(&tab.id)));
        assert_eq!(
            after.engine_sessions,
            sessions - 2,
            "the deleted space's sessions were destroyed"
        );

        // Reopen Closed Tab is the undo: the deleted space's first tab
        // comes back into the fallback.
        key(&mut client, "cmd-shift-t");
        let _ = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::TabReopened { .. }),
            since,
        );
        let reopened = client.snapshot().expect("reopened snapshot");
        let restored = reopened
            .all_tabs
            .iter()
            .find(|tab| tab.id == doomed_tabs[0])
            .expect("the first deleted tab reopened");
        assert_eq!(restored.space, home);
        assert_eq!(restored.url, page_two);

        // The last space of a profile offers no delete.
        let last = open_menu(&mut client, home);
        assert!(!last.tab_menu.rows[DELETE].enabled);
        client
            .send_ok(&bbr_testkit::Request::TabMenuClose)
            .expect("dismiss");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
