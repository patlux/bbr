use crate::*;
use bbr_testkit::protocol::Request;

#[test]
#[ignore]
fn wait_clear_reload_close_and_stale_events_use_single_authority() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, mut harness, socket, token, mut client, mut since) = start("renderer-recovery");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let first = client.snapshot().expect("initial").active_web_view.unwrap();
        let neighbor = open_tab(
            &mut client,
            &mut since,
            &fixtures.url("/page2.html"),
            Some(PAGE_TWO_TITLE),
        );
        client.send_ok(&Request::SelectTab { index: 0 }).unwrap();
        navigate(&mut client, &fixtures.url(BUSY_PATH));
        wait_fixture_hit(&fixtures, BUSY_PATH);
        let hung = wait_real_unresponsive(&mut client, first);
        assert_unresponsive_surface(&hung, first);

        // Wait keeps the same session and keeps monitoring. The deterministic
        // recovery completion below models the adapter answer after a page
        // becomes alive; the real busy fixture itself intentionally never
        // yields.
        client.send_ok(&Request::ClickUnresponsiveWait).unwrap();
        let waited = client.snapshot().expect("wait action snapshot");
        assert_eq!(waited.active_web_view, Some(first));
        assert_eq!(waited.engine_sessions, 2);
        client
            .send_ok(&Request::InjectWebEvent {
                event: WebEvent::UnresponsiveChanged(first, false),
            })
            .unwrap();
        let cleared = wait_snapshot(&mut client, |s| !s.tabs[0].unresponsive);
        assert_eq!(cleared.tabs[0].web_view, Some(first));
        assert!(!cleared
            .visible_ids
            .iter()
            .any(|id| id == "bbr-unresponsive-panel"));

        // Mark again (the still-busy renderer misses a real probe), traverse
        // Wait → Reload, and activate Reload. The same tab survives with a
        // fresh disposable session in the same profile.
        let _ = wait_real_unresponsive(&mut client, first);
        client.send_ok(&Request::ClickUnresponsiveReload).unwrap();
        let reloaded = wait_snapshot(&mut client, |s| {
            s.tabs[0].web_view.is_some_and(|view| view != first) && !s.tabs[0].unresponsive
        });
        let fresh = reloaded.tabs[0].web_view.expect("fresh recovered view");
        assert_eq!(reloaded.tabs.len(), 2);
        assert_eq!(reloaded.engine_sessions, 2);
        assert_eq!(reloaded.tabs[1].web_view, Some(neighbor));

        // Late probe/dialog-class completions for the destroyed incarnation
        // are dropped by the ordinary UnknownWebView discipline.
        client
            .send_ok(&Request::InjectWebEvent {
                event: WebEvent::UnresponsiveChanged(first, true),
            })
            .unwrap();
        let stale = client.snapshot().expect("stale event snapshot");
        assert_eq!(stale.active_web_view, Some(fresh));
        assert!(!stale.tabs[0].unresponsive);
        assert_eq!(stale.engine_sessions, 2);

        // The Close button uses the ordinary close authority. Focus starts at
        // Wait; two Tabs reach Close.
        navigate(&mut client, &fixtures.url(BUSY_PATH));
        let _ = wait_real_unresponsive(&mut client, fresh);
        client.send_ok(&Request::ClickUnresponsiveClose).unwrap();
        let closed = wait_snapshot(&mut client, |s| s.tabs.len() == 1);
        assert_eq!(closed.active_web_view, Some(neighbor));
        assert_eq!(closed.engine_sessions, 1);
    }));
    finish_case(result, &mut harness, &socket, &token);
}
