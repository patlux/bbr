//! Controller contract tests for the transient Peek preview lifecycle
//! (Link Preview / Peek packet).
//!
//! Every scenario drives the [`BrowserController`] through the exact
//! `Rc<RefCell<dyn WebEngine>>` seam against the deterministic
//! [`FakeEngine`], wired the same way the real app wires it. Coverage map
//! (link-preview-peek.md, "Evidence"):
//!
//! - Request policy: eligible direct activation → `Preview`; script → `Block`;
//!   ineligible direct activation → ordinary `OpenTab` (never silently lost);
//!   explicit `OpenTab`/`Peek` intents route through the same authority.
//! - Transient identity: `PreviewId` allocation, snapshot fields, no persisted
//!   `Tab`/recently-closed/history row.
//! - Source-profile session ownership: preview sessions are created in the
//!   resolved source profile's store; stale/mismatched sources fail closed
//!   without mutation.
//! - One preview per pane: a new eligible request from inside the preview
//!   navigates the existing preview (same session); any other source replaces
//!   it (the old session is destroyed first).
//! - Command routing: while the preview is open, browser navigation actions
//!   route to the preview session, never the covered origin.
//! - Dismissal: `close_preview` destroys the temporary session and leaves the
//!   warm origin untouched; origin-removal staleness closes the preview.
//! - Promotion: "Open as Tab" adopts the SAME `WebViewId`/session without
//!   reload or profile crossing; a crashed preview refuses promotion.

use bbr_web::{
    decide, NewWindowDecision, NewWindowFeatures, NewWindowInput, NewWindowIntent,
    NewWindowOutcome, NewWindowRequest, NewWindowTarget, ProfileId, WebAction, WebEvent, WebViewId,
};

use crate::fake::EngineOp;
use crate::harness::{open, plain};
use crate::{ControllerError, ControllerEvent};

/// A decided `Preview`-disposition session for `source`/`profile`/`url`.
fn preview_session(source: WebViewId, profile: ProfileId, url: &str) -> bbr_web::NewWindowSession {
    let request = NewWindowRequest {
        source,
        source_profile: profile,
        url: url.into(),
        user_initiated: true,
        target: NewWindowTarget::NewWindow,
        input: NewWindowInput::Primary,

        features: NewWindowFeatures::default(),
    };
    match decide(&request, NewWindowDecision::Preview) {
        NewWindowOutcome::CreateSession { session } => session,
        other => panic!("expected preview session, got {other:?}"),
    }
}

/// A decided `OpenTab`-disposition session (the ordinary-tab path).
fn tab_session(source: WebViewId, profile: ProfileId, url: &str) -> bbr_web::NewWindowSession {
    let request = NewWindowRequest {
        source,
        source_profile: profile,
        url: url.into(),
        user_initiated: true,
        target: NewWindowTarget::NewWindow,
        input: NewWindowInput::Primary,

        features: NewWindowFeatures::default(),
    };
    match decide(&request, NewWindowDecision::OpenTab) {
        NewWindowOutcome::CreateSession { session } => session,
        other => panic!("expected tab session, got {other:?}"),
    }
}

fn open_preview(
    c: &mut crate::BrowserController,
    source: WebViewId,
    profile: ProfileId,
    url: &str,
) -> ControllerEvent {
    c.open_preview_from_new_window(source, &preview_session(source, profile, url))
        .expect("open preview succeeds")
}

// ---------------------------------------------------------------------------
// Request policy
// ---------------------------------------------------------------------------

#[test]
fn eligible_direct_activation_resolves_to_preview_and_script_blocks() {
    let request = NewWindowRequest {
        source: WebViewId(1),
        source_profile: ProfileId(1),
        url: "https://example.com/link".into(),
        user_initiated: true,
        target: NewWindowTarget::NewWindow,
        input: NewWindowInput::Primary,

        features: NewWindowFeatures::default(),
    };
    assert_eq!(
        bbr_web::default_decision(&request),
        NewWindowDecision::Preview
    );

    let mut script = request.clone();
    script.user_initiated = false;
    assert_eq!(bbr_web::default_decision(&script), NewWindowDecision::Block);

    // Explicit ordinary-tab intent bypasses eligibility for direct
    // activations only; a script request still blocks.
    assert_eq!(
        bbr_web::decision_for_intent(&request, NewWindowIntent::OpenTab),
        NewWindowDecision::OpenTab
    );
    assert_eq!(
        bbr_web::decision_for_intent(&script, NewWindowIntent::OpenTab),
        NewWindowDecision::Block
    );
}

