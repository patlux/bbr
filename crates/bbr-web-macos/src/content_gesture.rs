//! Native content-gesture tracking over the bound WKWebView panes
//! (task packet `web-content-navigation-gestures`).
//!
//! One [`NativeContentGestureBridge`] per browser window observes scroll
//! wheel events through a public **local** event monitor
//! ([`NSEvent::addLocalMonitorForEventsMatchingMask_handler`]) — never a
//! global event tap — and returns every event untouched, so native
//! scrolling, text selection, magnification, page zoom, and responder-chain
//! shortcuts keep their exact AppKit behavior.
//!
//! Only gestures that **begin inside a live web-view frame** (the pane under
//! the pointer) are tracked; events over the sidebar, Portal, palette, find,
//! menus, or other chrome resolve no web view and are ignored, so those
//! surfaces keep their own hit-testing and input (the sidebar space swipe is
//! GPUI's own `on_sidebar_scroll_wheel` authority).
//!
//! The tracker normalizes raw platform deltas — including the natural-scroll
//! preference, through [`NSEvent::isDirectionInvertedFromDevice`] — into
//! **device/finger** movement, then maps finger direction to logical
//! `Back`/`Forward` intent. Portable crates never see raw deltas.
//!
//! A committed logical gesture is reported **exactly once** through the
//! reporter sink as a [`ContentGestureCommit`] after the native commit
//! threshold on `Ended`. Cancellation, vertical dominance, boundary
//! rubber-banding, momentum, and repeated terminal events commit nothing.
//! The deterministic phase logic lives in [`crate::gesture_machine`].

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSEvent, NSEventMask, NSEventPhase, NSView, NSWindow};
use objc2_web_kit::WKWebView;

use bbr_web::{ContentGestureCommit, ContentGesturePhase, WebViewId};

use crate::gesture_machine::GestureMachine;

/// Shared reporter sink: the engine installs one and the app pumps commits
/// into `BbrApp::on_content_gesture_commit` (the same `Rc<RefCell<…>>`
/// pattern as the event/incident sinks).
pub(crate) type ContentGestureSink = Rc<RefCell<Option<Box<dyn FnMut(ContentGestureCommit)>>>>;

/// Narrow native diagnostics for the testkit snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentGestureNativeDiagnostics {
    /// The local event monitor is installed and observing scroll wheels.
    pub monitor_installed: bool,
    /// Gestures that began inside a live web-view frame.
    pub gestures_over_web: u64,
    /// Logical history commits reported to the app (exactly one per
    /// committed gesture).
    pub commits_reported: u64,
    /// Gestures cancelled by an explicit cancellation phase.
    pub cancelled: u64,
    /// Gestures abandoned because vertical movement dominated.
    pub vertical_dominant: u64,
}

/// The bridge's shared state (monitor block + reporter).
struct ContentGestureInner {
    parent_window: Retained<NSWindow>,
    /// Live web views by id: the hit-test resolution for the pointer (a
    /// gesture beginning over a registered view is the bridge's territory).
    web_views: HashMap<WebViewId, Retained<WKWebView>>,
    machine: GestureMachine,
    reporter: ContentGestureSink,
    counters: ContentGestureNativeDiagnostics,
}

/// The one native content-gesture bridge per BBR browser window.
pub struct NativeContentGestureBridge {
    inner: Rc<RefCell<ContentGestureInner>>,
    /// The local event monitor; removed on drop.
    monitor: Option<Retained<AnyObject>>,
    /// The monitor handler block, retained for the monitor's lifetime.
    monitor_block: Option<block2::RcBlock<dyn Fn(NonNull<NSEvent>) -> *mut NSEvent>>,
}

impl NativeContentGestureBridge {
    /// Attach the bridge to the browser window: install the local scroll
    /// wheel monitor.
    ///
    /// # Safety
    ///
    /// `parent_ns_view` must be the live GPUI root `NSView*` of the browser
    /// window, captured on the main thread; the bridge retains it.
    pub unsafe fn attach(
        mtm: MainThreadMarker,
        parent_ns_view: *mut std::ffi::c_void,
        reporter: ContentGestureSink,
    ) -> Option<Box<Self>> {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "content-gesture bridge attached off the main thread"
        );
        let parent_view: Retained<NSView> =
            unsafe { Retained::retain(parent_ns_view.cast::<NSView>()) }?;
        let Some(parent_window) = parent_view.window() else {
            log::warn!("content-gesture bridge: root view has no window yet");
            return None;
        };
        let inner = Rc::new(RefCell::new(ContentGestureInner {
            parent_window: parent_window.clone(),
            web_views: HashMap::new(),
            machine: GestureMachine::new(),
            reporter,
            counters: ContentGestureNativeDiagnostics {
                monitor_installed: true,
                ..Default::default()
            },
        }));

