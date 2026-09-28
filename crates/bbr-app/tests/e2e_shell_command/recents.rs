//! Blueprint D04 (DESIGN §11.4): an empty ⌘T offers the recently used tabs
//! (never the one on screen) and Reopen Closed Tab when something can be
//! reopened, so ⌘T ↵ returns to the last thing; typing `>` switches the
//! palette to actions — every row a browser command with its shortcut —
//! and Return on an action runs it. Every step synchronizes on typed
//! snapshot state or semantic events under a deadline.

use super::*;

fn flush_ranking(client: &mut Client) -> bbr_testkit::protocol::Snapshot {
    client
        .send_ok(&bbr_testkit::Request::FlushCommandRanking)
        .expect("flush ranking");
    client.snapshot().expect("ranking snapshot")
}

fn mode_chip(snapshot: &bbr_testkit::protocol::Snapshot) -> Option<String> {
    fn walk(nodes: &[bbr_testkit::protocol::SemanticNode]) -> Option<String> {
        for node in nodes {
            if node.id == "bbr-palette-mode" {
                return node.name.clone();
            }
            if let Some(found) = walk(&node.children) {
                return Some(found);
            }
        }
        None
    }
    walk(&snapshot.semantics)
}

#[test]
#[ignore]
fn empty_new_tab_offers_recents_and_the_angle_bracket_switches_to_actions() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-recents", &page_one);

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
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let two = client.snapshot().expect("two tabs");
        let one_key = tab_key(&two, &page_one, None);
        let two_key = tab_key(&two, &page_two, None);

        // Empty ⌘T: the previously used tab first; the tab on screen is not
        // offered; nothing to reopen yet.
        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let recents = flush_ranking(&mut client);
        assert_eq!(recents.command_bar_mode, CommandBarMode::NewTab);
        assert_eq!(recents.command_result_state, CommandResultState::Results);
        let keys: Vec<&str> = recents
            .command_results
            .iter()
            .map(|r| r.key.as_str())
            .collect();
        assert_eq!(
            keys.first(),
            Some(&one_key.as_str()),
            "last thing first: {keys:?}"
        );
        assert!(!keys.contains(&two_key.as_str()), "never the tab on screen");
        assert!(!keys.contains(&"action:3"), "nothing to reopen yet");
        assert_eq!(mode_chip(&recents).as_deref(), Some("newtab"));

        // ⌘T ↵ returns to the last thing.
        key(&mut client, "enter");
        let (seq, _) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::TabSelected { .. }),
            since,
        );
        since = seq;
        let back = client.snapshot().expect("returned snapshot");
        assert_eq!(back.active_web_view, Some(first_view));
        assert_eq!(back.tabs.len(), 2, "no tab was created");

        // Close the other tab: the empty ⌘T now also offers Reopen Closed
        // Tab, after the recents.
        client
            .send_ok(&bbr_testkit::Request::CloseTab { index: 1 })
            .expect("close tab two");
        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let with_reopen = flush_ranking(&mut client);
        assert_eq!(
            with_reopen.command_results.last().map(|r| r.key.as_str()),
            Some("action:3"),
            "Reopen Closed Tab comes last"
        );
        let reopen = with_reopen.command_results.last().expect("reopen row");
        assert_eq!(reopen.subtitle.as_deref(), Some("⇧⌘T"), "its shortcut");

        // `>` switches to actions: only browser commands, the mode chip says
        // so, and Return runs the selected action (Reopen Closed Tab).
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText { text: ">".into() })
            .expect("type >");
        let actions = flush_ranking(&mut client);
        assert!(!actions.command_results.is_empty());
        assert!(actions
            .command_results
            .iter()
            .all(|row| row.group == "action"));
        assert_eq!(mode_chip(&actions).as_deref(), Some("actions"));
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "> reopen".into(),
            })
            .expect("type > reopen");
        let filtered = flush_ranking(&mut client);
        assert_eq!(
            filtered.command_results.first().map(|r| r.key.as_str()),
            Some("action:3")
        );
        key(&mut client, "enter");
        let _ = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::TabReopened { .. }),
            since,
        );
        let reopened = client.snapshot().expect("reopened snapshot");
        assert_eq!(reopened.tabs.len(), 2, "the closed tab came back");

        // A `>` query without a match never searches or navigates.
        key(&mut client, "cmd-t");
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "> qqqzzz".into(),
            })
            .expect("type unmatched");
        let none = flush_ranking(&mut client);
        assert!(none.command_results.is_empty());
        assert_eq!(none.command_result_state, CommandResultState::Empty);
        let tabs_before = none.tabs.len();
        key(&mut client, "enter");
        let after = client.snapshot().expect("after unmatched enter");
        assert_eq!(after.tabs.len(), tabs_before, "no search tab was opened");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
