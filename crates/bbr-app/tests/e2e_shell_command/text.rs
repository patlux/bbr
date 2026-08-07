//! Command-text editing E2E: Cmd-L selection, replace-selection typing,
//! Escape origin restoration (web and chrome origins), the repeated-Cmd-L
//! origin-transient regression, and burst-typing generation discipline.

use super::*;

#[test]
#[ignore]
fn cmd_l_selects_url_typing_replaces_and_escape_restores_origin() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-base", &page_one);

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

        // Cmd-L selects the current URL and focuses the field.
        let focused = cmd_l(&mut client, &mut since);
        assert_eq!(focused.command_bar_text, page_one);
        assert_eq!(
            focused.command_selection,
            Some([0, page_one.len()]),
            "Cmd-L selects the whole active URL"
        );
        assert_eq!(focused.command_cursor, page_one.len());
        assert_ids(&focused, &["bbr-command-bar", "bbr-command-cursor"]);

        // Typing replaces the selection (real keystrokes through the
        // production key-down handler). 'q'/'z' match no tab/action, so the
        // deterministic flush must publish only the settle search fallback.
        type_char(&mut client, 'q');
        type_char(&mut client, 'z');
        let typed = client.snapshot().expect("typed snapshot");
        assert_eq!(typed.command_bar_text, "qz");
        assert_eq!(typed.command_cursor, 2);
        assert_eq!(
            typed.command_selection, None,
            "typing collapses the selection"
        );
        assert!(typed.chrome_focused);

        // Backspace through the text model.
        key(&mut client, "backspace");
        let backspaced = client.snapshot().expect("backspace snapshot");
        assert_eq!(backspaced.command_bar_text, "q");
        assert_eq!(backspaced.command_cursor, 1);

        // Horizontal cursor movement stays on char boundaries.
        key(&mut client, "left");
        let moved = client.snapshot().expect("cursor-left snapshot");
        assert_eq!(moved.command_cursor, 0);
        key(&mut client, "right");
        assert_eq!(
            client
                .snapshot()
                .expect("cursor-right snapshot")
                .command_cursor,
            1
        );

        // The deterministic flush publishes ranked results (the typed text
        // matches nothing local, so the settle search fallback row is first).
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush ranking");
        let published = client.snapshot().expect("published snapshot");
        assert_eq!(published.command_result_state, CommandResultState::Results);
        assert_eq!(
            published.command_results[0].key, "search:q",
            "the encoded search intent is the fallback row"
        );
        assert_eq!(published.command_results[0].group, "fallback");
        assert_eq!(
            published.command_results[0].title, "Search for q",
            "the fallback row names the deterministic search intent"
        );
        // The settled DuckDuckGo endpoint (`?q=<percent-encoded terms>`) is
        // asserted by the encoded intent and unit-tested in
        // `command_activate`; the E2E never hits the live provider.
        assert_ids(
            &published,
            &["bbr-command-results", "bbr-command-result-search:q"],
        );

        // The §15 semantic tree mirrors the rendered result surface: a List
        // under the command field with one selected Result row.
        let field = find(&published.semantics, "bbr-command-bar").expect("command field");
        let list = field
            .children
            .iter()
            .find(|node| node.id == "bbr-command-results")
            .expect("semantic result list");
        assert_eq!(list.role, SemanticRole::List);
        assert_eq!(list.children.len(), 1);
        let row = &list.children[0];
        assert_eq!(row.role, SemanticRole::Result);
        assert!(row.selected, "the first fallback row is selected");

        // Escape restores the exact origin text and the web focus domain.
        key(&mut client, "escape");
        since = wait_chrome_focus(&mut client, false, since);
        let restored = client.snapshot().expect("restored snapshot");
        assert_eq!(restored.command_bar_mode, CommandBarMode::Inactive);
        assert!(!restored.chrome_focused);
        assert_eq!(
            restored.command_bar_text, page_one,
            "Escape restores the exact origin URL"
        );
        assert_eq!(restored.command_result_state, CommandResultState::Inactive);
        assert_eq!(restored.active_web_view, Some(first_view));
        assert!(
            !restored
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-command-result")),
            "the result surface is dismissed"
        );

        // Chrome-origin invocation: Cmd-L from a chrome control (the focused
        // tab list) restores the chrome focus domain on Escape, with the
        // exact origin text.
        client
            .send_ok(&dsl::click_tab(0))
            .expect("focus the tab list");
        since = wait_chrome_focus(&mut client, true, since);
        let _ = cmd_l(&mut client, &mut since);
        key(&mut client, "escape");
        let chrome_restored = client.snapshot().expect("chrome restored snapshot");
        assert_eq!(chrome_restored.command_bar_mode, CommandBarMode::Inactive);
        assert!(
            chrome_restored.chrome_focused,
            "Escape restores the originating chrome focus domain"
        );
        assert_eq!(
            chrome_restored.command_bar_text, page_one,
            "the chrome origin URL is restored exactly"
        );
        assert_eq!(chrome_restored.active_web_view, Some(first_view));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn repeated_cmd_l_preserves_the_origin_return_transient() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-repeat", &page_one);

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

        // Web origin: Cmd-L, edit, repeat Cmd-L, Escape. The repeated
        // Cmd-L must NOT overwrite the original return transient: Escape
        // restores the original web origin (exact URL + web focus), never
        // the intermediate edit state.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "edited".into(),
            })
            .expect("set edited text");
        let edited = client.snapshot().expect("edited snapshot");
        assert_eq!(edited.command_bar_text, "edited");
        key(&mut client, "cmd-l");
        let repeated = client.snapshot().expect("repeated cmd-l snapshot");
        assert_eq!(repeated.command_bar_mode, CommandBarMode::Location);
        assert_eq!(
            repeated.command_bar_text, page_one,
            "the repeated Cmd-L refreshes the live URL"
        );
        assert_eq!(
            repeated.command_selection,
            Some([0, page_one.len()]),
            "the refreshed URL is fully selected for replace-typing"
        );
        key(&mut client, "escape");
        since = wait_chrome_focus(&mut client, false, since);
        let restored = client.snapshot().expect("restored snapshot");
        assert_eq!(restored.command_bar_mode, CommandBarMode::Inactive);
        assert!(
            !restored.chrome_focused,
            "Escape restores the original web focus domain"
        );
        assert_eq!(
            restored.command_bar_text, page_one,
            "Escape restores the exact original origin URL, not 'edited'"
        );
        assert_eq!(restored.active_web_view, Some(first_view));

        // Chrome origin: the same flow from the focused tab list restores
        // the original chrome focus domain.
        client
            .send_ok(&dsl::click_tab(0))
            .expect("focus the tab list");
        since = wait_chrome_focus(&mut client, true, since);
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "edited2".into(),
            })
            .expect("set edited2 text");
        key(&mut client, "cmd-l");
        key(&mut client, "escape");
        let chrome_restored = client.snapshot().expect("chrome restored snapshot");
        assert_eq!(chrome_restored.command_bar_mode, CommandBarMode::Inactive);
        assert!(
            chrome_restored.chrome_focused,
            "Escape restores the original chrome focus domain"
        );
        assert_eq!(
            chrome_restored.command_bar_text, page_one,
            "the original chrome origin URL is restored exactly"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn burst_typing_publishes_only_the_final_generation_after_flush() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-burst", &page_one);

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
        let page_two_key = tab_key(&client.snapshot().expect("burst keys"), &page_two, None);

        let _ = cmd_l(&mut client, &mut since);
        // A burst of edits without any flush: each mutation bumps the
        // generation and restarts the debounce; nothing is published until
        // the deterministic flush.
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText { text: "p".into() })
            .expect("burst p");
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText { text: "pa".into() })
            .expect("burst pa");
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "page2".into(),
            })
            .expect("burst page2");
        // The flush cancels the pending debounce timer and publishes the
        // current generation synchronously: only the final text's results
        // can land — stale generations are invalidated, never published.
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush burst");
        let published = client.snapshot().expect("burst published snapshot");
        assert_eq!(published.command_result_state, CommandResultState::Results);
        assert_eq!(
            published
                .command_results
                .first()
                .map(|row| row.key.as_str()),
            Some(page_two_key.as_str()),
            "the final generation's page2 tab ranks first"
        );
        assert_eq!(published.command_results[0].group, "current_tab");
        assert_eq!(
            published
                .command_results
                .iter()
                .filter(|row| row.key.starts_with("tab:"))
                .count(),
            1,
            "no stale-generation tab row for the earlier 'p'/'pa' queries"
        );

        // Enter on the exact-tab row activates that tab in web focus.
        key(&mut client, "enter");
        let activated = client.snapshot().expect("activated snapshot");
        assert_eq!(activated.command_bar_mode, CommandBarMode::Inactive);
        assert!(!activated.chrome_focused);
        assert_eq!(activated.active_web_view, Some(second_view));
        assert_eq!(activated.command_bar_text, page_two);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
