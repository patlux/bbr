//! Real-app productive crash/recovery transport cases (renderer lifecycle
//! acceptance decision, A4).
//!
//! `Request::InjectWebEvent(PageCrashed)` feeds the same app event
//! pump/controller path as the native delegate sink (`on_web_event`), so
//! these scenarios prove the complete BBR-owned path from a typed crash
//! signal through the productive transport, controller session registry,
//! and UI. Injection proves only BBR-owned transport — it does **not**
//! claim to emulate the external WebKit → callback/error hop (A5, an
//! explicitly unpassed Apple platform contract).

use crate::*;
use bbr_testkit::protocol::{dsl, Request, TabVisualState};

/// Inject a `PageCrashed` for `view` through the productive typed event
/// pump and wait for the controller to apply it.
fn crash_view(client: &mut Client, view: WebViewId, since: &mut u64) {
    client
        .send_ok(&dsl::inject_web_event(WebEvent::PageCrashed(view)))
        .expect("inject crash through the productive pump");
    *since = wait_web_event(
        client,
        |event| matches!(event, WebEvent::PageCrashed(v) if *v == view),
        *since,
    );
}

#[test]
#[ignore]
fn active_crash_panel_exact_one_destroy_and_restore_in_same_profile_store() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, mut harness, socket, token, mut client, mut since) =
        start("renderer-crash-active");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let first = client.snapshot().expect("initial").active_web_view.unwrap();
        let neighbor = open_tab(
            &mut client,
            &mut since,
            &fixtures.url("/page2.html"),
            Some(PAGE_TWO_TITLE),
        );
        // The new tab activates on open; the crash panel is the active-tab
        // surface, so return to the first tab before crashing it.
        client.send_ok(&Request::SelectTab { index: 0 }).unwrap();
        let before = client.snapshot().expect("pre-crash snapshot");
        assert_eq!(before.engine_sessions, 2);
        assert_eq!(before.active_web_view, Some(first));

        // The active tab crashes through the productive typed pump: the
        // production crash panel/status semantics present, exactly one
        // disposable session is destroyed, and the tab identity survives.
        crash_view(&mut client, first, &mut since);
        let crashed = wait_snapshot(&mut client, |s| s.tabs[0].crashed);
        assert_eq!(crashed.tabs.len(), 2, "tab identity survives the crash");
        assert_eq!(crashed.engine_sessions, 1, "exactly one session destroyed");
        assert_eq!(crashed.tabs[0].web_view, None, "crashed session destroyed");
        assert_eq!(
            crashed.tabs[1].web_view,
            Some(neighbor),
            "sibling session untouched"
        );
        assert_eq!(crashed.tab_states[0], TabVisualState::Error);
        assert_ids(
            &crashed,
            &[
                "bbr-crash-panel",
                "bbr-crash-reload",
                "bbr-crash-close",
                "bbr-tab-row-0",
                "bbr-tab-state-0",
            ],
        );

        // Restore (the crash panel's Reload authority) recreates exactly one
        // fresh session for the same tab in the same profile store and
        // clears only that tab's crash state.
        client.send_ok(&dsl::reload()).expect("reload crashed tab");
        let recovered = wait_snapshot(&mut client, |s| {
            s.tabs[0].web_view.is_some_and(|view| view != first) && !s.tabs[0].crashed
        });
        let fresh = recovered.tabs[0].web_view.expect("fresh recovered view");
        assert_eq!(recovered.tabs.len(), 2, "same tab identity");
        assert_eq!(recovered.engine_sessions, 2, "exactly one fresh session");
        assert_eq!(recovered.active_web_view, Some(fresh));
        assert_eq!(
            recovered.tabs[0].url,
            fixtures.url("/page1.html"),
            "same URL in the same profile store"
        );
        assert_eq!(
            recovered.tabs[1].web_view,
            Some(neighbor),
            "sibling still live"
        );
        assert_eq!(
            recovered.active_profile, before.active_profile,
            "same profile store"
        );
    }));
    finish_case(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn split_member_crash_preserves_layout_and_sibling_consistency() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, mut harness, socket, token, mut client, mut since) =
        start("renderer-crash-split");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let first = client.snapshot().expect("initial").active_web_view.unwrap();
        let second = open_tab(
            &mut client,
            &mut since,
            &fixtures.url("/page2.html"),
            Some(PAGE_TWO_TITLE),
        );
        client
            .send_ok(&Request::SetSplit {
                primary: 0,
                secondary: 1,
            })
            .unwrap();
        let split = wait_snapshot(&mut client, |s| {
            s.pane_layout.is_some_and(|p| p.secondary.is_some())
        });
        assert_eq!(split.pane_layout.unwrap().secondary, Some(second));

        // One split member crashes: its session is destroyed and the tab
        // discarded. The split must never publish a false live pane — with
        // one member's session gone, the honest answer is no pane layout.
        crash_view(&mut client, first, &mut since);
        let crashed = wait_snapshot(&mut client, |s| s.tabs[0].crashed);
        assert_eq!(
            crashed.engine_sessions, 1,
            "crashed member session destroyed"
        );
        assert_eq!(
            crashed.tabs[1].web_view,
            Some(second),
            "sibling session survives"
        );
        assert!(!crashed.tabs[1].crashed, "sibling is not crashed");
        assert_eq!(
            crashed.pane_layout, None,
            "no false live pane after a member crash"
        );

        // Restore the crashed member: a fresh session returns and the
        // sibling stays consistent. Reload is the active-tab authority, so
        // select the crashed member first.
        client.send_ok(&Request::SelectTab { index: 0 }).unwrap();
        client
            .send_ok(&dsl::reload())
            .expect("reload crashed member");
        let recovered = wait_snapshot(&mut client, |s| {
            s.tabs[0].web_view.is_some_and(|view| view != first) && !s.tabs[0].crashed
        });
        assert_eq!(recovered.engine_sessions, 2);
        assert_eq!(
            recovered.tabs[1].web_view,
            Some(second),
            "sibling unchanged"
        );
    }));
    finish_case(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn multi_view_crash_burst_and_stale_event_after_recreation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, mut harness, socket, token, mut client, mut since) =
        start("renderer-crash-burst");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let first = client.snapshot().expect("initial").active_web_view.unwrap();
        let second = open_tab(
            &mut client,
            &mut since,
            &fixtures.url("/page2.html"),
            Some(PAGE_TWO_TITLE),
        );
        let third = open_tab(
            &mut client,
            &mut since,
            &fixtures.url("/__fx/slow"),
            Some(SLOW_TITLE),
        );
        let before = client.snapshot().expect("pre-burst snapshot");
        assert_eq!(before.engine_sessions, 3);

        // A shared-process burst: crash the middle view, then the last.
        // Invariants hold after every event — exactly one session destroyed
        // per crash, tab identity preserved, unaffected tabs keep sessions.
        crash_view(&mut client, second, &mut since);
        let after_first = wait_snapshot(&mut client, |s| s.tabs[1].crashed);
        assert_eq!(
            after_first.engine_sessions, 2,
            "exactly one session destroyed"
        );
        assert_eq!(after_first.tabs.len(), 3, "tab identity survives");
        assert_eq!(
            after_first.tabs[0].web_view,
            Some(first),
            "unaffected tab keeps session"
        );
        assert_eq!(
            after_first.tabs[2].web_view,
            Some(third),
            "unaffected tab keeps session"
        );

        crash_view(&mut client, third, &mut since);
        let after_second = wait_snapshot(&mut client, |s| s.tabs[2].crashed);
        assert_eq!(
            after_second.engine_sessions, 1,
            "exactly one more session destroyed"
        );
        assert_eq!(after_second.tabs.len(), 3, "tab identity survives");
        assert_eq!(
            after_second.tabs[0].web_view,
            Some(first),
            "unaffected tab keeps session"
        );
        assert!(after_second.tabs[1].crashed && after_second.tabs[2].crashed);

        // Recreate the first crashed tab, then deliver a stale crash for its
        // old incarnation: the fresh session is untouched.
        client.send_ok(&Request::SelectTab { index: 1 }).unwrap();
        client.send_ok(&dsl::reload()).expect("reload crashed tab");
        let recovered = wait_snapshot(&mut client, |s| {
            s.tabs[1].web_view.is_some_and(|view| view != second) && !s.tabs[1].crashed
        });
        let fresh = recovered.tabs[1].web_view.expect("fresh recovered view");
        assert_eq!(recovered.engine_sessions, 2);
        crash_view(&mut client, second, &mut since);
        let stale = wait_snapshot(&mut client, |s| s.active_web_view == Some(fresh));
        assert_eq!(stale.engine_sessions, 2, "stale crash destroys nothing");
        assert_eq!(
            stale.tabs[1].web_view,
            Some(fresh),
            "fresh session unaffected"
        );
        assert!(
            !stale.tabs[1].crashed,
            "stale crash cannot re-crash the fresh session"
        );
    }));
    finish_case(result, &mut harness, &socket, &token);
}
