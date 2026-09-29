//! Native portal window bridge (task packet `native-portal-host`).
//!
//! One implementation-owned, transparent, non-activating portal window per
//! BBR browser window. GPUI owns portal content and state; this module owns
//! only the AppKit side, per the packet's constraint list: native z-order
//! (child-window attachment ordered above the parent), parent-attachment,
//! frame synchronization to the parent content rect, visibility, and
//! hit-test routing. Pure frame math and the typed diagnostics snapshot live
//! in [`crate::portal_frame`].
//!
//! ## Safety invariant
//!
//! [`NativePortal`] retains the GPUI portal root `NSView` (captured once via
//! `raw_window_handle`), its `NSWindow`, and the browser window's `NSWindow`.
//! All entry points require a `MainThreadMarker`. The raw pointer handed to
//! the notification observer ([`PortalSyncObserver`]) points at the portal's
//! **heap allocation**: [`attach`](NativePortal::attach) allocates the portal
//! on the boxed heap *before* installing the observer, and the box is never
//! moved afterwards (moving the `Box<NativePortal>` inside the app's entity
//! arena moves the pointer, never the allocation), so the observer's pointer
//! is stable for the portal's whole lifetime. The pointer is only
//! dereferenced on the main thread while the observer is owned by the portal
//! (the observer is removed before the portal is dropped).
//!
//! ## Window policy
//!
//! The portal window is created by GPUI as a `WindowKind::PopUp` (an
//! `NSPanel` with the non-activating style), initially hidden and
//! non-focusing. The bridge hardens the policy natively: borderless style,
//! transparent (never opaque), no shadow, non-movable, excluded from the
//! Windows menu, hidden when the app deactivates, and no appearance
//! animation. It is attached as a child of the browser window ordered above,
//! so AppKit keeps it above the parent's WKWebViews; it never becomes key or
//! main (non-activating panel + `orderFront:` only).

use objc2::rc::Retained;
use objc2::{class, msg_send, sel, MainThreadMarker};
use objc2_app_kit::{
    NSView, NSWindow, NSWindowAnimationBehavior, NSWindowDidChangeScreenNotification,
    NSWindowDidDeminiaturizeNotification, NSWindowDidEnterFullScreenNotification,
    NSWindowDidMiniaturizeNotification, NSWindowDidMoveNotification, NSWindowDidResizeNotification,
    NSWindowStyleMask, NSWindowWillCloseNotification,
};
use objc2_foundation::{NSNotificationCenter, NSObject};

use crate::portal_deferred::{defer_window_op, DeferredWindowOp};
pub use crate::portal_frame::PortalNativeDiagnostics;
use crate::portal_frame::{should_sync_frame, PortalFrame};
use crate::portal_lifecycle::PortalSyncObserver;

/// Why [`NativePortal::attach`] could not establish the portal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalAttachError {
    NullParentView,
    NullPortalView,
    NoParentWindow,
    NoPortalWindow,
}

/// The one native portal host per BBR browser window.
///
/// `None`-able fields become `Some` after [`attach`](Self::attach) and are
/// cleared by [`teardown`](Self::teardown) / `Drop`; every entry point
/// tolerates a not-yet-attached (or already-torn-down) portal as a no-op.
///
/// The portal is always allocated on the boxed heap: [`attach`](Self::attach)
/// returns `Box<NativePortal>`, and the sync observer holds a raw pointer to
/// that allocation. Moving the `Box` (into the app bridge, into the entity
/// arena) never moves the allocation, so the observer's pointer stays valid
/// (module-level safety invariant).
pub struct NativePortal {
    /// Retained GPUI portal root view (keeps the raw pointer valid).
    portal_view: Option<Retained<NSView>>,
    /// Retained portal `NSWindow` (keeps the raw pointer valid).
    /// `pub(crate)`: the lifecycle-hardening and diagnostics impl blocks in
    /// the sibling `portal_lifecycle` / `portal_frame` modules read it.
    pub(crate) portal_window: Option<Retained<NSWindow>>,
    /// Retained browser-window `NSWindow` (the parent). `pub(crate)`: see
    /// `portal_window`.
    pub(crate) parent_window: Option<Retained<NSWindow>>,
    observer: Option<Retained<PortalSyncObserver>>,
    /// Child-window attachment established (diagnostics evidence).
    /// `pub(crate)`: read by the `portal_frame` diagnostics impl.
    pub(crate) attached: bool,
    /// Commanded visibility (the presentation is armed). `pub(crate)`: read
    /// by the `portal_lifecycle` restore/re-show paths and `portal_frame`
    /// diagnostics.
    pub(crate) visible: bool,
    torn_down: bool,
    /// Last frame pinned to the parent content rect (idempotence cache).
    /// `pub(crate)`: read by the `portal_frame` diagnostics impl.
    pub(crate) frame: Option<PortalFrame>,
    /// Parent content rect the frame was derived from (diagnostics).
    /// `pub(crate)`: read by the `portal_frame` diagnostics impl.
    pub(crate) parent_content_rect: Option<PortalFrame>,
    /// Hit-region mode (blueprint D08): the pointer monitor and the
    /// declared capture regions; `None` until a surface first declares
    /// regions. `pub(crate)`: owned by the `portal_hit` impl block.
    pub(crate) hit: Option<crate::portal_hit::PortalHitMonitor>,
}

