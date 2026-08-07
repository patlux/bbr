//! Truthful navigation-action E2E: Back/Forward rejected when the
//! engine-authoritative nav flags say there is no history, Reload/Stop
//! success governed by the controller's navigate result (Ok → terminal web
//! activation; no active session → rejected), and session counts never
//! change on a rejection.

use super::*;

#[test]
#[ignore]
fn back_forward_reload_stop_report_truthful_outcomes() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-navtruth", &page_one);

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
        let default_profile = first.active_profile;

        // The fresh session has no back/forward history: the flags are the
        // engine-authoritative truth the rejection reads.
        let initial = client.snapshot().expect("initial nav snapshot");
        assert!(
            !initial.can_go_back && !initial.can_go_forward,
            "a fresh single-page session has no history"
        );

        // Rank a Back action row and activate it: no back history, so the
        // action is rejected truthfully — the surface stays open with the
        // activation-error state and nothing was destroyed.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "back".into(),
            })
            .expect("query back");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush back");
        let back_rows = client.snapshot().expect("back rows");
        let back_row = back_rows
            .command_results
            .iter()
            .find(|row| row.key == "action:7")
            .expect("back action row");
        assert_eq!(back_row.group, "action");
        key(&mut client, "enter");
        let back_error = client.snapshot().expect("back error snapshot");
        assert_eq!(
            back_error.command_result_state,
            CommandResultState::ActivationError,
            "back without history is rejected, never a silent success"
        );
        assert_ids(&back_error, &["bbr-command-overlay", "bbr-command-error"]);
        assert_eq!(back_error.command_bar_mode, CommandBarMode::Location);
        assert!(
            back_error.chrome_focused,
            "the surface stays open for editing"
        );
        assert_eq!(back_error.engine_sessions, 1, "nothing was destroyed");
        assert_eq!(back_error.active_web_view, Some(first_view));

        // Forward: same truthful rejection without forward history.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "forward".into(),
            })
            .expect("query forward");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush forward");
        let forward_rows = client.snapshot().expect("forward rows");
        let forward_row = forward_rows
            .command_results
            .iter()
            .find(|row| row.key == "action:8")
            .expect("forward action row");
        assert_eq!(forward_row.group, "action");
        key(&mut client, "enter");
        let forward_error = client.snapshot().expect("forward error snapshot");
        assert_eq!(
            forward_error.command_result_state,
            CommandResultState::ActivationError,
            "forward without history is rejected truthfully"
        );
        assert_eq!(forward_error.command_bar_mode, CommandBarMode::Location);
        assert!(forward_error.chrome_focused);
        assert_eq!(forward_error.engine_sessions, 1);

        // Reload with a live session: the controller's navigate result is Ok,
        // so the action is a terminal web activation — palette closed, web
        // focus, the same view/session (reload never recreates sessions).
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "reload".into(),
            })
            .expect("query reload");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush reload");
        let reload_rows = client.snapshot().expect("reload rows");
        let reload_row = reload_rows
            .command_results
            .iter()
            .find(|row| row.key == "action:9")
            .expect("reload action row");
        assert_eq!(reload_row.group, "action");
        key(&mut client, "enter");
        let reloaded = client.snapshot().expect("reloaded snapshot");
        assert_eq!(
            reloaded.command_bar_mode,
            CommandBarMode::Inactive,
            "successful reload closes the palette"
        );
        assert!(!reloaded.chrome_focused, "reload activates into the page");
        assert_eq!(reloaded.active_web_view, Some(first_view));
        assert_eq!(reloaded.engine_sessions, 1, "reload reuses the session");

        // Stop with a live session: the controller routes Ok, so the action
        // is terminal like Reload.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "stop".into(),
            })
            .expect("query stop");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush stop");
        let stop_rows = client.snapshot().expect("stop rows");
        let stop_row = stop_rows
            .command_results
            .iter()
            .find(|row| row.key == "action:10")
            .expect("stop action row");
        assert_eq!(stop_row.group, "action");
        key(&mut client, "enter");
        let stopped = client.snapshot().expect("stopped snapshot");
        assert_eq!(stopped.command_bar_mode, CommandBarMode::Inactive);
        assert!(!stopped.chrome_focused);
        assert_eq!(stopped.active_web_view, Some(first_view));
        assert_eq!(stopped.engine_sessions, 1);

        // Reload in an empty space: no active session, so the controller's
        // navigate result is Err and the action is rejected truthfully.
        client
            .send_ok(&dsl::create_space(default_profile, "Empty"))
            .expect("create empty space");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let empty = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::activate_space(empty))
            .expect("activate empty");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: empty },
            since,
        );
        let empty_active = client.snapshot().expect("empty space snapshot");
        assert_eq!(
            empty_active.active_web_view, None,
            "empty space has no view"
        );
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "reload".into(),
            })
            .expect("query reload in empty space");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush reload in empty space");
        let reload_rows = client.snapshot().expect("empty reload rows");
        let reload_row = reload_rows
            .command_results
            .iter()
            .find(|row| row.key == "action:9")
            .expect("reload action row in empty space");
        assert_eq!(reload_row.group, "action");
        key(&mut client, "enter");
        let reload_error = client.snapshot().expect("empty reload error snapshot");
        assert_eq!(
            reload_error.command_result_state,
            CommandResultState::ActivationError,
            "reload without an active session is governed by the navigate result"
        );
        assert_eq!(reload_error.command_bar_mode, CommandBarMode::Location);
        assert!(reload_error.chrome_focused);
        assert_eq!(
            reload_error.engine_sessions, 1,
            "the warm tabs stay warm across the rejection"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
