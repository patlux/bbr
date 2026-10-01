//! Deferred AppKit window mutations for the native portal host (task packet
//! `native-portal-host`).
//!
//! AppKit window mutations (`setFrame:`, `setStyleMask:`, `addChildWindow:`,
//! `orderFront:`, `orderOut:`, `close:`) synchronously post window notifications into GPUI, whose
//! platform callbacks re-borrow the `App` RefCell — panicking when the
//! composition root or any update already holds it. GPUI's own mac platform
//! defers such work to the next run-loop iteration for the same reason
//! (`on_thermal_state_change` in `gpui_macos/src/platform.rs`); this module
//! is the same pattern using only the run loop that already runs the app (no
//! new dependency or link target). [`defer_window_op`] schedules one
//! mutation on a bare main run-loop turn, outside any GPUI borrow.

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask};
use objc2_foundation::{NSObject, NSRect};

/// `NSWindowAbove` (1) — the crate's `objc2-app-kit` features do not enable
/// `NSGraphics`, which gates `NSWindowOrderingMode`, so the ordering constant
/// is passed through the runtime selector with the documented raw value.
const NSWINDOW_ABOVE: i64 = 1;

/// One deferred window mutation applied on a bare main run-loop turn.
#[derive(Clone)]
pub(crate) enum DeferredWindowOp {
    /// Strip the titled style mask (borderless non-activating panel).
    StyleMask(NSWindowStyleMask),
    /// Pin the window frame (screen coordinates).
    Frame(NSRect),
    /// Attach the portal as a child of the parent ordered above, then
    /// immediately order it out so it starts hidden. Deferred because
    /// `addChildWindow:` on an on-screen parent synchronously orders the
    /// child in and posts window notifications into GPUI, which re-borrows
    /// the `App` while the composition root still holds it (the same
    /// re-entrancy hazard as `setFrame:`/`setStyleMask:`).
    AttachChild {
        parent: Retained<NSWindow>,
        portal: Retained<NSWindow>,
    },
    /// Reconcile current parent/application facts and the latest intent.
    Visibility {
        parent: Retained<NSWindow>,
        intent: std::rc::Rc<crate::portal_visibility::VisibilityIntent>,
    },
    /// Close the portal window. Deferred because `close:` synchronously
    /// posts `NSWindowWillCloseNotification`, which GPUI handles by removing
    /// the window from its arena — a mutation that panics while any `App`
    /// borrow (e.g. a testkit dispatch's `update_window`) is held (the same
    /// re-entrancy hazard as the other ops).
    Close,
    /// Enter/exit native fullscreen on the parent (`toggleFullScreen:`),
    /// adding the primary fullscreen collection behavior first when the
    /// window is not explicitly fullscreen-capable. Deferred: the fullscreen
    /// transition synchronously resizes/moves the window and posts
    /// move/resize notifications into GPUI, which re-borrows the `App`
    /// (native-portal-hardening lifecycle driver).
    ToggleFullscreen,
    /// Minimize the parent (`miniaturize:`) — deferred for the same
    /// notification re-entrancy reason.
    Minimize,
    /// Restore the parent (`deminiaturize:`) — deferred for the same
    /// notification re-entrancy reason.
    Deminimize,
    /// Set the window's collection behavior (native-portal-hardening
    /// Window Server space-assignment driver).
    CollectionBehavior(NSWindowCollectionBehavior),
    /// Close the parent browser window through the real AppKit close path
    /// (`performClose:`). Deferred: the close synchronously posts
    /// `NSWindowWillCloseNotification`, which GPUI handles by removing the
    /// window from its arena and re-borrowing the `App` (the same
    /// re-entrancy hazard as `Close`).
    CloseParent,
    #[cfg(feature = "testkit")]
    AppVisibility(bool),
}

/// Ivars of [`PortalDeferredWork`]: the retained window and the operation.
struct DeferredWorkIvars {
    window: Retained<NSWindow>,
    op: DeferredWindowOp,
}

