//! Pure native-portal frame geometry and diagnostics (task packet
//! `native-portal-host`).
//!
//! Plain data and pure math only: [`PortalFrame`] (screen coordinates in
//! points), the deterministic frame-sync decision, the host-layout → screen
//! conversion, the sync evidence predicate, and the typed native diagnostics
//! snapshot. Nothing here touches a window or the main thread, so the math
//! is unit-testable without any AppKit machinery; [`super::portal`] is the
//! only consumer and owns the AppKit side.

use bbr_web::WebRect;
use objc2_foundation::{NSPoint, NSRect, NSSize};

/// Epsilon (points) used for idempotent frame synchronization: sub-1/1000 pt
/// drift from AppKit coordinate rounding is never re-applied.
pub const FRAME_EPSILON: f64 = 0.001;

/// A frame in screen coordinates (points; AppKit's base coordinate system).
///
/// Kept as plain data so diagnostics and the future testkit snapshot never
/// depend on an `NSRect` (which is not serde-serializable) and so the
/// conversion math is unit-testable without any AppKit machinery.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalFrame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl PortalFrame {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Bottom edge (screen coordinates grow upwards).
    pub fn max_y(self) -> f64 {
        self.y + self.height
    }

    /// True when every component is within `epsilon` (points).
    pub fn approx_eq(self, other: Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() <= epsilon
            && (self.y - other.y).abs() <= epsilon
            && (self.width - other.width).abs() <= epsilon
            && (self.height - other.height).abs() <= epsilon
    }
}

impl From<NSRect> for PortalFrame {
    fn from(rect: NSRect) -> Self {
        Self::new(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        )
    }
}

impl From<PortalFrame> for NSRect {
    fn from(frame: PortalFrame) -> Self {
        NSRect::new(
            NSPoint::new(frame.x, frame.y),
            NSSize::new(frame.width, frame.height),
        )
    }
}

/// Deterministic frame-sync decision: `true` when the candidate differs from
/// the last synced frame within [`FRAME_EPSILON`] (a no-op sync is skipped,
/// so an idle or untouched window makes no repeated `setFrame:` traffic).
pub fn should_sync_frame(cached: Option<PortalFrame>, candidate: PortalFrame) -> bool {
    cached.is_none_or(|last| !last.approx_eq(candidate, FRAME_EPSILON))
}

/// Convert a host-layout rect (top-left origin, points, relative to the
/// parent window's content area — the same convention as
/// [`bbr_web::WebRect`]) into an AppKit screen frame, given the parent's
/// content rect in screen coordinates.
///
/// The vertical origin is mirrored exactly like [`bbr_web::flip_rect`], then
/// offset by the content rect's screen origin: screen y is measured from the
/// content area's top edge, which sits at `content.y + content.height`.
pub fn portal_screen_frame(content: PortalFrame, host_rect: WebRect) -> PortalFrame {
    PortalFrame {
        x: content.x + host_rect.x as f64,
        y: content.max_y() - (host_rect.y as f64 + host_rect.height as f64),
        width: host_rect.width as f64,
        height: host_rect.height as f64,
    }
}

/// True when the last synced frame pins the portal to the parent content
/// rect (the packet's frame-synchronization evidence; pure so it is
/// unit-testable).
pub fn frame_matches_content(frame: Option<PortalFrame>, content: Option<PortalFrame>) -> bool {
    match (frame, content) {
        (Some(frame), Some(content)) => frame.approx_eq(content, FRAME_EPSILON),
        (None, None) => true,
        _ => false,
    }
}

