//! Deterministic fake [`WebEngine`] for controller contract tests
//! (`cfg(test)` only: its only consumer is this crate's own suites, so per the
//! task packet's delivery choice it lives here rather than in a speculative
//! `bbr-web-fake` crate).
//!
//! The fake records every engine call into an operation log, owns fake
//! sessions keyed by [`WebViewId`], rejects duplicate creation and
//! missing-session actions, and emits controlled [`WebEvent`]s through the
//! registered sink. It is framework-free: no GPUI, no macOS, no wall clock.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use bbr_web::{ProfileId, WebAction, WebEngine, WebEvent, WebRect, WebViewId};

use crate::EngineRef;

/// One recorded engine call, in order.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineOp {
    AttachSlot {
        rect: WebRect,
    },
    Create {
        view: WebViewId,
        profile: ProfileId,
        url: String,
    },
    Show {
        view: WebViewId,
        visible: bool,
    },
    Action {
        view: WebViewId,
        action: WebAction,
    },
    Destroy {
        view: WebViewId,
    },
    FocusWeb {
        view: WebViewId,
    },
    FocusChrome,
}

/// Fake per-session web state.
#[derive(Debug, Clone, PartialEq)]
pub struct FakeSession {
    pub profile: ProfileId,
    pub visible: bool,
    pub url: String,
    pub title: String,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

impl FakeSession {
    fn new(profile: ProfileId, url: &str) -> Self {
        Self {
            profile,
            visible: false,
            url: url.to_string(),
            title: String::new(),
            loading: false,
            can_go_back: false,
            can_go_forward: false,
        }
    }
}

type EventSink = Rc<RefCell<Option<Box<dyn FnMut(WebEvent)>>>>;

/// The fake engine. Shared handles (`ops`, `sink`) let tests observe calls
/// and inject events even when the engine is type-erased behind
/// `Rc<RefCell<dyn WebEngine>>`.
pub struct FakeEngine {
    ops: Rc<RefCell<Vec<EngineOp>>>,
    sessions: HashMap<WebViewId, FakeSession>,
    sink: EventSink,
}

impl FakeEngine {
    /// Create a fake plus the shared operation log and event sink.
    pub fn new() -> Self {
        Self {
            ops: Rc::new(RefCell::new(Vec::new())),
            sessions: HashMap::new(),
            sink: Rc::new(RefCell::new(None)),
        }
    }

    /// Wrap the fake into the exact `dyn WebEngine` handle the controller
    /// drives, keeping the observation handles.
    pub fn into_harness(self) -> FakeHarness {
        let ops = self.ops.clone();
        let sink = self.sink.clone();
        let engine: EngineRef = Rc::new(RefCell::new(self));
        FakeHarness { engine, ops, sink }
    }

    pub fn ops(&self) -> &Rc<RefCell<Vec<EngineOp>>> {
        &self.ops
    }

    pub fn session(&self, view: WebViewId) -> Option<&FakeSession> {
        self.sessions.get(&view)
    }

    pub fn live_session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Clear the recorded operations without touching sessions.
    pub fn reset_ops(&self) {
        self.ops.borrow_mut().clear();
    }

    /// Emit one event through the registered sink (no-op without a sink).
    pub fn emit(&self, event: WebEvent) {
        if let Some(sink) = self.sink.borrow_mut().as_mut() {
            sink(event);
        }
    }

    /// Push the full navigation state for a session (what a real WKWebView's
    /// delegate emits after didFinish / on a title KVO).
    pub fn emit_state(&self, view: WebViewId) {
        let Some(session) = self.sessions.get(&view) else {
            return;
        };
        self.emit(WebEvent::UrlChanged(view, session.url.clone()));
        if !session.title.is_empty() {
            self.emit(WebEvent::TitleChanged(view, session.title.clone()));
        }
        self.emit(WebEvent::CanGoBackChanged(view, session.can_go_back));
        self.emit(WebEvent::CanGoForwardChanged(view, session.can_go_forward));
        self.emit(WebEvent::LoadingChanged(view, session.loading));
    }

    // Simulate a navigation completing on a session: record the history
    /// availability, then re-emit the full state.
    pub fn navigate_session(&mut self, view: WebViewId, url: &str, title: &str, can_go_back: bool) {
        let Some(session) = self.sessions.get_mut(&view) else {
            return;
        };
        session.url = url.to_string();
        session.title = title.to_string();
        session.can_go_back = can_go_back;
        session.can_go_forward = false;
        session.loading = false;
        self.emit_state(view);
    }

