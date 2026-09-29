//! Blueprint D02: the pointer observer behind the floating rail's edge
//! trigger (DESIGN §2).
//!
//! The hidden rail's trigger must see the pointer over the page, where the
//! WKWebView — not GPUI — receives the events. A public **local** event
//! monitor observes every pointer move the app receives (over the page, the
//! chrome or the Portal) and hands a sample in the browser window's
//! coordinates (points, top-left origin — GPUI's space, which the Portal
//! shares) to the app. The monitor only observes: it always returns the
//! event untouched, so the page keeps every move and drag it would get.

use std::ffi::c_void;
use std::ptr::NonNull;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSEvent, NSEventMask, NSView};

/// One observed pointer position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObservedPointer {
    /// Window points, top-left origin (may lie outside the window).
    pub x: f64,
    pub y: f64,
    /// The event time in milliseconds (monotonic, since system start).
    pub time_ms: f64,
    /// A mouse button is held.
    pub pressed: bool,
    /// Blueprint D11: this sample is a primary-button press (a press
    /// outside site settings closes them).
    pub down: bool,
}

/// The installed monitor; removing it (drop) stops the samples.
pub struct PointerObserver {
    monitor: Option<Retained<AnyObject>>,
    _block: block2::RcBlock<dyn Fn(NonNull<NSEvent>) -> *mut NSEvent>,
}

impl PointerObserver {
    /// Observe pointer moves relative to `view`'s window.
    ///
    /// # Safety
    /// `view` must be the live GPUI root `NSView*` of the browser window,
    /// and the call must run on the main thread. `sink` runs on the main
    /// thread inside AppKit's event dispatch: it must not re-enter GPUI
    /// synchronously (hand the sample to a channel).
    pub unsafe fn install(
        view: *mut c_void,
        sink: impl Fn(ObservedPointer) + 'static,
    ) -> Option<Self> {
        MainThreadMarker::new()?;
        // SAFETY (caller contract): a live NSView.
        let view: Retained<NSView> = unsafe { Retained::retain(view.cast::<NSView>()) }?;
        let block: block2::RcBlock<dyn Fn(NonNull<NSEvent>) -> *mut NSEvent> =
            block2::RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
                // SAFETY: AppKit hands a valid event for the handler's call.
                let native = unsafe { event.as_ref() };
                if let Some(window) = view.window() {
                    let screen = NSEvent::mouseLocation();
                    let frame = window.frame();
                    sink(ObservedPointer {
                        x: screen.x - frame.origin.x,
                        y: frame.size.height - (screen.y - frame.origin.y),
                        time_ms: native.timestamp() * 1000.0,
                        pressed: NSEvent::pressedMouseButtons() != 0,
                        down: native.r#type() == objc2_app_kit::NSEventType::LeftMouseDown,
                    });
                }
                event.as_ptr()
            });
        let mask =
            NSEventMask::MouseMoved | NSEventMask::LeftMouseDragged | NSEventMask::LeftMouseDown;
        // SAFETY: main thread; the handler returns the original event (pure
        // observation). AppKit copies the block for the monitor's lifetime;
        // it is retained here as well until the monitor is removed.
        let monitor =
            unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &block) };
        Some(Self {
            monitor,
            _block: block,
        })
    }
}

impl Drop for PointerObserver {
    fn drop(&mut self) {
        if MainThreadMarker::new().is_some() {
            if let Some(monitor) = self.monitor.take() {
                // SAFETY: main thread; installed by `addLocalMonitor…`,
                // removed exactly once.
                unsafe { NSEvent::removeMonitor(&monitor) };
            }
        }
    }
}
