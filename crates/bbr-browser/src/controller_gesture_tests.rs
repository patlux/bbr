//! N1 controller contract tests — the portable content-gesture decision
//! matrix, session-local opener provenance lifecycle, and the atomic
//! Back-at-settled-root close-child/activate-opener transaction
//! (task packet `web-content-navigation-gestures`).
//!
//! Every scenario drives the [`BrowserController`] over the deterministic
//! [`FakeEngine`] through the same `Rc<RefCell<dyn WebEngine>>` seam the
//! composition root uses, with engine events applied through the controller's
//! `apply_event` exactly like the wired sink. The fake's operation log proves
//! exactly-once dispatch/close/activate; no macOS frameworks, filesystem, or
//! wall clock.

use bbr_web::{
    decide, ContentGestureCommit, ContentGestureDirection, NewWindowDecision, NewWindowFeatures,
    NewWindowOutcome, NewWindowRequest, NewWindowTarget, ProfileId, WebAction, WebEvent, WebViewId,
};

use crate::fake::EngineOp;
use crate::harness::{open, plain};
use crate::{BrowserController, ControllerEvent};

fn new_window_session(
    source: WebViewId,
    profile: ProfileId,
    url: &str,
) -> bbr_web::NewWindowSession {
    let request = NewWindowRequest {
        source,
        source_profile: profile,
        url: url.into(),
        user_initiated: true,
        target: NewWindowTarget::NewWindow,
        features: NewWindowFeatures::default(),
    };
    match decide(&request, NewWindowDecision::OpenTab) {
        NewWindowOutcome::CreateSession { session } => session,
        other => panic!("expected session, got {other:?}"),
    }
}

/// Open an opener tab, then a regular child through the productive
/// new-window path (the only provenance creator). Returns `(child_view,
/// opener_view)` after the child's initial document committed.
fn open_child(c: &mut BrowserController) -> (WebViewId, WebViewId) {
    open(c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();
    c.apply_event(WebEvent::DidCommitNavigation(child));
    (child, opener)
}

/// Open a child whose initial navigation failed (settled failure root).
fn open_child_with_failure(c: &mut BrowserController) -> (WebViewId, WebViewId) {
    open(c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://broken.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();
    c.apply_event(WebEvent::NavigationFailed(
        child,
        "connection refused".into(),
    ));
    (child, opener)
}

/// Resolve the persistent `TabId` of the tab currently hosting `view`
/// (provenance is `TabId`-keyed by contract — never `WebViewId`-keyed).
fn tab_of(c: &BrowserController, view: WebViewId) -> bbr_domain::TabId {
    c.snapshot()
        .all_tabs
        .iter()
        .find(|t| t.web_view == Some(view))
        .expect("tab hosting view")
        .id
}

fn back(view: WebViewId) -> ContentGestureCommit {
    ContentGestureCommit {
        view,
        direction: ContentGestureDirection::Back,
    }
}

fn forward(view: WebViewId) -> ContentGestureCommit {
    ContentGestureCommit {
        view,
        direction: ContentGestureDirection::Forward,
    }
}

// ---------------------------------------------------------------------------
// Gesture decision matrix
// ---------------------------------------------------------------------------

#[test]
fn back_with_history_dispatches_exactly_one_back_to_the_bound_view() {
    let (mut c, h) = plain();
    open(&mut c, "https://a.example");
    let view = c.snapshot().active_view.unwrap();
    c.apply_event(WebEvent::CanGoBackChanged(view, true));
    h.reset_ops();

    assert_eq!(
        c.on_content_gesture(back(view)),
        Some(ControllerEvent::GestureHistoryRouted { view })
    );
    assert_eq!(
        h.ops(),
        vec![EngineOp::Action {
            view,
            action: WebAction::Back,
        }],
        "exactly one Back action, never a tab mutation"
    );
    // The tab/session identity is untouched by a history gesture.
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 1);
    assert_eq!(snap.active_view, Some(view));
}

#[test]
fn forward_with_history_dispatches_exactly_one_forward_to_the_bound_view() {
    let (mut c, h) = plain();
    open(&mut c, "https://a.example");
    let view = c.snapshot().active_view.unwrap();
    c.apply_event(WebEvent::CanGoBackChanged(view, true));
    c.apply_event(WebEvent::CanGoForwardChanged(view, true));
    h.reset_ops();

    assert_eq!(
        c.on_content_gesture(forward(view)),
        Some(ControllerEvent::GestureHistoryRouted { view })
    );
    assert_eq!(
        h.ops(),
        vec![EngineOp::Action {
            view,
            action: WebAction::Forward,
        }]
    );
}

#[test]
fn disabled_forward_is_a_no_op() {
    let (mut c, h) = plain();
    open(&mut c, "https://a.example");
    let view = c.snapshot().active_view.unwrap();
    h.reset_ops();
    assert_eq!(c.on_content_gesture(forward(view)), None);
    assert!(h.ops().is_empty(), "disabled Forward dispatches nothing");
    assert_eq!(c.snapshot().tabs.len(), 1);
}

#[test]
fn root_back_without_provenance_is_a_no_op_and_never_closes() {
    let (mut c, h) = plain();
    open(&mut c, "https://a.example");
    let view = c.snapshot().active_view.unwrap();
    // A settled root (committed initial document) but NO opener relation:
    // the ordinary tab must never close merely because history is empty.
    c.apply_event(WebEvent::DidCommitNavigation(view));
    h.reset_ops();
    assert_eq!(c.on_content_gesture(back(view)), None);
    assert!(h.ops().is_empty(), "no dispatch, no close, no focus change");
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 1);
    assert_eq!(snap.active_view, Some(view));
}

