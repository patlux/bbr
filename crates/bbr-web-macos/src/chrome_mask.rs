//! The four inverse card corners are holes in the GPUI layer, not translucent
//! paint over square internal pages. Native WebKit already clips the same arc.
//! Public paths only; no page pixels, view replacement or private renderer API.

use objc2::rc::Retained;
use objc2_app_kit::{NSColor, NSView};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGMutablePath;
use objc2_quartz_core::{kCAFillRuleEvenOdd, CALayer, CAShapeLayer, CATransaction};

pub(super) struct CardMask {
    root: Retained<CALayer>,
    mask: Retained<CAShapeLayer>,
    corners: Retained<CAShapeLayer>,
    last: Option<([f32; 4], f32, [f32; 4], CGRect)>,
}

impl CardMask {
    pub(super) fn new(root: &NSView, material: &NSView) -> Option<Self> {
        let root = root.layer()?;
        // Do not displace a mask owned by another subsystem.
        if root.mask().is_some() {
            return None;
        }
        material.setWantsLayer(true);
        let backdrop = material.layer()?;
        let mask = CAShapeLayer::new();
        let corners = CAShapeLayer::new();
        // SAFETY: immutable public AppKit/Core Animation constants.
        unsafe {
            mask.setFillRule(kCAFillRuleEvenOdd);
            corners.setFillRule(kCAFillRuleEvenOdd);
        }
        backdrop.addSublayer(&corners);
        // SAFETY: the mask is exclusively owned here and is not a hierarchy ancestor.
        unsafe { root.setMask(Some(&mask)) };
        Some(Self {
            root,
            mask,
            corners,
            last: None,
        })
    }

    #[cfg(feature = "testkit")]
    pub(super) fn clips_only_card_corners(&self) -> bool {
        use objc2_core_graphics::CGPath;
        let Some((card, radius, _, bounds)) = self.last else {
            return false;
        };
        if self.root.mask().as_deref() != Some(self.mask.as_ref()) {
            return false;
        }
        let Some(path) = self.mask.path() else {
            return false;
        };
        let [x, y, width, height] = card.map(f64::from);
        let y = if self.root.isGeometryFlipped() {
            y
        } else {
            bounds.size.height - y - height
        };
        // SAFETY: read-only containment on the actual installed native path.
        let contains = |x, y| unsafe {
            CGPath::contains_point(Some(&path), std::ptr::null(), CGPoint::new(x, y), true)
        };
        let center = contains(x + width / 2.0, y + height / 2.0);
        let rail = x <= 1.0 || contains(x / 2.0, bounds.size.height / 2.0);
        let corners = radius <= 0.0
            || [
                (x + 0.25, y + 0.25),
                (x + width - 0.25, y + 0.25),
                (x + 0.25, y + height - 0.25),
                (x + width - 0.25, y + height - 0.25),
            ]
            .into_iter()
            .all(|(x, y)| !contains(x, y));
        center && rail && corners
    }

    pub(super) fn set_enabled(&self, enabled: bool) {
        // SAFETY: exclusive ownership; no ancestor can become its own mask.
        unsafe { self.root.setMask(enabled.then_some(self.mask.as_ref())) };
        self.corners.setHidden(!enabled);
    }

    pub(super) fn update(&mut self, card: [f32; 4], radius: f32, tint: [f32; 4]) {
        let bounds = self.root.bounds();
        if self.last == Some((card, radius, tint, bounds)) {
            return;
        }
        let [x, y, w, h] = card.map(f64::from);
        let y = if self.root.isGeometryFlipped() {
            y
        } else {
            bounds.size.height - y - h
        };
        let rect = CGRect::new(CGPoint::new(x, y), CGSize::new(w, h));
        let r = f64::from(radius).min(w / 2.0).min(h / 2.0).max(0.0);
        let mask_path = CGMutablePath::new();
        let corner_path = CGMutablePath::new();
        // SAFETY: live locally owned mutable paths, no transform pointer, finite
        // production layout coordinates. Even-odd fill removes only the wedges.
        unsafe {
            CGMutablePath::add_rect(Some(&mask_path), std::ptr::null(), bounds);
            if r > 0.0 {
                for path in [&mask_path, &corner_path] {
                    CGMutablePath::add_rect(Some(path), std::ptr::null(), rect);
                    CGMutablePath::add_rounded_rect(Some(path), std::ptr::null(), rect, r, r);
                }
            }
        }
        let [red, green, blue, alpha] = tint.map(f64::from);
        let color = NSColor::colorWithSRGBRed_green_blue_alpha(red, green, blue, alpha);
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        self.mask.setFrame(bounds);
        self.mask.setPath(Some(&mask_path));
        self.corners.setFrame(bounds);
        self.corners
            .setGeometryFlipped(self.root.isGeometryFlipped());
        self.corners.setPath(Some(&corner_path));
        self.corners.setFillColor(Some(&color.CGColor()));
        CATransaction::commit();
        self.last = Some((card, radius, tint, bounds));
    }
}

impl Drop for CardMask {
    fn drop(&mut self) {
        // SAFETY: remove only the mask installed by this owner.
        if self.root.mask().as_deref() == Some(self.mask.as_ref()) {
            unsafe { self.root.setMask(None) };
        }
        self.corners.removeFromSuperlayer();
    }
}
