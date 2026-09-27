//! Content-card clipping (DESIGN.md §2): the native slot is clipped to the
//! card's corner radius by its own backing layer. Public AppKit / Core
//! Animation only (`wantsLayer`, `layer`, `cornerRadius`, `masksToBounds`);
//! the web content is never sampled, textured, or re-rendered.

use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::NSView;

/// Clip `view` to `radius` points through its backing layer; `0` removes
/// the clip (full-bleed). Idempotent: an unchanged radius makes no call.
pub(crate) fn clip_to_radius(view: &NSView, radius: f64) {
    if corner_radius(view) == Some(radius) {
        return;
    }
    view.setWantsLayer(true);
    // SAFETY: main thread (the engine asserts it); `layer`,
    // `setCornerRadius:` and `setMasksToBounds:` are public CALayer API on
    // the view's own backing layer, called with their documented types.
    unsafe {
        let layer: Option<Retained<AnyObject>> = msg_send![view, layer];
        if let Some(layer) = layer {
            let _: () = msg_send![&*layer, setCornerRadius: radius];
            let _: () = msg_send![&*layer, setMasksToBounds: radius > 0.0];
        }
    }
}

/// The backing layer's corner radius, or `None` when the view has no layer
/// yet (the typed evidence for the native clip).
pub(crate) fn corner_radius(view: &NSView) -> Option<f64> {
    // SAFETY: main thread; read-only public CALayer API.
    unsafe {
        let layer: Option<Retained<AnyObject>> = msg_send![view, layer];
        layer.map(|layer| msg_send![&*layer, cornerRadius])
    }
}
