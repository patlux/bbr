//! Dense-list controller tests: the deterministic session budget
//! (`enforce_session_budget`) over the fake engine, lifecycle counts in the
//! snapshot, and the no-500-concurrent-views invariant.
//!
//! The budget runs through the production controller mutation paths
//! (`open_tab`/`pin_tab`/`suspend_tab`/`select_tab` + `enforce_session_budget`)
//! against the deterministic `FakeEngine`, so the engine op log records
//! exactly which sessions were created and destroyed and the live-session
//! peak can be asserted directly.

use bbr_web::{WebEvent, WebViewId};

use crate::fake::EngineOp;
use crate::harness::{open, plain};
use crate::{ControllerEvent, TabLifecycle};

/// Seed `count` tabs in the default space, pinning the first `pinned` and
/// returning the resulting snapshot counts.
#[test]
fn dense_seed_keeps_exactly_the_live_cap_with_a_bounded_transient_peak() {
    let (mut c, h) = plain();
    let url = |i: usize| format!("https://dense{i}.example");
    let mut peak_sessions = 0usize;

    // Open every tab through the production open path; immediately make it
    // hidden and suspend the ones outside the keep set (pinned 0..10 +
    // recent 460..500 + active 0). Each transient session is destroyed before
    // the next open, so the concurrent WKWebView population stays at
    // live_cap + 1 — never 500.
    for i in 0..500 {
        open(&mut c, &url(i));
        let keep = !(10..460).contains(&i);
        if !keep {
            // The fresh tab is visible (Active); hiding it first is the
            // legal Warm → Suspended path the budget policy uses.
            c.select_tab(0).expect("hide transient");
            let event = c.suspend_tab(i).expect("suspend transient");
            assert!(matches!(event, ControllerEvent::TabSuspended { index } if index == i));
        }
        peak_sessions = peak_sessions.max(c.snapshot().engine_sessions);
    }
    assert!(
        peak_sessions <= 51,
        "preparing dense state must never hold 500 live sessions; peak {peak_sessions}"
    );

    // Activate tab 0 (pinned) as the visible active tab.
    c.select_tab(0).expect("activate pinned tab");
    let budget = c
        .enforce_session_budget(50)
        .expect("budget applies cleanly");
    assert_eq!(
        budget,
        ControllerEvent::LifecycleBudgetApplied {
            live: 50,
            suspended: 0
        },
        "seeding already matches the 50-session cap"
    );

    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 500);
    assert_eq!(snap.engine_sessions, 50);
    let counts = snap.lifecycle_counts;
    assert_eq!(
        counts,
        crate::LifecycleCounts {
            active: 1,
            warm: 49,
            suspended: 450,
            discarded: 0,
        }
    );
    // Session identity: exactly the kept 50 tabs own a view, the active and
    // every pinned tab among them.
    let live_views: Vec<WebViewId> = snap.tabs.iter().filter_map(|t| t.web_view).collect();
    assert_eq!(live_views.len(), 50);
    assert!(snap.tabs.iter().enumerate().all(|(index, t)| {
        if !(10..460).contains(&index) {
            t.web_view.is_some()
        } else {
            t.web_view.is_none()
        }
    }));
    assert_eq!(snap.tabs[0].lifecycle, TabLifecycle::Active);
    assert_eq!(h.engine.borrow().live_session_count(), 50);
    // The engine op log proves every non-kept session was destroyed.
    let ops = h.ops();
    let creates = ops
        .iter()
        .filter(|op| matches!(op, EngineOp::Create { .. }))
        .count();
    let destroys = ops
        .iter()
        .filter(|op| matches!(op, EngineOp::Destroy { .. }))
        .count();
    assert_eq!(
        creates, 500,
        "every seeded tab ran the production create path"
    );
    assert_eq!(destroys, 450, "every non-kept session was destroyed");
}

#[test]
fn budget_suspends_least_recently_used_and_keeps_identity_stability() {
    let (mut c, h) = plain();
    for i in 0..12 {
        open(&mut c, &format!("https://budget{i}.example"));
    }
    // Pin 0 and 1; select 5 (recency: 5 newest, then 11, 10, …).
    c.pin_tab(0).expect("pin 0");
    c.pin_tab(1).expect("pin 1");
    c.select_tab(5).expect("select 5");
    let before = c.snapshot();
    let kept_views: std::collections::HashMap<usize, WebViewId> = before
        .tabs
        .iter()
        .filter(|t| t.index < 2 || t.index == 5 || t.index == 11)
        .map(|t| (t.index, t.web_view.unwrap()))
        .collect();

    // Cap 4: kept = visible (5) + pinned (0, 1) + most recent (11). The rest
    // suspend; their views are destroyed but the identities survive.
    let event = c.enforce_session_budget(4).expect("budget");
    assert_eq!(
        event,
        ControllerEvent::LifecycleBudgetApplied {
            live: 4,
            suspended: 8
        }
    );
    let snap = c.snapshot();
    assert_eq!(snap.engine_sessions, 4);
    assert_eq!(snap.lifecycle_counts.warm, 3);
    assert_eq!(snap.lifecycle_counts.suspended, 8);
    for (index, tab) in snap.tabs.iter().enumerate() {
        if index == 5 {
            assert_eq!(
                tab.lifecycle,
                TabLifecycle::Active,
                "the visible selected tab stays Active"
            );
            assert_eq!(
                tab.web_view,
                Some(kept_views[&5]),
                "the visible tab keeps its session"
            );
        } else if index < 2 || index == 11 {
            assert_eq!(
                tab.web_view,
                Some(kept_views[&index]),
                "kept tab {index} keeps its session"
            );
            assert_eq!(tab.lifecycle, TabLifecycle::Warm);
        } else {
            assert!(tab.web_view.is_none(), "suspended tab {index} has no view");
            assert_eq!(tab.lifecycle, TabLifecycle::Suspended);
        }
    }
    assert_eq!(snap.lifecycle_counts.active, 1);
    // Re-running is a stable no-op (identity stability for repeat calls).
    let again = c.enforce_session_budget(4).expect("budget again");
    assert_eq!(
        again,
        ControllerEvent::LifecycleBudgetApplied {
            live: 4,
            suspended: 0
        }
    );
    let _ = h;
}

