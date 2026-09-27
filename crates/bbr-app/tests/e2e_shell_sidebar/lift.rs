//! Real-app v2 rail lift E2E (task packet `v2-rail`, DESIGN §2 D2, §10.4,
//! §18.2): the selected row is the only rail surface painted with the
//! static `elev:lift`. The typed `lifted_ids` evidence is resolved by the
//! same row-state composition and folder-member selection the render paints
//! with, so the suite proves:
//!
//! 1. exactly the selected tab row lifts, and the lift follows selection;
//! 2. hover and press paint their washes (`hover`/`active` states) without
//!    ever lifting;
//! 3. a selected folder member lifts on its nested row as well as on the
//!    pinned row of the same tab — both presentations of the one selected
//!    tab, never another row.

use super::*;
use bbr_testkit::protocol::TabVisualState;

/// The one lifted tab row id for `index`.
fn row_id(index: usize) -> String {
    format!("bbr-tab-row-{index}")
}

#[test]
#[ignore]
fn only_the_selected_row_lifts_through_hover_press_and_folders() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-v2-lift", &page_one);

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
        let _second = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush two tabs");

        // 1. Exactly the selected row lifts.
        let two = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two.active_tab_index, Some(1));
        assert_eq!(two.tab_states[1], TabVisualState::Selected);
        assert_eq!(two.lifted_ids, vec![row_id(1)], "one lifted row");

        // 2. Hover and press paint washes, never a lift; the selected row
        // keeps the only lift.
        client.send_ok(&dsl::hover_tab_row(0)).expect("hover row 0");
        let hovered = client.snapshot().expect("hovered snapshot");
        assert_eq!(hovered.tab_states[0], TabVisualState::Hover);
        assert_eq!(hovered.lifted_ids, vec![row_id(1)], "hover never lifts");
        client.send_ok(&dsl::unhover_tab_row()).expect("unhover");
        client.send_ok(&dsl::press_tab_row(0)).expect("press row 0");
        let pressed = client.snapshot().expect("pressed snapshot");
        assert_eq!(pressed.tab_states[0], TabVisualState::Active);
        assert_eq!(pressed.lifted_ids, vec![row_id(1)], "press never lifts");
        client.send_ok(&dsl::release_tab_row()).expect("release");

        // The lift follows the committed selection.
        client.send_ok(&dsl::select_tab(0)).expect("select row 0");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush selection");
        let moved = client.snapshot().expect("moved selection snapshot");
        assert_eq!(moved.active_tab_index, Some(0));
        assert_eq!(moved.lifted_ids, vec![row_id(0)], "lift follows selection");

        // 3. A selected folder member lifts on its nested row and on the
        // pinned row of the same tab (grouping pins the member).
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("commit New Folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush folder");
        let folder_id = client.snapshot().expect("folder snapshot").folders[0].id;
        client
            .send_ok(&dsl::move_tab_into_folder(1, folder_id))
            .expect("move tab 1 into folder");
        client
            .send_ok(&dsl::click_folder_toggle(folder_id))
            .expect("expand folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush expanded folder");
        let expanded = client.snapshot().expect("expanded snapshot");
        let member_row = expanded
            .tabs
            .iter()
            .position(|tab| tab.title == PAGE_TWO_TITLE)
            .expect("member tab row");
        assert!(expanded.tabs[member_row].pinned, "grouping pins the member");
        assert!(
            !expanded
                .lifted_ids
                .contains(&"bbr-folder-item-0-0".to_string()),
            "an unselected member never lifts"
        );
        client
            .send_ok(&dsl::click_folder_member(folder_id, 0))
            .expect("select folder member");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush member selection");
        let member = client.snapshot().expect("member selected snapshot");
        assert_eq!(member.active_tab_index, Some(member_row));
        assert_eq!(
            member.lifted_ids,
            vec![row_id(member_row), "bbr-folder-item-0-0".to_string()],
            "both presentations of the one selected tab lift, nothing else"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
