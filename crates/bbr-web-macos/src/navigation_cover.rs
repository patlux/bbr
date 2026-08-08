//! Dark native cover shown above a WKWebView while a navigation is loading.
//!
//! WebKit paints a white document plane between provisional navigation and the
//! destination's first completed render even when `underPageBackgroundColor`
//! is dark. This sibling AppKit view masks that transient plane. It is
//! click-through, follows the WKWebView frame, and disappears only after the
//! navigation finishes or fails.

use objc2::rc::Retained;
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSColor, NSRectFill, NSView};
use objc2_foundation::{NSPoint, NSRect};

define_class!(
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    #[name = "BBRNavigationCover"]
    pub(crate) struct NavigationCover;

    impl NavigationCover {
        #[unsafe(method(isOpaque))]
        fn is_opaque(&self) -> bool {
            true
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty_rect: NSRect) {
            let color = NSColor::colorWithSRGBRed_green_blue_alpha(
                0x15 as f64 / 255.0,
                0x17 as f64 / 255.0,
                0x19 as f64 / 255.0,
                1.0,
            );
            color.set();
            NSRectFill(self.bounds());
        }

        #[unsafe(method_id(hitTest:))]
        fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> {
            None
        }
    }
);

impl NavigationCover {
    pub(crate) fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        let cover: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };
        cover.setAutoresizingMask(
            objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        cover
    }
}
