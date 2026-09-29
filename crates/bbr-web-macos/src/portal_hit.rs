//! Hit-region Portal mode (blueprint D08, DESIGN §13).
//!
//! By default the portal panel captures every pointer event over the
//! browser window while it is shown — right for modal sheets, menus and
//! bars that dismiss on an outside press. A transient surface that must not
//! block the page (a toast, a zoom readout) instead declares **capture
//! regions**: the panel then lets the pointer through everywhere else.
//!
//! AppKit routes a click to the topmost window under the pointer unless that
//! window `ignoresMouseEvents`. A public **local** event monitor observes
//! every pointer move the app receives (from the parent WKWebViews while the
//! panel ignores events, from the panel while it does not) and flips
//! `ignoresMouseEvents` so the panel accepts events exactly while the
//! pointer is inside a region. Moves always precede the click at the same
//! position, so the flag is already right when a press arrives; setting the
//! regions also applies the current pointer position at once. The monitor
//! only observes and always returns the event untouched.

use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSEvent, NSEventMask, NSWindow};

use crate::portal::NativePortal;

/// One capture region in portal-window points (top-left origin, the GPUI
/// coordinate space of both the portal and its parent content view).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalHitRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl PortalHitRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn contains(&self, point: (f64, f64)) -> bool {
        point.0 >= self.x
            && point.0 < self.x + self.width
            && point.1 >= self.y
            && point.1 < self.y + self.height
    }
}

/// The pure routing decision: the portal captures the pointer at `point`
/// when it captures everything (`None`) or when a region contains it.
pub fn portal_captures(regions: Option<&[PortalHitRect]>, point: (f64, f64)) -> bool {
    regions.is_none_or(|regions| regions.iter().any(|rect| rect.contains(point)))
}

/// The monitor and its shared region state (owned by the portal bridge).
pub(crate) struct PortalHitMonitor {
    regions: Rc<RefCell<Option<Vec<PortalHitRect>>>>,
    monitor: Option<Retained<AnyObject>>,
    _block: block2::RcBlock<dyn Fn(NonNull<NSEvent>) -> *mut NSEvent>,
}

impl PortalHitMonitor {
    fn install(portal: Retained<NSWindow>) -> Self {
        let regions: Rc<RefCell<Option<Vec<PortalHitRect>>>> = Rc::new(RefCell::new(None));
        let shared = regions.clone();
        let block: block2::RcBlock<dyn Fn(NonNull<NSEvent>) -> *mut NSEvent> =
            block2::RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
                if let Some(mtm) = MainThreadMarker::new() {
                    apply_pointer(mtm, &portal, shared.borrow().as_deref());
                }
                event.as_ptr()
            });
        let mask = NSEventMask::MouseMoved
            | NSEventMask::LeftMouseDragged
            | NSEventMask::RightMouseDragged
            | NSEventMask::MouseEntered
            | NSEventMask::MouseExited;
        // SAFETY: main thread (the bridge's entry points require it); the
        // handler returns the original event (pure observation). AppKit
        // copies the block for the monitor's lifetime; it is retained here
        // as well until the monitor is removed.
        let monitor =
            unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &block) };
        Self {
            regions,
            monitor,
            _block: block,
        }
    }

    fn remove(&mut self) {
        if let Some(monitor) = self.monitor.take() {
            // SAFETY: main thread; the monitor was installed by
            // `addLocalMonitor…` and is removed exactly once.
            unsafe { NSEvent::removeMonitor(&monitor) };
        }
    }
}

impl Drop for PortalHitMonitor {
    fn drop(&mut self) {
        if MainThreadMarker::new().is_some() {
            self.remove();
        }
    }
}

/// The pointer in portal-window points (top-left origin).
fn pointer_in(portal: &NSWindow, mtm: MainThreadMarker) -> (f64, f64) {
    let screen = NSEvent::mouseLocation();
    let frame = portal.frame();
    let _ = mtm;
    (
        screen.x - frame.origin.x,
        frame.size.height - (screen.y - frame.origin.y),
    )
}

fn apply_pointer(mtm: MainThreadMarker, portal: &NSWindow, regions: Option<&[PortalHitRect]>) {
    let captures = portal_captures(regions, pointer_in(portal, mtm));
    if portal.ignoresMouseEvents() == captures {
        portal.setIgnoresMouseEvents(!captures);
    }
}

impl NativePortal {
    /// Declare the capture regions: `None` captures every pointer event
    /// (modal presentations), `Some(regions)` lets the pointer through
    /// outside them. Applies the current pointer position immediately.
    pub fn set_hit_regions(&mut self, mtm: MainThreadMarker, regions: Option<Vec<PortalHitRect>>) {
        let Some(portal) = self.portal_window.clone() else {
            return;
        };
        if regions.is_some() && self.hit.is_none() {
            self.hit = Some(PortalHitMonitor::install(portal.clone()));
        }
        if let Some(hit) = self.hit.as_ref() {
            *hit.regions.borrow_mut() = regions.clone();
        }
        apply_pointer(mtm, &portal, regions.as_deref());
    }

    /// The declared capture regions (`None`: the panel captures everything).
    pub fn hit_regions(&self) -> Option<Vec<PortalHitRect>> {
        self.hit
            .as_ref()
            .and_then(|hit| hit.regions.borrow().clone())
    }

    /// Run the monitor's routing decision for a pointer at `point`
    /// (portal-window points) and apply it — the exact path a pointer move
    /// takes, addressable without moving the user's cursor (testkit).
    /// Returns whether the panel now captures the pointer.
    pub fn route_pointer_at(&mut self, mtm: MainThreadMarker, point: (f64, f64)) -> bool {
        let _ = mtm;
        let Some(portal) = self.portal_window.as_deref() else {
            return false;
        };
        let regions = self.hit_regions();
        let captures = portal_captures(regions.as_deref(), point);
        portal.setIgnoresMouseEvents(!captures);
        captures
    }

    /// Whether the panel currently lets pointer events through
    /// (`ignoresMouseEvents`, measured from the window).
    pub fn ignores_mouse_events(&self) -> bool {
        self.portal_window
            .as_deref()
            .is_some_and(|portal| portal.ignoresMouseEvents())
    }

    /// Remove the monitor (teardown).
    pub(crate) fn remove_hit_monitor(&mut self) {
        if let Some(mut hit) = self.hit.take() {
            hit.remove();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_everything_without_regions_and_only_inside_regions_otherwise() {
        assert!(portal_captures(None, (5.0, 5.0)));
        let toast = [PortalHitRect::new(100.0, 600.0, 240.0, 40.0)];
        assert!(portal_captures(Some(&toast), (100.0, 600.0)));
        assert!(portal_captures(Some(&toast), (339.0, 639.0)));
        assert!(
            !portal_captures(Some(&toast), (340.0, 620.0)),
            "right edge is outside"
        );
        assert!(!portal_captures(Some(&toast), (50.0, 50.0)));
        assert!(
            !portal_captures(Some(&[]), (100.0, 600.0)),
            "no region: pass through"
        );
    }
}
