//! Native context-menu bridge (task packet `bbr-context-menus-over-web`).
//!
//! The bridge owns the AppKit side of the ownership routing between
//! WebKit-owned page menus and BBR-owned chrome menus, per the packet's
//! constraint list: AppKit owns only event observation, menu classification,
//! and cancellation; GPUI owns the menu's content and state (rendered inside
//! the one `NativeAboveWeb` portal window).
//!
//! ## Ownership routing (the documented product/native decision)
//!
//! Right-clicks over a live WKWebView are observed by a local event monitor
//! (main run loop). The bridge never suppresses a right-click: WebKit always
//! gets the event and builds its own menu, so link/image/selection/form
//! menus keep their native path untouched. The bridge then classifies the
//! menu WebKit actually constructed at `NSMenuWillOpenNotification` time:
//!
//! - A menu whose non-empty item titles are all page-level (`Back`,
//!   `Forward`, `Reload`, `Inspect Element`) is the generic page menu —
//!   BBR-owned. The bridge cancels it ([`NSMenu::cancelTrackingWithoutAnimation`],
//!   verified to unblock `popUpMenuPositioningItem:` synchronously) and
//!   emits [`WebEvent::ContextMenuRequested`] with the pointer location so
//!   the app presents the BBR menu through the portal.
//! - Any other menu (element-specific items such as `Open Link`, `Save
//!   Image`, `Copy`, `Share…`, …) is left entirely alone — WebKit owns.
//!
//! The classification is deliberately conservative: an unrecognized or empty
//! item set never cancels WebKit's menu (the native menu wins), so BBR can
//! never wrongly suppress a WebKit-owned page menu. Item titles are
//! WebKit's own (system-localized); on non-English systems an unrecognized
//! localized page menu falls back to WebKit-owned, which is the safe
//! direction.
//!
//! ## Safety invariant
//!
//! All entry points require a `MainThreadMarker` (the local monitor and the
//! notification observer fire on the main thread). The monitor block and the
//! observer share the bridge's boxed heap state through an `Rc<RefCell<…>>`;
//! the observer is removed before the bridge is dropped.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSEvent, NSEventMask, NSMenu, NSPasteboard, NSPasteboardTypeString, NSView, NSWindow,
};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObject, NSString};
use objc2_web_kit::WKWebView;

use bbr_web::{WebEvent, WebRect, WebViewId};

use crate::delegate::EventSink;

/// The pure page-vs-element ownership classification (deterministic, unit-
/// tested) lives in the crate-level [`crate::menu_classify`] module; this
/// module owns only the AppKit bridge machinery.
pub use crate::menu_classify::{classify_menu, menu_titles, MenuOwnership};

/// Write `url` to the system general pasteboard as a string (the BBR
/// context menu's Copy Link entry; main thread only). The caller passes the
/// view's current URL — the engine implements [`WebAction::CopyLink`] this
/// way, never a caller-supplied string.
pub(crate) fn copy_to_pasteboard(url: &str) {
    if MainThreadMarker::new().is_none() {
        log::warn!("copy to pasteboard off the main thread");
        return;
    }
    let pasteboard = NSPasteboard::generalPasteboard();
    let string = NSString::from_str(url);
    // SAFETY: main thread; `setString:forType:` on the general pasteboard is
    // a plain synchronous write with no retained-state hazard.
    unsafe {
        pasteboard.clearContents();
        pasteboard.setString_forType(&string, NSPasteboardTypeString);
    }
}

/// One armed right-click awaiting WebKit's menu classification.
struct PendingRightClick {
    view: WebViewId,
    /// Pointer location in window content coordinates (top-left origin,
    /// points — the same space the portal placement math uses).
    point: WebRect,
}

/// Bridge counters (narrow diagnostics for the testkit snapshot).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextMenuNativeDiagnostics {
    /// The monitor is installed and observing right-clicks.
    pub monitor_installed: bool,
    /// Right-clicks observed over a live WKWebView (all of them: every
    /// WebKit menu is classified, never suppressed blindly).
    pub right_clicks_over_web: u64,
    /// Classified as the generic page menu → the BBR menu was requested.
    pub page_owned: u64,
    /// Classified as element-owned → WebKit's native menu stayed untouched.
    pub element_owned: u64,
}

/// The bridge's shared state (monitor block + observer).
struct MenuBridgeInner {
    parent_view: Retained<NSView>,
    parent_window: Retained<NSWindow>,
    /// Live web views by id: the hit-test resolution for the pointer (a
    /// right-click over a registered view is the bridge's territory).
    web_views: HashMap<WebViewId, Retained<WKWebView>>,
    pending: Option<PendingRightClick>,
    counters: ContextMenuNativeDiagnostics,
}

