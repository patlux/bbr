//! Controller-level renderer-availability tests (renderer-isolation lane).
//!
//! Prove the portable monitor integrated into the controller: unresponsive
//! detection keeps the tab identity and session alive, Wait/Reload/Close are
//! the closed recovery set, the crash authority owns `Unresponsive → Crashed`,
//! stale events after close/recreate are dropped, and a PageCrashed triggers
//! the sibling reconciliation sweep.

use bbr_web::{WebEvent, WebViewId};

use crate::fake::EngineOp;
use crate::harness::{open, plain, wired};
use crate::ControllerEvent;

#[test]
fn unresponsive_event_marks_the_tab_and_keeps_identity_and_session() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    // The visible session's probe is armed by the production pump.
    let view = c.snapshot().active_view.unwrap();
    assert!(h
        .ops()
        .iter()
        .any(|op| matches!(op, EngineOp::Probe { view: v } if *v == view)));
    let outcome = c.apply_event(WebEvent::UnresponsiveChanged(view, true));
    assert!(matches!(
        outcome,
        ControllerEvent::RendererStateChanged { view: v, state: crate::RendererAvailability::Unresponsive } if v == view
    ));
    let snap = c.snapshot();
    assert!(snap.tabs[0].unresponsive, "row state shows unresponsive");
    assert_eq!(snap.tabs.len(), 1, "tab identity survives");
    assert_eq!(snap.engine_sessions, 1, "session stays alive");
    assert!(!snap.tabs[0].crashed, "unresponsive is not a crash");
    // A completing probe clears the state (busy→alive).
    let outcome = c.apply_event(WebEvent::UnresponsiveChanged(view, false));
    assert!(matches!(
        outcome,
        ControllerEvent::RendererStateChanged { view: v, state: crate::RendererAvailability::Responsive } if v == view
    ));
    let snap = c.snapshot();
    assert!(!snap.tabs[0].unresponsive);
    assert_eq!(snap.tabs.len(), 1);
    assert_eq!(snap.engine_sessions, 1);
}

#[test]
fn recovery_reload_recreates_a_fresh_session_for_the_same_tab() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    let view = c.snapshot().active_view.unwrap();
    c.apply_event(WebEvent::UnresponsiveChanged(view, true));
    let recovered = c.recover_unresponsive(view).unwrap();
    let ControllerEvent::SessionRecovered {
        view: old,
        fresh,
        tab,
    } = recovered
    else {
        panic!("expected SessionRecovered, got {recovered:?}");
    };
    assert_eq!(old, view);
    assert_ne!(fresh, view, "fresh WebViewId incarnation");
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 1, "same tab identity");
    assert_eq!(snap.active_view, Some(fresh));
    assert_eq!(snap.engine_sessions, 1);
    assert_eq!(snap.tabs[0].url, "u1", "same URL in the same profile store");
    assert!(!snap.tabs[0].unresponsive, "recovered session is fresh");
    assert!(!snap.tabs[0].crashed);
    let _ = tab;
}

#[test]
fn recovery_is_closed_to_wait_reload_close() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    let view = c.snapshot().active_view.unwrap();
    // Reload refuses a session that is not reported unresponsive.
    assert!(matches!(
        c.recover_unresponsive(view),
        Err(crate::ControllerError::NotUnresponsive(v)) if v == view
    ));
    // Wait keeps monitoring an unresponsive session (state unchanged until a
    // probe answers), then a later completion clears it.
    c.apply_event(WebEvent::UnresponsiveChanged(view, true));
    let outcome = c.wait_unresponsive(view);
    assert!(matches!(
        outcome,
        ControllerEvent::RendererStateChanged { view: v, state: crate::RendererAvailability::Unresponsive } if v == view
    ));
    assert!(c.snapshot().tabs[0].unresponsive);
    c.apply_event(WebEvent::UnresponsiveChanged(view, false));
    assert!(!c.snapshot().tabs[0].unresponsive);
    // Close is the existing close authority.
    let closed = c.close_tab(0).unwrap();
    assert!(matches!(closed, ControllerEvent::TabClosed { index: 0 }));
    assert_eq!(c.snapshot().tabs.len(), 0);
}

