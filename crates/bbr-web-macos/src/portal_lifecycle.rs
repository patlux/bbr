//! Window-lifecycle hardening for the one native portal host (task packet
//! `native-portal-hardening`).
//!
//! The portal's AppKit notification observer ([`PortalSyncObserver`], the
//! `BBRPortalSyncObserver` class) and the lifecycle driver methods it
//! dispatches on real AppKit notifications — fullscreen enter/leave,
//! minimize/restore, Window Server space-assignment transitions, and the
//! parent's close — plus the testkit drivers the hardening E2E suite uses.
//! Every transition leaves the host correct or cleanly torn down: never
//! orphaned, never occluding the web view, never intercepting input. Split
//! from `portal.rs` (the host core) so both files stay under the ~500-line
//! budget; the observer and the drivers are inherent impls of
//! [`crate::portal::NativePortal`] reading its `pub(crate)`
//! window/visibility fields.

use objc2::rc::Retained;
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationDidBecomeActiveNotification, NSApplicationDidHideNotification,
    NSApplicationDidResignActiveNotification, NSApplicationDidUnhideNotification, NSWindow,
    NSWindowCollectionBehavior, NSWindowDidBecomeKeyNotification,
    NSWindowDidChangeScreenNotification, NSWindowDidDeminiaturizeNotification,
    NSWindowDidEnterFullScreenNotification, NSWindowDidExitFullScreenNotification,
    NSWindowDidMiniaturizeNotification, NSWindowDidResignKeyNotification, NSWindowStyleMask,
    NSWindowWillCloseNotification, NSWorkspace, NSWorkspaceActiveSpaceDidChangeNotification,
};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObject};

use crate::portal::NativePortal;
use crate::portal_deferred::{defer_window_op, DeferredWindowOp};

/// Ivars of [`PortalSyncObserver`]: a raw pointer to the owning portal's
/// boxed heap allocation.
///
/// Main-thread-only: the observer is created by and retained by the portal,
/// and removed before the portal is dropped, so the pointer is always valid
/// while the observer can fire. The portal is allocated on the boxed heap in
/// [`NativePortal::attach`] before the observer is installed and never moves
/// afterwards (see the portal module's safety invariant).
///
/// `pub(crate)`: the `pub(crate)` observer class exposes its ivar type.
pub(crate) struct PortalSyncIvars {
    portal: *mut NativePortal,
}

define_class!(
    /// Receives the parent window's move/resize notifications and re-pins
    /// the portal frame to the parent content rect. Frame synchronization is
    /// AppKit's job per the packet's constraint list, so it is observed
    /// natively and never routed through GPUI.
    #[unsafe(super = NSObject)]
    #[ivars = PortalSyncIvars]
    #[thread_kind = MainThreadOnly]
    #[name = "BBRPortalSyncObserver"]
    pub(crate) struct PortalSyncObserver;

    impl PortalSyncObserver {
        #[unsafe(method(portalSync:))]
        fn portal_sync(&self, _notification: Option<&NSNotification>) {
            // SAFETY: `portalSync:` fires on the main thread (window
            // notifications are posted there), and the portal outlives the
            // observer, which it owns and removes first. The pointer targets
            // the portal's boxed heap allocation, which never moves.
            let portal = unsafe { &mut *self.ivars().portal };
            if let Some(mtm) = MainThreadMarker::new() {
                portal.sync_frame(mtm);
                portal.sync_visibility(mtm);
            }
        }

        #[unsafe(method(portalVisibility:))]
        fn portal_visibility(&self, _notification: Option<&NSNotification>) {
            // Pinned allocation; observer is removed before NativePortal drop.
            let portal = unsafe { &*self.ivars().portal };
            portal.sync_visibility(self.mtm());
        }

        /// Window-lifecycle hardening (task packet `native-portal-hardening`):
        /// fullscreen enter/leave, minimize/restore, display changes, and the
        /// parent's close all arrive here through real AppKit notifications
        /// on the parent `NSWindow`. Each leaves the host correct or cleanly
        /// torn down — never orphaned, never occluding the web view, never
        /// intercepting input.
        #[unsafe(method(portalLifecycle:))]
        fn portal_lifecycle(&self, notification: Option<&NSNotification>) {
            let Some(notification) = notification else {
                return;
            };
            let Some(mtm) = MainThreadMarker::new() else {
                return;
            };
            // SAFETY: `portalLifecycle:` fires on the main thread (window
            // notifications are posted there), and the portal outlives the
            // observer, which it owns and removes first. The pointer targets
            // the portal's boxed heap allocation, which never moves.
            let portal = unsafe { &mut *self.ivars().portal };
            let name = notification.name();
            // The notification-name statics are extern `NSString` statics
            // (unsafe to read; reading them is the only unsafe part).
            if unsafe { name.isEqualToString(NSWindowWillCloseNotification) } {
                // The browser window is closing while a presentation may be
                // open: tear the portal down deterministically (remove the
                // observer, order out, drop the child relation, close the
                // portal window). Idempotent; the app's own entity drop
                // afterwards is a no-op.
                portal.teardown(mtm);
            } else if unsafe { name.isEqualToString(NSWindowDidMiniaturizeNotification) } {
                portal.parent_miniaturized(mtm);
            } else if unsafe { name.isEqualToString(NSWindowDidDeminiaturizeNotification) } {
                portal.parent_deminiaturized(mtm);
            } else if unsafe { name.isEqualToString(NSWindowDidEnterFullScreenNotification) }
                || unsafe { name.isEqualToString(NSWindowDidExitFullScreenNotification) }
            {
                portal.fullscreen_did_change(mtm);
            } else if unsafe { name.isEqualToString(NSWindowDidChangeScreenNotification) } {
                // Display change (including a different backing scale
                // factor): re-pin in points; the frame is point-based, so a
                // scale change alone never shifts it (belt-and-braces for a
                // content-rect change that skipped the move/resize events).
                portal.sync_frame(mtm);
                portal.sync_visibility(mtm);
            }
        }
    }
);