/// Typed snapshot of the native portal host (DESIGN §15 semantics; the
/// testkit snapshot mirrors these fields verbatim).
#[derive(Debug, Clone, PartialEq)]
pub struct PortalNativeDiagnostics {
    /// The portal `NSWindow` is attached as a child of the browser window.
    pub child_attached: bool,
    /// Logical visibility commanded by the app (orderFront/orderOut).
    pub visible: bool,
    /// The AppKit window's actual `isVisible` state.
    pub native_visible: bool,
    /// Last frame synchronized to the parent content rect (screen coords).
    pub frame: Option<PortalFrame>,
    /// The parent content rect the frame was derived from (screen coords).
    pub parent_content_rect: Option<PortalFrame>,
    /// The last sync pinned the portal exactly to the parent content rect.
    pub synchronized: bool,
    /// The borderless non-activating-panel policy is applied.
    pub nonactivating_panel: bool,
    pub parent_window_number: i64,
    pub portal_window_number: i64,
    /// The AppKit window number frontmost at the probe point (the last
    /// synced frame's center — the centered probe card's point). `None`
    /// when no frame has been synced yet.
    ///
    /// This is an absolute, cross-application query: on a locked desktop or
    /// when another app's window covers the point it reports that window,
    /// not the portal. The deterministic hit-test evidence is
    /// [`PortalNativeDiagnostics::portal_owns_probe_point`].
    pub frontmost_at_probe_point: Option<i64>,
    /// Measured owner classification of
    /// [`PortalNativeDiagnostics::frontmost_at_probe_point`]: whether that
    /// window number belongs to this process (one of this app's own
    /// `NSWindow` numbers, membership-tested against `NSApplication.windows`
    /// on the macOS host). `false` when no window is frontmost or when the
    /// frontmost window belongs to another application (a foreign
    /// always-on-top window or a system overlay covering the probe point).
    /// Lets the testkit distinguish a provably foreign overlay from a
    /// BBR-owned window without dropping the raw absolute query.
    pub frontmost_at_probe_point_is_this_app: bool,
    /// The AppKit window number frontmost at the probe point strictly BELOW
    /// the portal window (`windowNumberAtPoint:belowWindowWithWindowNumber:`
    /// with the portal as the reference). Raw diagnostic only: with
    /// all-spaces/PopUp window behavior the window server's answer can be an
    /// unrelated overlay, so the deterministic evidence is
    /// [`PortalNativeDiagnostics::portal_owns_probe_point`]. `None` when the
    /// portal has no frame or its window number is not yet allocated.
    pub below_portal_at_probe_point: Option<i64>,
    /// Deterministic AppKit hit-test evidence at the probe point (the
    /// centered probe card's point, inside the portal frame): the portal is
    /// an attached child window ordered above its parent, it is visible, its
    /// frame is pinned to the parent content rect (so it covers the point),
    /// and it never became key/main. AppKit hit-tests child windows above
    /// their parent's views, so when this is true a click at the probe point
    /// is routed to the portal window — never to the WKWebView underneath.
    /// Environment-independent: no cross-application window queries, so a
    /// locked desktop or another app's overlay cannot change it.
    pub portal_owns_probe_point: bool,
    /// The non-key policy holds: the portal is never key merely by showing
    /// (non-activating panel + `orderFront:` only).
    pub portal_is_key: bool,
    /// The non-main policy holds: the portal is never main merely by
    /// showing.
    pub portal_is_main: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_frame_roundtrips_through_nsrect() {
        let frame = PortalFrame::new(1.5, 2.5, 800.0, 600.0);
        let rect: NSRect = frame.into();
        assert_eq!(rect.origin.x, 1.5);
        assert_eq!(rect.origin.y, 2.5);
        assert_eq!(rect.size.width, 800.0);
        assert_eq!(rect.size.height, 600.0);
        assert_eq!(PortalFrame::from(rect), frame);
    }

    #[test]
    fn frames_approx_eq_tolerates_rounding_only() {
        let a = PortalFrame::new(10.0, 20.0, 800.0, 600.0);
        let sub_pixel = PortalFrame::new(10.0004, 20.0, 800.0, 600.0);
        assert!(
            a.approx_eq(sub_pixel, FRAME_EPSILON),
            "sub-1/1000 pt drift is rounding, not a sync"
        );
        let real_change = PortalFrame::new(10.5, 20.0, 800.0, 600.0);
        assert!(
            !a.approx_eq(real_change, FRAME_EPSILON),
            "a half-point move must re-sync"
        );
    }