/// Ivars of the will-open observer: the shared bridge state and the event
/// sink (the same `Rc`-shared sink the engine hands its delegates).
struct MenuWillOpenIvars {
    inner: Rc<RefCell<MenuBridgeInner>>,
    sink: EventSink,
}

define_class!(
    /// Receives `NSMenuWillOpenNotification` and applies the ownership
    /// routing to the menu WebKit is about to show. Main-thread-only; the
    /// bridge owns the observer and removes it before dropping.
    #[unsafe(super = NSObject)]
    #[ivars = MenuWillOpenIvars]
    #[thread_kind = MainThreadOnly]
    #[name = "BBRContextMenuWillOpenObserver"]
    struct MenuWillOpenObserver;

    impl MenuWillOpenObserver {
        #[unsafe(method(contextMenuWillOpen:))]
        fn context_menu_will_open(&self, notification: Option<&NSNotification>) {
            let Some(notification) = notification else {
                return;
            };
            let Some(menu) = notification.object().and_then(|object| {
                object.downcast::<NSMenu>().ok()
            }) else {
                return;
            };
            let mut inner = self.ivars().inner.borrow_mut();
            let Some(pending) = inner.pending.take() else {
                // Not one of our armed right-clicks (a left-click select
                // popup, a future BBR NSMenu, …): never touch it.
                return;
            };
            // Empty/unclassifiable menus are left alone AND keep the arm
            // (WebKit posts trailing empty menus after the real one; the
            // real menu is the first with items, so acting only here is
            // deterministic).
            let titles = menu_titles(&menu);
            match classify_menu(&titles) {
                MenuOwnership::PageOnly => {
                    // SAFETY: main thread; the menu is live (the notification
                    // fires inside its popup call stack).
                    menu.cancelTrackingWithoutAnimation();
                    inner.counters.page_owned += 1;
                    if let Some(sink) = self.ivars().sink.borrow_mut().as_mut() {
                        sink(WebEvent::ContextMenuRequested(pending.view, pending.point));
                    }
                }
                MenuOwnership::ElementOwned => {
                    inner.counters.element_owned += 1;
                }
            }
        }
    }
);

impl MenuWillOpenObserver {
    fn new(
        mtm: MainThreadMarker,
        inner: Rc<RefCell<MenuBridgeInner>>,
        sink: EventSink,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(MenuWillOpenIvars { inner, sink });
        // SAFETY: `init` on the main thread (ChromeResponder pattern).
        unsafe { msg_send![super(this), init] }
    }
}

/// The item titles of a menu are read through the classifier module
/// ([`menu_classify::menu_titles`]); the observer applies the ownership
/// routing to the menu WebKit is about to show.
///
/// The one native context-menu bridge per BBR browser window.
pub struct NativeContextMenuBridge {
    inner: Rc<RefCell<MenuBridgeInner>>,
    /// The local event monitor; removed on drop.
    monitor: Option<Retained<AnyObject>>,
    /// The monitor handler block, retained for the monitor's lifetime
    /// (AppKit copies it, but the `RcBlock` keeps it alive regardless).
    monitor_block: Option<block2::RcBlock<dyn Fn(NonNull<NSEvent>) -> *mut NSEvent>>,
    /// The `NSMenuWillOpenNotification` observer; removed on drop.
    observer: Option<Retained<MenuWillOpenObserver>>,
}

impl NativeContextMenuBridge {
    /// Attach the bridge to the browser window: install the local
    /// right-click monitor and the menu-will-open observer.
    ///
    /// # Safety
    ///
    /// `parent_ns_view` must be the live GPUI root `NSView*` of the browser
    /// window, captured on the main thread; the bridge retains it.
    pub unsafe fn attach(
        mtm: MainThreadMarker,
        parent_ns_view: *mut std::ffi::c_void,
        sink: EventSink,
    ) -> Option<Box<Self>> {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "context-menu bridge attached off the main thread"
        );
        let parent_view: Retained<NSView> =
            unsafe { Retained::retain(parent_ns_view.cast::<NSView>()) }?;
        let Some(parent_window) = parent_view.window() else {
            log::warn!("context-menu bridge: root view has no window yet");
            return None;
        };
        let inner = Rc::new(RefCell::new(MenuBridgeInner {
            parent_view: parent_view.clone(),
            parent_window: parent_window.clone(),
            web_views: HashMap::new(),
            pending: None,
            counters: ContextMenuNativeDiagnostics {
                monitor_installed: true,
                ..Default::default()
            },
        }));

        let mut bridge = Box::new(Self {
            inner: inner.clone(),
            monitor: None,
            monitor_block: None,
            observer: None,
        });
        bridge.install_monitor(mtm, &inner);
        bridge.install_observer(mtm, &inner, sink);
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
    pub fn diagnostics(&self, mtm: MainThreadMarker) -> ContextMenuNativeDiagnostics {
        let _ = mtm;
        self.inner.borrow().counters.clone()
    }

