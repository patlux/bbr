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
    engine_kind: &'static str,
) -> EngineRef {
    #[cfg(target_os = "macos")]
    {
        let mtm = objc2::MainThreadMarker::new().expect("engine built on main thread");
        let engine = unsafe {
            bbr_web_macos::WebEngineWk::attach(
                mtm,
                ns_view,
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
///
/// Geometry follows the same contract as the fake and macOS engines: it is
/// addressed per live [`WebViewId`] (unknown ids are no-ops), an unchanged
/// per-view rect produces no repeated update, and destroying a view removes
/// its cached geometry.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub struct NullEngine {
    pub actions: Vec<String>,
    /// Live view id → last published rect. `create_web_view` registers the
    /// id (default frame until the keyed slot publishes), `destroy` removes
    /// the entry; unknown ids are never inserted.
    pub geometry: std::collections::HashMap<WebViewId, WebRect>,
    /// Recorded geometry updates: only **changed** rects of **live** views,
    /// in call order — diagnostic evidence on platforms without a native
    /// engine (idle windows and unknown ids publish nothing).
    pub slots: Vec<(WebViewId, WebRect)>,
    pub events: EventSink,
}

impl Default for NullEngine {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            geometry: std::collections::HashMap::new(),
            slots: Vec::new(),
            events: std::cell::RefCell::new(None),
        }
    }
}

impl std::fmt::Debug for NullEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NullEngine")
            .field("actions", &self.actions)
            .field("geometry", &self.geometry)
            .field("slots", &self.slots)
            .finish()
    }
}

impl WebEngine for NullEngine {
    fn set_web_view_rect(&mut self, id: WebViewId, rect: WebRect, _content_height: f32) {
        // Unknown ids are no-ops (no live view to frame) and an unchanged
        // rect is suppressed, mirroring the fake/macOS engines' suppression
        // of redundant native frame updates.
        let Some(last) = self.geometry.get_mut(&id) else {
            return;
        };
        if *last == rect {
            return;
        }
        *last = rect;
        self.slots.push((id, rect));
    }

    fn create_web_view(&mut self, id: WebViewId, profile: ProfileId, initial_url: &str) {
        self.actions
            .push(format!("create {id:?} in {profile:?} at {initial_url}"));
        // A newly created view keeps a default frame until its keyed slot
        // publishes (the view starts hidden, so the frame is never visible).
        self.geometry.insert(id, WebRect::default());
    }

    fn destroy_web_view(&mut self, id: WebViewId) {
        self.actions.push(format!("destroy {id:?}"));
        // Destroyed ids lose their cached geometry; later rect calls for
        // the id are unknown-id no-ops.
        self.geometry.remove(&id);
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

    fn focus_native_web(&mut self, _id: WebViewId) {
        self.actions.push("focus_web".into());
    }

    fn focus_host_chrome(&mut self) {
        self.actions.push("focus_chrome".into());
    }

    fn live_session_count(&self) -> usize {
        0
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

// Tests reference the null engine, so it is live in test builds on macOS
// even though release/dead-code builds only need it on other targets.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_engine_ignores_unknown_ids() {
        let mut engine = NullEngine::default();
        engine.set_web_view_rect(WebViewId(99), WebRect::new(0.0, 0.0, 10.0, 10.0), 700.0);
        assert!(engine.slots.is_empty(), "unknown ids are no-ops");
        assert!(
            engine.geometry.is_empty(),
            "unknown ids never enter the live geometry map"
        );
    }

    #[test]
    fn null_engine_keys_geometry_by_live_id_and_suppresses_unchanged_rects() {
        let mut engine = NullEngine::default();
        engine.create_web_view(WebViewId(1), ProfileId(1), "u1");
        engine.create_web_view(WebViewId(2), ProfileId(1), "u2");
        let primary = WebRect::new(10.0, 0.0, 400.0, 600.0);
        let secondary = WebRect::new(410.0, 0.0, 400.0, 600.0);
        engine.set_web_view_rect(WebViewId(1), primary, 700.0);
        engine.set_web_view_rect(WebViewId(2), secondary, 700.0);
        assert_eq!(
            engine.slots,
            vec![(WebViewId(1), primary), (WebViewId(2), secondary)]
        );
        // Unchanged rects are suppressed (no repeated frame update).
        engine.set_web_view_rect(WebViewId(1), primary, 700.0);
        engine.set_web_view_rect(WebViewId(2), secondary, 700.0);
        assert_eq!(engine.slots.len(), 2, "unchanged rects are suppressed");
        // A changed rect publishes only for that view.
        engine.set_web_view_rect(WebViewId(1), WebRect::new(20.0, 0.0, 400.0, 600.0), 700.0);
        assert_eq!(engine.slots.len(), 3, "changed rect publishes for its view");
        assert_eq!(engine.slots[2].0, WebViewId(1));
    }

    #[test]
    fn null_engine_destroy_removes_cached_geometry() {
        let mut engine = NullEngine::default();
        engine.create_web_view(WebViewId(1), ProfileId(1), "u1");
        let rect = WebRect::new(0.0, 0.0, 400.0, 600.0);
        engine.set_web_view_rect(WebViewId(1), rect, 700.0);
        assert_eq!(engine.geometry.get(&WebViewId(1)), Some(&rect));
        engine.destroy_web_view(WebViewId(1));
        assert!(
            engine.geometry.is_empty(),
            "a destroyed id loses its cached geometry"
        );
        // A rect call after destroy is an unknown-id no-op again.
        engine.set_web_view_rect(WebViewId(1), rect, 700.0);
        assert_eq!(engine.slots.len(), 1, "post-destroy rects are ignored");
    }

    #[test]
    fn null_engine_keeps_action_recording() {
        let mut engine = NullEngine::default();
        engine.create_web_view(WebViewId(1), ProfileId(1), "u1");
        engine.action(WebViewId(1), bbr_web::WebAction::Reload);
        engine.show(WebViewId(1), true);
        engine.destroy_web_view(WebViewId(1));
        assert_eq!(engine.actions.len(), 4);
        assert!(engine.actions[0].starts_with("create "));
        assert!(engine.actions.iter().any(|a| a == "reload"));
        assert!(engine.actions.iter().any(|a| a.starts_with("show ")));
        assert!(engine.actions[3].starts_with("destroy "));
    }
}
