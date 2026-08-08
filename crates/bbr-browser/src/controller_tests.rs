//! Controller contract tests — API surface, first vertical slice, and
//! lifecycle/moves (browser-controller.md, "Tests and evidence").
//!
//! Every scenario drives the [`BrowserController`] through the exact
//! `Rc<RefCell<dyn WebEngine>>` seam against the deterministic
//! [`FakeEngine`], wired the same way the real app wires it: the engine's
//! sink forwards [`WebEvent`]s into `apply_event`. No macOS frameworks,
//! filesystem, or wall clock.
//!
//! Coverage map (browser-controller.md):
//! - API slice: setup, injected id generator, typed outcomes, snapshots.
//! - First vertical slice: open/select/close/reopen + navigation routing.
//! - Lifecycle: suspend/activate/crash, same- and cross-profile moves.
//! - Registry invariants after every mutation (asserted inside each op).
//! - Split and event-application coverage live in `controller_event_tests`.

use bbr_domain::{AppearanceError, BbrGlyph, SpaceEmoji, SpaceIdentity, SpaceTheme};
use bbr_web::{ProfileId, WebAction, WebEvent, WebViewId};

use crate::fake::EngineOp;
use crate::harness::{open, plain, wired};
use crate::{ControllerError, ControllerEvent};

// ---------------------------------------------------------------------------
// API slice
// ---------------------------------------------------------------------------

#[test]
fn setup_creates_a_profile_space_and_empty_snapshot() {
    let (c, h) = plain();
    let snap = c.snapshot();
    assert!(snap.tabs.is_empty());
    assert_eq!(snap.active_index, None);
    assert_eq!(snap.active_view, None);
    assert!(!snap.web_view_hosted);
    assert_eq!(snap.engine_sessions, 0);
    assert_eq!(h.engine.borrow().live_session_count(), 0);
    // Profile identity flows into created sessions (ProfileId(1) by domain
    // setup allocates the first profile before any tab).
    assert_eq!(c.active_profile(), ProfileId(1));
    // The empty pane names the owning profile from the snapshot (§11.8), so
    // the shell never hardcodes a profile label.
    assert_eq!(snap.active_profile_title.as_deref(), Some("Default"));
}

#[test]
fn profile_setup_and_space_activation_are_typed() {
    let (mut c, h) = plain();
    let event = c.create_profile_space("Private", "Private").unwrap();
    let (profile, space) = match event {
        ControllerEvent::ProfileCreated { profile, space } => (profile, space),
        other => panic!("unexpected event: {other:?}"),
    };
    assert_eq!(profile, ProfileId(3));
    open(&mut c, "https://default");
    let default_view = c.snapshot().active_view.unwrap();
    h.reset_ops();

    assert_eq!(
        c.activate_space(space).unwrap(),
        ControllerEvent::SpaceActivated { space }
    );
    assert!(c.snapshot().tabs.is_empty());
    // The active profile title follows activation into the new space.
    assert_eq!(
        c.snapshot().active_profile_title.as_deref(),
        Some("Private")
    );
    assert!(h.recorded(&[EngineOp::Show {
        view: default_view,
        visible: false,
    }]));

    h.reset_ops();
    open(&mut c, "https://private");
    assert!(h.recorded(&[
        EngineOp::Create {
            view: WebViewId(2),
            profile,
            url: "https://private".into(),
        },
        EngineOp::Show {
            view: WebViewId(2),
            visible: true,
        },
    ]));
}

#[test]
fn injected_allocator_yields_deterministic_sequential_ids() {
    let (mut c, h) = plain();
    open(&mut c, "https://a");
    open(&mut c, "https://b");
    assert!(h.recorded(&[
        EngineOp::Create {
            view: WebViewId(1),
            profile: ProfileId(1),
            url: "https://a".into()
        },
        EngineOp::Show {
            view: WebViewId(1),
            visible: true
        },
        EngineOp::Create {
            view: WebViewId(2),
            profile: ProfileId(1),
            url: "https://b".into()
        },
        EngineOp::Show {
            view: WebViewId(1),
            visible: false
        },
        EngineOp::Show {
            view: WebViewId(2),
            visible: true
        },
    ]));
}

