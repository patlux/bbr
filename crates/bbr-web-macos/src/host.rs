//! The host bridge: owns the GPUI root `NSView` and the AppKit-side helpers
//! for geometry and focus hand-off.
//!
//! ## Safety invariant
//!
//! [`HostBridge`] stores the GPUI root view as a retained `NSView` obtained
//! once from `raw_window_handle`. The pointer is valid for as long as the
//! owning window is alive; the bridge is owned by the engine, whose lifetime
//! is tied to the window's root view (via the slot element and the command
//! bar), so the two lifetimes coincide. The bridge is **only** used on the
//! main thread — every entry point accepts a `MainThreadMarker`, so a
//! non-main caller cannot construct one without explicitly proving otherwise.
//!
//! ## The responder overlay
//!
//! GPUI's root view (`GPUIView`, registered by GPUI itself) does **not**
//! override `acceptsFirstResponder`, so `makeFirstResponder(gpui_view)` is
//! refused by AppKit. To hand native keyboard focus back to the chrome we add
//! a transparent `NSView` subclass ([`ChromeResponder`]) whose only behavior
//! is `acceptsFirstResponder → YES`; every other `NSResponder` method uses the
//! default implementation, which forwards up the responder chain to its
//! `nextResponder` — the GPUI root view. Key events therefore arrive at GPUI
//! exactly as if `GPUIView` itself were the first responder, with no GPUI fork
//! required. The overlay is never hit-testable (`hitTest:` returns `nil`), so
//! it cannot steal clicks from the WKWebView or from GPUI chrome.

use objc2::rc::Retained;
use objc2::{define_class, msg_send, ClassType, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::NSView;
use objc2_foundation::{NSPoint, NSRect, NSSize};

use bbr_web::WebRect;

define_class!(
    /// A transparent, click-through view that can become the AppKit first
    /// responder and forwards all unhandled input up the responder chain to GPUI.
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    #[name = "BBRChromeResponder"]
    struct ChromeResponder;

    impl ChromeResponder {
        /// We can become first responder even though we never appear in hit
        /// tests. This is the entire point of the class.
        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        /// Never intercept pointer events: the overlay is invisible to
        /// hit-testing, so clicks continue to reach the WKWebView or GPUI
        /// chrome normally.
        #[unsafe(method_id(hitTest:))]
        fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> {
            None
        }
    }
);

impl ChromeResponder {
    /// Create a new overlay view (main thread only).
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: `init` is NSView's... send to super per wry's pattern.
        unsafe { msg_send![super(this), init] }
    }
}

pub(crate) struct HostBridge {
    /// Retains the GPUI root view so the pointer stays valid while we hold
    /// it. AppKit views retain their superviews, so retaining the root view
    /// from here creates no cycle (the window owns the root view; we are a
    /// leaf borrower). `None` until [`HostBridge::attach_root_view`].
    root_view: Option<Retained<NSView>>,

    /// The transparent first-responder acceptor, a subview of the root view.
    responder_overlay: Option<Retained<ChromeResponder>>,
}

impl HostBridge {
    pub(crate) fn new() -> Self {
        Self {
            root_view: None,
            responder_overlay: None,
        }
    }