    #[test]
    fn should_sync_frame_skips_unchanged_and_syncs_changed() {
        let content = PortalFrame::new(100.0, 200.0, 800.0, 500.0);
        assert!(
            should_sync_frame(None, content),
            "first sync always applies"
        );
        assert!(
            !should_sync_frame(Some(content), content),
            "unchanged frame is a no-op"
        );
        let moved = PortalFrame::new(140.0, 200.0, 800.0, 500.0);
        assert!(
            should_sync_frame(Some(content), moved),
            "changed frame re-syncs"
        );
    }

    #[test]
    fn portal_screen_frame_full_content_is_content_origin() {
        let content = PortalFrame::new(100.0, 200.0, 800.0, 500.0);
        let host = WebRect::new(0.0, 0.0, 800.0, 500.0);
        assert_eq!(
            portal_screen_frame(content, host),
            PortalFrame::new(100.0, 200.0, 800.0, 500.0),
            "a full-content host rect pins the portal to the content rect"
        );
    }

    #[test]
    fn portal_screen_frame_flips_vertical_origin() {
        // Host layout is top-left origin: y=0 is the content's top edge,
        // which sits at screen y = content.y + content.height (200 + 500).
        let content = PortalFrame::new(100.0, 200.0, 800.0, 500.0);
        let host = WebRect::new(0.0, 0.0, 400.0, 300.0);
        assert_eq!(
            portal_screen_frame(content, host),
            PortalFrame::new(100.0, 400.0, 400.0, 300.0),
            "top-aligned host rect maps to the content top in screen coords"
        );
    }

    #[test]
    fn portal_screen_frame_applies_host_offset() {
        let content = PortalFrame::new(100.0, 200.0, 800.0, 500.0);
        let host = WebRect::new(40.0, 60.0, 320.0, 200.0);
        assert_eq!(
            portal_screen_frame(content, host),
            PortalFrame::new(140.0, 440.0, 320.0, 200.0),
            "host offset shifts x; y mirrors from the content top edge"
        );
    }

    #[test]
    fn frame_matches_content_is_pure_sync_evidence() {
        let content = Some(PortalFrame::new(100.0, 200.0, 800.0, 500.0));
        let same = Some(PortalFrame::new(100.0004, 200.0, 800.0, 500.0));
        assert!(frame_matches_content(same, content));
        let drifted = Some(PortalFrame::new(105.0, 200.0, 800.0, 500.0));
        assert!(!frame_matches_content(drifted, content));
        assert!(
            frame_matches_content(None, None),
            "never synced counts as in sync"
        );
        assert!(!frame_matches_content(None, content));
    }

    #[test]
    fn diagnostics_snapshot_is_pure_data() {
        let diagnostics = PortalNativeDiagnostics {
            child_attached: true,
            visible: false,
            native_visible: false,
            frame: Some(PortalFrame::new(0.0, 0.0, 1080.0, 700.0)),
            parent_content_rect: Some(PortalFrame::new(0.0, 0.0, 1080.0, 700.0)),
            synchronized: true,
            nonactivating_panel: true,
            parent_window_number: 11,
            portal_window_number: 12,
            frontmost_at_probe_point: Some(12),
            frontmost_at_probe_point_is_this_app: true,
            below_portal_at_probe_point: Some(11),
            portal_owns_probe_point: true,
            portal_is_key: false,
            portal_is_main: false,
        };
        assert!(diagnostics.child_attached);
        assert!(!diagnostics.visible);
        assert!(diagnostics.synchronized);
        assert!(diagnostics.nonactivating_panel);
        assert_eq!(diagnostics.frame, diagnostics.parent_content_rect);
        assert_eq!(diagnostics.frontmost_at_probe_point, Some(12));
        assert!(diagnostics.frontmost_at_probe_point_is_this_app);
        assert!(diagnostics.portal_owns_probe_point);
        assert!(!diagnostics.portal_is_key);
        assert!(!diagnostics.portal_is_main);
    }
}