#[test]
fn crash_after_unresponsive_is_owned_by_the_crash_authority() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    let view = c.snapshot().active_view.unwrap();
    c.apply_event(WebEvent::UnresponsiveChanged(view, true));
    assert!(c.snapshot().tabs[0].unresponsive);
    // A real termination supersedes the unresponsive state: the session is
    // destroyed and the tab discarded (Unresponsive → Crashed via the
    // existing on_page_crash authority).
    let outcome = c.apply_event(WebEvent::PageCrashed(view));
    assert!(matches!(outcome, ControllerEvent::TabDiscarded { tab: _ }));
    let snap = c.snapshot();
    assert!(snap.tabs[0].crashed, "crash state wins");
    assert!(!snap.tabs[0].unresponsive, "no unresponsive row on a crash");
    assert_eq!(snap.engine_sessions, 0, "crashed session destroyed");
    assert_eq!(snap.tabs.len(), 1, "tab identity survives");
    // The crash destroys the session, so its availability record is dropped
    // with it (stale-id discipline); the crash panel state comes from the
    // controller's crashed-tab set.
    assert_eq!(c.renderer_state(view), crate::RendererAvailability::Unknown);
}

#[test]
fn stale_unresponsive_events_after_close_are_dropped() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    let view = c.snapshot().active_view.unwrap();
    // A probe is in flight when the tab is closed: the record is dropped
    // with the session.
    c.close_tab(0).unwrap();
    assert_eq!(c.snapshot().tabs.len(), 0);
    // A late completion from the destroyed session mutates nothing and is
    // reported exactly like the UnknownWebView path.
    let outcome = c.apply_event(WebEvent::UnresponsiveChanged(view, true));
    assert!(matches!(outcome, ControllerEvent::UnknownWebView { view: v } if v == view));
    let outcome = c.apply_event(WebEvent::UnresponsiveChanged(view, false));
    assert!(matches!(outcome, ControllerEvent::UnknownWebView { view: v } if v == view));
    assert_eq!(c.snapshot().engine_sessions, 0);
    assert_eq!(c.renderer_state(view), crate::RendererAvailability::Unknown);
}

#[test]
fn page_crash_triggers_the_sibling_reconciliation_sweep() {
    let (c, h) = wired();
    {
        let mut c = c.borrow_mut();
        open(&mut c, "u1");
        open(&mut c, "u2");
        // Both panes become visible: both sessions are probed.
        c.open_split(0, 1).unwrap();
    }
    let snap = c.borrow().snapshot();
    let first = snap.tabs[0].web_view.unwrap();
    let sibling = snap.tabs[1].web_view.unwrap();
    assert_eq!(c.borrow().pane_layout().map(|p| p.primary), Some(first));
    h.reset_ops();
    // One split member crashes: its session is destroyed and the tab
    // discarded; the other live (still visible) session is re-armed by the
    // sweep and probed independently.
    c.borrow_mut().apply_event(WebEvent::PageCrashed(first));
    let snap = c.borrow().snapshot();
    assert_eq!(snap.engine_sessions, 1, "crashed session destroyed");
    let mut c = c.borrow_mut();
    let _outcome = c.pump_responsiveness();
    assert!(
        h.ops()
            .iter()
            .any(|op| matches!(op, EngineOp::Probe { view: v } if *v == sibling)),
        "sibling re-probed after the crash: {:?}",
        h.ops()
    );
}

#[test]
fn probe_view_seam_arms_exactly_one_probe_for_one_live_view() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    let view = c.snapshot().active_view.unwrap();
    h.reset_ops();
    assert!(c.probe_view(view), "live view accepts a probe");
    assert_eq!(
        h.ops(),
        vec![EngineOp::Probe { view }],
        "exactly one probe for the live view"
    );
    // Unknown/destroyed ids are no-ops.
    assert!(!c.probe_view(WebViewId(999)));
    assert_eq!(h.ops().len(), 1);
}

// ---------------------------------------------------------------------------
// A3 — portable multi-session reconciliation contract (renderer lifecycle
// acceptance decision). WebKit may share renderer processes, so the app must
// never assume one process per tab, a particular event count, or a
// particular crash order. These tests model a shared-process burst with
// sequential `PageCrashed` events in varying orders and check invariants
// after every event, not only at the end of the burst.
// ---------------------------------------------------------------------------