#[test]
fn mutations_emit_typed_outcomes() {
    let (mut c, _h) = plain();
    assert_eq!(
        open(&mut c, "https://a"),
        ControllerEvent::TabOpened {
            index: 0,
            url: "https://a".into()
        }
    );
    assert_eq!(
        c.select_tab(0).unwrap(),
        ControllerEvent::TabSelected { index: 0 }
    );
    assert_eq!(
        c.close_tab(0).unwrap(),
        ControllerEvent::TabClosed { index: 0 }
    );
    assert_eq!(
        c.reopen_tab().unwrap(),
        ControllerEvent::TabReopened {
            index: 0,
            url: "https://a".into()
        }
    );
    assert_eq!(
        c.navigate(WebAction::Reload).unwrap(),
        ControllerEvent::NavigationRouted { view: WebViewId(2) }
    );
}

// ---------------------------------------------------------------------------
// First vertical slice
// ---------------------------------------------------------------------------

#[test]
fn open_tab_creates_a_session_and_shows_it() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 1);
    assert_eq!(snap.active_index, Some(0));
    assert_eq!(snap.tabs[0].url, "u1");
    assert!(snap.tabs[0].active);
    assert_eq!(snap.active_view, Some(WebViewId(1)));
    assert!(snap.web_view_hosted);
    assert_eq!(snap.engine_sessions, 1);
    assert!(h.recorded(&[
        EngineOp::Create {
            view: WebViewId(1),
            profile: ProfileId(1),
            url: "u1".into()
        },
        EngineOp::Show {
            view: WebViewId(1),
            visible: true
        },
    ]));
}

#[test]
fn opening_a_second_tab_hides_the_first_but_keeps_its_session_warm() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    h.reset_ops();
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 2);
    assert_eq!(snap.active_index, Some(1));
    assert_eq!(snap.engine_sessions, 2, "previous session stays warm");
    assert!(!c.snapshot().tabs[0].active);
    // No sessions were created or destroyed by the selection itself.
    assert!(h.ops().is_empty());
}

#[test]
fn selecting_a_tab_hides_the_previous_session_and_shows_the_selected_one() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    h.reset_ops();
    c.select_tab(0).unwrap();
    assert!(h.recorded(&[
        EngineOp::Show {
            view: WebViewId(1),
            visible: true
        },
        EngineOp::Show {
            view: WebViewId(2),
            visible: false
        },
    ]));
    let snap = c.snapshot();
    assert_eq!(snap.active_index, Some(0));
    assert_eq!(snap.active_view, Some(WebViewId(1)));
    assert_eq!(snap.engine_sessions, 2, "both sessions stay live");
}

#[test]
fn closing_the_active_tab_destroys_its_session_and_activates_the_neighbor() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    h.reset_ops();
    c.close_tab(1).unwrap();
    assert!(h.recorded(&[
        EngineOp::Destroy { view: WebViewId(2) },
        EngineOp::Show {
            view: WebViewId(1),
            visible: true
        },
    ]));
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 1);
    assert_eq!(snap.active_index, Some(0));
    assert_eq!(snap.engine_sessions, 1);
}

#[test]
fn closing_the_last_tab_leaves_the_space_empty() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    h.reset_ops();
    c.close_tab(0).unwrap();
    assert!(h.recorded(&[EngineOp::Destroy { view: WebViewId(1) }]));
    let snap = c.snapshot();
    assert!(snap.tabs.is_empty());
    assert_eq!(snap.active_index, None);
    assert_eq!(snap.engine_sessions, 0);
}

#[test]
fn reopen_tab_restores_identity_with_a_fresh_session() {
    let (c, h) = wired();
    {
        let mut c = c.borrow_mut();
        open(&mut c, "https://a");
    }
    // Navigation events persist the live URL/title onto the tab identity
    // through the exact sink wiring the composition root uses.
    h.emit(WebEvent::UrlChanged(WebViewId(1), "https://b".into()));
    h.emit(WebEvent::TitleChanged(WebViewId(1), "B".into()));
    let mut c = c.borrow_mut();
    c.close_tab(0).unwrap();
    h.reset_ops();
    assert_eq!(
        c.reopen_tab().unwrap(),
        ControllerEvent::TabReopened {
            index: 0,
            url: "https://b".into()
        }
    );
    assert!(h.recorded(&[
        EngineOp::Create {
            view: WebViewId(2),
            profile: ProfileId(1),
            url: "https://b".into()
        },
        EngineOp::Show {
            view: WebViewId(2),
            visible: true
        },
    ]));
    let snap = c.snapshot();
    assert_eq!(snap.tabs[0].url, "https://b");
    assert_eq!(snap.tabs[0].title, "B");
    assert_eq!(snap.active_view, Some(WebViewId(2)));
}

