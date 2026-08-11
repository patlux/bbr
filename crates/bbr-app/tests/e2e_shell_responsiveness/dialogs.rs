use crate::*;
use bbr_testkit::protocol::Request;

#[test]
#[ignore]
fn dialogs_auto_dismiss_and_beforeunload_never_blocks_chrome_close() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, mut harness, socket, token, mut client, mut since) = start("renderer-dialogs");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let dialog_url = fixtures.url(DIALOG_PATH);
        let dialog_view = open_tab(
            &mut client,
            &mut since,
            &dialog_url,
            Some("BBR Hang Dialog"),
        );
        wait_fixture_hit(&fixtures, DIALOG_PATH);

        // The fixture immediately runs alert/confirm/prompt. Public
        // WKUIDelegate handlers answer them deterministically, so a client-
        // world content probe completes and observes the result page rather
        // than a modal dialog blocking the app.
        let probe = probe_content(&mut client, dialog_view);
        assert_eq!(probe.dom_title, "BBR Hang Dialog");
        assert!(probe.has_body);
        key(&mut client, "cmd-l");
        let chrome = wait_snapshot(&mut client, |s| s.chrome_focused);
        assert_eq!(chrome.active_web_view, Some(dialog_view));
        key(&mut client, "escape");
        key(&mut client, "cmd-w");
        let after_dialog = wait_snapshot(&mut client, |s| s.tabs.len() == 1);
        assert_eq!(after_dialog.engine_sessions, 1);

        let before = fixtures.url(BEFOREUNLOAD_PATH);
        let before_view = open_tab(
            &mut client,
            &mut since,
            &before,
            Some("BBR Hang Beforeunload"),
        );
        wait_fixture_hit(&fixtures, BEFOREUNLOAD_PATH);
        let ready = probe_content(&mut client, before_view);
        assert_eq!(ready.dom_title, "BBR Hang Beforeunload");

        // Chrome close destroys the WKWebView immediately. The pinned public
        // WKUIDelegate surface has no beforeunload callback, therefore no page
        // prompt or page-script round trip can delay this normal close path.
        key(&mut client, "cmd-w");
        let closed = wait_snapshot(&mut client, |s| s.tabs.len() == 1);
        assert_eq!(closed.engine_sessions, 1);
        assert_ne!(closed.active_web_view, Some(before_view));

        // A synthetic late completion scoped to the destroyed id is harmless.
        client
            .send_ok(&Request::InjectWebEvent {
                event: WebEvent::UnresponsiveChanged(before_view, false),
            })
            .unwrap();
        let stable = client.snapshot().expect("post-stale snapshot");
        assert_eq!(stable.tabs.len(), 1);
        assert_eq!(stable.engine_sessions, 1);
    }));
    finish_case(result, &mut harness, &socket, &token);
}