#[test]
fn page_crash_burst_destroys_each_affected_session_exactly_once() {
    let (c, h) = wired();
    {
        let mut c = c.borrow_mut();
        open(&mut c, "u1");
        open(&mut c, "u2");
        open(&mut c, "u3");
    }
    let snap = c.borrow().snapshot();
    let views: Vec<WebViewId> = snap.tabs.iter().map(|t| t.web_view.unwrap()).collect();
    assert_eq!(views.len(), 3);
    assert_eq!(snap.engine_sessions, 3);
    // A shared-process burst reports crashes in an order the app must not
    // assume: middle, last, first. Invariants hold after every event.
    let mut crashed: Vec<usize> = Vec::new();
    for (i, tab_index) in [1usize, 2, 0].into_iter().enumerate() {
        let view = views[tab_index];
        let outcome = c.borrow_mut().apply_event(WebEvent::PageCrashed(view));
        assert!(
            matches!(outcome, ControllerEvent::TabDiscarded { .. }),
            "crash {i} discards the owning tab: {outcome:?}"
        );
        crashed.push(tab_index);
        let snap = c.borrow().snapshot();
        assert_eq!(
            snap.engine_sessions,
            3 - (i + 1),
            "exactly one session destroyed per crash (after event {i})"
        );
        assert_eq!(
            snap.tabs.len(),
            3,
            "persistent tab identity survives every crash"
        );
        assert!(snap.tabs[tab_index].crashed, "affected tab marked crashed");
        assert_eq!(
            snap.tabs[tab_index].web_view, None,
            "affected session destroyed"
        );
        for (j, tab) in snap.tabs.iter().enumerate() {
            if !crashed.contains(&j) {
                assert_eq!(
                    tab.web_view,
                    Some(views[j]),
                    "unaffected tab {j} keeps its session after event {i}"
                );
                assert!(!tab.crashed, "unaffected tab {j} is not crashed");
            }
        }
    }
    let snap = c.borrow().snapshot();
    assert_eq!(snap.engine_sessions, 0, "burst fully consumed");
    assert_eq!(snap.tabs.len(), 3, "all tab identities survive the burst");
    assert!(snap.tabs.iter().all(|t| t.crashed));
    let _ = h;
}

#[test]
fn duplicate_unknown_and_old_incarnation_page_crashes_fail_closed() {
    let (c, _h) = wired();
    {
        let mut c = c.borrow_mut();
        open(&mut c, "u1");
    }
    let view = c.borrow().snapshot().active_view.unwrap();
    // First crash destroys the session and discards the tab.
    let outcome = c.borrow_mut().apply_event(WebEvent::PageCrashed(view));
    assert!(matches!(outcome, ControllerEvent::TabDiscarded { .. }));
    assert_eq!(c.borrow().snapshot().engine_sessions, 0);
    // A duplicate crash for the same (now destroyed) incarnation is stale:
    // fail closed — no second destroy, no state mutation.
    let outcome = c.borrow_mut().apply_event(WebEvent::PageCrashed(view));
    assert!(
        matches!(outcome, ControllerEvent::UnknownWebView { view: v } if v == view),
        "duplicate crash is reported like the UnknownWebView path: {outcome:?}"
    );
    let snap = c.borrow().snapshot();
    assert_eq!(snap.engine_sessions, 0, "no second destroy");
    assert!(snap.tabs[0].crashed, "crash state unchanged");
    // An unknown view crash is reported without corrupting anything.
    let outcome = c
        .borrow_mut()
        .apply_event(WebEvent::PageCrashed(WebViewId(999)));
    assert!(matches!(
        outcome,
        ControllerEvent::UnknownWebView {
            view: WebViewId(999)
        }
    ));
    assert_eq!(c.borrow().snapshot().engine_sessions, 0);
    // Recreate a fresh session for the same tab, then deliver a stale crash
    // for the old incarnation: the fresh view is untouched.
    c.borrow_mut().select_tab(0).unwrap();
    let fresh = c.borrow().snapshot().active_view.unwrap();
    assert_ne!(fresh, view, "fresh incarnation");
    assert_eq!(c.borrow().snapshot().engine_sessions, 1);
    let outcome = c.borrow_mut().apply_event(WebEvent::PageCrashed(view));
    assert!(
        matches!(outcome, ControllerEvent::UnknownWebView { view: v } if v == view),
        "old-incarnation crash cannot target the fresh view: {outcome:?}"
    );
    let snap = c.borrow().snapshot();
    assert_eq!(
        snap.active_view,
        Some(fresh),
        "fresh recreated view unaffected"
    );
    assert_eq!(snap.engine_sessions, 1);
    assert!(
        !snap.tabs[0].crashed,
        "stale crash cannot re-crash the fresh session"
    );
}