#[test]
fn navigation_routes_all_actions_to_the_focused_tab() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    h.reset_ops();
    for action in [
        WebAction::Back,
        WebAction::Forward,
        WebAction::Reload,
        WebAction::Stop,
        WebAction::Load {
            url: "https://x".into(),
        },
    ] {
        c.navigate(action).unwrap();
    }
    let ops = h.ops();
    assert_eq!(ops.len(), 5);
    assert!(ops.iter().all(|op| matches!(
        op,
        EngineOp::Action {
            view: WebViewId(2),
            ..
        }
    )));
    // After selecting the first tab, routing follows the new focus.
    // (Selection itself reconciles visibility, so the full sequence proves
    // that navigation then routes to the freshly focused pane without
    // redundant show/hide traffic.)
    h.reset_ops();
    c.select_tab(0).unwrap();
    c.navigate(WebAction::Back).unwrap();
    assert!(h.recorded(&[
        EngineOp::Show {
            view: WebViewId(1),
            visible: true
        },
        EngineOp::Show {
            view: WebViewId(2),
            visible: false
        },
        EngineOp::Action {
            view: WebViewId(1),
            action: WebAction::Back
        },
    ]));
}

#[test]
fn out_of_bounds_indices_are_typed_errors() {
    let (mut c, _h) = plain();
    assert_eq!(
        c.select_tab(3),
        Err(ControllerError::IndexOutOfBounds { index: 3, len: 0 })
    );
    assert_eq!(
        c.close_tab(0),
        Err(ControllerError::IndexOutOfBounds { index: 0, len: 0 })
    );
    assert!(matches!(
        c.navigate(WebAction::Back),
        Err(ControllerError::NoActiveTab)
    ));
}

// ---------------------------------------------------------------------------
// Lifecycle and moves
// ---------------------------------------------------------------------------

#[test]
fn suspending_a_hidden_tab_destroys_its_session_but_keeps_identity() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    h.reset_ops();
    assert_eq!(
        c.suspend_tab(0).unwrap(),
        ControllerEvent::TabSuspended { index: 0 }
    );
    assert!(h.recorded(&[EngineOp::Destroy { view: WebViewId(1) }]));
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 2, "identity kept");
    assert_eq!(snap.engine_sessions, 1, "only the active session survives");
}

#[test]
fn suspending_a_visible_tab_is_rejected() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    assert!(
        c.suspend_tab(0).is_err(),
        "visible tabs cannot be suspended"
    );
}

#[test]
fn selecting_a_suspended_tab_recreates_and_shows_its_session() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    c.suspend_tab(0).unwrap();
    h.reset_ops();
    c.select_tab(0).unwrap();
    assert!(h.recorded(&[
        EngineOp::Create {
            view: WebViewId(3),
            profile: ProfileId(1),
            url: "u1".into()
        },
        EngineOp::Show {
            view: WebViewId(3),
            visible: true
        },
        EngineOp::Show {
            view: WebViewId(2),
            visible: false
        },
    ]));
    let snap = c.snapshot();
    assert_eq!(snap.active_index, Some(0));
    assert_eq!(snap.active_view, Some(WebViewId(3)));
    assert_eq!(snap.engine_sessions, 2);
}

#[test]
fn moving_within_one_profile_keeps_the_live_session() {
    let (mut c, h) = plain();
    let space2 = {
        let profile = c.active_profile();
        c.create_space(profile, "Second").unwrap()
    };
    open(&mut c, "u1");
    h.reset_ops();
    let event = c.move_tab(0, space2).unwrap();
    // Domain ids come from one sequential counter: a fresh controller has
    // consumed 1 (profile) and 2 (space), the test space is 3, the tab 4.
    assert_eq!(
        event,
        ControllerEvent::TabMoved {
            tab: bbr_domain::TabId(4),
            to_space: space2,
            recreated: false
        }
    );
    assert!(h.recorded(&[EngineOp::Show {
        view: WebViewId(1),
        visible: false,
    }]));
    assert_eq!(c.snapshot().engine_sessions, 1);
    h.reset_ops();
    c.activate_space(space2).unwrap();
    assert!(h.recorded(&[EngineOp::Show {
        view: WebViewId(1),
        visible: true,
    }]));
    assert_eq!(c.snapshot().active_view, Some(WebViewId(1)));
}

