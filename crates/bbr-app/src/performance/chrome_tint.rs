//! Chrome-tint presentation identity for the presentation probe.
//!
//! The `chrome_transparency_change` benchmark commits only from a browser-
//! window render that painted the armed tint coverage with the native
//! material in the armed state. The render records what it painted
//! (`BbrApp::chrome_background`); arming clears the previous observation,
//! so a frame still showing the old coverage can never satisfy a sample.
//! Same child-module pattern as `super::zoom`.

use super::{Pending, PerformanceProbe, State};

/// The painted tint the pending sample must commit from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ChromeTintExpectation {
    pub coverage: f32,
    pub glass: bool,
}

/// What the browser-window render painted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ChromeTintObservation {
    pub coverage: f32,
    pub glass: bool,
}

impl PerformanceProbe {
    /// Arm the painted tint the pending sample must commit from.
    pub fn expect_chrome_tint(&self, coverage: f32, glass: bool) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.chrome_tint = Some(ChromeTintExpectation { coverage, glass });
            state.latest_chrome_tint = None;
        }
    }

    /// Record the tint a browser-window render painted (the milestone).
    pub fn observe_chrome_tint(&self, coverage: f32, glass: bool) {
        self.inner.borrow_mut().latest_chrome_tint =
            Some(ChromeTintObservation { coverage, glass });
    }
}

/// True when no tint is armed, or the latest render painted exactly the
/// armed coverage and material state. Armed without a render: never.
pub(super) fn chrome_tint_matches(pending: &Pending, state: &State) -> bool {
    let Some(expected) = pending.chrome_tint else {
        return true;
    };
    state.latest_chrome_tint.is_some_and(|observed| {
        observed.glass == expected.glass && (observed.coverage - expected.coverage).abs() < 1e-4
    })
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};

    use super::super::PerformanceProbe;

    #[test]
    fn only_a_render_of_the_armed_tint_commits() {
        let probe = PerformanceProbe::new(Instant::now(), None);
        probe.cancel();
        let committed = std::rc::Rc::new(std::cell::Cell::new(0));
        let sink = committed.clone();
        probe.set_sink(Box::new(move |_| sink.set(sink.get() + 1)));
        probe.observe_render_state(
            Some(bbr_web::WebViewId(1)),
            true,
            false,
            bbr_testkit::protocol::CommandBarMode::Inactive,
        );
        probe
            .begin(
                BenchmarkOperationId(7),
                BenchmarkKind::ChromeTransparencyChange,
            )
            .unwrap();
        probe.expect_browser_state(Some(bbr_web::WebViewId(1)), true);
        probe.expect_chrome_tint(0.30, true);
        let rect = bbr_web::WebRect::new(300.0, 0.0, 700.0, 900.0);
        // A frame that still paints the previous coverage is rejected.
        probe.observe_chrome_tint(0.55, true);
        (probe.presentation_callback())(rect);
        assert_eq!(committed.get(), 0);
        // The right coverage with glass off is rejected too.
        probe.observe_chrome_tint(0.30, false);
        (probe.presentation_callback())(rect);
        assert_eq!(committed.get(), 0);
        probe.observe_chrome_tint(0.30, true);
        (probe.presentation_callback())(rect);
        assert_eq!(committed.get(), 1);
    }
}