#[test]
fn selecting_a_suspended_tab_recreates_exactly_one_session() {
    let (mut c, h) = plain();
    for i in 0..8 {
        open(&mut c, &format!("https://recreate{i}.example"));
    }
    // Shrink to a 2-session budget: 6 suspended, the 2 most recent live.
    let _ = c.enforce_session_budget(2).expect("budget");
    let before = c.snapshot();
    assert_eq!(before.engine_sessions, 2);
    let suspended_index = 0;
    assert!(before.tabs[suspended_index].web_view.is_none());

    // Selecting the suspended tab recreates exactly one session and presents
    // it; the session count grows by exactly one.
    let ops_before = h.ops().len();
    let event = c.select_tab(suspended_index).expect("select suspended");
    assert!(matches!(event, ControllerEvent::TabSelected { index: 0 }));
    let snap = c.snapshot();
    assert_eq!(snap.engine_sessions, 3);
    assert_eq!(snap.tabs[suspended_index].lifecycle, TabLifecycle::Active);
    let fresh = snap.tabs[suspended_index].web_view.expect("recreated view");
    let creates_after = h
        .ops()
        .into_iter()
        .skip(ops_before)
        .filter(|op| matches!(op, EngineOp::Create { view, .. } if *view == fresh))
        .count();
    assert_eq!(
        creates_after, 1,
        "exactly one fresh session for the selection"
    );
}

#[test]
fn warm_selection_never_recreates_its_session() {
    let (mut c, h) = plain();
    for i in 0..6 {
        open(&mut c, &format!("https://warm{i}.example"));
    }
    let _ = c.enforce_session_budget(2).expect("budget");
    let before = c.snapshot();
    // The two most recent tabs (4, 5) are warm and own views.
    let warm_view = before.tabs[4].web_view.expect("warm view");
    let ops_before = h.ops();
    c.select_tab(4).expect("select warm");
    let snap = c.snapshot();
    assert_eq!(
        snap.tabs[4].web_view,
        Some(warm_view),
        "warm selection reuses the existing session"
    );
    assert_eq!(
        snap.engine_sessions, 2,
        "no session churn on warm selection"
    );
    // Selection may show/hide sessions, but it never creates or destroys one.
    let churn: Vec<String> = h
        .ops()
        .into_iter()
        .skip(ops_before.len())
        .filter(|op| !matches!(op, EngineOp::Show { .. }))
        .map(|op| format!("{op:?}"))
        .collect();
    assert!(
        churn.is_empty(),
        "warm selection issued non-show engine ops: {churn:?}"
    );
}

#[test]
fn seeded_tabs_carry_deterministic_recency_in_the_snapshot() {
    let (mut c, _h) = plain();
    for i in 0..5 {
        open(&mut c, &format!("https://recency{i}.example"));
    }
    let first = c.snapshot().tabs[0].last_used;
    c.select_tab(1).expect("select 1");
    let snap = c.snapshot();
    assert!(
        snap.tabs[1].last_used > first,
        "selection bumps the tab's recency stamp"
    );
    assert!(
        snap.tabs[1].last_used > snap.tabs[4].last_used,
        "the selected tab is the most recent"
    );
}

#[test]
fn crashed_and_web_events_keep_lifecycle_counts_consistent() {
    let (mut c, h) = plain();
    for i in 0..4 {
        open(&mut c, &format!("https://crash{i}.example"));
    }
    let snap = c.snapshot();
    let active_view = snap.active_view.expect("active view");
    // A page crash discards the visible tab: its session is destroyed.
    c.apply_event(WebEvent::PageCrashed(active_view));
    let snap = c.snapshot();
    assert_eq!(snap.lifecycle_counts.discarded, 1);
    assert_eq!(snap.engine_sessions, 3);
    assert_eq!(snap.lifecycle_counts.live(), 3);
    assert!(snap.tabs[3].crashed);
    let _ = h;
}
