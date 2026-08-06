//! Real-app visible-tab shell E2E (Phase-4 first vertical slice).
//!
//! Covers the controller-backed vertical tab surface over real WKWebViews:
//! pointer open/select/close/reopen through the production listener path,
//! keyboard traversal through real keystrokes, and empty-space handling. No
//! sleeps: every step synchronizes on semantic events or snapshot state under
//! a deadline. The shared harness lives in `e2e_common`; the pinned/regular
//! regions, row states, fallbacks, and crash recovery suite lives in
//! `e2e_shell_states.rs`.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::CommandBarMode;
use e2e_common::*;

#[test]
#[ignore]
fn pointer_opens_selects_closes_reopens_visible_tabs() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");

    let (mut harness, socket, token) = spawn_harness("pointer", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        assert_eq!(initial.tabs.len(), 1);
        assert_eq!(initial.engine_sessions, 1);
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // §15 semantic ids: the sidebar shell and the single visible row.
        assert_ids(
            &initial,
            &[
                "bbr-root",
                "bbr-sidebar",
                "bbr-brand",
                "bbr-command-bar",
                "bbr-command-text",
                "bbr-status",
                "bbr-sidebar-toggle",
                "bbr-tab-list",
                "bbr-tab-row-0",
                "bbr-tab-title-0",
                "bbr-tab-favicon-0",
                "bbr-tab-state-0",
                "bbr-tab-close-0",
            ],
        );
        assert!(
            !initial
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-tab-row-1")),
            "one tab renders exactly one row"
        );

        // Pointer open: a second visible row.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let two = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two.tabs.len(), 2);
        assert_eq!(two.engine_sessions, 2);
        assert_eq!(two.active_tab_index, Some(1));
        assert_eq!(
            two.tab_states,
            vec![TabVisualState::Default, TabVisualState::Selected]
        );
        assert_ids(
            &two,
            &[
                "bbr-tab-row-1",
                "bbr-tab-title-1",
                "bbr-tab-favicon-1",
                "bbr-tab-state-1",
                "bbr-tab-close-1",
            ],
        );

        // Pointer select: activate row 0 through the production mouse-down
        // listener path. The pointer keeps the chrome focus domain.
        client.send_ok(&dsl::click_tab(0)).expect("click first row");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        let selected = client.snapshot().expect("first selected");
        assert_eq!(selected.active_web_view, Some(first_view));
        assert_eq!(selected.active_tab_index, Some(0));
        assert_eq!(selected.engine_sessions, 2, "warm select keeps sessions");
        assert!(selected.chrome_focused, "pointer keeps chrome focus domain");
        assert_eq!(selected.tabs[0].web_view, Some(first_view));
        assert_eq!(
            selected.tab_states,
            vec![TabVisualState::Selected, TabVisualState::Default]
        );

        // Warm pointer selection must not reload the target: capture the
        // selection sequence, select back, and prove no engine load event
        // for the first view crossed the wire in between.
        client
            .send_ok(&dsl::click_tab(1))
            .expect("click second row");
        let (seq_second, _) = client
            .wait_for_event_since(
                |event| matches!(event, Event::BrowserChanged { kind } if *kind == BrowserChange::TabSelected { index: 1 }),
                since,
                deadline(),
            )
            .expect("second selected");
        let reselected = client.snapshot().expect("second reselected");
        assert_eq!(reselected.active_web_view, Some(second_view));
        assert_eq!(reselected.tabs[0].web_view, Some(first_view));
        assert_eq!(reselected.engine_sessions, 2);

        let reloads = client
            .attach(Some(since))
            .expect("replay selection window")
            .into_iter()
            .filter(|(seq, event)| {
                *seq < seq_second
                    && matches!(
                        event,
                        Event::Web(WebEvent::LoadingChanged(view, true))
                            | Event::Web(WebEvent::UrlChanged(view, _))
                            if *view == first_view
                    )
            })
            .count();
        assert_eq!(reloads, 0, "warm selection must not reload the tab");
        since = seq_second;

        // Pointer close of the first row's close control: destroys exactly
        // its session, neighbor becomes active, chrome focus preserved.
        client
            .send_ok(&dsl::click_tab_close(0))
            .expect("click first close");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let closed = client.snapshot().expect("closed snapshot");
        assert_eq!(closed.tabs.len(), 1);
        assert_eq!(closed.engine_sessions, 1);
        assert_eq!(closed.active_web_view, Some(second_view));
        assert!(closed.chrome_focused);
        assert_eq!(closed.command_bar_mode, CommandBarMode::Inactive);

        // Reopen restores the closed tab's identity (title/URL) with a fresh
        // native session and keeps the invoking chrome focus domain.
        client.send_ok(&dsl::reopen_tab()).expect("reopen tab");
        since = wait_browser_change(&mut client, BrowserChange::TabReopened { index: 0 }, since);
        let reopened = client.snapshot().expect("reopened snapshot");
        assert_eq!(reopened.tabs.len(), 2);
        assert_eq!(reopened.active_tab_index, Some(0));
        assert_eq!(reopened.engine_sessions, 2);
        assert_eq!(reopened.tabs[0].url, page_one, "URL identity preserved");
        assert!(reopened.chrome_focused, "reopen preserves chrome domain");
        let fresh_view = reopened.active_web_view.expect("fresh view");
        assert_ne!(fresh_view, first_view, "fresh session incarnation");
        if reopened.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(&mut client, fresh_view, PAGE_ONE_TITLE, since);
        }
        let restored = client.snapshot().expect("restored snapshot");
        assert_eq!(
            restored.tabs[0].title, PAGE_ONE_TITLE,
            "title identity preserved"
        );

        // Empty space: closing the final tab through the pointer path leaves
        // the empty New Tab surface (no about:blank, no session).
        client
            .send_ok(&dsl::click_tab_close(0))
            .expect("close first row");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        client
            .send_ok(&dsl::click_tab_close(0))
            .expect("close last row");
        let _ = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let empty = client.snapshot().expect("empty space snapshot");
        assert!(empty.tabs.is_empty());
        assert_eq!(empty.engine_sessions, 0);
        assert_eq!(empty.active_web_view, None);
        assert_eq!(empty.command_bar_mode, CommandBarMode::NewTab);
        assert!(empty.chrome_focused);
        // §11.8 pane + §15 ids: the empty New Tab surface replaces the rows.
        assert_ids(&empty, &["bbr-empty-new-tab", "bbr-empty-new-tab-button"]);
        assert!(
            !empty
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-tab-row")),
            "empty space renders no tab rows"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Keyboard traversal of the visible tab list with real keystrokes: Cmd-T +
/// Enter opens, arrow keys move selection, backspace closes, Cmd-Shift-T
/// reopens. The tab list keeps chrome focus across the sequence.

#[test]
#[ignore]
fn keyboard_navigates_visible_tab_list() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");

    let (mut harness, socket, token) = spawn_harness("keyboard", &page_one);

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

        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };

        // Keyboard open: Cmd-T stages, real keystrokes commit the new tab.
        keystroke(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let staged = client.snapshot().expect("staged snapshot");
        assert_eq!(staged.command_bar_mode, CommandBarMode::NewTab);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_two.clone(),
            })
            .expect("set destination");
        keystroke(&mut client, "enter");
        since = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 1 }, since);
        let opened = client.snapshot().expect("opened snapshot");
        assert_eq!(opened.tabs.len(), 2);
        assert_eq!(opened.engine_sessions, 2);
        let second_view = opened.active_web_view.expect("second view");
        if opened.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(&mut client, second_view, PAGE_TWO_TITLE, since);
        }

        // Enter moves focus into the new page; a pointer click on a row then
        // brings the tab list into chrome focus (production path).
        assert!(!client.snapshot().expect("web focus").chrome_focused);
        client
            .send_ok(&dsl::click_tab(0))
            .expect("click first row to focus the list");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        // The pointer listener emits the focus change before the selection
        // event, so the chrome-domain assertion reads the authoritative
        // snapshot rather than racing the event cursor.
        assert!(
            client.snapshot().expect("tab list focused").chrome_focused,
            "pointer row click focuses the tab list (chrome domain)"
        );

        // Real arrow-key traversal: down selects the second row, up returns.
        keystroke(&mut client, "down");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let down = client.snapshot().expect("down snapshot");
        assert_eq!(down.active_tab_index, Some(1));
        assert_eq!(down.active_web_view, Some(second_view));
        assert_eq!(
            down.engine_sessions, 2,
            "keyboard warm select keeps sessions"
        );
        assert!(down.chrome_focused, "keyboard traversal stays in chrome");

        keystroke(&mut client, "up");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        let up = client.snapshot().expect("up snapshot");
        assert_eq!(up.active_tab_index, Some(0));
        assert_eq!(up.active_web_view, Some(first_view));
        assert!(up.chrome_focused);

        // Backspace closes the active row; Cmd-Shift-T reopens it. Both
        // preserve the chrome focus domain.
        keystroke(&mut client, "backspace");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let after_close = client.snapshot().expect("closed snapshot");
        assert_eq!(after_close.tabs.len(), 1);
        assert_eq!(after_close.active_tab_index, Some(0));
        assert_eq!(after_close.active_web_view, Some(second_view));
        assert_eq!(after_close.engine_sessions, 1);
        assert!(after_close.chrome_focused);

        keystroke(&mut client, "cmd-shift-t");
        since = wait_browser_change(&mut client, BrowserChange::TabReopened { index: 0 }, since);
        let after_reopen = client.snapshot().expect("reopened snapshot");
        assert_eq!(after_reopen.tabs.len(), 2);
        assert_eq!(after_reopen.active_tab_index, Some(0));
        assert_eq!(after_reopen.engine_sessions, 2);
        assert!(after_reopen.chrome_focused);
        let fresh_view = after_reopen.active_web_view.expect("fresh view");
        assert_ne!(fresh_view, first_view);
        if after_reopen.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(&mut client, fresh_view, PAGE_ONE_TITLE, since);
        }
        assert_eq!(
            client.snapshot().expect("restored title").tabs[0].title,
            PAGE_ONE_TITLE
        );

        // Enter activates the selected row: the chrome traversal commits and
        // keyboard focus moves into the page (DESIGN.md §11.1 / interaction
        // contract). Selection and session identity are untouched.
        keystroke(&mut client, "enter");
        since = wait_chrome_focus(&mut client, false, since);
        let activated = client.snapshot().expect("activated snapshot");
        assert_eq!(activated.active_tab_index, Some(0));
        assert_eq!(activated.active_web_view, Some(fresh_view));
        assert_eq!(activated.engine_sessions, 2, "activation never recreates");
        assert!(!activated.chrome_focused, "Enter moves focus into the page");
        assert_eq!(activated.command_bar_mode, CommandBarMode::Inactive);

        // The page keeps keyboard focus afterwards; a later pointer row click
        // returns to the tab list (chrome domain) without losing selection.
        assert!(!client.snapshot().expect("web focus stays").chrome_focused);
        client
            .send_ok(&dsl::click_tab(1))
            .expect("click second row after activation");
        let _ = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let back = client.snapshot().expect("back in chrome");
        assert!(back.chrome_focused);
        assert_eq!(back.active_tab_index, Some(1));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Keyboard close of the final tab must open the empty New Tab command