#[test]
fn moving_across_profiles_recreates_the_session_in_the_destination_store() {
    let (mut c, h) = plain();
    let profile2 = c.create_profile("Private");
    let space_p2 = c.create_space(profile2, "Private Space").unwrap();
    open(&mut c, "https://secret.example");
    h.reset_ops();
    let event = c.move_tab(0, space_p2).unwrap();
    // Domain ids: 1 profile, 2 space (controller setup), 3 profile (Private),
    // 4 space, so the fresh tab is 5 and the Private profile is 3.
    assert_eq!(
        event,
        ControllerEvent::TabMoved {
            tab: bbr_domain::TabId(5),
            to_space: space_p2,
            recreated: true
        }
    );
    assert!(h.recorded(&[
        EngineOp::Destroy { view: WebViewId(1) },
        EngineOp::Create {
            view: WebViewId(2),
            profile: ProfileId(3),
            url: "https://secret.example".into()
        },
    ]));
    assert_eq!(c.snapshot().engine_sessions, 1);
    h.reset_ops();
    c.activate_space(space_p2).unwrap();
    assert!(h.recorded(&[EngineOp::Show {
        view: WebViewId(2),
        visible: true,
    }]));
    assert_eq!(c.snapshot().active_view, Some(WebViewId(2)));
}

#[test]
fn page_crash_marks_the_owning_tab_discarded_and_destroys_its_session() {
    let (c, h) = wired();
    {
        let mut c = c.borrow_mut();
        open(&mut c, "u1");
    }
    // The wired sink routes the engine event into apply_event exactly like
    // the composition root; the effect is observable through state (events
    // are not echoed back to the controller call site in the app, either).
    h.emit(WebEvent::PageCrashed(WebViewId(1)));
    let snap = c.borrow().snapshot();
    assert_eq!(snap.tabs.len(), 1, "identity survives a crash");
    assert_eq!(snap.engine_sessions, 0, "crashed session is destroyed");
    // Activation recreates the session with a fresh incarnation.
    {
        let mut c = c.borrow_mut();
        c.select_tab(0).unwrap();
    }
    let snap = c.borrow().snapshot();
    assert_eq!(snap.active_view, Some(WebViewId(2)));
    assert_eq!(snap.engine_sessions, 1);
}

#[test]
fn discarding_via_crash_reuses_the_tab_id_for_recreation() {
    let (c, h) = wired();
    {
        let mut c = c.borrow_mut();
        open(&mut c, "u1");
    }
    h.reset_ops();
    h.emit(WebEvent::PageCrashed(WebViewId(1)));
    {
        let mut c = c.borrow_mut();
        c.select_tab(0).unwrap();
    }
    assert!(h.recorded(&[
        EngineOp::Destroy { view: WebViewId(1) },
        EngineOp::Create {
            view: WebViewId(2),
            profile: ProfileId(1),
            url: "u1".into()
        },
        EngineOp::Show {
            view: WebViewId(2),
            visible: true
        },
    ]));
    // The new session's events target the same tab again.
    h.emit(WebEvent::UrlChanged(WebViewId(2), "https://new".into()));
    assert_eq!(c.borrow().snapshot().tabs[0].url, "https://new");
}

// ---------------------------------------------------------------------------
// Visible-tab shell slice: pinned region, reopen, and per-tab visual state.
// ---------------------------------------------------------------------------

#[test]
fn pin_moves_the_tab_into_the_pinned_region_without_recreating_its_session() {
    let (mut c, h) = plain();
    open(&mut c, "https://a");
    open(&mut c, "https://b");
    let a_view = c.snapshot().tabs[0].web_view;
    h.reset_ops();
    let event = c.pin_tab(1).unwrap();
    // `TabPinned.index` reports the tab's new position: the domain moves the
    // pinned tab to the end of the pinned region (index 0 when empty).
    assert_eq!(event, ControllerEvent::TabPinned { index: 0 });
    let snap = c.snapshot();
    assert!(snap.tabs[0].pinned, "pinned tab moves to the front");
    assert!(!snap.tabs[1].pinned);
    assert_eq!(snap.tabs[1].web_view, a_view, "session identity preserved");
    assert_eq!(snap.engine_sessions, 2, "pin never touches sessions");
    // Re-pinning an already-pinned tab is a no-op (domain guard).
    assert!(c.pin_tab(0).is_ok());
}