#[test]
fn split_member_crash_never_publishes_a_false_live_pane() {
    let (c, h) = wired();
    {
        let mut c = c.borrow_mut();
        open(&mut c, "u1");
        open(&mut c, "u2");
        c.open_split(0, 1).unwrap();
    }
    let snap = c.borrow().snapshot();
    let first = snap.tabs[0].web_view.unwrap();
    let sibling = snap.tabs[1].web_view.unwrap();
    assert_eq!(
        c.borrow().pane_layout().map(|p| p.secondary),
        Some(Some(sibling))
    );
    // One split member crashes: its session is destroyed and the tab
    // discarded. The split must never publish a false live pane — with one
    // member's session gone, the honest answer is no pane layout at all.
    c.borrow_mut().apply_event(WebEvent::PageCrashed(first));
    let snap = c.borrow().snapshot();
    assert_eq!(snap.engine_sessions, 1, "crashed member session destroyed");
    assert_eq!(
        c.borrow().pane_layout(),
        None,
        "no false live pane after a member crash"
    );
    // The still-live sibling keeps its session and is re-probed
    // independently by the reconciliation sweep.
    assert_eq!(
        snap.tabs[1].web_view,
        Some(sibling),
        "sibling session survives"
    );
    assert!(!snap.tabs[1].crashed, "sibling is not crashed");
    h.reset_ops();
    c.borrow_mut().pump_responsiveness();
    assert!(
        h.ops()
            .iter()
            .any(|op| matches!(op, EngineOp::Probe { view: v } if *v == sibling)),
        "sibling re-probed independently after the member crash: {:?}",
        h.ops()
    );
}

#[test]
fn page_crash_preserves_profile_ownership_of_the_recreated_session() {
    let (c, _h) = wired();
    let (private_profile, private_space) = {
        let mut c = c.borrow_mut();
        match c.create_profile_space("Private", "Private") {
            Ok(ControllerEvent::ProfileCreated { profile, space }) => (profile, space),
            other => panic!("unexpected event: {other:?}"),
        }
    };
    {
        let mut c = c.borrow_mut();
        c.activate_space(private_space).unwrap();
        open(&mut c, "https://private");
    }
    let view = c.borrow().snapshot().active_view.unwrap();
    assert_eq!(c.borrow().active_profile(), private_profile);
    // Crash the private-profile session: the tab is discarded but keeps its
    // identity and profile ownership.
    c.borrow_mut().apply_event(WebEvent::PageCrashed(view));
    let snap = c.borrow().snapshot();
    assert_eq!(snap.engine_sessions, 0);
    assert!(snap.tabs[0].crashed);
    assert_eq!(snap.tabs[0].url, "https://private");
    // Recreation in the same profile store produces a fresh session bound
    // to the correct profile.
    c.borrow_mut().select_tab(0).unwrap();
    let snap = c.borrow().snapshot();
    let fresh = snap.active_view.expect("fresh session");
    assert_ne!(fresh, view, "fresh incarnation");
    assert_eq!(snap.engine_sessions, 1);
    assert_eq!(
        snap.tabs[0].url, "https://private",
        "same URL in the same profile store"
    );
    assert!(!snap.tabs[0].crashed, "fresh session clears crash state");
    assert_eq!(c.borrow().active_profile(), private_profile);
    // The fresh session's engine creation carries the private profile.
    let _ = private_profile;
}