    /// Claim ownership of the GPUI root view.
    ///
    /// # Safety
    ///
    /// `ns_view` must be a valid, still-alive `NSView*` (the GPUI root view).
    /// It is retained here, so afterwards the pointer stays valid until this
    /// bridge is dropped.
    pub(crate) unsafe fn attach_root_view(
        &mut self,
        mtm: MainThreadMarker,
        ns_view: *mut std::ffi::c_void,
    ) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "attach_root_view off main thread"
        );
        debug_assert!(!ns_view.is_null(), "GPUI root view must not be null");
        // SAFETY: caller promises a live NSView*; retaining is always sound.
        let Some(root): Option<Retained<NSView>> =
            (unsafe { Retained::retain(ns_view.cast::<NSView>()) })
        else {
            return;
        };

        // Install the responder overlay as a subview covering the root view,
        // pinned to its bounds. It is invisible to hit-testing (see the
        // class definition) so it changes no input routing.
        let overlay = ChromeResponder::new(mtm);
        let frame = root.bounds();
        overlay.setFrame(frame);
        overlay.setAutoresizingMask(
            objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        root.addSubview(&overlay);

        // Present the window without AppKit's grow-from-corner appearance
        // animation: the animation frames are driven by the display link and
        // freeze at arbitrary intermediate sizes while the app runs in the
        // background (the E2E/benchmark harness, `--bbr-hidden`), which made
        // window geometry nondeterministic under test. BBR windows are
        // chrome-like; appearing at their final frame matches the product's
        // "instant shell" principle and keeps programmatic geometry stable.
        if let Some(window) = root.window() {
            window.setAnimationBehavior(objc2_app_kit::NSWindowAnimationBehavior::None);
        }

        self.root_view = Some(root);
        self.responder_overlay = Some(overlay);
    }

    pub(crate) fn root_view(&self) -> Option<&NSView> {
        self.root_view.as_deref()
    }

    pub(crate) fn responder_overlay(&self) -> Option<&NSView> {
        self.responder_overlay
            .as_deref()
            .map(ChromeResponder::as_super)
    }

    /// Height of the root view in points — the container used by the
    /// y-flip when converting host layout rects into AppKit frames.
    pub(crate) fn content_height(&self) -> f32 {
        self.root_view()
            .map(|view| view.bounds().size.height as f32)
            .unwrap_or(0.0)
    }

    /// Convert a host-layout rect (top-left origin, points, relative to the
    /// window) into an AppKit frame (bottom-left origin) for a subview of the
    /// root view. Uses the root view's actual height so the flip can never
    /// drift from the true container height.
    ///
    /// AppKit's `NSView` coordinate system is bottom-left origin (these views
    /// are not flipped), so the vertical origin is mirrored via
    /// [`bbr_web::flip_rect`].
    pub(crate) fn frame_rect(&self, rect: WebRect) -> NSRect {
        let h = self.content_height();
        let flipped = bbr_web::flip_rect(rect, h);
        NSRect::new(
            NSPoint::new(flipped.x as f64, flipped.y as f64),
            NSSize::new(flipped.width as f64, flipped.height as f64),
        )
    }

    /// Give native keyboard focus to `view` (a subview of the root view,
    /// e.g. a WKWebView). AppKit responds by querying `acceptsFirstResponder`
    /// on the view and, once accepted, routing key events to it.
    pub(crate) fn focus_web_view(&self, mtm: MainThreadMarker, web_view: &NSView) {
        debug_assert!(MainThreadMarker::new().is_some(), "focus off main thread");
        self.make_first_responder(mtm, Some(web_view));
    }

    /// Give native keyboard focus back to the GPUI chrome (via the responder
    /// overlay, which forwards up to the GPUI root view). The caller must
    /// also move GPUI's own focus tree (`window.focus`) — AppKit's first
    /// responder and GPUI's focus tree are separate systems.
    pub(crate) fn focus_chrome(&self, mtm: MainThreadMarker) {
        debug_assert!(MainThreadMarker::new().is_some(), "focus off main thread");
        let overlay = self.responder_overlay();
        self.make_first_responder(mtm, overlay);
    }

    fn make_first_responder(&self, _mtm: MainThreadMarker, view: Option<&NSView>) {
        let Some(root) = self.root_view.as_deref() else {
            log::warn!("host bridge has no root view yet; skipping first-responder change");
            return;
        };
        let Some(window) = root.window() else {
            log::warn!("root view has no window yet; skipping first-responder change");
            return;
        };
        // A `&NSView` coerces to `&NSResponder` via the superclass chain.
        let responder: Option<&objc2_app_kit::NSResponder> = view.map(|v| v.as_super());
        // SAFETY: main thread only; `view` (when Some) is part of the
        // window's view hierarchy, so this cannot make the responder chain
        // invalid.
        let accepted = window.makeFirstResponder(responder);
        if !accepted {
            log::warn!("makeFirstResponder: rejected");
        }
    }
}

/// Blueprint D19 (test harness): move the window of a GPUI root view by
/// `(dx, dy)` points, `dy` downwards as in GPUI's screen coordinates.
///
/// # Safety
///
/// `ns_view` must be a live `NSView*`, used on the main thread.
pub unsafe fn move_window_by(ns_view: *mut std::ffi::c_void, dx: f64, dy: f64) {
    // SAFETY: the caller promises a live NSView* on the main thread.
    let Some(view) = (unsafe { ns_view.cast::<NSView>().as_ref() }) else {
        return;
    };
    if let Some(window) = view.window() {
        let frame = window.frame();
        window.setFrameOrigin(NSPoint::new(frame.origin.x + dx, frame.origin.y - dy));
    }
}