#[test]
fn stale_view_commits_are_ignored() {
    let (mut c, h) = plain();
    open(&mut c, "https://a.example");
    h.reset_ops();
    assert_eq!(c.on_content_gesture(back(WebViewId(999))), None);
    assert!(h.ops().is_empty());
}

// ---------------------------------------------------------------------------
// Provenance lifecycle
// ---------------------------------------------------------------------------

#[test]
fn new_window_records_identity_based_provenance_but_plain_open_never_does() {
    let (mut c, _h) = plain();
    open(&mut c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    assert!(c.opener_pairs().is_empty(), "plain open records nothing");
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();
    assert_eq!(c.opener_of(tab_of(&c, child)), Some(tab_of(&c, opener)));
    assert_eq!(
        c.opener_pairs(),
        vec![(tab_of(&c, child), tab_of(&c, opener))]
    );

    // Cmd-T style plain open of another tab invents nothing.
    open(&mut c, "https://other.example");
    let other = c.snapshot().active_view.unwrap();
    assert_eq!(c.opener_of(tab_of(&c, other)), None);
    assert_eq!(c.opener_pairs().len(), 1);
}

#[test]
fn provenance_survives_reorder_move_suspend_and_navigation() {
    let (mut c, _h) = plain();
    open(&mut c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();
    let child_tab = tab_of(&c, child);

    // Reorder within the region: identity untouched.
    c.reorder_tab(1, 0).expect("reorder");
    assert_eq!(c.opener_of(child_tab), Some(tab_of(&c, opener)));

    // Navigation events do not rewrite the relation.
    c.apply_event(WebEvent::UrlChanged(
        child,
        "https://child.example/2".into(),
    ));
    c.apply_event(WebEvent::CanGoBackChanged(child, true));
    assert_eq!(c.opener_of(child_tab), Some(tab_of(&c, opener)));

    // Hide the child, then suspend it: its session is destroyed but the
    // identity (and the relation) survives; reactivation recreates the
    // session and keeps provenance.
    c.select_tab(1).expect("select opener");
    c.suspend_tab(0).expect("suspend hidden child");
    assert_eq!(c.opener_of(child_tab), Some(tab_of(&c, opener)));
    c.select_tab(0).expect("reactivate child");
    assert_eq!(c.opener_of(child_tab), Some(tab_of(&c, opener)));
    assert_eq!(c.opener_of(child_tab), Some(tab_of(&c, opener)));

    // A same-profile move preserves the live session and the relation.
    let other = c.create_space(ProfileId(1), "Other").unwrap();
    c.move_tab(0, other).expect("move child");
    assert_eq!(c.opener_of(child_tab), Some(tab_of(&c, opener)));
}

#[test]
fn closing_child_or_opener_invalidates_the_relation() {
    let (mut c, _h) = plain();
    open(&mut c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();

    // Close the child: relation gone (a reopen restores the identity with no
    // provenance — reopen never invents an opener).
    let child_tab = tab_of(&c, child);
    c.close_tab(1).expect("close child");
    assert_eq!(c.opener_of(child_tab), None);
    c.reopen_tab().expect("reopen");
    let reopened_view = c.snapshot().active_view.unwrap();
    assert_eq!(
        tab_of(&c, reopened_view),
        child_tab,
        "reopen keeps the persistent TabId"
    );
    assert_eq!(
        c.opener_of(child_tab),
        None,
        "reopen never invents provenance"
    );

    // Close the opener: its children lose provenance.
    c.open_tab_from_new_window(
        reopened_view,
        &new_window_session(reopened_view, ProfileId(1), "https://grandchild.example"),
    )
    .expect("open grandchild");
    let grandchild = c.snapshot().active_view.unwrap();
    let grandchild_tab = tab_of(&c, grandchild);
    assert_eq!(c.opener_of(grandchild_tab), Some(child_tab));
    // Close the child (the grandchild's exact opener): its child loses
    // provenance.
    c.close_tab(1).expect("close child (grandchild opener)");
    assert_eq!(c.opener_of(grandchild_tab), None);
}

// ---------------------------------------------------------------------------
// Atomic Back-at-root opener return
// ---------------------------------------------------------------------------

#[test]
fn back_at_settled_child_root_closes_child_and_activates_exact_opener() {
    let (mut c, h) = plain();
    let (child, opener) = open_child(&mut c);
    let snap_before = c.snapshot();
    let child_tab = snap_before
        .all_tabs
        .iter()
        .find(|t| t.web_view == Some(child))
        .expect("child tab id")
        .id;
    let opener_tab = snap_before
        .all_tabs
        .iter()
        .find(|t| t.web_view == Some(opener))
        .expect("opener tab id")
        .id;
    let space = snap_before.active_space;
    h.reset_ops();

    let event = c.on_content_gesture(back(child)).expect("returns");
    let snap = c.snapshot();
    match event {
        ControllerEvent::OpenerReturn {
            child: event_child,
            opener: event_opener,
            from_space,
            from_profile,
            to_space,
            to_profile,
            closed_view,
            presented_view,
            recreated,
        } => {
            assert_eq!(event_child, child_tab);
            assert_eq!(event_opener, opener_tab);
            assert_eq!(closed_view, child);
            assert_eq!(presented_view, opener);
            assert!(!recreated, "warm opener session is reused");
            assert_eq!(from_space, space);
            assert_eq!(to_space, space);
            assert_eq!(from_profile, ProfileId(1));
            assert_eq!(to_profile, ProfileId(1));
        }
        other => panic!("unexpected event: {other:?}"),
    }
    // The child is gone through the normal close path; the opener is shown.
    assert_eq!(snap.tabs.len(), 1);
    assert_eq!(snap.active_view, Some(opener));
    assert_eq!(snap.engine_sessions, 1);
    assert!(h.recorded(&[
        EngineOp::Destroy { view: child },
        EngineOp::Show {
            view: opener,
            visible: true,
        },
        EngineOp::FocusWeb { view: opener },
    ]));
    assert_eq!(
        h.ops()
            .iter()
            .filter(|op| matches!(op, EngineOp::Action { .. }))
            .count(),
        0,
        "opener return is close+activate, never a history action"
    );
}

#[test]
fn child_with_older_history_navigates_first_and_returns_only_at_root() {
    let (mut c, h) = plain();
    let (child, opener) = open_child(&mut c);

    // The child navigates within its own history: Back dispatches a real
    // history action, never the fallback.
    c.apply_event(WebEvent::CanGoBackChanged(child, true));
    h.reset_ops();
    assert_eq!(
        c.on_content_gesture(back(child)),
        Some(ControllerEvent::GestureHistoryRouted { view: child })
    );
    assert_eq!(
        h.ops(),
        vec![EngineOp::Action {
            view: child,
            action: WebAction::Back,
        }]
    );
    assert_eq!(c.snapshot().tabs.len(), 2, "no tab closed");

    // Back at root (history empty again): the atomic return applies.
    c.apply_event(WebEvent::CanGoBackChanged(child, false));
    h.reset_ops();
    assert!(matches!(
        c.on_content_gesture(back(child)),
        Some(ControllerEvent::OpenerReturn { .. })
    ));
    assert_eq!(c.snapshot().tabs.len(), 1);
    assert_eq!(c.snapshot().active_view, Some(opener));
}

#[test]
fn provisional_load_is_never_a_settled_root() {
    let (mut c, h) = plain();
    // Create the child through the productive path but WITHOUT the initial
    // document committing: the session is still in an ambiguous provisional
    // load, so a Back gesture must be a whole no-op.
    open(&mut c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();
    c.apply_event(WebEvent::LoadingChanged(child, true));
    h.reset_ops();
    assert_eq!(c.on_content_gesture(back(child)), None);
    assert!(h.ops().is_empty(), "no close during a provisional load");
    assert_eq!(c.snapshot().tabs.len(), 2);
}

#[test]
fn settled_failure_or_crash_states_allow_the_return() {
    // A settled navigation failure (initial load failed) is a settled root:
    // the session survives, so the Back-at-root return applies.
    let (mut c, _h) = plain();
    let (failed_child, failed_opener) = open_child_with_failure(&mut c);
    assert!(matches!(
        c.on_content_gesture(back(failed_child)),
        Some(ControllerEvent::OpenerReturn {
            presented_view,
            closed_view,
            ..
        }) if presented_view == failed_opener && closed_view == failed_child
    ));

    // A crash destroys the child's native session: the dead view can never
    // be a gesture target (stale-view no-op), and the crash handler must
    // never close the tab — it stays recoverable through its error state.
    let (mut c, _h) = plain();
    let (crashed_child, _crashed_opener) = open_child(&mut c);
    c.apply_event(WebEvent::PageCrashed(crashed_child));
    assert_eq!(c.on_content_gesture(back(crashed_child)), None);
    assert_eq!(c.snapshot().tabs.len(), 2, "crash never closes a tab");
}

#[test]
fn hidden_background_child_is_never_a_gesture_target() {
    let (mut c, h) = plain();
    // Open the child in an inactive source space: it is hidden/warm, so a
    // Back gesture naming it must be a whole no-op.
    open(&mut c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    let other = c.create_space(ProfileId(1), "Other").unwrap();
    c.activate_space(other).unwrap();
    open(&mut c, "https://active.example");
    let active_view = c.snapshot().active_view.unwrap();
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child in inactive source space");
    let child_view = c
        .snapshot()
        .all_tabs
        .iter()
        .find(|t| t.url == "https://child.example")
        .and_then(|t| t.web_view)
        .expect("child session exists");
    c.apply_event(WebEvent::DidCommitNavigation(child_view));

    h.reset_ops();
    assert_eq!(c.on_content_gesture(back(child_view)), None);
    assert!(h.ops().is_empty(), "hidden child is not a gesture target");
    assert_eq!(c.snapshot().active_view, Some(active_view));
    assert_eq!(c.snapshot().tabs.len(), 1);
}

#[test]
fn return_activates_opener_across_spaces_and_keeps_the_warm_session() {
    let (mut c, h) = plain();
    open(&mut c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();
    let child_space = c.snapshot().active_space;
    c.apply_event(WebEvent::DidCommitNavigation(child));

    // Move the *opener* into a second space of the same profile: the child
    // stays in the active space, its provenance survives, and the opener's
    // live session moves with it (warm move).
    let other = c.create_space(ProfileId(1), "Other").unwrap();
    c.move_tab(0, other).expect("move opener");
    assert_eq!(
        c.snapshot().active_space,
        child_space,
        "source space repaired"
    );
    assert_eq!(c.snapshot().active_view, Some(child));
    h.reset_ops();

    assert!(matches!(
        c.on_content_gesture(back(child)),
        Some(ControllerEvent::OpenerReturn {
            from_space,
            from_profile,
            to_space,
            to_profile,
            recreated: false,
            presented_view,
            ..
        }) if from_space == child_space && from_profile == ProfileId(1)
            && to_space == other && to_profile == ProfileId(1)
            && presented_view == opener
    ));
    assert_eq!(c.snapshot().active_space, other);
    assert_eq!(c.snapshot().active_view, Some(opener));
}

#[test]
fn return_recreates_a_suspended_opener_session() {
    let (mut c, h) = plain();
    open(&mut c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    let opener_tab = tab_of(&c, opener);
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();

    // Suspend the hidden opener (the child is active): its session is
    // destroyed, its identity and provenance survive.
    c.suspend_tab(0).expect("suspend hidden opener");
    assert_eq!(c.opener_of(tab_of(&c, child)), Some(opener_tab));
    c.apply_event(WebEvent::DidCommitNavigation(child));
    h.reset_ops();

    let event = c.on_content_gesture(back(child)).expect("returns");
    let snap = c.snapshot();
    match event {
        ControllerEvent::OpenerReturn {
            presented_view,
            recreated,
            closed_view,
            ..
        } => {
            assert!(recreated, "suspended opener session is rebuilt");
            assert_eq!(closed_view, child);
            // A rebuilt session is a fresh view for the same opener tab.
            assert_ne!(presented_view, opener);
            assert_eq!(snap.active_view, Some(presented_view));
        }
        other => panic!("unexpected event: {other:?}"),
    }
    assert_eq!(snap.tabs.len(), 1, "only the opener remains");
    assert_eq!(snap.engine_sessions, 1);
}

#[test]
fn return_from_split_child_dissolves_the_split_and_presents_opener() {
    let (mut c, h) = plain();
    open(&mut c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();
    c.apply_event(WebEvent::DidCommitNavigation(child));

    // Split opener (index 0) and child (index 1).
    c.open_split(0, 1).expect("split");
    let layout = c.pane_layout().expect("split layout");
    assert_eq!(layout.primary, opener);
    assert_eq!(layout.secondary, Some(child));

    h.reset_ops();
    assert!(matches!(
        c.on_content_gesture(back(child)),
        Some(ControllerEvent::OpenerReturn {
            closed_view,
            presented_view,
            ..
        }) if closed_view == child && presented_view == opener
    ));
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 1, "the child is closed");
    assert_eq!(snap.active_view, Some(opener));
    assert_eq!(
        c.pane_layout().map(|l| l.secondary),
        Some(None),
        "the split is dissolved into single-pane mode"
    );
}

#[test]
fn stale_or_missing_opener_is_a_no_op() {
    let (mut c, h) = plain();
    open(&mut c, "https://opener.example");
    let opener = c.snapshot().active_view.unwrap();
    c.open_tab_from_new_window(
        opener,
        &new_window_session(opener, ProfileId(1), "https://child.example"),
    )
    .expect("open child");
    let child = c.snapshot().active_view.unwrap();
    c.apply_event(WebEvent::DidCommitNavigation(child));

    // Close the opener directly: the relation is invalidated, so the child's
    // Back at root is a no-op (never closes an arbitrary tab).
    let child_tab = tab_of(&c, child);
    c.close_tab(0).expect("close opener");
    h.reset_ops();
    assert_eq!(c.on_content_gesture(back(child)), None);
    assert!(h.ops().is_empty());
    assert_eq!(c.snapshot().tabs.len(), 1);
    assert_eq!(c.snapshot().active_view, Some(child));
    assert_eq!(c.opener_of(child_tab), None);
}
