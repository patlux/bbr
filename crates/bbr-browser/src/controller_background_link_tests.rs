//! DD-LINK controller contract tests — the portable grouped background-child
//! insertion matrix (task packet `background-link-tabs`).
//!
//! Every scenario drives the [`BrowserController`] over the deterministic
//! [`FakeEngine`] through the same `Rc<RefCell<dyn WebEngine>>` seam the
//! composition root uses. Coverage map (background-link-tabs.md, "Unit and
//! contract"):
//!
//! - The background disposition creates exactly one unselected regular child
//!   and records exact opener provenance; plain/script dispositions are
//!   unchanged (their own suites prove that).
//! - Insertion matrix: no prior child (first child lands immediately after
//!   the opener), several children (contiguous group in click order),
//!   closed child (next resumes after the surviving member), reordered /
//!   manually-separated child (deterministic contiguous-run policy, never
//!   hidden adjacency), pinned/regular boundary (pinned opener's child lands
//!   at the first regular position), inactive opener space, stale opener, and
//!   invalid source profile.
//! - No selection, focus-domain, split, or unrelated-session mutation: the
//!   opener stays the active view, the child session is created hidden, and
//!   the engine op log shows exactly one `Create` + `Show(false)` for the
//!   child and nothing else.

use bbr_web::{
    decide, NewWindowDecision, NewWindowFeatures, NewWindowInput, NewWindowOutcome,
    NewWindowRequest, NewWindowTarget, ProfileId, WebViewId,
};

use crate::fake::EngineOp;
use crate::harness::{open, plain};
use crate::{BrowserController, ControllerError, ControllerEvent};

/// Build a decided `OpenBackgroundTab` session for `source`/`profile`/`url`.
fn background_session(
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
        input: NewWindowInput::Background,
        features: NewWindowFeatures::default(),
    };
    match decide(&request, bbr_web::new_window::default_decision(&request)) {
        NewWindowOutcome::CreateSession { session } => {
            assert_eq!(
                session.disposition(),
                bbr_web::new_window::NewWindowDisposition::OpenBackgroundTab
            );
            session
        }
        other => panic!("expected background session, got {other:?}"),
    }
}

/// Open one background child from `opener` (a live view). Returns the new
/// child's view and the `NewWindowOpened` event carrying the insertion index.
fn open_background_child(
    c: &mut BrowserController,
    opener: WebViewId,
    url: &str,
) -> (WebViewId, ControllerEvent) {
    let profile = c.snapshot().active_profile;
    let event = c
        .open_background_tab_from_new_window(opener, &background_session(opener, profile, url))
        .expect("background child opens");
    let child = match &event {
        ControllerEvent::NewWindowOpened { view, .. } => *view,
        other => panic!("expected NewWindowOpened, got {other:?}"),
    };
    (child, event)
}

