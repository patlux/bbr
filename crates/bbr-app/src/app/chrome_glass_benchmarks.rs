//! Testkit-only Settings › Appearance transparency benchmark.
//!
//! `benchmark_chrome_transparency` starts at the exact event the slider
//! sends (`SettingsEvent::Slide`) and commits only from the browser-window
//! render that painted the armed tint coverage with the native material in
//! the armed state. The expectation comes from the percent and the theme
//! mode, not from the preference the mutation writes, so a frame that still
//! paints the old coverage (or no frame at all) never satisfies the sample.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use bbr_ui::surfaces::settings::SettingsEvent;
use gpui::{Context, Window};

use super::settings_transparency::TRANSPARENCY_ROW;
use super::BbrApp;

impl BbrApp {
    /// Refuses (false) when glass cannot show at all, or when the percent is
    /// already set: an unchanged value presents nothing to measure.
    pub fn benchmark_chrome_transparency(
        &mut self,
        operation_id: BenchmarkOperationId,
        percent: u8,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let percent = percent.min(100);
        let system = self.effective_accessibility();
        #[cfg(target_os = "macos")]
        let installed = self.chrome_material.borrow().is_some();
        #[cfg(not(target_os = "macos"))]
        let installed = false;
        if !installed
            || system.reduce_transparency
            || system.increase_contrast
            || self.preferences.read().chrome_transparency == percent
        {
            return false;
        }
        if self
            .performance
            .begin(operation_id, BenchmarkKind::ChromeTransparencyChange)
            .is_err()
        {
            return false;
        }
        let (coverage, glass) = if percent == 0 {
            (self.active_chrome_tint().a, false)
        } else {
            (
                super::chrome_glass::tint_coverage(self.theme().mode, percent),
                true,
            )
        };
        let active_view = self.controller.read().snapshot().active_view;
        self.performance
            .expect_browser_state(active_view, self.sidebar_visible);
        self.performance.expect_chrome_tint(coverage, glass);
        self.on_settings_event(
            SettingsEvent::Slide(TRANSPARENCY_ROW.into(), percent),
            Some(window),
            cx,
        );
        true
    }
}
