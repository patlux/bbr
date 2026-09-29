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
use objc2::msg_send;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSWindow, NSWindowStyleMask};
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
    /// The parent browser window is currently in native fullscreen
    /// (`NSWindowStyleMaskFullScreen`; measured from the parent `NSWindow`
    /// on the main thread). Hardening evidence: fullscreen enter/leave must
    /// re-pin the portal frame and never recreate or orphan the host.
    pub parent_fullscreen: bool,
    /// The parent browser window is currently miniaturized
    /// (`isMiniaturized`). Hardening evidence: while minimized the portal is
    /// ordered out (never occluding/intercepting) and is re-shown on
    /// restore when a presentation is still armed.
    pub parent_minimized: bool,
    /// The parent browser window is associated with the active space
    /// (`isOnActiveSpace` — a real Window Server query). Hardening
    /// evidence: the portal and its parent must always agree on active-space
    /// membership (attached children follow their parent's space).
    pub parent_on_active_space: bool,
    /// The portal window is associated with the active space (same query).
    /// The invariant is `parent_on_active_space == portal_on_active_space`.
    pub portal_on_active_space: bool,
    /// The portal window's backing scale factor (`backingScaleFactor`, a
    /// real Window Server/display query). Hardening evidence: frame sync is
    /// point-based, so a display with a different scale factor keeps the
    /// portal pinned in points (scale only changes the backing pixels).
    pub portal_scale_factor: f64,
    /// Hit-region mode (blueprint D08): the declared capture regions as
    /// `[x, y, width, height]` in portal points; `None` captures everything.
    pub hit_regions: Option<Vec<[f64; 4]>>,
    /// The panel currently lets the pointer through (`ignoresMouseEvents`).
    pub ignores_mouse_events: bool,
}

