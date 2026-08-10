//! Deterministic fake [`WebEngine`] for controller contract tests
//! (`cfg(test)` only: its only consumer is this crate's own suites, so per the
//! task packet's delivery choice it lives here rather than in a speculative
//! `bbr-web-fake` crate — documented in ARCHITECTURE.md §9).
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
    SetRect {
        view: WebViewId,
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
    Raise {
        view: WebViewId,
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
    /// Per-view geometry cache: the last rect each live view received.
    geometry: HashMap<WebViewId, WebRect>,
    sink: EventSink,
}

impl Default for FakeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeEngine {
    /// Create a fake plus the shared operation log and event sink.
    pub fn new() -> Self {
        Self {
            ops: Rc::new(RefCell::new(Vec::new())),
            sessions: HashMap::new(),
            geometry: HashMap::new(),
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
    fn set_web_view_rect(&mut self, id: WebViewId, rect: WebRect, _content_height: f32) {
        // Geometry is addressed per live view: unknown ids are no-ops and an
        // unchanged rect produces no repeated frame update (recorded as a
        // duplicate op, exactly like the macOS engine skips `setFrame:`).
        if !self.sessions.contains_key(&id) {
            return;
        }
        if self.geometry.get(&id) == Some(&rect) {
            return;
        }
        self.geometry.insert(id, rect);
        self.ops
            .borrow_mut()
            .push(EngineOp::SetRect { view: id, rect });
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
        self.geometry.remove(&id);
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

    fn raise_web_view(&mut self, id: WebViewId) {
        if !self.sessions.contains_key(&id) {
            panic!("fake engine: raise of unknown session {id:?}");
        }
        self.ops.borrow_mut().push(EngineOp::Raise { view: id });
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

    /// Clear the recorded operations without touching sessions.
    pub fn reset_ops(&self) {
        *self.ops.borrow_mut() = Vec::new();
    }
}