/// Open an opener tab and return its live view.
fn open_opener(c: &mut BrowserController, url: &str) -> WebViewId {
    open(c, url);
    c.snapshot().active_view.expect("opener view")
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

/// The ordered URLs of the active space's tabs.
fn tab_urls(c: &BrowserController) -> Vec<String> {
    c.snapshot()
        .tabs
        .iter()
        .map(|t| t.url.clone())
        .collect::<Vec<_>>()
}

#[test]
fn background_child_lands_immediately_after_opener_and_stays_hidden() {
    let (mut c, h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    h.reset_ops();

    let (child, event) = open_background_child(&mut c, opener, "https://child.example");
    let ControllerEvent::NewWindowOpened { index, view, .. } = event else {
        unreachable!();
    };
    assert_eq!(index, 1, "first child inserts immediately after the opener");
    assert_eq!(view, child);
    assert_eq!(
        tab_urls(&c),
        vec![
            "https://opener.example".to_string(),
            "https://child.example".to_string()
        ]
    );
    let snap = c.snapshot();
    assert_eq!(snap.active_view, Some(opener), "opener stays selected");
    assert!(!snap.tabs[1].active, "child is not selected");
    assert_eq!(snap.engine_sessions, 2);
    assert_eq!(
        c.opener_pairs(),
        vec![(tab_of(&c, child), tab_of(&c, opener))],
        "exact opener provenance is recorded"
    );
    // The child session is created and stays hidden: the engine saw exactly
    // one Create and never a Show for the child (its desired visibility is
    // false from creation), and the opener's visibility is untouched — no
    // focus, split, or unrelated-session op ever runs.
    assert_eq!(
        h.ops(),
        vec![EngineOp::Create {
            view: child,
            profile: ProfileId(1),
            url: "https://child.example".into(),
        }]
    );
}

#[test]
fn repeated_children_form_a_contiguous_group_in_click_order() {
    let (mut c, _h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    let (x, _) = open_background_child(&mut c, opener, "https://x.example");
    let (y, _) = open_background_child(&mut c, opener, "https://y.example");
    let (z, _) = open_background_child(&mut c, opener, "https://z.example");
    assert_eq!(
        tab_urls(&c),
        vec![
            "https://opener.example".to_string(),
            "https://x.example".to_string(),
            "https://y.example".to_string(),
            "https://z.example".to_string()
        ]
    );
    let snap = c.snapshot();
    assert_eq!(snap.active_view, Some(opener), "opener stays selected");
    assert_eq!(snap.engine_sessions, 4);
    // Every child records identity-based provenance to the exact opener.
    let pairs = c.opener_pairs();
    assert_eq!(pairs.len(), 3);
    let children = [x, y, z].map(|view| tab_of(&c, view));
    for (child_view, opener_tab) in pairs {
        assert_eq!(opener_tab, tab_of(&c, opener));
        assert!(
            children.contains(&child_view),
            "child {child_view:?} is one of X/Y/Z"
        );
    }
}

#[test]
fn unrelated_tabs_retain_relative_order_after_the_group() {
    let (mut c, _h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    // An unrelated tab already exists AFTER the opener; the group must stay
    // contiguous immediately after the opener with C after it.
    open(&mut c, "https://unrelated.example");
    let _ = open_background_child(&mut c, opener, "https://x.example");
    let _ = open_background_child(&mut c, opener, "https://y.example");
    assert_eq!(
        tab_urls(&c),
        vec![
            "https://opener.example".to_string(),
            "https://x.example".to_string(),
            "https://y.example".to_string(),
            "https://unrelated.example".to_string()
        ]
    );
}

#[test]
fn closed_child_lets_the_next_insertion_resume_after_the_survivor() {
    let (mut c, _h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    let (x, _) = open_background_child(&mut c, opener, "https://x.example");
    let (_y, _) = open_background_child(&mut c, opener, "https://y.example");
    // Close the second child (index 2): its provenance is invalidated and the
    // next child must land after the surviving x, before nothing else.
    c.close_tab(2).expect("close y");
    assert_eq!(
        c.opener_pairs(),
        vec![(tab_of(&c, x), tab_of(&c, opener))],
        "closing the child invalidates only its relation"
    );
    let (z, _) = open_background_child(&mut c, opener, "https://z.example");
    assert_eq!(
        tab_urls(&c),
        vec![
            "https://opener.example".to_string(),
            "https://x.example".to_string(),
            "https://z.example".to_string()
        ]
    );
    let _ = z;
}

#[test]
fn manually_separated_child_is_never_inferred_by_adjacency() {
    let (mut c, _h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    let (_x, _) = open_background_child(&mut c, opener, "https://x.example");
    let (y, _) = open_background_child(&mut c, opener, "https://y.example");
    // The user moves y away from the contiguous group (reorder 2 → 4 after
    // appending a filler). The next insertion must follow the deterministic
    // contiguous-run policy and land after x, never after the moved y.
    open(&mut c, "https://filler.example");
    open(&mut c, "https://filler2.example");
    c.reorder_tab(2, 4).expect("move y away");
    assert_eq!(
        tab_urls(&c),
        vec![
            "https://opener.example".to_string(),
            "https://x.example".to_string(),
            "https://filler.example".to_string(),
            "https://filler2.example".to_string(),
            "https://y.example".to_string()
        ]
    );
    let (z, event) = open_background_child(&mut c, opener, "https://z.example");
    let ControllerEvent::NewWindowOpened { index, .. } = event else {
        unreachable!();
    };
    assert_eq!(index, 2, "insertion resumes after the contiguous run (x)");
    assert_eq!(
        tab_urls(&c),
        vec![
            "https://opener.example".to_string(),
            "https://x.example".to_string(),
            "https://z.example".to_string(),
            "https://filler.example".to_string(),
            "https://filler2.example".to_string(),
            "https://y.example".to_string()
        ]
    );
    let _ = (y, z);
}

#[test]
fn pinned_opener_child_lands_at_the_first_regular_position() {
    let (mut c, _h) = plain();
    open(&mut c, "https://pinned.example");
    open(&mut c, "https://regular.example");
    c.pin_tab(0).expect("pin the opener");
    let snap = c.snapshot();
    assert!(
        snap.tabs[0].pinned && !snap.tabs[1].pinned,
        "boundary set up"
    );
    let opener = snap.tabs[0].web_view.expect("pinned opener view");
    let (x, event) = open_background_child(&mut c, opener, "https://x.example");
    let ControllerEvent::NewWindowOpened { index, .. } = event else {
        unreachable!();
    };
    // A regular child can never live inside the pinned region: the first
    // child lands at the pinned/regular boundary (the nearest legal position
    // after the pinned opener).
    assert_eq!(index, 1);
    assert_eq!(
        tab_urls(&c),
        vec![
            "https://pinned.example".to_string(),
            "https://x.example".to_string(),
            "https://regular.example".to_string()
        ]
    );
    let (_y, _) = open_background_child(&mut c, opener, "https://y.example");
    assert_eq!(
        tab_urls(&c),
        vec![
            "https://pinned.example".to_string(),
            "https://x.example".to_string(),
            "https://y.example".to_string(),
            "https://regular.example".to_string()
        ],
        "further children extend the group in click order"
    );
    let _ = x;
}

#[test]
fn inactive_source_space_stays_inactive_and_child_is_hidden_warm() {
    let (mut c, _h) = plain();
    // Work profile + source space with an opener; then a separate active
    // space with its own active tab.
    let ControllerEvent::ProfileCreated { profile, space } = c
        .create_profile_space("Work", "Source")
        .expect("create work profile with source space")
    else {
        panic!("expected ProfileCreated");
    };
    c.activate_space(space).expect("activate source space");
    let opener = open_opener(&mut c, "https://opener.example");
    let other_space = c
        .create_space(profile, "Active")
        .expect("create active space");
    c.activate_space(other_space)
        .expect("activate active space");
    let other_view = open_opener(&mut c, "https://other.example");
    let active_before = c.snapshot().active_view;

    let (child, event) = open_background_child(&mut c, opener, "https://child.example");
    let ControllerEvent::NewWindowOpened {
        index,
        space: child_space,
        profile: child_profile,
        ..
    } = event
    else {
        unreachable!();
    };
    assert_eq!(index, 1, "first child after the opener in the source space");
    assert_eq!(child_space, space);
    assert_eq!(child_profile, profile);
    let snap = c.snapshot();
    assert_eq!(snap.active_view, active_before, "active space unchanged");
    assert_eq!(snap.active_view, Some(other_view));
    // The child is a warm hidden session in the inactive source space: its
    // global tab snapshot names the source space/profile and no visible
    // session count changed.
    let global = snap
        .all_tabs
        .iter()
        .find(|t| t.web_view == Some(child))
        .expect("child in global snapshot");
    assert_eq!(global.space, space);
    assert_eq!(global.profile, profile);
    assert_eq!(snap.engine_sessions, 3);
}

#[test]
fn stale_opener_rejects_without_any_mutation() {
    let (mut c, h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    let before = c.snapshot();
    let stale = WebViewId(u64::MAX);
    h.reset_ops();
    let err = c
        .open_background_tab_from_new_window(
            stale,
            &background_session(stale, before.active_profile, "https://child.example"),
        )
        .expect_err("stale source fails closed");
    assert_eq!(err, ControllerError::UnknownWebView(stale));
    let after = c.snapshot();
    assert_eq!(after.tabs, before.tabs);
    assert_eq!(after.engine_sessions, before.engine_sessions);
    assert_eq!(after.active_view, before.active_view);
    assert!(h.ops().is_empty(), "no engine mutation for a stale source");
    let _ = opener;
}

#[test]
fn invalid_source_profile_rejects_without_any_mutation() {
    let (mut c, h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    let before = c.snapshot();
    h.reset_ops();
    let err = c
        .open_background_tab_from_new_window(
            opener,
            &background_session(opener, ProfileId(99), "https://child.example"),
        )
        .expect_err("profile mismatch fails closed");
    assert_eq!(
        err,
        ControllerError::NewWindowProfileMismatch {
            source: ProfileId(1),
            requested: ProfileId(99),
        }
    );
    let after = c.snapshot();
    assert_eq!(after.tabs, before.tabs);
    assert_eq!(after.engine_sessions, before.engine_sessions);
    assert!(
        h.ops().is_empty(),
        "no engine mutation for a profile mismatch"
    );
}

#[test]
fn background_open_never_dissolves_a_split_or_touches_the_opener_session() {
    let (mut c, h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    // A second tab in a split pair (open_split activates both panes; the
    // second tab is the focused pane because it was active before the split).
    open(&mut c, "https://second.example");
    let second = c.snapshot().active_view.expect("second view");
    c.open_split(0, 1).expect("open a split");
    let layout_before = c.pane_layout().expect("split layout");
    assert!(layout_before.secondary.is_some(), "split exists");
    h.reset_ops();

    let (child, _) = open_background_child(&mut c, opener, "https://child.example");
    let snap = c.snapshot();
    // The split survives (the background child never dissolves unrelated
    // state), the same two panes stay framed, and the focused pane is
    // unchanged.
    let layout_after = c.pane_layout().expect("split layout after");
    assert_eq!(layout_after, layout_before, "split composition unchanged");
    assert_eq!(snap.active_view, Some(second), "focused pane unchanged");
    // Only the hidden child session was created; the split sessions were
    // never touched (no extra shows/actions) and no Show ran for the child
    // (hidden from creation).
    assert_eq!(
        h.ops(),
        vec![EngineOp::Create {
            view: child,
            profile: ProfileId(1),
            url: "https://child.example".into(),
        }]
    );
}

#[test]
fn child_activation_reuses_its_already_created_session() {
    let (mut c, _h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    let (child, _) = open_background_child(&mut c, opener, "https://child.example");
    let before = c.snapshot();
    let child_index = before
        .tabs
        .iter()
        .position(|t| t.web_view == Some(child))
        .expect("child index");
    c.select_tab(child_index).expect("activate the child");
    let after = c.snapshot();
    assert_eq!(
        after.active_view,
        Some(child),
        "activating the child reuses its existing warm session"
    );
    assert_eq!(
        after.engine_sessions, before.engine_sessions,
        "no recreation"
    );
    assert_eq!(
        c.opener_pairs(),
        vec![(tab_of(&c, child), tab_of(&c, opener))],
        "provenance survives activation"
    );
}

#[test]
fn script_claiming_background_input_is_still_blocked() {
    let (mut c, h) = plain();
    let opener = open_opener(&mut c, "https://opener.example");
    let before = c.snapshot();
    h.reset_ops();
    let request = NewWindowRequest {
        source: opener,
        source_profile: before.active_profile,
        url: "https://script.example".into(),
        user_initiated: false,
        target: NewWindowTarget::NewWindow,
        input: NewWindowInput::Background,
        features: NewWindowFeatures::default(),
    };
    assert_eq!(
        bbr_web::new_window::default_decision(&request),
        NewWindowDecision::Block,
        "input metadata never fabricates a script gesture"
    );
    assert!(h.ops().is_empty());
}