impl PortalSyncObserver {
    pub(crate) fn observe_visibility(&self, mtm: MainThreadMarker, parent: &NSWindow) {
        let center = NSNotificationCenter::defaultCenter();
        let app = NSApplication::sharedApplication(mtm);
        // Main-thread notifications; removed before the pinned owner drops.
        unsafe {
            for name in [
                NSWindowDidBecomeKeyNotification,
                NSWindowDidResignKeyNotification,
            ] {
                center.addObserver_selector_name_object(
                    self,
                    sel!(portalVisibility:),
                    Some(name),
                    Some(parent),
                );
            }
            for name in [
                NSApplicationDidBecomeActiveNotification,
                NSApplicationDidResignActiveNotification,
                NSApplicationDidHideNotification,
                NSApplicationDidUnhideNotification,
            ] {
                center.addObserver_selector_name_object(
                    self,
                    sel!(portalVisibility:),
                    Some(name),
                    Some(&app),
                );
            }
            center.addObserver_selector_name_object(
                self,
                sel!(portalLifecycle:),
                Some(NSWindowDidExitFullScreenNotification),
                Some(parent),
            );
            NSWorkspace::sharedWorkspace()
                .notificationCenter()
                .addObserver_selector_name_object(
                    self,
                    sel!(portalVisibility:),
                    Some(NSWorkspaceActiveSpaceDidChangeNotification),
                    None,
                );
        }
    }

    /// Create and initialize the observer on the main thread with a raw
    /// pointer to the owning portal's boxed heap allocation.
    pub(crate) fn new(mtm: MainThreadMarker, portal: *mut NativePortal) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(PortalSyncIvars { portal });
        // SAFETY: `init` on the main thread (ChromeResponder pattern).
        unsafe { msg_send![super(this), init] }
    }
}

/// Test-only real application lifecycle driver, applied outside the GPUI borrow.
#[cfg(feature = "testkit")]
pub fn drive_app_visibility(mtm: MainThreadMarker, hidden: bool) {
    let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
    if let Some(window) = app.windows().firstObject() {
        defer_window_op(mtm, window, DeferredWindowOp::AppVisibility(hidden));
    }
}

impl NativePortal {
    /// The parent was miniaturized (real `NSWindowDidMiniaturizeNotification`):
    /// order the portal out so it can never occlude the desktop or intercept
    /// input while the browser window is minimized. The commanded `visible`
    /// flag stays armed — the presentation is paused, not dismissed — so a
    /// restore re-shows the same presentation on the same host window.
    pub fn parent_miniaturized(&mut self, mtm: MainThreadMarker) {
        self.sync_visibility(mtm);
    }