        let mut bridge = Box::new(Self {
            inner: inner.clone(),
            monitor: None,
            monitor_block: None,
        });
        bridge.install_monitor(mtm, &inner);
        Some(bridge)
    }

    /// Register a live web view for hit-test resolution (engine
    /// `create_web_view`). Duplicate ids replace the retained view.
    pub fn register_web_view(
        &self,
        mtm: MainThreadMarker,
        id: WebViewId,
        view: &Retained<WKWebView>,
    ) {
        let _ = mtm;
        self.inner.borrow_mut().web_views.insert(id, view.clone());
    }

    /// Unregister a destroyed web view (engine `destroy_web_view`).
    pub fn unregister_web_view(&self, mtm: MainThreadMarker, id: WebViewId) {
        let _ = mtm;
        self.inner.borrow_mut().web_views.remove(&id);
    }

    /// Narrow main-thread diagnostics (testkit snapshot semantics).
    pub fn diagnostics(&self, mtm: MainThreadMarker) -> ContentGestureNativeDiagnostics {
        let _ = mtm;
        self.inner.borrow().counters.clone()
    }

    /// The shared reporter sink handle (the engine installs the app sink
    /// through [`NativeContentGestureBridge`]; replaces any previous one).
    pub fn reporter_handle(&self) -> ContentGestureSink {
        self.inner.borrow().reporter.clone()
    }

    fn install_monitor(&mut self, mtm: MainThreadMarker, inner: &Rc<RefCell<ContentGestureInner>>) {
        let shared = inner.clone();
        let block: block2::RcBlock<dyn Fn(NonNull<NSEvent>) -> *mut NSEvent> =
            block2::RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
                let event_ptr = event.as_ptr();
                // SAFETY: local monitors run on the main thread; the event
                // pointer is live for the duration of the callback.
                let event = unsafe { event.as_ref() };
                if let Some(mtm) = MainThreadMarker::new() {
                    monitor_handle(mtm, event, &shared);
                }
                // Always pass the event through: the bridge only observes,
                // so native scrolling/selection/zoom are never disabled.
                event_ptr
            });
        // SAFETY: main thread; the handler always returns the original event
        // (pure observation). AppKit copies the handler block for the
        // monitor's lifetime; the `RcBlock` is additionally retained here.
        let monitor = unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::ScrollWheel, &block)
        };
        self.monitor = monitor;
        self.monitor_block = Some(block);
        let _ = mtm;
    }
}

/// The engine-held content-gesture wiring (N1): owns the one native bridge
/// and exposes the attach/report/diagnostics/register/unregister surface the
/// engine calls from `webview.rs`. Splitting the wiring here keeps the
/// engine module focused under the ~500-line module budget.
pub(crate) struct ContentGestureHost {
    bridge: Option<Box<NativeContentGestureBridge>>,
}

impl ContentGestureHost {
    /// Attach the one bridge (or leave it absent when the root view has no
    /// window yet — the same deferred attach behavior as the menu bridge).
    pub(crate) fn attach(
        mtm: MainThreadMarker,
        ns_view: *mut std::ffi::c_void,
        reporter: ContentGestureSink,
    ) -> Self {
        Self {
            bridge: unsafe { NativeContentGestureBridge::attach(mtm, ns_view, reporter) },
        }
    }

    /// Install the app delivery sink for every committed logical content
    /// gesture. Replaces any previous reporter.
    pub(crate) fn set_reporter(&self, reporter: Box<dyn FnMut(ContentGestureCommit)>) {
        if let Some(bridge) = self.bridge.as_ref() {
            *bridge.reporter_handle().borrow_mut() = Some(reporter);
        }
    }

    /// Narrow diagnostics of the native bridge; `None` when no bridge is
    /// attached.
    pub(crate) fn diagnostics(&self) -> Option<ContentGestureNativeDiagnostics> {
        let mtm = MainThreadMarker::new()?;
        self.bridge.as_ref().map(|bridge| bridge.diagnostics(mtm))
    }

    /// Register a live web view for hit-test resolution (engine
    /// `create_web_view`).
    pub(crate) fn register_web_view(
        &self,
        mtm: MainThreadMarker,
        id: WebViewId,
        view: &Retained<WKWebView>,
    ) {
        if let Some(bridge) = self.bridge.as_ref() {
            bridge.register_web_view(mtm, id, view);
        }
    }