    fn install_monitor(&mut self, mtm: MainThreadMarker, inner: &Rc<RefCell<MenuBridgeInner>>) {
        let shared = inner.clone();
        let block: block2::RcBlock<dyn Fn(NonNull<NSEvent>) -> *mut NSEvent> =
            block2::RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
                let event_ptr = event.as_ptr();
                // SAFETY: local monitors run on the main thread; the event
                // pointer is live for the duration of the callback.
                let event = unsafe { event.as_ref() };
                if let Some(mtm) = MainThreadMarker::new() {
                    let pass = monitor_handle(mtm, event, &shared);
                    if pass {
                        event_ptr
                    } else {
                        std::ptr::null_mut()
                    }
                } else {
                    event_ptr
                }
            });
        // SAFETY: main thread; the handler returns the original event (pass)
        // or null (swallow — never taken for web right-clicks in v1). AppKit
        // copies the handler block for the monitor's lifetime; the `RcBlock`
        // is additionally retained here until the bridge drops.
        let monitor = unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(
                NSEventMask::RightMouseDown,
                &block,
            )
        };
        self.monitor = monitor;
        self.monitor_block = Some(block);
        let _ = mtm;
    }

    fn install_observer(
        &mut self,
        mtm: MainThreadMarker,
        inner: &Rc<RefCell<MenuBridgeInner>>,
        sink: EventSink,
    ) {
        let observer = MenuWillOpenObserver::new(mtm, inner.clone(), sink);
        let center = NSNotificationCenter::defaultCenter();
        let name = NSString::from_str("NSMenuWillOpenNotification");
        // SAFETY: main thread; `observer` is retained by `self`; observing
        // the default center posts on the main thread.
        unsafe {
            center.addObserver_selector_name_object(
                &observer,
                sel!(contextMenuWillOpen:),
                Some(&name),
                None,
            );
        }
        self.observer = Some(observer);
    }

    fn remove_observer(&mut self, mtm: MainThreadMarker) {
        let _ = mtm;
        if let Some(observer) = self.observer.take() {
            let center = NSNotificationCenter::defaultCenter();
            // SAFETY: main thread; the observer was registered with the
            // default center and is still alive (we hold it).
            unsafe {
                center.removeObserver(&observer);
            }
        }
    }
}

/// The monitor's per-event decision. Returns `true` to pass the event
/// through (always, for web right-clicks in v1 — WebKit builds its menu and
/// the will-open observer classifies it), `false` to swallow it.
fn monitor_handle(
    mtm: MainThreadMarker,
    event: &NSEvent,
    inner: &Rc<RefCell<MenuBridgeInner>>,
) -> bool {
    let Some(event_window) = event.window(mtm) else {
        return true;
    };
    let mut state = inner.borrow_mut();
    // Pointer identity, not `PartialEq` (Retained<NSWindow> has none).
    if !std::ptr::eq(&*event_window, &*state.parent_window) {
        return true;
    }
    // Pointer location in window content coordinates (AppKit bottom-left
    // origin); flip to the host top-left origin used by the portal math.
    let location = event.locationInWindow();
    let content_height = state.parent_view.bounds().size.height as f32;
    let point = WebRect::new(
        location.x as f32,
        content_height - location.y as f32,
        0.0,
        0.0,
    );
    // Resolve the hit view and walk up to a registered web view.
    let Some(content_view) = state.parent_window.contentView() else {
        return true;
    };
    let Some(hit) = content_view.hitTest(location) else {
        return true;
    };
    let mut view = Some(hit);
    let mut matched: Option<WebViewId> = None;
    while let Some(current) = view {
        if let Some((id, _)) = state.web_views.iter().find(|(_, registered)| {
            // Pointer identity between a registered `WKWebView` and the hit
            // view (or one of its superviews) on the same object address.
            let registered_ptr = Retained::as_ptr(registered) as *const std::ffi::c_void;
            let hit_ptr = &*current as *const NSView as *const std::ffi::c_void;
            registered_ptr == hit_ptr
        }) {
            matched = Some(*id);
            break;
        }
        view = unsafe { current.superview() };
    }
    let Some(view_id) = matched else {
        // Right-click over chrome, not a web view: untouched.
        return true;
    };
    state.counters.right_clicks_over_web += 1;
    // Arm the classification: WebKit now builds its menu for this click; the
    // will-open observer decides ownership and (page-only) cancels it and
    // requests the BBR menu. The event always passes through, so WebKit's
    // native menu can never be suppressed without classification.
    state.pending = Some(PendingRightClick {
        view: view_id,
        point,
    });
    true
}

impl Drop for NativeContextMenuBridge {
    fn drop(&mut self) {
        if let Some(mtm) = MainThreadMarker::new() {
            self.remove_observer(mtm);
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
