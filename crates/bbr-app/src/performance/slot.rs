//! Observed native slot geometry (testkit-only).
//!
//! The last engine-received native content rect is the geometry the
//! synchronous presentation commits carry; this module owns its observation
//! so `performance.rs` stays under the ~500-line budget.

use std::rc::Rc;

use bbr_web::WebRect;

use super::PerformanceProbe;

impl PerformanceProbe {
    /// Observe the rect the native slot actually handed to the engine on the
    /// last paint (the WKWebView frame), for the semantic snapshot.
    pub fn observe_slot_rect(&self, rect: WebRect) {
        self.inner.borrow_mut().content_rect = Some(rect);
    }

    /// The last engine-received native content rect, as `[x, y, w, h]`.
    pub fn content_rect(&self) -> Option<[f32; 4]> {
        self.inner
            .borrow()
            .content_rect
            .map(|r| [r.x, r.y, r.width, r.height])
    }

    /// A callback that observes every slot rect (testkit-only geometry
    /// observation, independent of any pending measurement).
    pub fn slot_geometry_callback(&self) -> Rc<dyn Fn(WebRect)> {
        let probe = self.clone();
        Rc::new(move |rect| probe.observe_slot_rect(rect))
    }
}