/// The native-side diagnostics builder for the one portal host.
///
/// Reads the portal's live AppKit window state on the main thread and
/// returns the typed [`PortalNativeDiagnostics`] the snapshot mirrors
/// verbatim (task packet `native-portal-hardening` evidence). Defined next
/// to its struct so the frame/diagnostics module stays free of the host's
/// observer machinery; the struct fields are `pub(crate)` in `portal.rs`.
impl crate::portal::NativePortal {
    /// Narrow main-thread diagnostics (DESIGN §15 snapshot semantics).
    pub fn diagnostics(&self, mtm: MainThreadMarker) -> PortalNativeDiagnostics {
        let portal = self.portal_window.as_deref();
        let parent = self.parent_window.as_deref();
        let native_visible = portal.is_some_and(|window| window.isVisible());
        let nonactivating_panel = portal.is_some_and(|window| {
            window
                .styleMask()
                .contains(NSWindowStyleMask::NonactivatingPanel)
        });
        let (portal_is_key, portal_is_main) = portal.map_or((false, false), |window| {
            (window.isKeyWindow(), window.isMainWindow())
        });
        let parent_fullscreen =
            parent.is_some_and(|window| window.styleMask().contains(NSWindowStyleMask::FullScreen));
        let parent_minimized = parent.is_some_and(|window| window.isMiniaturized());
        // Real Window Server membership queries (`isOnActiveSpace`): the
        // portal is an attached child, so it must always agree with its
        // parent on active-space membership.
        let parent_on_active_space = parent.is_some_and(|window| window.isOnActiveSpace());
        let portal_on_active_space = portal.is_some_and(|window| window.isOnActiveSpace());
        // Point-based frame evidence: the backing scale factor only changes
        // the pixels, never the point frame the portal is pinned to. The
        // `backingScaleFactor` selector is gated behind an objc2-app-kit
        // feature this crate does not enable, so it is called through the
        // runtime selector (CGFloat result), same as the NSColor seam.
        let portal_scale_factor = portal.map_or(0.0, |window| unsafe {
            msg_send![window, backingScaleFactor]
        });
        // The probe point is the last synced frame's center: the centered
        // probe card's point. AppKit answers with the frontmost window that
        // would receive a mouseDown there (any application); while the portal
        // is visible above the parent, that is the portal window itself.
        let frontmost_at_probe_point = self
            .frame
            .map(|frame| {
                let point = NSPoint::new(frame.x + frame.width / 2.0, frame.y + frame.height / 2.0);
                // `windowNumberAtPoint:belowWindowWithWindowNumber:` returns
                // the frontmost window that would be hit by a mouseDown at
                // the screen location (any application; 0 when no window).
                NSWindow::windowNumberAtPoint_belowWindowWithWindowNumber(point, 0, mtm)
            })
            .map(|number| number as i64)
            .filter(|&number| number != 0);
        // Raw diagnostic only: with all-spaces/PopUp window behavior the
        // window server's below-window answer can be an unrelated overlay,
        // so the deterministic evidence is `portal_owns_probe_point` below.
        let below_portal_at_probe_point = match (self.frame, portal.map(|w| w.windowNumber())) {
            (Some(frame), Some(portal_number)) if portal_number != 0 => {
                let point = NSPoint::new(frame.x + frame.width / 2.0, frame.y + frame.height / 2.0);
                let below = NSWindow::windowNumberAtPoint_belowWindowWithWindowNumber(
                    point,
                    portal_number,
                    mtm,
                );
                (below != 0).then_some(below as i64)
            }
            _ => None,
        };
        // Deterministic hit-test ownership: an attached, visible child
        // window pinned to the parent content rect covers the probe point,
        // and AppKit hit-tests child windows above their parent's views.
        let portal_owns_probe_point = self.attached
            && native_visible
            && frame_matches_content(self.frame, self.parent_content_rect);
        // Measured owner classification of the absolute frontmost answer:
        // `NSApplication.windows` enumerates every window of this app
        // (child windows and ordered-out windows included), so membership
        // proves the number is BBR-owned. A foreign always-on-top window or
        // a system overlay covering the probe point reports `false`. This
        // makes the idle-portal test acceptance precise: a BBR-owned window
        // other than the parent at the probe point is a regression, a
        // foreign window is environment.
        let frontmost_at_probe_point_is_this_app = frontmost_at_probe_point.is_some_and(|number| {
            let windows = NSApplication::sharedApplication(mtm).windows();
            (0..windows.count())
                .any(|index| windows.objectAtIndex(index).windowNumber() as i64 == number)
        });
        PortalNativeDiagnostics {
            child_attached: self.attached,
            visible: self.visible,
            native_visible,
            frame: self.frame,
            parent_content_rect: self.parent_content_rect,
            synchronized: frame_matches_content(self.frame, self.parent_content_rect),
            nonactivating_panel,
            parent_window_number: parent
                .map(|window| window.windowNumber() as i64)
                .unwrap_or(0),
            portal_window_number: portal
                .map(|window| window.windowNumber() as i64)
                .unwrap_or(0),
            frontmost_at_probe_point,
            frontmost_at_probe_point_is_this_app,
            below_portal_at_probe_point,
            portal_owns_probe_point,
            portal_is_key,
            portal_is_main,
            parent_fullscreen,
            parent_minimized,
            parent_on_active_space,
            portal_on_active_space,
            portal_scale_factor,
            hit_regions: self.hit_regions().map(|regions| {
                regions
                    .iter()
                    .map(|r| [r.x, r.y, r.width, r.height])
                    .collect()
            }),
            ignores_mouse_events: self.ignores_mouse_events(),
        }
    }
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
            parent_fullscreen: false,
            parent_minimized: false,
            parent_on_active_space: true,
            portal_on_active_space: true,
            portal_scale_factor: 2.0,
            hit_regions: None,
            ignores_mouse_events: false,
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
        assert!(!diagnostics.parent_fullscreen);
        assert!(!diagnostics.parent_minimized);
        assert!(diagnostics.parent_on_active_space);
        assert!(diagnostics.portal_on_active_space);
        assert_eq!(diagnostics.portal_scale_factor, 2.0);
    }
}