#[test]
fn ineligible_direct_activation_falls_back_to_ordinary_open_tab() {
    // A non-web scheme never forces a preview and never vanishes.
    let mailto = NewWindowRequest {
        source: WebViewId(1),
        source_profile: ProfileId(1),
        url: "mailto:hello@example.com".into(),
        user_initiated: true,
        target: NewWindowTarget::NewWindow,
        input: NewWindowInput::Primary,

        features: NewWindowFeatures::default(),
    };
    assert_eq!(
        bbr_web::default_decision(&mailto),
        NewWindowDecision::OpenTab
    );
    assert_eq!(
        bbr_web::decision_for_intent(&mailto, NewWindowIntent::Peek),
        NewWindowDecision::OpenTab
    );
}

#[test]
fn explicit_open_tab_intent_still_creates_exactly_one_regular_tab_never_a_preview() {
    // The preserved ordinary-regular-tab path: an explicit `OpenTab`-intent
    // session (the `NewWindowIntent::OpenTab` action) opens exactly one
    // regular tab/session in the source Space/Profile and never enters the
    // transient preview state. This is the invariant the Link Preview packet
    // must preserve beside the Peek default.
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    let tabs_before = c.snapshot().tabs.len();
    h.reset_ops();

    let event = c
        .open_tab_from_new_window(
            source,
            &tab_session(source, ProfileId(1), "https://child.example"),
        )
        .expect("ordinary tab path succeeds");
    assert_eq!(
        event,
        ControllerEvent::NewWindowOpened {
            index: 1,
            url: "https://child.example".into(),
            space: c.snapshot().active_space,
            profile: ProfileId(1),
            view: WebViewId(2),
        }
    );
    let snapshot = c.snapshot();
    assert_eq!(
        snapshot.tabs.len(),
        tabs_before + 1,
        "exactly one regular tab"
    );
    assert!(snapshot.preview.is_none(), "no transient preview created");
    assert_eq!(
        snapshot.engine_sessions, 2,
        "origin warm + one fresh regular session"
    );
    assert_eq!(
        snapshot.tabs[1].web_view,
        Some(snapshot.active_view.unwrap())
    );
    assert_eq!(snapshot.tabs[1].url, "https://child.example");
}

// ---------------------------------------------------------------------------
// Transient identity and source-profile session ownership
// ---------------------------------------------------------------------------

#[test]
fn preview_opens_a_source_profile_session_above_the_warm_origin_without_a_tab() {
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    let tabs_before = c.snapshot().tabs.len();
    h.reset_ops();

    let event = open_preview(&mut c, source, ProfileId(1), "https://preview.example");
    let view = match event {
        ControllerEvent::PreviewOpened {
            id,
            view,
            source: s,
            profile,
            url,
        } => {
            assert_eq!(id.0, 0, "fresh PreviewId allocation starts at zero");
            assert_eq!(s, source);
            assert_eq!(profile, ProfileId(1));
            assert_eq!(url, "https://preview.example");
            view
        }
        other => panic!("unexpected event: {other:?}"),
    };

    let snapshot = c.snapshot();
    // No regular tab was created and the origin keeps its exact session.
    assert_eq!(snapshot.tabs.len(), tabs_before);
    assert_eq!(snapshot.tabs[0].web_view, Some(source));
    assert_eq!(snapshot.active_view, Some(source));
    assert_eq!(snapshot.engine_sessions, 2, "origin warm + preview session");
    let preview = snapshot.preview.expect("preview snapshot present");
    assert_eq!(preview.view, view);
    assert_eq!(preview.source, source);
    assert_eq!(preview.profile, ProfileId(1));
    assert_eq!(preview.url, "https://preview.example");
    // Blueprint D09: the preview is the one pane; the covered layout is the
    // origin's own, so the shell keeps the origin framed underneath.
    let layout = c.pane_layout().expect("preview layout");
    assert_eq!((layout.primary, layout.focused), (view, view));
    let covered = c.covered_pane_layout().expect("covered layout");
    assert_eq!((covered.primary, covered.secondary), (source, None));

    // The engine saw create → raise (above origin) → show for the preview.
    assert!(h.recorded(&[
        EngineOp::Create {
            view: WebViewId(2),
            profile: ProfileId(1),
            url: "https://preview.example".into(),
        },
        EngineOp::Raise { view: WebViewId(2) },
        EngineOp::Show {
            view: WebViewId(2),
            visible: true,
        },
    ]));
    // The fake session was created in the source profile's store (the
    // engine's Create op records the validated profile).
    assert!(h.ops().iter().any(|op| matches!(
        op,
        EngineOp::Create {
            view: v,
            profile: ProfileId(1),
            url,
        } if *v == view && url == "https://preview.example"
    )));
}