    /// Simulate a loading cycle on a session.
    pub fn set_loading(&mut self, view: WebViewId, loading: bool) {
        if let Some(session) = self.sessions.get_mut(&view) {
            session.loading = loading;
        }
        self.emit(WebEvent::LoadingChanged(view, loading));
    }

    /// Simulate a failed navigation.
    pub fn fail_navigation(&mut self, view: WebViewId, message: &str) {
        if let Some(session) = self.sessions.get_mut(&view) {
            session.loading = false;
        }
        self.emit(WebEvent::NavigationFailed(view, message.to_string()));
    }

    /// Simulate a WebKit content process termination.
    pub fn crash(&mut self, view: WebViewId) {
        if let Some(session) = self.sessions.get_mut(&view) {
            session.loading = false;
        }
        self.emit(WebEvent::PageCrashed(view));
    }
}

impl WebEngine for FakeEngine {
    fn attach_slot(&mut self, rect: WebRect, _content_height: f32) {
        self.ops.borrow_mut().push(EngineOp::AttachSlot { rect });
    }

    fn create_web_view(&mut self, id: WebViewId, profile: ProfileId, initial_url: &str) {
        if self.sessions.contains_key(&id) {
            panic!("fake engine: duplicate creation of {id:?}");
        }
        self.ops.borrow_mut().push(EngineOp::Create {
            view: id,
            profile,
            url: initial_url.to_string(),
        });
        self.sessions
            .insert(id, FakeSession::new(profile, initial_url));
    }

    fn destroy_web_view(&mut self, id: WebViewId) {
        if self.sessions.remove(&id).is_none() {
            panic!("fake engine: destroy of unknown session {id:?}");
        }
        self.ops.borrow_mut().push(EngineOp::Destroy { view: id });
    }

    fn show(&mut self, id: WebViewId, visible: bool) {
        let session = match self.sessions.get_mut(&id) {
            Some(s) => s,
            None => panic!("fake engine: show of unknown session {id:?}"),
        };
        session.visible = visible;
        self.ops
            .borrow_mut()
            .push(EngineOp::Show { view: id, visible });
    }

    fn action(&mut self, id: WebViewId, action: WebAction) {
        if !self.sessions.contains_key(&id) {
            panic!("fake engine: action on missing session {id:?}");
        }
        self.ops
            .borrow_mut()
            .push(EngineOp::Action { view: id, action });
    }

    fn set_events(&mut self, sink: Box<dyn FnMut(WebEvent)>) {
        *self.sink.borrow_mut() = Some(sink);
    }

    fn focus_native_web(&mut self, id: WebViewId) {
        if !self.sessions.contains_key(&id) {
            panic!("fake engine: focus of unknown session {id:?}");
        }
        self.ops.borrow_mut().push(EngineOp::FocusWeb { view: id });
    }

    fn focus_host_chrome(&mut self) {
        self.ops.borrow_mut().push(EngineOp::FocusChrome);
    }

    fn live_session_count(&self) -> usize {
        self.sessions.len()
    }
}

/// The fake behind the exact `dyn WebEngine` handle the controller drives,
/// plus the shared observation handles a test needs.
pub struct FakeHarness {
    pub engine: EngineRef,
    pub ops: Rc<RefCell<Vec<EngineOp>>>,
    pub sink: EventSink,
}

impl FakeHarness {
    /// Push a synthetic engine event through the registered sink.
    pub fn emit(&self, event: WebEvent) {
        if let Some(sink) = self.sink.borrow_mut().as_mut() {
            sink(event);
        }
    }

    /// The recorded operation sequence so far.
    pub fn ops(&self) -> Vec<EngineOp> {
        self.ops.borrow().clone()
    }

    /// True when the recorded sequence equals `ops`.
    pub fn recorded(&self, ops: &[EngineOp]) -> bool {
        self.ops.borrow().as_slice() == ops
    }
}

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
        assert!(harness.recorded(&[EngineOp::Create {
            view: WebViewId(1),
            profile: ProfileId(7),
            url: "https://a.example".into()
        }]));
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
        assert!(!f.session(WebViewId(1)).unwrap().loading, "failure stops loading");
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
