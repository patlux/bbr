//! The macOS safe-titlebar adapter (packet `titlebar-navigation-controls`):
//! the native query that turns the window's real titlebar/layout guide into
//! the plain portable [`bbr_web::SafeTitlebarGuide`] the UI composes from.
//!
//! The guide carries exactly two facts the BBR-owned titlebar row needs:
//!
//! - the **titlebar strip height** (`frame` − `contentLayoutRect`), so the
//!   sidebar rail's top strip and the content inset match the native
//!   titlebar; and
//! - the **traffic-light region** (the close button's frame), so the
//!   sidebar toggle starts immediately after the real traffic lights —
//!   never a hardcoded x/y offset.
//!
//! Everything here is plain geometry read from the live window on the main
//! thread; no layout decision lives in this module and no traffic-light
//! coordinate is ever hardcoded. The pure coordinate conversion (an AppKit
//! bottom-left frame → the host top-left guide) is extracted into a
//! testable function so the flip math is proven without macOS.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSButton, NSView, NSWindowButton};
use objc2_foundation::NSRect;

use bbr_web::{SafeTitlebarGuide, WebRect};

/// Convert an AppKit window-coordinate frame (bottom-left origin, y up)
/// into the host top-left origin used by [`bbr_web::WebRect`]. Pure math so
/// the flip is unit-testable without macOS.
fn flip_window_frame(frame: NSRect, window_height: f64) -> WebRect {
    WebRect::new(
        frame.origin.x as f32,
        (window_height - (frame.origin.y + frame.size.height)) as f32,
        frame.size.width as f32,
        frame.size.height as f32,
    )
}

/// The live safe-titlebar guide of the window hosting `root`, or `None`
/// when the root has no window yet. Main thread only (AppKit).
pub fn safe_titlebar_guide(_mtm: MainThreadMarker, root: &NSView) -> Option<SafeTitlebarGuide> {
    debug_assert!(
        MainThreadMarker::new().is_some(),
        "titlebar guide off main thread"
    );
    let window = root.window()?;
    // `contentLayoutRect` is the non-obscured content area in window
    // coordinates; the difference to the window frame is the titlebar strip
    // height (the same measurement GPUI's `titlebar_height()` uses).
    let frame = window.frame();
    let content_layout = window.contentLayoutRect();
    let height = (frame.size.height - content_layout.size.height).max(0.0) as f32;
    // The close button is the leading traffic light; its frame is the
    // region the sidebar toggle must clear. Standard traffic lights keep
    // their default placement (GPUI moves them only when a custom
    // `traffic_light_position` is set, which BBR never does).
    let close_button: Option<Retained<NSButton>> =
        window.standardWindowButton(NSWindowButton::CloseButton);
    let traffic_lights = close_button
        .as_deref()
        .map(|button| flip_window_frame(button.frame(), frame.size.height))
        .unwrap_or_default();
    Some(SafeTitlebarGuide {
        height,
        traffic_lights,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_foundation::{NSPoint, NSSize};

    #[test]
    fn flip_converts_bottom_left_to_host_top_left() {
        // A 20×14 button sitting at (20, 26) in a 1000-high window
        // (bottom-left origin) is at top 1000 − (26 + 14) = 960.
        let frame = NSRect::new(NSPoint::new(20.0, 26.0), NSSize::new(20.0, 14.0));
        let guide = flip_window_frame(frame, 1000.0);
        assert_eq!(guide.x, 20.0);
        assert_eq!(guide.y, 960.0);
        assert_eq!(guide.width, 20.0);
        assert_eq!(guide.height, 14.0);
    }

    #[test]
    fn flip_keeps_the_region_inside_the_window() {
        // A full-height button at the left edge never leaves the window.
        let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(50.0, 30.0));
        let guide = flip_window_frame(frame, 30.0);
        assert_eq!(guide.x, 0.0);
        assert_eq!(guide.y, 0.0);
        assert!(guide.y + guide.height <= 30.0);
    }
}