impl NativePortal {
    /// Attach the portal to the browser window.
    ///
    /// The portal is allocated on the boxed heap here, **before** the sync
    /// observer is installed, so the observer's raw pointer targets a stable
    /// allocation; returning the `Box` (and later moving it inside the app's
    /// entity arena) never invalidates that pointer.
    ///
    /// # Safety
    ///
    /// `parent_ns_view` must be the live GPUI root `NSView*` of the browser
    /// window and `portal_ns_view` the live GPUI root `NSView*` of the portal
    /// window, both captured on the main thread; the bridge retains both.
    pub unsafe fn attach(
        mtm: MainThreadMarker,
        parent_ns_view: *mut std::ffi::c_void,
        portal_ns_view: *mut std::ffi::c_void,
    ) -> Result<Box<Self>, PortalAttachError> {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "NativePortal::attach off main thread"
        );
        let Some(parent_view): Option<Retained<NSView>> =
            (unsafe { Retained::retain(parent_ns_view.cast::<NSView>()) })
        else {
            return Err(PortalAttachError::NullParentView);
        };
        let Some(portal_view): Option<Retained<NSView>> =
            (unsafe { Retained::retain(portal_ns_view.cast::<NSView>()) })
        else {
            return Err(PortalAttachError::NullPortalView);
        };
        let Some(parent_window) = parent_view.window() else {
            return Err(PortalAttachError::NoParentWindow);
        };
        let Some(portal_window) = portal_view.window() else {
            return Err(PortalAttachError::NoPortalWindow);
        };

