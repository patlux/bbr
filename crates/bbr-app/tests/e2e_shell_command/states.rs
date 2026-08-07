//! Command result-state E2E: the typed empty / loading / results states
//! expose stable §15 ids and typed snapshots (the search-fallback guarantee
//! means there is no production no-match state in this slice).

use super::*;

#[test]
#[ignore]
fn empty_and_loading_states_expose_stable_ids_and_typed_snapshots() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-states", &page_one);

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

        // Empty: the New-Tab staging surface starts with no text.
        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let empty = client.snapshot().expect("empty state snapshot");
        assert_eq!(empty.command_bar_mode, CommandBarMode::NewTab);
        assert_eq!(empty.command_result_state, CommandResultState::Empty);
        assert_eq!(empty.command_bar_text, "");
        assert_ids(&empty, &["bbr-command-results", "bbr-command-empty"]);
        assert!(
            !empty
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-command-result-")),
            "no result rows while empty"
        );

        // Loading: a keystroke schedules a debounce generation; until the
        // flush publishes, the surface truthfully shows the loading row.
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText { text: "pag".into() })
            .expect("set loading text");
        let loading = client.snapshot().expect("loading state snapshot");
        assert_eq!(loading.command_result_state, CommandResultState::Loading);
        assert_ids(&loading, &["bbr-command-results", "bbr-command-loading"]);

        // The deterministic flush publishes the current generation.
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush loading");
        let results = client.snapshot().expect("results state snapshot");
        assert_eq!(results.command_result_state, CommandResultState::Results);
        assert!(
            results
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-command-result-")),
            "result rows render after the flush"
        );

        // The mode-independent surface dismisses with Escape back to web.
        key(&mut client, "escape");
        since = wait_chrome_focus(&mut client, false, since);
        let dismissed = client.snapshot().expect("dismissed snapshot");
        assert_eq!(dismissed.command_bar_mode, CommandBarMode::Inactive);
        assert_eq!(dismissed.command_result_state, CommandResultState::Inactive);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
