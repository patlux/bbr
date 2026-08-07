//! Command activation E2E: Up/Down selection, Enter activation of
//! tabs/spaces/actions, Cmd-Enter new-tab activation, and the terminal
//! Close Tab aftermath.

use super::*;

#[test]
#[ignore]
fn arrows_move_selection_and_enter_activates_tabs_spaces_and_actions() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-act", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let default_profile = first.active_profile;
        let page_two_key = tab_key(
            &client.snapshot().expect("activation keys"),
            &page_two,
            None,
        );

        // A query matching both fixture titles yields ≥ 2 deterministic rows.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText { text: "bbr".into() })
            .expect("query bbr");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush bbr");
        let rows = client.snapshot().expect("bbr rows snapshot");
        assert_eq!(rows.command_result_state, CommandResultState::Results);
        assert!(
            rows.command_results.len() >= 2,
            "both fixture titles match 'bbr': {:?}",
            rows.command_results
        );
        assert!(rows.command_results[0].selected);
        assert!(!rows.command_results[1].selected);

        // Up/Down move the selection without leaving the field.
        key(&mut client, "down");
        let down = client.snapshot().expect("down snapshot");
        assert!(down.command_results[1].selected);
        assert!(down.chrome_focused, "selection never leaves the field");
        assert_eq!(down.command_bar_mode, CommandBarMode::Location);
        key(&mut client, "up");
        assert!(client.snapshot().expect("up snapshot").command_results[0].selected);

        // Enter on the exact-tab row (page_two) selects that tab and moves
        // focus into the page.
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_two.clone(),
            })
            .expect("set page_two destination");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush page_two");
        let exact = client.snapshot().expect("exact tab snapshot");
        assert_eq!(exact.command_results[0].key, page_two_key);
        assert_eq!(exact.command_results[0].group, "exact_tab");
        key(&mut client, "enter");
        let selected = client.snapshot().expect("tab selected snapshot");
        assert_eq!(selected.command_bar_mode, CommandBarMode::Inactive);
        assert!(!selected.chrome_focused, "Enter activates into the page");
        assert_eq!(selected.active_web_view, Some(second_view));
        assert_eq!(selected.active_tab_index, Some(1));
        assert_eq!(selected.command_bar_text, page_two);

        // Space activation through the command surface: a second space in the
        // default profile; Enter on its row activates it and keeps the chrome
        // focus domain.
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading space");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "reading".into(),
            })
            .expect("query reading");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush reading");
        let space_rows = client.snapshot().expect("reading rows snapshot");
        let space_row = space_rows
            .command_results
            .iter()
            .find(|row| row.key == format!("space:{}", reading.0))
            .expect("reading space row");
        assert_eq!(space_row.group, "space");
        assert_eq!(space_row.subtitle.as_deref(), Some("Space"));
        key(&mut client, "enter");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let space_active = client.snapshot().expect("reading space active snapshot");
        assert_eq!(space_active.active_space, reading);
        assert_eq!(space_active.command_bar_mode, CommandBarMode::Inactive);
        assert!(
            space_active.chrome_focused,
            "space activation keeps the chrome focus domain"
        );
        assert_eq!(space_active.active_web_view, None, "reading is empty");
        assert_eq!(space_active.engine_sessions, 2, "warm tabs stay warm");
        assert_eq!(space_active.active_tab_index, None);

        // Action activation: "close" ranks the destructive Close Tab action
        // row; the empty space has no active tab, so Enter surfaces the
        // truthful activation-error state instead of a silent no-op.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "close".into(),
            })
            .expect("query close");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush close");
        let close_rows = client.snapshot().expect("close rows snapshot");
        let close_row = close_rows
            .command_results
            .iter()
            .find(|row| row.key == "action:2")
            .expect("close tab action row");
        assert_eq!(close_row.group, "action");
        assert!(close_row.destructive, "close tab is the destructive action");
        assert_ids(
            &close_rows,
            &["bbr-command-results", "bbr-command-result-action:2"],
        );
        key(&mut client, "enter");
        let error = client.snapshot().expect("activation error snapshot");
        assert_eq!(
            error.command_result_state,
            CommandResultState::ActivationError,
            "closing a tab in an empty space is rejected truthfully"
        );
        assert_ids(&error, &["bbr-command-results", "bbr-command-error"]);
        assert_eq!(error.command_bar_mode, CommandBarMode::Location);
        assert!(error.chrome_focused, "the surface stays open for editing");
        assert_eq!(error.engine_sessions, 2, "nothing was destroyed");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn cmd_enter_opens_exactly_one_new_tab_and_focuses_it() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-newtab", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let before = client.snapshot().expect("pre cmd-enter snapshot");
        let page_one_key = tab_key(&before, &page_one, None);
        let before_tabs = before.tabs.len();
        let before_sessions = before.engine_sessions;

        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_one.clone(),
            })
            .expect("set page_one destination");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush page_one");
        let exact = client.snapshot().expect("exact row snapshot");
        assert_eq!(exact.command_results[0].key, page_one_key);
        key(&mut client, "cmd-enter");
        let after = client.snapshot().expect("cmd-enter snapshot");
        assert_eq!(after.tabs.len(), before_tabs + 1, "exactly one new tab");
        assert_eq!(
            after.engine_sessions,
            before_sessions + 1,
            "exactly one new live session"
        );
        assert_eq!(after.command_bar_mode, CommandBarMode::Inactive);
        assert!(!after.chrome_focused, "the fresh page owns focus");
        let new_view = after.active_web_view.expect("fresh view");
        assert_ne!(new_view, first_view);
        assert_ne!(new_view, second_view);
        assert_eq!(after.active_tab_index, Some(before_tabs));
        assert_eq!(after.command_bar_text, page_one);
        if after.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, new_view, PAGE_ONE_TITLE, since);
        }
        // The opened tab is a real page: its deterministic fixture title
        // presents on the fresh session.
        assert_eq!(
            client.snapshot().expect("title snapshot").title.as_deref(),
            Some(PAGE_ONE_TITLE)
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn close_tab_action_is_terminal_and_selects_the_neighbor() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-close", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let before = client.snapshot().expect("before close snapshot");
        assert_eq!(before.tabs.len(), 2);
        assert_eq!(before.engine_sessions, 2);
        assert_eq!(before.active_tab_index, Some(1));
        assert_eq!(before.active_web_view, Some(second_view));

        // Rank the destructive Close Tab action and activate it.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "close".into(),
            })
            .expect("query close");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush close");
        let close_rows = client.snapshot().expect("close rows snapshot");
        let close_row = close_rows
            .command_results
            .iter()
            .find(|row| row.key == "action:2")
            .expect("close tab action row");
        assert!(close_row.destructive, "close tab is the destructive action");
        key(&mut client, "enter");

        // Terminal aftermath: the neighbor (page_one) is selected with its
        // warm session, the closed tab's session is destroyed, and the
        // result surface is dismissed with the chrome focus domain.
        let after = client.snapshot().expect("after close snapshot");
        assert_eq!(after.tabs.len(), 1, "the closed tab is gone");
        assert_eq!(after.engine_sessions, 1, "the closed session is destroyed");
        assert_eq!(after.active_tab_index, Some(0), "the neighbor is selected");
        assert_eq!(
            after.active_web_view,
            Some(first_view),
            "the warm neighbor view is reused"
        );
        assert_eq!(
            after.command_bar_mode,
            CommandBarMode::Inactive,
            "the palette is closed after a successful close"
        );
        assert_eq!(after.command_result_state, CommandResultState::Inactive);
        assert!(
            !after
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-command-result")),
            "no result rows remain after the terminal action"
        );
        assert!(
            after.chrome_focused,
            "the initiating chrome focus domain is preserved"
        );
        assert_eq!(after.command_bar_text, page_one);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