#[test]
fn unpin_moves_the_tab_to_the_regular_region_without_touching_sessions() {
    let (mut c, h) = plain();
    open(&mut c, "https://a");
    open(&mut c, "https://b");
    let _ = c.pin_tab(1).unwrap();
    let b_view = c.snapshot().tabs[0].web_view;
    h.reset_ops();
    let event = c.unpin_tab(0).unwrap();
    assert_eq!(event, ControllerEvent::TabUnpinned { index: 1 });
    let snap = c.snapshot();
    assert!(
        !snap.tabs[0].pinned,
        "unpinned tab leaves the pinned region"
    );
    assert!(!snap.tabs[1].pinned);
    assert_eq!(snap.tabs[1].web_view, b_view, "session identity preserved");
    assert_eq!(snap.engine_sessions, 2, "unpin never touches sessions");
    // Unpinning an already-regular tab is a no-op (domain guard).
    assert!(c.unpin_tab(1).is_ok());
}

#[test]
fn reorder_within_a_region_preserves_selection_session_and_order_elsewhere() {
    let (mut c, h) = plain();
    open(&mut c, "https://a");
    open(&mut c, "https://b");
    open(&mut c, "https://c");
    let _ = c.pin_tab(2).unwrap(); // pinned: [c, a, b] -> pinned [c], regular [a, b]
    let views: Vec<_> = c.snapshot().tabs.iter().map(|t| t.web_view).collect();
    h.reset_ops();
    // Reorder within the regular region: [c, a, b] -> [c, b, a].
    let event = c.reorder_tab(2, 1).unwrap();
    assert_eq!(event, ControllerEvent::TabReordered { from: 2, to: 1 });
    let snap = c.snapshot();
    let urls: Vec<&str> = snap.tabs.iter().map(|t| t.url.as_str()).collect();
    assert_eq!(urls, vec!["https://c", "https://b", "https://a"]);
    assert!(snap.tabs[0].pinned, "pinned region untouched");
    assert_eq!(
        snap.tabs.iter().map(|t| t.web_view).collect::<Vec<_>>(),
        [views[0], views[2], views[1]],
        "reorder never recreates sessions; each view follows its tab"
    );
    assert_eq!(snap.engine_sessions, 3);
}

#[test]
fn reorder_across_the_pinned_boundary_is_rejected_whole() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a");
    open(&mut c, "https://b");
    let _ = c.pin_tab(1).unwrap(); // [b(pinned), a]
    let before: Vec<String> = c.snapshot().tabs.iter().map(|t| t.title.clone()).collect();
    // Moving the pinned tab into the regular region and vice versa are both
    // rejected; the complete operation is a no-op (no partial mutation).
    assert!(c.reorder_tab(0, 1).is_err());
    assert!(c.reorder_tab(1, 0).is_err());
    let after: Vec<String> = c.snapshot().tabs.iter().map(|t| t.title.clone()).collect();
    assert_eq!(before, after, "rejected reorder mutates nothing");
    assert_eq!(c.snapshot().engine_sessions, 2, "sessions untouched");
}

#[test]
fn reorder_keeps_the_selected_tab_selected() {
    let (mut c, _h) = plain();
    open(&mut c, "https://a");
    open(&mut c, "https://b");
    open(&mut c, "https://c");
    // Select the last tab, then reorder it to the front.
    let _ = c.select_tab(2).unwrap();
    let _ = c.reorder_tab(2, 0).unwrap();
    let snap = c.snapshot();
    assert_eq!(snap.active_index, Some(0), "selected tab follows the move");
    assert!(snap.tabs[0].active);
}

#[test]
fn reopen_restores_identity_with_a_fresh_session() {
    // `wired()` routes engine events through `apply_event` exactly like the
    // composition root; `plain()` has no sink, so emitted title/loading
    // events would be dropped.
    let (controller, h) = wired();
    let (a_view, a_title) = {
        let mut c = controller.borrow_mut();
        open(&mut c, "https://a");
        let a_view = c.snapshot().active_view.unwrap();
        drop(c);
        // The wired sink routes the emitted event into `apply_event`; the
        // controller borrow must be released first (single-threaded Rc).
        h.emit(WebEvent::TitleChanged(a_view, "Alpha".into()));
        (a_view, controller.borrow().snapshot().tabs[0].title.clone())
    };
    assert_eq!(a_title, "Alpha", "title event reached the domain state");
    {
        let mut c = controller.borrow_mut();
        let _ = c.close_tab(0).unwrap();
        assert_eq!(c.snapshot().engine_sessions, 0);
    }
    h.reset_ops();
    let event = controller.borrow_mut().reopen_tab().unwrap();
    assert_eq!(
        event,
        ControllerEvent::TabReopened {
            index: 0,
            url: "https://a".into()
        }
    );
    let snap = controller.borrow().snapshot();
    assert_eq!(snap.tabs.len(), 1);
    assert_eq!(snap.tabs[0].title, "Alpha", "preserved title identity");
    assert_eq!(snap.tabs[0].url, "https://a");
    assert_ne!(snap.active_view, Some(a_view), "fresh session incarnation");
    assert_eq!(snap.engine_sessions, 1);
    assert!(h.recorded(&[
        EngineOp::Create {
            view: WebViewId(2),
            profile: ProfileId(1),
            url: "https://a".into()
        },
        EngineOp::Show {
            view: WebViewId(2),
            visible: true
        },
    ]));
}