/// surface exactly like the pointer path (interaction contract: closing the
/// final tab leaves the space empty and offers an obvious way to open a new
/// tab; no about:blank is created). Regression for the keyboard path that
/// used to reset the staged surface back to the tab list.

#[test]
#[ignore]
fn keyboard_close_of_final_tab_opens_empty_new_tab_surface() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");

    let (mut harness, socket, token) = spawn_harness("empty-keyboard", &page_one);

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

        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };

        // Seed a second tab so the final close is the unambiguous last one.
        client
            .send_ok(&bbr_testkit::Request::OpenTab {
                url: page_one.clone(),
            })
            .expect("open second tab");
        since = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 1 }, since);
        let two = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two.tabs.len(), 2);
        let second_view = two.active_web_view.expect("second view");
        if two.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, second_view, PAGE_ONE_TITLE, since);
        }

        // Focus the tab list through the production pointer path, then close
        // the active row. The first close leaves a tab behind, so the list
        // keeps focus for repeated keys.
        client
            .send_ok(&dsl::click_tab(1))
            .expect("focus list on row 1");
        let _ = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        assert!(client.snapshot().expect("list focused").chrome_focused);
        keystroke(&mut client, "backspace");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 1 }, since);
        let one = client.snapshot().expect("one-tab snapshot");
        assert_eq!(one.tabs.len(), 1);
        assert_eq!(one.active_web_view, Some(first_view));
        assert!(one.chrome_focused, "repeated keys stay in the tab list");
        assert_eq!(one.command_bar_mode, CommandBarMode::Inactive);

        // The final backspace must open the empty New Tab command surface and
        // keep it open: NewTab mode + chrome (command-bar) focus, no session.
        keystroke(&mut client, "backspace");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 0 }, since);
        let empty = client.snapshot().expect("empty keyboard snapshot");
        assert!(empty.tabs.is_empty());
        assert_eq!(empty.engine_sessions, 0);
        assert_eq!(empty.active_web_view, None);
        assert_eq!(empty.command_bar_mode, CommandBarMode::NewTab);
        assert!(empty.chrome_focused, "command surface owns chrome focus");
        // §11.8 pane + §15 ids: the empty New Tab surface renders.
        assert_ids(&empty, &["bbr-empty-new-tab", "bbr-empty-new-tab-button"]);

        // Enter on the empty command surface with no destination creates
        // nothing; the staged surface stays open for the next destination.
        keystroke(&mut client, "enter");
        let still_empty = client.snapshot().expect("still empty after Enter");
        assert!(still_empty.tabs.is_empty());
        assert_eq!(still_empty.engine_sessions, 0);
        assert_eq!(still_empty.command_bar_mode, CommandBarMode::NewTab);

        // A committed destination re-opens the flow from the empty state.
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_one.clone(),
            })
            .expect("set destination");
        keystroke(&mut client, "enter");
        let _ = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 0 }, since);
        let reborn = client.snapshot().expect("reborn snapshot");
        assert_eq!(reborn.tabs.len(), 1);
        assert_eq!(reborn.engine_sessions, 1);
        assert!(
            !reborn.chrome_focused,
            "committed New Tab moves to the page"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