#[test]
fn preview_rejects_stale_source_and_profile_mismatch_without_mutation() {
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    let before = c.snapshot();
    h.reset_ops();

    let stale = c.open_preview_from_new_window(
        WebViewId(999),
        &preview_session(WebViewId(999), ProfileId(1), "https://child.example"),
    );
    assert_eq!(stale, Err(ControllerError::UnknownWebView(WebViewId(999))));
    let mismatch = c.open_preview_from_new_window(
        source,
        &preview_session(source, ProfileId(77), "https://child.example"),
    );
    assert_eq!(
        mismatch,
        Err(ControllerError::NewWindowProfileMismatch {
            source: ProfileId(1),
            requested: ProfileId(77),
        })
    );
    assert_eq!(c.snapshot(), before);
    assert!(h.ops().is_empty());
}

#[test]
fn preview_is_transient_state_never_a_tab_recently_closed_or_history_row() {
    let (mut c, _h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    open_preview(&mut c, source, ProfileId(1), "https://preview.example");

    let snapshot = c.snapshot();
    assert!(snapshot.preview.is_some());
    // The preview produced no domain tab and no extra space membership: the
    // active space still owns exactly the origin tab.
    assert_eq!(snapshot.tabs.len(), 1);
    assert_eq!(
        snapshot
            .spaces
            .iter()
            .map(|space| space.tab_count)
            .sum::<usize>(),
        1,
        "no tab was created for the preview"
    );
}

// ---------------------------------------------------------------------------
// One preview per pane
// ---------------------------------------------------------------------------

#[test]
fn a_request_from_inside_the_preview_navigates_the_existing_preview_session() {
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    let opened = open_preview(&mut c, source, ProfileId(1), "https://preview.example");
    let preview_view = match opened {
        ControllerEvent::PreviewOpened { view, .. } => view,
        _ => unreachable!(),
    };
    h.reset_ops();

    let event = c
        .open_preview_from_new_window(
            preview_view,
            &preview_session(preview_view, ProfileId(1), "https://next.example"),
        )
        .expect("internal preview request navigates the existing preview");
    assert_eq!(
        event,
        ControllerEvent::PreviewNavigated { view: preview_view }
    );
    // Same session, same WebViewId: no create/destroy/raise/show.
    assert!(h.recorded(&[EngineOp::Action {
        view: preview_view,
        action: WebAction::Load {
            url: "https://next.example".into()
        },
    }]));
    let snapshot = c.snapshot();
    assert_eq!(snapshot.engine_sessions, 2);
    assert_eq!(snapshot.preview.unwrap().view, preview_view);
}

#[test]
fn a_request_from_another_source_replaces_the_existing_preview() {
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    open(&mut c, "https://second.example");
    // Two tabs; the second is active.
    let second = c.snapshot().active_view.unwrap();
    let source = c.snapshot().tabs[0].web_view.unwrap();
    let opened = open_preview(&mut c, second, ProfileId(1), "https://preview.example");
    let first_preview = match opened {
        ControllerEvent::PreviewOpened { view, .. } => view,
        _ => unreachable!(),
    };
    assert_eq!(first_preview, WebViewId(3));
    h.reset_ops();

    // A request from the OTHER live tab's session replaces the preview:
    // the old preview session is destroyed first, then a fresh one is
    // created (never stacked).
    let replaced = open_preview(&mut c, source, ProfileId(1), "https://replacement.example");
    let new_view = match replaced {
        ControllerEvent::PreviewOpened { view, .. } => view,
        _ => panic!("expected a fresh PreviewOpened, got {replaced:?}"),
    };
    assert_eq!(new_view, WebViewId(4), "fresh session for the replacement");
    assert!(h.recorded(&[
        EngineOp::Destroy {
            view: first_preview
        },
        EngineOp::Create {
            view: WebViewId(4),
            profile: ProfileId(1),
            url: "https://replacement.example".into(),
        },
        EngineOp::Raise { view: WebViewId(4) },
        EngineOp::Show {
            view: WebViewId(4),
            visible: true,
        },
    ]));
    let snapshot = c.snapshot();
    assert_eq!(snapshot.engine_sessions, 3, "2 tabs + 1 preview");
    assert_eq!(snapshot.preview.unwrap().view, new_view);
}

// ---------------------------------------------------------------------------
// Command routing
// ---------------------------------------------------------------------------

#[test]
fn browser_navigation_commands_route_to_the_preview_while_it_owns_focus() {
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    let opened = open_preview(&mut c, source, ProfileId(1), "https://preview.example");
    let preview_view = match opened {
        ControllerEvent::PreviewOpened { view, .. } => view,
        _ => unreachable!(),
    };
    h.reset_ops();

    for action in [
        WebAction::Back,
        WebAction::Forward,
        WebAction::Reload,
        WebAction::Stop,
    ] {
        assert_eq!(
            c.navigate(action.clone()).unwrap(),
            ControllerEvent::NavigationRouted { view: preview_view },
            "action {action:?} routes to the preview"
        );
    }
    assert!(h.recorded(&[
        EngineOp::Action {
            view: preview_view,
            action: WebAction::Back,
        },
        EngineOp::Action {
            view: preview_view,
            action: WebAction::Forward,
        },
        EngineOp::Action {
            view: preview_view,
            action: WebAction::Reload,
        },
        EngineOp::Action {
            view: preview_view,
            action: WebAction::Stop,
        },
    ]));
}

#[test]
fn preview_navigation_events_mirror_into_the_preview_snapshot() {
    let (c, h) = crate::harness::wired();
    {
        let controller = &mut *c.borrow_mut();
        let _ = open(controller, "https://source.example");
        let source = controller.snapshot().active_view.unwrap();
        let _ = open_preview(controller, source, ProfileId(1), "https://preview.example");
    }
    let preview_view = c.borrow().snapshot().preview.unwrap().view;

    // The wired event transport mirrors preview events into nav state and
    // never treats them as unknown views.
    h.emit(WebEvent::TitleChanged(preview_view, "Preview Title".into()));
    h.emit(WebEvent::UrlChanged(
        preview_view,
        "https://preview.example/page".into(),
    ));
    h.emit(WebEvent::LoadingChanged(preview_view, true));
    h.emit(WebEvent::CanGoBackChanged(preview_view, true));
    h.emit(WebEvent::LoadingChanged(preview_view, false));

    let preview = c.borrow().snapshot().preview.unwrap();
    assert_eq!(preview.title.as_deref(), Some("Preview Title"));
    assert_eq!(preview.url, "https://preview.example/page");
    assert!(!preview.loading);
    assert!(preview.can_go_back);
    assert!(!preview.crashed);
}

// ---------------------------------------------------------------------------
// Dismissal and origin removal
// ---------------------------------------------------------------------------

#[test]
fn dismissal_destroys_the_preview_session_and_leaves_the_warm_origin_untouched() {
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    let opened = open_preview(&mut c, source, ProfileId(1), "https://preview.example");
    let preview_view = match opened {
        ControllerEvent::PreviewOpened { view, .. } => view,
        _ => unreachable!(),
    };
    h.reset_ops();

    let event = c.close_preview().unwrap();
    assert_eq!(event, ControllerEvent::PreviewClosed { view: preview_view });
    let snapshot = c.snapshot();
    assert!(snapshot.preview.is_none());
    assert_eq!(
        snapshot.engine_sessions, 1,
        "only the origin session survives"
    );
    assert_eq!(snapshot.tabs[0].web_view, Some(source));
    assert_eq!(snapshot.active_view, Some(source));
    assert!(h.recorded(&[EngineOp::Destroy { view: preview_view }]));
    // A second dismissal is a typed no-op error, never a double destroy.
    assert_eq!(c.close_preview(), Err(ControllerError::NoPreviewOpen));
}

#[test]
fn selecting_another_tab_ends_the_preview_by_origin_removal() {
    let (mut c, h) = plain();
    open(&mut c, "https://a.example");
    open(&mut c, "https://b.example");
    let b = c.snapshot().active_view.unwrap();
    open_preview(&mut c, b, ProfileId(1), "https://preview.example");
    let preview_view = c.snapshot().preview.unwrap().view;
    h.reset_ops();

    // Selecting tab A moves the active tab away from the preview origin: the
    // preview is torn down deterministically (no stacked preview over a
    // pane that no longer shows its source).
    c.select_tab(0).unwrap();
    let snapshot = c.snapshot();
    assert!(snapshot.preview.is_none());
    assert_eq!(
        snapshot.engine_sessions, 2,
        "both tabs warm, preview destroyed"
    );
    assert!(h.ops().contains(&EngineOp::Destroy { view: preview_view }));
}

#[test]
fn switching_spaces_ends_the_preview() {
    let (mut c, h) = plain();
    open(&mut c, "https://a.example");
    let source = c.snapshot().active_view.unwrap();
    open_preview(&mut c, source, ProfileId(1), "https://preview.example");
    let preview_view = c.snapshot().preview.unwrap().view;
    let other = c.create_space(ProfileId(1), "Other").unwrap();
    h.reset_ops();

    c.activate_space(other).unwrap();
    let snapshot = c.snapshot();
    assert!(snapshot.preview.is_none());
    assert_eq!(snapshot.engine_sessions, 1, "preview session destroyed");
    assert!(h.ops().contains(&EngineOp::Destroy { view: preview_view }));
}

// ---------------------------------------------------------------------------
// Promotion (Open as Tab)
// ---------------------------------------------------------------------------

#[test]
fn promotion_adopts_the_same_web_view_session_without_reload_or_profile_crossing() {
    let (mut c, h) = plain();
    open(&mut c, "https://source.example");
    let source = c.snapshot().active_view.unwrap();
    let opened = open_preview(&mut c, source, ProfileId(1), "https://preview.example");
    let preview_view = match opened {
        ControllerEvent::PreviewOpened { view, .. } => view,
        _ => unreachable!(),
    };
    h.reset_ops();

    let event = c.promote_preview().unwrap();
    let (_tab, index, view) = match event {
        ControllerEvent::PreviewPromoted {
            tab, index, view, ..
        } => (tab, index, view),
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(
        view, preview_view,
        "the SAME WebViewId becomes a regular tab"
    );
    assert_eq!(index, 1);

    let snapshot = c.snapshot();
    assert!(snapshot.preview.is_none());
    assert_eq!(snapshot.tabs.len(), 2);
    assert_eq!(snapshot.engine_sessions, 2);
    assert_eq!(snapshot.tabs[1].web_view, Some(preview_view));
    assert_eq!(snapshot.tabs[1].url, "https://preview.example");
    assert_eq!(snapshot.active_view, Some(preview_view));
    // No engine create/raise/destroy: the native session was adopted
    // atomically; only the origin became hidden (warm) under the now-regular
    // promoted tab.
    assert!(h.recorded(&[EngineOp::Show {
        view: source,
        visible: false,
    }]));
}

#[test]
fn promotion_preserves_live_navigation_state() {
    let (c, h) = crate::harness::wired();
    {
        let controller = &mut *c.borrow_mut();
        let _ = open(controller, "https://source.example");
        let source = controller.snapshot().active_view.unwrap();
        let _ = open_preview(controller, source, ProfileId(1), "https://preview.example");
    }
    let preview_view = c.borrow().snapshot().preview.unwrap().view;
    // Navigate inside the preview: the session's live URL/title/history.
    h.emit(WebEvent::UrlChanged(
        preview_view,
        "https://preview.example/next".into(),
    ));
    h.emit(WebEvent::TitleChanged(preview_view, "Preview Next".into()));
    h.emit(WebEvent::CanGoBackChanged(preview_view, true));

    let event = c.borrow_mut().promote_preview().unwrap();
    let (_, _, view) = match event {
        ControllerEvent::PreviewPromoted {
            tab, index, view, ..
        } => (tab, index, view),
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(view, preview_view);
    let snapshot = c.borrow().snapshot();
    let promoted = &snapshot.tabs[1];
    assert_eq!(promoted.web_view, Some(preview_view));
    // The promoted tab is the active view: the live session navigation
    // state (url/title/history) is preserved — nothing was reloaded.
    assert_eq!(
        snapshot.nav.url.as_deref(),
        Some("https://preview.example/next")
    );
    assert_eq!(snapshot.nav.title.as_deref(), Some("Preview Next"));
    assert!(snapshot.nav.can_go_back);
    // The persistent domain tab identity is seeded with the live title at
    // promotion: no reload means no fresh TitleChanged will ever arrive.
    assert_eq!(promoted.title, "Preview Next");
}

#[test]
fn a_crashed_preview_refuses_promotion_and_still_dismisses() {
    let (c, h) = crate::harness::wired();
    {
        let controller = &mut *c.borrow_mut();
        let _ = open(controller, "https://source.example");
        let source = controller.snapshot().active_view.unwrap();
        let _ = open_preview(controller, source, ProfileId(1), "https://preview.example");
    }
    let preview_view = c.borrow().snapshot().preview.unwrap().view;
    h.emit(WebEvent::PageCrashed(preview_view));
    assert!(c.borrow().snapshot().preview.unwrap().crashed);

    assert_eq!(
        c.borrow_mut().promote_preview(),
        Err(ControllerError::PreviewCrashed),
        "only a live session can be adopted without recreation"
    );
    // Dismissal after a crash still destroys the session cleanly.
    let event = c.borrow_mut().close_preview().unwrap();
    assert_eq!(event, ControllerEvent::PreviewClosed { view: preview_view });
    assert!(c.borrow().snapshot().preview.is_none());
    assert_eq!(c.borrow().snapshot().engine_sessions, 1);
}

// ---------------------------------------------------------------------------
// Cross-profile isolation
// ---------------------------------------------------------------------------

#[test]
fn a_preview_from_a_non_default_profile_uses_that_profiles_store() {
    let (mut c, h) = plain();
    c.create_profile_space("Work", "Work").unwrap();
    // Work profile is ProfileId(3) (1 default + 1 private in the shared
    // setup + this one); activate its space and open a tab there.
    let work_space = c.snapshot().spaces.last().unwrap().id;
    c.activate_space(work_space).unwrap();
    open(&mut c, "https://work.example");
    let source = c.snapshot().active_view.unwrap();
    let profile = c.snapshot().active_profile;
    assert_eq!(profile, ProfileId(3));

    let opened = open_preview(&mut c, source, profile, "https://preview.work");
    let preview_view = match opened {
        ControllerEvent::PreviewOpened {
            view, profile: p, ..
        } => {
            assert_eq!(p, ProfileId(3));
            view
        }
        _ => unreachable!(),
    };
    assert_eq!(
        h.ops().iter().find_map(|op| match op {
            EngineOp::Create { view, profile, .. } if *view == preview_view => Some(*profile),
            _ => None,
        }),
        Some(ProfileId(3)),
        "preview session lives in the source profile's store"
    );
    // A request claiming a different profile never crosses the boundary.
    assert_eq!(
        c.open_preview_from_new_window(source, &preview_session(source, ProfileId(2), "https://x")),
        Err(ControllerError::NewWindowProfileMismatch {
            source: ProfileId(3),
            requested: ProfileId(2),
        })
    );
}