    /// The parent was restored (real `NSWindowDidDeminiaturizeNotification`):
    /// re-pin the frame and, when a presentation is still armed, order the
    /// portal front again. AppKit itself restores children of deminiaturized
    /// windows, but the explicit re-show is deterministic regardless.
    pub fn parent_deminiaturized(&mut self, mtm: MainThreadMarker) {
        self.sync_frame(mtm);
        self.sync_visibility(mtm);
    }

    /// The parent entered or left native fullscreen (real
    /// `NSWindowDidEnterFullScreenNotification` / `DidExitFullScreen`):
    /// re-pin the frame to the (fullscreen or restored) content rect and,
    /// when a presentation is armed, re-order the portal front — the
    /// fullscreen transition moves the window to the fullscreen space and
    /// back, and the attached child must follow without being recreated or
    /// orphaned.
    pub fn fullscreen_did_change(&mut self, mtm: MainThreadMarker) {
        self.sync_frame(mtm);
        self.sync_visibility(mtm);
    }

    /// Drive the parent's real fullscreen transition (`toggleFullScreen:`).
    /// Idempotent: no-op when the window is already in the requested state.
    /// The mutation itself is deferred to a bare main run-loop turn: the
    /// fullscreen transition synchronously resizes/moves the window and
    /// posts move/resize notifications into GPUI, which re-borrows the
    /// `App` while a testkit dispatch holds it (the same re-entrancy rule
    /// as every other AppKit window mutation; see [`DeferredWindowOp`]).
    pub fn set_fullscreen(&mut self, mtm: MainThreadMarker, fullscreen: bool) {
        let Some(parent) = self.parent_window.as_deref() else {
            return;
        };
        let currently_fullscreen = parent.styleMask().contains(NSWindowStyleMask::FullScreen);
        if currently_fullscreen == fullscreen {
            return;
        }
        defer_window_op(
            mtm,
            self.parent_window.clone().expect("parent live"),
            DeferredWindowOp::ToggleFullscreen,
        );
    }

    /// Drive the parent's real minimize/restore (`miniaturize:` /
    /// `deminiaturize:`). Idempotent; deferred to a bare main run-loop turn
    /// (minimize/restore posts window notifications into GPUI, see
    /// [`DeferredWindowOp`]).
    pub fn set_minimized(&mut self, mtm: MainThreadMarker, minimized: bool) {
        let Some(parent) = self.parent_window.as_deref() else {
            return;
        };
        let already = parent.isMiniaturized();
        if minimized == already {
            return;
        }
        let op = if minimized {
            DeferredWindowOp::Minimize
        } else {
            DeferredWindowOp::Deminimize
        };
        defer_window_op(mtm, self.parent_window.clone().expect("parent live"), op);
    }

    /// Drive a real Window Server space-assignment transition on the parent:
    /// set/clear the `CanJoinAllSpaces` collection behavior, so the window
    /// server places the parent (and its attached child) on every space.
    /// Idempotent; the mutation is deferred to a bare main run-loop turn.
    pub fn set_all_spaces(&mut self, mtm: MainThreadMarker, all_spaces: bool) {
        let Some(parent) = self.parent_window.as_deref() else {
            return;
        };
        let behavior = parent.collectionBehavior();
        let target = if all_spaces {
            behavior | NSWindowCollectionBehavior::CanJoinAllSpaces
        } else {
            behavior - NSWindowCollectionBehavior::CanJoinAllSpaces
        };
        if target != behavior {
            defer_window_op(
                mtm,
                self.parent_window.clone().expect("parent live"),
                DeferredWindowOp::CollectionBehavior(target),
            );
        }
    }

    /// Close the parent browser window through the real AppKit close path
    /// (`performClose:` — the exact route the user's close button takes, so
    /// the delegate's close decision is honored). Deferred to a bare main
    /// run-loop turn: the close synchronously posts
    /// `NSWindowWillCloseNotification`, which GPUI handles by removing the
    /// window from its arena and re-borrowing the `App`. The portal's own
    /// will-close observer then tears the host down deterministically.
    pub fn close_parent(&mut self, mtm: MainThreadMarker) {
        if self.parent_window.is_none() {
            return;
        }
        defer_window_op(
            mtm,
            self.parent_window.clone().expect("parent live"),
            DeferredWindowOp::CloseParent,
        );
    }
}
