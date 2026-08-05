//! Engine construction at the composition root.
//!
//! On macOS this builds the real [`bbr_web_macos::WebEngineWk`] attached to
//! the GPUI root view (captured via `raw_window_handle`). On other targets
//! it builds a portable null engine so the workspace still evaluates — and
//! the null engine doubles as the fake engine for platform-free unit tests.

use std::cell::RefCell;
use std::rc::Rc;

use bbr_web::{ProfileId, WebAction, WebEngine, WebEvent, WebRect, WebViewId};

/// The engine as held by the app: main-thread-only, shared with the slot
/// element and the test bridge.
pub type EngineRef = Rc<RefCell<dyn WebEngine>>;
type EventSink = RefCell<Option<Box<dyn FnMut(WebEvent)>>>;

/// Public runtime knobs for engine construction.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Use hermetic non-persistent website data stores (E2E mode).
    pub non_persistent: bool,
}

/// Build the platform engine for the current process.
///
/// On macOS, `ns_view` is the GPUI root view captured from the window handle
/// on the main thread; it is retained by the engine. On other platforms the
/// returned engine is a [`NullEngine`] and `ns_view` is ignored.
///
/// # Safety
///
/// `ns_view` must be a valid live `NSView*` when `cfg(target_os = "macos")`.
pub(crate) unsafe fn build_engine(
    ns_view: *mut std::ffi::c_void,
    config: &EngineConfig,
    web_view_id: WebViewId,
    engine_kind: &'static str,
) -> EngineRef {
    #[cfg(target_os = "macos")]
    {
        let mtm = objc2::MainThreadMarker::new().expect("engine built on main thread");
        let engine = unsafe {
            bbr_web_macos::WebEngineWk::attach(
                mtm,
                ns_view,
                web_view_id,
                bbr_web_macos::EngineOptions {
                    non_persistent: config.non_persistent,
                },
            )
        };
        let _ = engine_kind;
        Rc::new(RefCell::new(engine))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Rc::new(RefCell::new(NullEngine::default()))
    }
}

/// A no-op engine used on non-macOS targets (and as the fake engine in
/// platform-free tests). It records actions so tests can assert routing.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub struct NullEngine {
    pub actions: Vec<String>,
    pub slots: Vec<WebRect>,
    pub events: EventSink,
}

impl Default for NullEngine {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            slots: Vec::new(),
            events: std::cell::RefCell::new(None),
        }
    }
}

impl std::fmt::Debug for NullEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NullEngine")
            .field("actions", &self.actions)
            .field("slots", &self.slots)
            .finish()
    }
}

impl WebEngine for NullEngine {
    fn attach_slot(&mut self, rect: WebRect, _content_height: f32) {
        self.slots.push(rect);
    }

    fn create_web_view(&mut self, id: WebViewId, profile: ProfileId, initial_url: &str) {
        self.actions
            .push(format!("create {id:?} in {profile:?} at {initial_url}"));
    }

    fn destroy_web_view(&mut self, id: WebViewId) {
        self.actions.push(format!("destroy {id:?}"));
    }

    fn show(&mut self, id: WebViewId, visible: bool) {
        self.actions.push(format!("show {id:?} visible={visible}"));
    }

    fn action(&mut self, _id: WebViewId, action: WebAction) {
        match action {
            WebAction::Load { url } => self.actions.push(format!("load {url}")),
            WebAction::Back => self.actions.push("back".into()),
            WebAction::Forward => self.actions.push("forward".into()),
            WebAction::Reload => self.actions.push("reload".into()),
            WebAction::Stop => self.actions.push("stop".into()),
        }
    }

    fn set_events(&mut self, sink: Box<dyn FnMut(WebEvent)>) {
        *self.events.borrow_mut() = Some(sink);
    }

    fn focus_native_web(&mut self) {
        self.actions.push("focus_web".into());
    }

    fn focus_host_chrome(&mut self) {
        self.actions.push("focus_chrome".into());
    }
}

#[cfg_attr(target_os = "macos", allow(dead_code))]
impl NullEngine {
    /// Emit a synthetic event through the sink (fake-engine tests).
    pub fn emit(&self, event: WebEvent) {
        if let Some(sink) = self.events.borrow_mut().as_mut() {
            sink(event);
        }
    }
}
