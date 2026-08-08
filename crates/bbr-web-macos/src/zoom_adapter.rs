//! # Native WebKit page-zoom adapter (macOS)
//!
//! Applies the portable per-site zoom level ([`bbr_web::site_settings::ZoomPercent`],
//! task packet `site-settings-zoom-contract`) to a real `WKWebView` by setting
//! its `pageZoom` property. `pageZoom` is WebKit's page-scale factor: it scales
//! the whole layout (CSS pixels vs. viewport pixels) and survives navigation
//! within the same view, so a per-site level persists across same-site
//! navigations and is restored whenever the view is (re)created or the site
//! changes. `textZoom` is deliberately left untouched — BBR has no separate
//! text-only zoom, so text scales with the page.
//!
//! The mapping is the documented contract's "later adapter slice":
//! `pageZoom = percent / 100` (100% → 1.0, 125% → 1.25, …, 400% → 4.0).
//! The factor is derived from the integer-percent value, never from raw
//! unchecked floats, so the drift-free contract survives the wire.
//!
//! # Safety
//!
//! The adapter is main-thread-only (like every `bbr-web-macos` entry point);
//! the retained `WKWebView` clone is only dereferenced on the main thread
//! while the owning session is alive. The composition binds a fresh adapter
//! per application and drops it immediately — applying zoom never creates,
//! moves, or reselects a web session.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_web_kit::WKWebView;

use bbr_web::site_settings::ZoomPercent;

/// Map a portable integer-percent zoom level to WebKit's `pageZoom` factor.
///
/// `100%` maps to `1.0`, each step of 25 percentage points adds `0.25`, and
/// the documented bounds map to `0.25` (25%) and `4.0` (400%). The division
/// is exact for every representable level (a `u16` percent over `100.0`),
/// so the ladder never drifts.
pub fn page_zoom_factor(zoom: ZoomPercent) -> f64 {
    f64::from(zoom.as_u16()) / 100.0
}

/// Applies a [`ZoomPercent`] to one live `WKWebView` (`pageZoom`).
///
/// One-shot: the composition creates an adapter per application site change
/// and drops it after the synchronous property set, so no adapter outlives
/// the view or holds state across commands.
pub struct WebKitZoomAdapter {
    web_view: Retained<WKWebView>,
}

impl WebKitZoomAdapter {
    /// Bind the adapter to a retained clone of a live session's web view.
    pub fn new(mtm: MainThreadMarker, web_view: Retained<WKWebView>) -> Self {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "WebKitZoomAdapter constructed off the main thread"
        );
        let _ = mtm;
        Self { web_view }
    }

    /// Apply `zoom` natively: `pageZoom` becomes `percent / 100`.
    ///
    /// Synchronous and deterministic: the property set is the full product
    /// effect, so the caller can observe the applied level immediately after
    /// this call returns.
    pub fn apply(&self, zoom: ZoomPercent) {
        self.assert_main_thread();
        let factor = page_zoom_factor(zoom);
        // SAFETY: main thread; `self.web_view` is retained and alive for the
        // duration of this call; `pageZoom` accepts any finite factor.
        unsafe {
            self.web_view.setPageZoom(factor);
        }
    }

    fn assert_main_thread(&self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "WebKitZoomAdapter used off the main thread"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_maps_to_unit_factor() {
        assert_eq!(page_zoom_factor(ZoomPercent::default()), 1.0);
        assert_eq!(page_zoom_factor(ZoomPercent::new(100).unwrap()), 1.0);
    }

    #[test]
    fn each_step_adds_exactly_one_quarter() {
        // The full ladder is exact: 25..=400 percent maps to 0.25..=4.0 in
        // steps of 0.25, with no floating-point drift anywhere on the range.
        for percent in (ZoomPercent::MIN..=ZoomPercent::MAX).step_by(ZoomPercent::STEP as usize) {
            let zoom = ZoomPercent::new(percent).unwrap();
            let expected = f64::from(percent) / 100.0;
            assert_eq!(page_zoom_factor(zoom), expected);
            assert!((page_zoom_factor(zoom) * 4.0).fract().abs() < f64::EPSILON);
        }
    }

    #[test]
    fn bounds_map_to_quarter_and_four() {
        assert_eq!(page_zoom_factor(ZoomPercent::new(25).unwrap()), 0.25);
        assert_eq!(page_zoom_factor(ZoomPercent::new(400).unwrap()), 4.0);
    }

    #[test]
    fn arbitrary_levels_are_exact() {
        for percent in [137, 225, 333, 399] {
            let zoom = ZoomPercent::new(percent).unwrap();
            assert_eq!(page_zoom_factor(zoom), f64::from(percent) / 100.0);
        }
    }
}
