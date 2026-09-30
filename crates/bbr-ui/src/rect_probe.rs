//! Where controls drew, by semantic id (reality-gaps packet, item C).
//!
//! The real-input harness aims real AppKit clicks at controls; it needs the
//! rect each control was drawn at in the latest frame. A probe is an
//! invisible, absolutely positioned child that fills its control and, while
//! the frame is laid out, records the control's bounds under the control's
//! semantic id (`bbr-tab-close-<n>`, `bbr-footer-downloads`…) in the
//! [`ElementRects`] global — only when that global is installed (the test
//! build). Without the `rect-probes` feature no probe element exists at all,
//! so release and Gallery renders are unchanged.

use std::cell::RefCell;
use std::collections::BTreeMap;

use gpui::{Global, SharedString};

/// Every probed control's rect in the latest frame of each window: window
/// id → semantic id → `[x, y, width, height]` (points, top-left origin).
#[derive(Debug, Default)]
pub struct ElementRects {
    rects: RefCell<BTreeMap<u64, BTreeMap<String, [f32; 4]>>>,
}

impl Global for ElementRects {}

impl ElementRects {
    /// Forget a window's rects before it draws a new frame.
    pub fn begin_frame(&self, window: u64) {
        self.rects.borrow_mut().remove(&window);
    }

    /// Record one control's rect.
    pub fn record(&self, window: u64, id: &str, rect: [f32; 4]) {
        self.rects
            .borrow_mut()
            .entry(window)
            .or_default()
            .insert(id.to_string(), rect);
    }

    /// A window's rects, sorted by id.
    pub fn for_window(&self, window: u64) -> Vec<(String, [f32; 4])> {
        self.rects
            .borrow()
            .get(&window)
            .map(|rects| rects.iter().map(|(id, rect)| (id.clone(), *rect)).collect())
            .unwrap_or_default()
    }
}

/// The probe child for the control `id` (`None` without the `rect-probes`
/// feature); attach it with `.children(rect_probe(id))`.
#[cfg(feature = "rect-probes")]
pub fn rect_probe(id: impl Into<SharedString>) -> Option<gpui::AnyElement> {
    use gpui::{canvas, IntoElement, Styled};
    let id = id.into();
    Some(
        canvas(
            move |bounds, window, cx| {
                if let Some(rects) = cx.try_global::<ElementRects>() {
                    rects.record(
                        window.window_handle().window_id().as_u64(),
                        &id,
                        [
                            f32::from(bounds.origin.x),
                            f32::from(bounds.origin.y),
                            f32::from(bounds.size.width),
                            f32::from(bounds.size.height),
                        ],
                    );
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element(),
    )
}

/// Without the `rect-probes` feature there is no probe.
#[cfg(not(feature = "rect-probes"))]
pub fn rect_probe(_id: impl Into<SharedString>) -> Option<gpui::AnyElement> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_frame_forgets_only_its_own_window() {
        let rects = ElementRects::default();
        rects.record(1, "bbr-footer-plus", [1.0, 2.0, 3.0, 4.0]);
        rects.record(2, "bbr-footer-plus", [5.0, 6.0, 7.0, 8.0]);
        rects.begin_frame(1);
        assert!(rects.for_window(1).is_empty());
        assert_eq!(
            rects.for_window(2),
            [("bbr-footer-plus".to_string(), [5.0, 6.0, 7.0, 8.0])]
        );
    }
}