    /// Unregister a destroyed web view (engine `destroy_web_view`).
    pub(crate) fn unregister_web_view(&self, mtm: MainThreadMarker, id: WebViewId) {
        if let Some(bridge) = self.bridge.as_ref() {
            bridge.unregister_web_view(mtm, id);
        }
    }
}

/// The monitor's per-event handling. Never swallows the event (the bridge is
/// a pure observer); tracks only gestures beginning over a registered web
/// view and reports at most one commit per gesture.
fn monitor_handle(
    mtm: MainThreadMarker,
    event: &NSEvent,
    inner: &Rc<RefCell<ContentGestureInner>>,
) {
    // Momentum and coarse (non-trackpad) wheels never drive history
    // gestures; precise trackpad phases drive the machine.
    if !event.hasPreciseScrollingDeltas() {
        return;
    }
    if event.momentumPhase() != NSEventPhase::None {
        return;
    }
    let phase = logical_phase(event.phase());
    let Some(event_window) = event.window(mtm) else {
        return;
    };
    let mut state = inner.borrow_mut();
    // Pointer identity, not `PartialEq` (Retained<NSWindow> has none).
    if !std::ptr::eq(&*event_window, &*state.parent_window) {
        return;
    }
    // Device-normalized finger deltas (points): compensate the natural-scroll
    // preference so `finger_dx > 0` always means "fingers moved right".
    let inverted = event.isDirectionInvertedFromDevice();
    let finger_dx = if inverted {
        -event.scrollingDeltaX()
    } else {
        event.scrollingDeltaX()
    };
    let finger_dy = if inverted {
        -event.scrollingDeltaY()
    } else {
        event.scrollingDeltaY()
    };

    let view = match phase {
        ContentGesturePhase::Started => {
            // Resolve the exact pane under the pointer; a gesture over
            // chrome (sidebar/Portal/palette/menus) is not tracked.
            let Some(view_id) = resolve_web_view_under(event, mtm, &state) else {
                return;
            };
            state.counters.gestures_over_web += 1;
            view_id
        }
        _ => {
            // Later phases keep feeding the exact pane the gesture began
            // over (the pointer may drift across chrome mid-gesture).
            let Some(view) = state.machine.tracking_view() else {
                return;
            };
            view
        }
    };

    let commit = state.machine.feed(phase, view, finger_dx, finger_dy);
    match (phase, &commit) {
        (ContentGesturePhase::Cancelled, _) => state.counters.cancelled += 1,
        (ContentGesturePhase::Moved, _) if state.machine.is_idle() => {
            state.counters.vertical_dominant += 1;
        }
        _ => {}
    }
    if let Some(commit) = commit {
        state.counters.commits_reported += 1;
        if let Some(reporter) = state.reporter.borrow_mut().as_mut() {
            reporter(commit);
        }
    }
}

/// Map AppKit's event phase to the portable logical phase.
fn logical_phase(phase: NSEventPhase) -> ContentGesturePhase {
    match phase {
        NSEventPhase::Began | NSEventPhase::MayBegin => ContentGesturePhase::Started,
        NSEventPhase::Changed => ContentGesturePhase::Moved,
        NSEventPhase::Ended => ContentGesturePhase::Ended,
        NSEventPhase::Cancelled => ContentGesturePhase::Cancelled,
        // Stationary and phase-less events carry no tracking intent.
        NSEventPhase::None | NSEventPhase::Stationary => ContentGesturePhase::Cancelled,
        _ => ContentGesturePhase::Cancelled,
    }
}

/// Resolve the registered web view under the event's pointer location by
/// walking the AppKit hit-test chain; `None` for chrome surfaces.
fn resolve_web_view_under(
    event: &NSEvent,
    mtm: MainThreadMarker,
    state: &ContentGestureInner,
) -> Option<WebViewId> {
    let location = event.locationInWindow();
    let content_view = state.parent_window.contentView()?;
    let hit = content_view.hitTest(location)?;
    let mut view = Some(hit);
    while let Some(current) = view {
        if let Some((id, _)) = state.web_views.iter().find(|(_, registered)| {
            let registered_ptr = Retained::as_ptr(registered) as *const std::ffi::c_void;
            let hit_ptr = &*current as *const NSView as *const std::ffi::c_void;
            registered_ptr == hit_ptr
        }) {
            return Some(*id);
        }
        view = unsafe { current.superview() };
    }
    let _ = mtm;
    None
}

impl Drop for NativeContentGestureBridge {
    fn drop(&mut self) {
        if MainThreadMarker::new().is_some() {
            if let Some(monitor) = self.monitor.take() {
                // SAFETY: main thread; the monitor was created by
                // `addLocalMonitor…` and is still installed.
                unsafe {
                    NSEvent::removeMonitor(&monitor);
                }
            }
        }
    }
}