#[test]
fn snapshot_exposes_loading_and_crash_state_per_tab() {
    let (c, h) = wired();
    let view = {
        let mut inner = c.borrow_mut();
        open(&mut inner, "https://a");
        let view = inner.snapshot().active_view.unwrap();
        drop(inner);
        // Crash events arrive through the engine sink; release the borrow
        // so `apply_event` can take it.
        view
    };
    h.emit(WebEvent::LoadingChanged(view, true));
    let snap = c.borrow().snapshot();
    assert!(snap.tabs[0].loading);
    assert!(!snap.tabs[0].crashed);
    assert!(snap.tabs[0].active);
    // A crash marks the tab error until a fresh session loads.
    h.emit(WebEvent::PageCrashed(view));
    let snap = c.borrow().snapshot();
    assert!(
        snap.tabs[0].crashed,
        "crash flag survives session destruction"
    );
    assert!(!snap.tabs[0].loading);
    assert_eq!(snap.tabs[0].web_view, None);
    c.borrow_mut().select_tab(0).unwrap(); // recreation clears the flag
    let snap = c.borrow().snapshot();
    assert!(
        !snap.tabs[0].crashed,
        "fresh session clears the crash state"
    );
}

// ---------------------------------------------------------------------------
// Space hierarchy: complete collections, per-space active tabs, and warm
// session preservation across view-only switches.
// ---------------------------------------------------------------------------