        let mut portal = Box::new(Self {
            portal_view: Some(portal_view),
            portal_window: Some(portal_window.clone()),
            parent_window: Some(parent_window.clone()),
            observer: None,
            attached: false,
            visible: false,
            torn_down: false,
            frame: None,
            parent_content_rect: None,
            hit: None,
        });
        portal.configure(mtm);
        // Defer the child attachment to a bare main run-loop turn:
        // `addChildWindow:` on an on-screen parent synchronously orders the
        // child in and posts window notifications into GPUI, re-borrowing
        // the App while the composition root still holds it (see
        // [`PortalDeferredWork`]). The deferred op attaches the portal above
        // the parent and immediately orders it out again, so the portal
        // starts hidden and never flashes as a standalone window.
        defer_window_op(
            mtm,
            portal_window.clone(),
            DeferredWindowOp::AttachChild {
                parent: parent_window.clone(),
                portal: portal_window.clone(),
            },
        );
        // The attach op is scheduled infallibly (both windows are live and
        // retained), so the bridge records the child relation immediately;
        // the E2E first snapshot is taken many run-loop turns later, after
        // the deferred op has applied.
        portal.attached = true;
        portal.hide(mtm);
        portal.sync_frame(mtm);
        // Install the observer last: from here on the raw pointer in the
        // observer targets this box's heap allocation, which never moves.
        portal.install_observer(mtm);
        Ok(portal)
    }

    /// Apply the implementation-owned window policy (idempotent; GPUI's
    /// `PopUp` creation already chose the non-activating panel class).
    fn configure(&self, _mtm: MainThreadMarker) {
        let Some(portal) = self.portal_window.clone() else {
            return;
        };
        // Borderless + non-activating: strip GPUI's default titled mask so
        // the portal has no titlebar region at all. Deferred to a bare main
        // run-loop turn: changing the style mask can synchronously fire
        // `NSWindowDidResizeNotification` into GPUI, which re-borrows the
        // App while the composition root still holds it (see
        // [`PortalDeferredWork`]).
        defer_window_op(
            _mtm,
            portal.clone(),
            DeferredWindowOp::StyleMask(NSWindowStyleMask::NonactivatingPanel),
        );
        // Transparent: never opaque; harden the app-seam's
        // `WindowBackgroundAppearance::Transparent` choice natively.
        portal.setOpaque(false);
        // SAFETY: main thread; `clearColor` is a valid NSColor object and
        // `setBackgroundColor:` accepts it (NSColor feature is off in this
        // crate, so the call is made through the runtime selector).
        unsafe {
            let clear: Retained<NSObject> = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![&*portal, setBackgroundColor: &*clear];
        }
        // No shadow, non-movable, and excluded from the Windows menu:
        // implementation-owned chrome, never a user-visible window identity.
        // NOTE: `hidesOnDeactivate` must NOT be set — empirically it prevents
        // the panel from ever being displayed by the window server (the
        // bridge orders the portal out itself when no presentation is
        // active, so the setting is redundant).
        portal.setHasShadow(false);
        portal.setMovable(false);
        portal.setExcludedFromWindowsMenu(true);
        // Never animate appearance: the portal must not flash a grow/scale.
        portal.setAnimationBehavior(NSWindowAnimationBehavior::None);
    }

    /// Re-pin the portal frame to the parent content rect (idempotent: an
    /// unchanged frame makes no `setFrame:` traffic).
    ///
    /// The frame is recorded immediately (diagnostics + idempotence) and
    /// applied on a bare main run-loop turn: `setFrame:` synchronously posts
    /// `NSWindowDidResize`/`DidMove` notifications that GPUI re-enters to
    /// update window bounds, which panics if the App is currently borrowed
    /// (GPUI's own mac platform defers exactly this hazard to the next run
    /// loop iteration).
    pub fn sync_frame(&mut self, _mtm: MainThreadMarker) {
        let Some(parent) = self.parent_window.as_deref() else {
            return;
        };
        let Some(portal) = self.portal_window.clone() else {
            return;
        };
        // The parent's content area in screen coordinates: the portal frame
        // is set to exactly this rect so it overlays the WKWebView pane
        // (child windows share the parent's screen coordinate space).
        let content_rect = PortalFrame::from(parent.contentRectForFrameRect(parent.frame()));
        self.parent_content_rect = Some(content_rect);
        if !should_sync_frame(self.frame, content_rect) {
            return;
        }
        self.frame = Some(content_rect);
        defer_window_op(_mtm, portal, DeferredWindowOp::Frame(content_rect.into()));
    }

    /// Order the portal window front. A non-activating panel never becomes
    /// key or main merely by showing, so focus stays in BBR's existing
    /// focus choreography.
    ///
    /// The commanded visibility becomes true immediately for lifecycle
    /// policy, while AppKit ordering is deferred to a bare main-run-loop turn.
    /// `orderFront:` can synchronously notify GPUI and re-borrow its `App`,
    /// exactly like the already-deferred frame/attachment/close mutations.
    pub fn show(&mut self, mtm: MainThreadMarker) {
        // Re-pin before showing: the parent may have moved while hidden.
        self.sync_frame(mtm);
        let Some(portal) = self.portal_window.clone() else {
            return;
        };
        self.visible = true;
        defer_window_op(mtm, portal, DeferredWindowOp::Show);
    }

    /// Order the portal window out (used when no native presentation is
    /// active and during teardown). The lifecycle flag clears immediately;
    /// native ordering converges on the next bare main-run-loop turn so GPUI
    /// is never re-entered while its `App` is borrowed.
    pub fn hide(&mut self, mtm: MainThreadMarker) {
        let Some(portal) = self.portal_window.clone() else {
            return;
        };
        self.visible = false;
        defer_window_op(mtm, portal, DeferredWindowOp::Hide);
    }

    /// Commanded visibility: `show`/`hide` on the active presentation path.
    pub fn set_visible(&mut self, mtm: MainThreadMarker, visible: bool) {
        if visible {
            self.show(mtm);
        } else {
            self.hide(mtm);
        }
    }

    fn install_observer(&mut self, mtm: MainThreadMarker) {
        let Some(parent_ptr) = self
            .parent_window
            .as_deref()
            .map(|window| window as *const NSWindow)
        else {
            return;
        };
        // SAFETY of the stored pointer: `self` is the boxed heap allocation
        // created in `attach`; the box is never moved afterwards, so the
        // pointer stays valid for the observer's whole lifetime (module-level
        // safety invariant).
        let observer = PortalSyncObserver::new(mtm, self as *mut Self);
        let center = NSNotificationCenter::defaultCenter();
        // SAFETY: main thread; `observer` is retained by `self`; `parent_ptr`
        // points at a live NSWindow; the default center observes on the
        // posting (main) thread.
        unsafe {
            center.addObserver_selector_name_object(
                &observer,
                sel!(portalSync:),
                Some(NSWindowDidMoveNotification),
                Some(&*parent_ptr),
            );
            center.addObserver_selector_name_object(
                &observer,
                sel!(portalSync:),
                Some(NSWindowDidResizeNotification),
                Some(&*parent_ptr),
            );
            // Window-lifecycle hardening (task packet
            // `native-portal-hardening`): fullscreen enter/leave, minimize/
            // restore, display changes, and the parent's close all re-enter
            // the host through the same observer on real AppKit
            // notifications, so every transition leaves the host correct or
            // cleanly torn down.
            center.addObserver_selector_name_object(
                &observer,
                sel!(portalLifecycle:),
                Some(NSWindowDidEnterFullScreenNotification),
                Some(&*parent_ptr),
            );
            center.addObserver_selector_name_object(
                &observer,
                sel!(portalLifecycle:),
                Some(NSWindowDidMiniaturizeNotification),
                Some(&*parent_ptr),
            );
            center.addObserver_selector_name_object(
                &observer,
                sel!(portalLifecycle:),
                Some(NSWindowDidDeminiaturizeNotification),
                Some(&*parent_ptr),
            );
            center.addObserver_selector_name_object(
                &observer,
                sel!(portalLifecycle:),
                Some(NSWindowDidChangeScreenNotification),
                Some(&*parent_ptr),
            );
            center.addObserver_selector_name_object(
                &observer,
                sel!(portalLifecycle:),
                Some(NSWindowWillCloseNotification),
                Some(&*parent_ptr),
            );
        }
        self.observer = Some(observer);
    }

    fn remove_observer(&mut self, _mtm: MainThreadMarker) {
        if let Some(observer) = self.observer.take() {
            let center = NSNotificationCenter::defaultCenter();
            // SAFETY: main thread; the observer was registered with the
            // default center and is still alive (we hold it).
            unsafe {
                center.removeObserver(&observer);
            }
        }
    }

    /// Deterministically detach and close the portal window: remove the
    /// observer, order the window out, drop the child relation, and close
    /// the window (GPUI's close path removes it from the window arena).
    pub fn teardown(&mut self, mtm: MainThreadMarker) {
        if self.torn_down {
            return;
        }
        self.torn_down = true;
        self.remove_observer(mtm);
        self.remove_hit_monitor();
        self.hide(mtm);
        if self.attached {
            if let Some(portal) = self.portal_window.as_deref() {
                if let Some(parent) = self.parent_window.as_deref() {
                    parent.removeChildWindow(portal);
                }
            }
            self.attached = false;
        }
        // Closing is deferred to a bare main run-loop turn: `close:` posts
        // `NSWindowWillCloseNotification` synchronously, which GPUI handles
        // by removing the window from its arena — a mutation that would
        // panic if the App is currently borrowed (e.g. inside a testkit
        // dispatch's `update_window`). The deferred work retains the window
        // until it closes, so dropping the bridge's retain below is safe.
        if let Some(portal) = self.portal_window.clone() {
            defer_window_op(mtm, portal, DeferredWindowOp::Close);
        }
        self.portal_view = None;
        self.portal_window = None;
        self.parent_window = None;
        self.frame = None;
        self.parent_content_rect = None;
    }
}

impl Drop for NativePortal {
    fn drop(&mut self) {
        // Detach-only on plain drop (app teardown): remove the observer and
        // the child relation and order the window out. Closing is the
        // explicit `teardown` path, or GPUI's own close at process end — a
        // native `close()` here could race GPUI's teardown.
        if self.torn_down {
            return;
        }
        if let Some(mtm) = MainThreadMarker::new() {
            self.remove_observer(mtm);
        }
        if let Some(portal) = self.portal_window.as_deref() {
            portal.orderOut(None);
        }
        if self.attached {
            if let Some(portal) = self.portal_window.as_deref() {
                if let Some(parent) = self.parent_window.as_deref() {
                    parent.removeChildWindow(portal);
                }
            }
            self.attached = false;
        }
    }
}