define_class!(
    /// Applies one window mutation on a later main run-loop turn.
    ///
    /// AppKit window mutations (`setFrame:`, `setStyleMask:`,
    /// `addChildWindow:`, `orderFront:`, `orderOut:`) synchronously post window notifications into GPUI,
    /// whose platform callbacks re-borrow the `App` RefCell — panicking when
    /// the composition root or any update already holds it. GPUI's own mac
    /// platform defers such work to the next run-loop iteration for the
    /// same reason (`on_thermal_state_change` in `gpui_macos/src/platform.rs`);
    /// this class is the same pattern using only the run loop that already
    /// runs the app (no new dependency or link target).
    #[unsafe(super = NSObject)]
    #[ivars = DeferredWorkIvars]
    #[thread_kind = MainThreadOnly]
    #[name = "BBRPortalDeferredWork"]
    struct PortalDeferredWork;

    impl PortalDeferredWork {
        #[unsafe(method(applyWork:))]
        fn apply_work(&self, _object: Option<&AnyObject>) {
            let ivars = self.ivars();
            match &ivars.op {
                DeferredWindowOp::StyleMask(mask) => ivars.window.setStyleMask(*mask),
                DeferredWindowOp::Frame(frame) => ivars.window.setFrame_display(*frame, false),
                DeferredWindowOp::Visibility { parent, intent } => {
                    crate::portal_visibility::apply(self.mtm(), &ivars.window, parent, intent);
                }
                DeferredWindowOp::Close => ivars.window.close(),
                DeferredWindowOp::ToggleFullscreen => {
                    // Deterministic fullscreen capability: titled windows are
                    // fullscreen-capable by default; make it explicit when
                    // the behavior bit is missing (safe reads/writes on the
                    // bare turn).
                    let behavior = ivars.window.collectionBehavior();
                    if !behavior.contains(NSWindowCollectionBehavior::FullScreenPrimary)
                        && !behavior.contains(NSWindowCollectionBehavior::FullScreenAuxiliary)
                    {
                        ivars.window.setCollectionBehavior(
                            behavior | NSWindowCollectionBehavior::FullScreenPrimary,
                        );
                    }
                    ivars.window.toggleFullScreen(None);
                }
                DeferredWindowOp::Minimize => ivars.window.miniaturize(None),
                DeferredWindowOp::Deminimize => ivars.window.deminiaturize(None),
                DeferredWindowOp::CollectionBehavior(behavior) => {
                    ivars.window.setCollectionBehavior(*behavior);
                }
                DeferredWindowOp::CloseParent => ivars.window.performClose(None),
                #[cfg(feature = "testkit")]
                DeferredWindowOp::AppVisibility(hidden) => {
                    let app = objc2_app_kit::NSApplication::sharedApplication(self.mtm());
                    if *hidden {
                        app.hide(None);
                    } else {
                        app.unhide(None);
                    }
                }
                DeferredWindowOp::AttachChild { parent, portal } => {
                    // GPUI PopUp defaults to level 101 and CanJoinAllSpaces.
                    // This is browser-owned chrome, never a global popup.
                    portal.setLevel(parent.level());
                    portal.setCollectionBehavior(NSWindowCollectionBehavior::FullScreenAuxiliary);
                    // SAFETY: main thread; both windows are live; the
                    // child-window attachment is an AppKit main-thread API
                    // (the generated `addChildWindow:ordered:` is gated
                    // behind the crate's disabled `NSGraphics` feature, so
                    // it is called through the runtime selector with the
                    // same semantics).
                    unsafe {
                        let _: () = msg_send![
                            &**parent,
                            addChildWindow: &**portal,
                            ordered: NSWINDOW_ABOVE
                        ];
                    }
                    // Attaching a child to an on-screen parent orders it in
                    // automatically; the portal must start hidden and never
                    // flash, so it is ordered out on the same bare turn.
                    portal.orderOut(None);
                }
            }
        }
    }
);

impl PortalDeferredWork {
    fn new(
        mtm: MainThreadMarker,
        window: Retained<NSWindow>,
        op: DeferredWindowOp,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(DeferredWorkIvars { window, op });
        // SAFETY: `init` on the main thread (ChromeResponder pattern).
        unsafe { msg_send![super(this), init] }
    }

    /// Schedule `applyWork:` on the main run loop after a 0 delay, so the
    /// mutation runs on a bare run-loop turn outside any GPUI borrow.
    /// `performSelector:withObject:afterDelay:` retains the receiver until
    /// the selector fires (then it is released), so no further retention is
    /// needed.
    fn schedule(&self) {
        // SAFETY: main thread; the selector runs on the main run loop in the
        // default mode, which is where the app's event loop is running.
        unsafe {
            let _: () = msg_send![
                self,
                performSelector: sel!(applyWork:),
                withObject: None::<&AnyObject>,
                afterDelay: 0.0
            ];
        }
    }
}

/// Run `op` on the portal window on a bare main run-loop turn, outside any
/// GPUI `App` borrow (see [`PortalDeferredWork`]).
pub(crate) fn defer_window_op(
    mtm: MainThreadMarker,
    window: Retained<NSWindow>,
    op: DeferredWindowOp,
) {
    let work = PortalDeferredWork::new(mtm, window, op);
    work.schedule();
}
