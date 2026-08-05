//! Contract tests for the deterministic fake [`WebEngine`] (fake-web-engine.md).
//!
//! The fake itself lives in [`crate::fake`]; this module is the
//! `#[cfg(test)]` suite that proves the contract the controller and future
//! native adapter must honour: creation records profile+URL, show/hide never
//! creates or destroys, destroy removes exactly once, actions and focus are
//! id-aware, events preserve ordering and IDs, sessions stay independent, and
//! the whole thing works behind the `Rc<RefCell<dyn WebEngine>>` seam the
//! controller drives.

use std::cell::RefCell;
use std::rc::Rc;

use bbr_web::{ProfileId, WebAction, WebEngine, WebEvent, WebViewId};

use crate::fake::{EngineOp, FakeEngine};

// ---------------------------------------------------------------------------
// Contract tests (fake-web-engine.md)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fake() -> FakeEngine {
        FakeEngine::new()
    }

    #[test]
    fn creation_records_profile_and_url() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "https://a.example");
        let ops = f.ops().borrow().clone();
        assert_eq!(
            ops,
            vec![EngineOp::Create {
                view: WebViewId(1),
                profile: ProfileId(7),
                url: "https://a.example".into()
            }]
        );
        let session = f.session(WebViewId(1)).unwrap();
        assert_eq!(session.profile, ProfileId(7));
        assert_eq!(session.url, "https://a.example");
    }

    #[test]
    fn show_and_hide_never_create_or_destroy() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "https://a.example");
        f.reset_ops();
        f.show(WebViewId(1), true);
        f.show(WebViewId(1), false);
        assert_eq!(f.live_session_count(), 1);
        assert_eq!(
            f.ops().borrow().clone(),
            vec![
                EngineOp::Show {
                    view: WebViewId(1),
                    visible: true
                },
                EngineOp::Show {
                    view: WebViewId(1),
                    visible: false
                },
            ]
        );
        assert!(!f.session(WebViewId(1)).unwrap().visible);
    }

    #[test]
    fn fake_runs_behind_the_dyn_web_engine_seam() {
        // The controller (and the real app) drive the engine through
        // `Rc<RefCell<dyn WebEngine>>`; prove the fake survives that exact
        // seam, including its observation handles.
        let harness = FakeEngine::new().into_harness();
        harness.engine.borrow_mut().create_web_view(
            WebViewId(1),
            ProfileId(7),
            "https://a.example",
        );
        assert_eq!(
            harness.ops(),
            vec![EngineOp::Create {
                view: WebViewId(1),
                profile: ProfileId(7),
                url: "https://a.example".into()
            }]
        );
        harness.engine.borrow_mut().show(WebViewId(1), true);
        harness.engine.borrow_mut().show(WebViewId(1), false);
        assert!(harness.recorded(&[
            EngineOp::Create {
                view: WebViewId(1),
                profile: ProfileId(7),
                url: "https://a.example".into()
            },
            EngineOp::Show {
                view: WebViewId(1),
                visible: true
            },
            EngineOp::Show {
                view: WebViewId(1),
                visible: false
            },
        ]));
        assert_eq!(
            harness.engine.borrow().live_session_count(),
            1,
            "show/hide must not create or destroy"
        );
        harness.engine.borrow_mut().destroy_web_view(WebViewId(1));
        assert_eq!(harness.engine.borrow().live_session_count(), 0);
        harness.emit(WebEvent::LoadingChanged(WebViewId(1), false));
    }

    #[test]
    #[should_panic(expected = "duplicate creation")]
    fn duplicate_creation_rejected() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "u1");
        f.create_web_view(WebViewId(1), ProfileId(7), "u2");
    }

    #[test]
    #[should_panic(expected = "missing session")]
    fn actions_against_missing_sessions_rejected() {
        let mut f = fake();
        f.action(WebViewId(99), WebAction::Reload);
    }

    #[test]
    #[should_panic(expected = "destroy of unknown")]
    fn destroy_removes_the_session_exactly_once() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "u");
        f.destroy_web_view(WebViewId(1));
        assert_eq!(f.live_session_count(), 0);
        // Second destroy of the same id is a contract violation.
        f.destroy_web_view(WebViewId(1));
    }

    #[test]
    fn actions_target_only_existing_ids() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "u");
        f.reset_ops();
        f.action(WebViewId(1), WebAction::Back);
        f.action(WebViewId(1), WebAction::Forward);
        f.action(WebViewId(1), WebAction::Reload);
        f.action(WebViewId(1), WebAction::Stop);
        f.action(
            WebViewId(1),
            WebAction::Load {
                url: "https://b".into(),
            },
        );
        assert_eq!(f.ops().borrow().len(), 5);
        assert!(f.ops().borrow().iter().all(|op| matches!(
            op,
            EngineOp::Action { view, .. } if *view == WebViewId(1)
        )));
    }

    #[test]
    fn event_injection_preserves_ordering_and_ids() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "u1");
        f.create_web_view(WebViewId(2), ProfileId(7), "u2");
        let received = Rc::new(RefCell::new(Vec::new()));
        let sink_recv = received.clone();
        f.set_events(Box::new(move |ev| sink_recv.borrow_mut().push(ev)));

        f.navigate_session(WebViewId(1), "https://a", "A", false);
        f.navigate_session(WebViewId(2), "https://b", "B", true);
        f.set_loading(WebViewId(1), true);
        f.crash(WebViewId(1));

        // Exact order: session1 state, session2 state, session2 can_go_back,
        // loading, crash — all with their own ids.
        assert_eq!(
            *received.borrow(),
            vec![
                WebEvent::UrlChanged(WebViewId(1), "https://a".into()),
                WebEvent::TitleChanged(WebViewId(1), "A".into()),
                WebEvent::CanGoBackChanged(WebViewId(1), false),
                WebEvent::CanGoForwardChanged(WebViewId(1), false),
                WebEvent::LoadingChanged(WebViewId(1), false),
                WebEvent::UrlChanged(WebViewId(2), "https://b".into()),
                WebEvent::TitleChanged(WebViewId(2), "B".into()),
                WebEvent::CanGoBackChanged(WebViewId(2), true),
                WebEvent::CanGoForwardChanged(WebViewId(2), false),
                WebEvent::LoadingChanged(WebViewId(2), false),
                WebEvent::LoadingChanged(WebViewId(1), true),
                WebEvent::PageCrashed(WebViewId(1)),
            ]
        );
    }

    #[test]
    fn two_sessions_remain_independent() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "u1");
        f.create_web_view(WebViewId(2), ProfileId(8), "u2");
        f.navigate_session(WebViewId(1), "https://a", "A", true);
        f.navigate_session(WebViewId(2), "https://b", "B", false);
        let s1 = f.session(WebViewId(1)).unwrap();
        let s2 = f.session(WebViewId(2)).unwrap();
        assert_eq!(s1.profile, ProfileId(7));
        assert_eq!(s2.profile, ProfileId(8));
        assert_eq!(s1.url, "https://a");
        assert_eq!(s2.url, "https://b");
        assert_ne!(s1, s2);
        f.destroy_web_view(WebViewId(1));
        assert!(f.session(WebViewId(2)).is_some());
        assert_eq!(f.live_session_count(), 1);
    }

    #[test]
    fn operation_log_resets_without_resetting_sessions() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "u");
        f.show(WebViewId(1), true);
        f.reset_ops();
        assert!(f.ops().borrow().is_empty(), "op log must clear");
        assert_eq!(f.live_session_count(), 1);
        assert!(f.session(WebViewId(1)).unwrap().visible);
    }

    #[test]
    fn failed_navigation_emits_an_error_event_and_stops_loading() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "https://bad.example");
        f.set_loading(WebViewId(1), true);
        let received = Rc::new(RefCell::new(Vec::new()));
        let sink_recv = received.clone();
        f.set_events(Box::new(move |ev| sink_recv.borrow_mut().push(ev)));
        f.fail_navigation(WebViewId(1), "certificate refused");
        assert_eq!(
            *received.borrow(),
            vec![WebEvent::NavigationFailed(
                WebViewId(1),
                "certificate refused".into()
            )]
        );
        assert!(
            !f.session(WebViewId(1)).unwrap().loading,
            "failure stops loading"
        );
    }

    #[test]
    fn focus_requests_are_id_aware() {
        let mut f = fake();
        f.create_web_view(WebViewId(1), ProfileId(7), "u");
        f.create_web_view(WebViewId(2), ProfileId(7), "u2");
        f.reset_ops();
        f.focus_native_web(WebViewId(2));
        f.focus_host_chrome();
        assert_eq!(
            f.ops().borrow().clone(),
            vec![
                EngineOp::FocusWeb { view: WebViewId(2) },
                EngineOp::FocusChrome,
            ]
        );
    }
}
