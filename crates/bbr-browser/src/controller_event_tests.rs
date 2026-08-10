//! Controller contract tests — split behavior and event application
//! (browser-controller.md, "Tests and evidence").
//!
//! Split behavior: pane activation, focus routing, teardown without orphaned
//! sessions. Event application: correct-tab mapping, nav-state tracking,
//! crash/failure outcomes, stale-id rejection, and registry/engine agreement
//! under churn. Uses the same fake-driven harness as `controller_tests`.

use bbr_web::{WebAction, WebEvent, WebViewId};

use crate::fake::EngineOp;
use crate::harness::{open, plain};
use crate::nav::NavState;
use crate::ControllerEvent;

// ---------------------------------------------------------------------------
// Split behavior
// ---------------------------------------------------------------------------

#[test]
fn opening_a_split_activates_and_shows_both_pane_sessions() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    h.reset_ops();
    c.open_split(0, 1).unwrap();
    assert!(h.recorded(&[EngineOp::Show {
        view: WebViewId(1),
        visible: true
    }]));
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 2);
    assert_eq!(snap.engine_sessions, 2, "both panes own live sessions");
    // Exactly one pane stays marked active: the previously focused tab (the
    // focused pane), while the other pane is shown but not focused.
    assert_eq!(snap.tabs.iter().filter(|t| t.active).count(), 1);
    assert!(snap.tabs[1].active, "secondary pane keeps the focus");
}

#[test]
fn the_focused_split_pane_receives_navigation_commands() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    c.open_split(0, 1).unwrap();
    // The domain kept the previously active tab (secondary) focused.
    h.reset_ops();
    c.navigate(WebAction::Reload).unwrap();
    assert!(h.recorded(&[EngineOp::Action {
        view: WebViewId(2),
        action: WebAction::Reload
    }]));
    // Selecting the other pane (a split member) re-targets routing.
    c.select_tab(0).unwrap();
    h.reset_ops();
    c.navigate(WebAction::Reload).unwrap();
    assert!(h.recorded(&[EngineOp::Action {
        view: WebViewId(1),
        action: WebAction::Reload
    }]));
}

#[test]
fn closing_the_split_warms_the_non_selected_pane_without_closing_its_tab() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    c.open_split(0, 1).unwrap();
    c.select_tab(0).unwrap(); // focus the primary pane
    h.reset_ops();
    assert_eq!(c.close_split().unwrap(), ControllerEvent::SplitClosed);
    assert!(h.recorded(&[EngineOp::Show {
        view: WebViewId(2),
        visible: false
    }]));
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 2, "both tabs stay open");
    assert_eq!(snap.active_index, Some(0));
    assert_eq!(snap.engine_sessions, 2, "non-selected pane stays warm");
}

#[test]
fn closing_a_split_pane_dissolves_the_split_without_orphaning_sessions() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    c.open_split(0, 1).unwrap();
    h.reset_ops();
    c.close_tab(0).unwrap();
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 1, "the other pane's tab survives");
    assert_eq!(snap.active_index, Some(0));
    assert_eq!(snap.engine_sessions, 1, "no orphaned sessions");
    assert!(h
        .ops()
        .iter()
        .any(|op| matches!(op, EngineOp::Destroy { view: WebViewId(1) })));
}

// ---------------------------------------------------------------------------
// Event application
// ---------------------------------------------------------------------------

#[test]
fn url_and_title_events_apply_to_the_owning_tab_only() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    assert_eq!(
        c.apply_event(WebEvent::UrlChanged(WebViewId(1), "https://a".into())),
        ControllerEvent::WebEvent {
            event: WebEvent::UrlChanged(WebViewId(1), "https://a".into())
        }
    );
    c.apply_event(WebEvent::TitleChanged(WebViewId(1), "A".into()));
    let snap = c.snapshot();
    assert_eq!(snap.tabs[0].url, "https://a");
    assert_eq!(snap.tabs[0].title, "A");
    assert_eq!(snap.nav.url.as_deref(), Some("https://a"));
    assert_eq!(snap.nav.title.as_deref(), Some("A"));
}

#[test]
fn loading_and_history_are_tracked_in_nav_state_not_domain_data() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    c.apply_event(WebEvent::LoadingChanged(WebViewId(1), true));
    c.apply_event(WebEvent::CanGoBackChanged(WebViewId(1), true));
    c.apply_event(WebEvent::CanGoForwardChanged(WebViewId(1), false));
    let snap = c.snapshot();
    assert!(snap.nav.loading);
    assert!(snap.nav.can_go_back);
    assert_eq!(
        snap.nav,
        NavState {
            url: None,
            title: None,
            loading: true,
            can_go_back: true,
            can_go_forward: false,
            crashed: false,
            failed: false,
            initial_committed: false,
        }
    );
    // The persistent tab identity is untouched by transient state.
    assert_eq!(snap.tabs[0].url, "u1");
}

#[test]
fn a_crash_is_a_typed_outcome_and_stops_loading() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    c.apply_event(WebEvent::LoadingChanged(WebViewId(1), true));
    let outcome = c.apply_event(WebEvent::PageCrashed(WebViewId(1)));
    assert_eq!(
        outcome,
        ControllerEvent::TabDiscarded {
            tab: bbr_domain::TabId(3)
        }
    );
    assert_eq!(c.snapshot().engine_sessions, 0);
}

#[test]
fn failed_navigation_stops_loading_and_emits_a_web_event_outcome() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    c.apply_event(WebEvent::LoadingChanged(WebViewId(1), true));
    let outcome = c.apply_event(WebEvent::NavigationFailed(WebViewId(1), "boom".into()));
    assert_eq!(
        outcome,
        ControllerEvent::WebEvent {
            event: WebEvent::NavigationFailed(WebViewId(1), "boom".into())
        }
    );
    assert!(!c.snapshot().nav.loading, "failure stops loading");
}

#[test]
fn stale_webview_events_are_reported_without_corrupting_state() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    let before = c.snapshot();
    let outcome = c.apply_event(WebEvent::UrlChanged(WebViewId(99), "https://evil".into()));
    assert_eq!(
        outcome,
        ControllerEvent::UnknownWebView {
            view: WebViewId(99)
        }
    );
    assert_eq!(c.snapshot(), before, "no state may change for unknown ids");
    // A late event after the session was destroyed is also reported.
    c.close_tab(0).unwrap();
    let outcome = c.apply_event(WebEvent::TitleChanged(WebViewId(1), "late".into()));
    assert_eq!(
        outcome,
        ControllerEvent::UnknownWebView { view: WebViewId(1) }
    );
}

// ---------------------------------------------------------------------------
// Invariants under churn
// ---------------------------------------------------------------------------

#[test]
fn registry_and_engine_agree_through_a_long_mutation_sequence() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    open(&mut c, "u3");
    c.suspend_tab(1).unwrap();
    c.select_tab(0).unwrap();
    c.close_tab(0).unwrap();
    c.reopen_tab().unwrap();
    c.navigate(WebAction::Reload).unwrap();
    // The controller's check_invariants ran after every mutation above; the
    // engine's physical count must match the registry's bookkeeping.
    assert_eq!(
        h.engine.borrow().live_session_count(),
        c.snapshot().engine_sessions
    );
    assert!(!c.snapshot().tabs.is_empty());
}