#[test]
fn snapshot_exposes_complete_profile_and_space_collections() {
    let (mut c, h) = plain();
    let default_space = c.snapshot().active_space;
    let (private_profile, private_space) = match c.create_profile_space("Private", "Private") {
        Ok(ControllerEvent::ProfileCreated { profile, space }) => (profile, space),
        other => panic!("unexpected event: {other:?}"),
    };
    let reading_space = match c.create_space_event(ProfileId(1), "Reading") {
        Ok(ControllerEvent::SpaceCreated { space, .. }) => space,
        other => panic!("unexpected event: {other:?}"),
    };
    open(&mut c, "https://default");
    c.activate_space(private_space).unwrap();
    open(&mut c, "https://private");

    let snap = c.snapshot();
    // Complete collections in creation order, not just the active space.
    assert_eq!(
        snap.profiles
            .iter()
            .map(|p| (p.index, p.id, p.title.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (0, ProfileId(1), "Default"),
            (1, private_profile, "Private"),
        ]
    );
    assert_eq!(
        snap.spaces
            .iter()
            .map(|s| (s.index, s.id, s.title.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (0, default_space, "Default"),
            (1, private_space, "Private"),
            (2, reading_space, "Reading"),
        ]
    );
    // Per-space active tab + live session: the private space's own tab is
    // active and hosted while the default space's tab stays warm.
    let private = snap.spaces.iter().find(|s| s.id == private_space).unwrap();
    assert!(private.active);
    assert_eq!(private.active_tab, Some(0));
    assert_eq!(private.active_view, snap.active_view);
    let default = snap.spaces.iter().find(|s| s.id == default_space).unwrap();
    assert!(!default.active);
    assert_eq!(default.active_tab, Some(0));
    assert_eq!(default.active_view, Some(WebViewId(1)));
    assert_eq!(snap.engine_sessions, 2);
    assert_eq!(h.engine.borrow().live_session_count(), 2);
    assert_eq!(snap.all_tabs.len(), 2);
    assert_ne!(snap.all_tabs[0].id, snap.all_tabs[1].id);
    assert_eq!(
        snap.all_tabs
            .iter()
            .map(|tab| (
                tab.space,
                tab.profile,
                tab.index,
                tab.url.as_str(),
                tab.web_view
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                default_space,
                ProfileId(1),
                0,
                "https://default",
                Some(WebViewId(1)),
            ),
            (
                private_space,
                private_profile,
                0,
                "https://private",
                Some(WebViewId(2)),
            ),
        ],
        "complete tab read model preserves owning identities, order, and warm sessions"
    );
    // Profile identity hues flow through the snapshot (DESIGN.md §12).
    assert_eq!(snap.profiles[0].hue.name(), "cobalt");
    assert_eq!(snap.profiles[1].hue.name(), "violet");
    assert_eq!(
        snap.active_profile_hue,
        Some(bbr_domain::ProfileHue::Violet),
        "active profile hue follows the active space"
    );
}

#[test]
fn each_space_remembers_its_active_tab() {
    let (mut c, h) = plain();
    open(&mut c, "https://a");
    open(&mut c, "https://b");
    c.select_tab(0).unwrap();
    let default_space = c.snapshot().active_space;
    let second = match c.create_space_event(ProfileId(1), "Two") {
        Ok(ControllerEvent::SpaceCreated { space, .. }) => space,
        other => panic!("unexpected event: {other:?}"),
    };
    c.activate_space(second).unwrap();
    open(&mut c, "https://c");

    // Space 1's active tab is tab 0; space 2's is tab 0 of its own list.
    let snap = c.snapshot();
    let s1 = snap.spaces.iter().find(|s| s.id == default_space).unwrap();
    let s2 = snap.spaces.iter().find(|s| s.id == second).unwrap();
    assert_eq!(s1.active_tab, Some(0));
    assert_eq!(s1.active_view, Some(WebViewId(1)));
    assert_eq!(s2.active_tab, Some(0));
    assert_eq!(s2.active_view, Some(WebViewId(3)));

    // Switching back restores space 1's active tab without recreation.
    h.reset_ops();
    c.activate_space(default_space).unwrap();
    let snap = c.snapshot();
    assert_eq!(snap.active_index, Some(0));
    assert_eq!(snap.active_view, Some(WebViewId(1)));
    assert_eq!(snap.engine_sessions, 3, "all sessions stay warm");
    assert!(
        h.recorded(&[
            EngineOp::Show {
                view: WebViewId(1),
                visible: true,
            },
            EngineOp::Show {
                view: WebViewId(3),
                visible: false,
            },
        ]),
        "domain order: default-space tab first, then the now-hidden space"
    );
}

#[test]
fn cross_profile_switch_preserves_warm_sessions_without_recreation() {
    let (mut c, h) = plain();
    let default_space = c.snapshot().active_space;
    open(&mut c, "https://default");
    let default_view = c.snapshot().active_view.unwrap();
    let (private_profile, private_space) = match c.create_profile_space("Private", "Private") {
        Ok(ControllerEvent::ProfileCreated { profile, space }) => (profile, space),
        other => panic!("unexpected event: {other:?}"),
    };
    c.activate_space(private_space).unwrap();
    open(&mut c, "https://private");
    let private_view = c.snapshot().active_view.unwrap();
    assert_ne!(private_profile, ProfileId(1), "private profile differs");

    // View-only switches across profile boundaries must not recreate either
    // session: each tab keeps its own data-store-backed session, and no
    // create/destroy op crosses the wire (the recreation policy fires only
    // when a tab actually moves across profiles).
    h.reset_ops();
    c.activate_space(default_space).unwrap();
    let snap = c.snapshot();
    assert_eq!(snap.active_profile, ProfileId(1));
    assert_eq!(snap.active_view, Some(default_view));
    assert_eq!(snap.engine_sessions, 2);
    assert!(
        h.recorded(&[
            EngineOp::Show {
                view: default_view,
                visible: true,
            },
            EngineOp::Show {
                view: private_view,
                visible: false,
            },
        ]),
        "cross-profile view-only switch must only show/hide, never recreate"
    );

    h.reset_ops();
    c.activate_space(private_space).unwrap();
    let snap = c.snapshot();
    assert_eq!(snap.active_profile, private_profile);
    assert_eq!(snap.active_view, Some(private_view));
    assert_eq!(snap.engine_sessions, 2);
    assert!(
        h.recorded(&[
            EngineOp::Show {
                view: default_view,
                visible: false,
            },
            EngineOp::Show {
                view: private_view,
                visible: true,
            },
        ]),
        "domain order: default-space tab first, then the now-visible private"
    );
}

#[test]
fn empty_space_activation_restores_no_active_tab_and_no_session_loss() {
    let (mut c, h) = plain();
    let default_space = c.snapshot().active_space;
    open(&mut c, "https://a");
    let a_view = c.snapshot().active_view.unwrap();
    let second = match c.create_space_event(ProfileId(1), "Empty") {
        Ok(ControllerEvent::SpaceCreated { space, .. }) => space,
        other => panic!("unexpected event: {other:?}"),
    };
    c.activate_space(second).unwrap();
    let snap = c.snapshot();
    assert!(snap.tabs.is_empty());
    assert_eq!(snap.active_index, None);
    assert_eq!(snap.active_view, None);
    assert!(!snap.web_view_hosted);
    assert_eq!(snap.engine_sessions, 1, "the other space's tab stays warm");
    // The empty pane names the owning profile from the snapshot.
    assert_eq!(snap.active_profile_title.as_deref(), Some("Default"));

    // Back to the populated space: warm tab restored, no recreation.
    c.activate_space(default_space).unwrap();
    let snap = c.snapshot();
    assert_eq!(snap.active_view, Some(a_view));
    assert_eq!(snap.engine_sessions, 1);
    assert_eq!(h.engine.borrow().live_session_count(), 1);
}

// ---------------------------------------------------------------------------
// Space appearance (portable appearance contract)
// ---------------------------------------------------------------------------

#[test]
fn space_appearance_derives_defaults_from_the_profile_hue() {
    let (c, _h) = plain();
    let snap = c.snapshot();
    let default_space = snap.active_space;
    // The default profile is Cobalt (first §12 hue): the derived appearance
    // maps the theme 1:1 from the hue and uses the default curated glyph.
    let appearance = c.space_appearance(default_space).expect("space exists");
    assert_eq!(
        appearance.theme,
        SpaceTheme::from_profile_hue(crate::ProfileHue::Cobalt)
    );
    assert_eq!(appearance.identity, SpaceIdentity::default());
    assert_eq!(appearance.theme.name(), "cobalt");
    assert_eq!(appearance.identity.name(), "code");
    // Unknown spaces derive no appearance (defensive; invariants forbid).
    assert_eq!(c.space_appearance(crate::SpaceId(999)), None);
}

#[test]
fn explicit_appearance_overrides_the_derived_default() {
    let (mut c, _h) = plain();
    let space = c.snapshot().active_space;
    // A validated curated emoji + the Pine space-only token.
    let emoji = SpaceEmoji::new("🔥").expect("curated emoji");
    c.set_space_appearance(space, SpaceTheme::Pine, SpaceIdentity::Emoji(emoji))
        .expect("set appearance");
    let appearance = c.space_appearance(space).expect("space exists");
    assert_eq!(appearance.theme, SpaceTheme::Pine);
    assert_eq!(
        appearance.identity,
        SpaceIdentity::Emoji(SpaceEmoji::new("🔥").expect("curated"))
    );
    // The explicit record is atomic: identity-only and theme-only setters
    // preserve the other half.
    c.set_space_identity(space, SpaceIdentity::Glyph(BbrGlyph::Terminal))
        .expect("set identity");
    let appearance = c.space_appearance(space).expect("space exists");
    assert_eq!(
        appearance.identity,
        SpaceIdentity::Glyph(BbrGlyph::Terminal)
    );
    assert_eq!(
        appearance.theme,
        SpaceTheme::Pine,
        "theme survives an identity-only update"
    );
    c.set_space_theme(space, SpaceTheme::Plum)
        .expect("set theme");
    let appearance = c.space_appearance(space).expect("space exists");
    assert_eq!(
        appearance.theme,
        SpaceTheme::Plum,
        "theme update preserves the explicit identity"
    );
    assert_eq!(
        appearance.identity,
        SpaceIdentity::Glyph(BbrGlyph::Terminal),
        "identity survives a theme-only update"
    );
}

#[test]
fn unknown_space_appearance_mutation_rejects_whole() {
    let (mut c, _h) = plain();
    let unknown = crate::SpaceId(4242);
    let err = c
        .set_space_appearance(unknown, SpaceTheme::Pine, SpaceIdentity::default())
        .expect_err("unknown space rejects whole");
    assert_eq!(err, AppearanceError::UnknownSpace(unknown));
    // None of the setters partially mutates: the store stays empty and the
    // controller invariants (including the appearance store's) hold.
    assert_eq!(
        c.set_space_theme(unknown, SpaceTheme::Plum),
        Err(err.clone())
    );
    assert_eq!(
        c.set_space_identity(unknown, SpaceIdentity::default()),
        Err(err)
    );
    assert_eq!(c.space_appearance(unknown), None);
}
