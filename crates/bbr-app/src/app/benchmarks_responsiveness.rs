//! Testkit-only renderer-responsiveness benchmark drivers.
//!
//! These are production-path, render-inclusive arms. They never synthesize a
//! successful state: the source must already be unresponsive; close/select
//! must reach the exact neighbor/session/focus result; an idle probe tick must
//! leave all observable state unchanged. Performance execution is currently
//! deferred, but the typed scenarios and guards remain compiled and tested.

use bbr_browser::RendererAvailability;
use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use bbr_web::WebViewId;
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    #[allow(clippy::too_many_arguments)]
    pub fn benchmark_unresponsive_chrome(
        &mut self,
        operation_id: BenchmarkOperationId,
        source: WebViewId,
        neighbor_index: usize,
        expected_neighbor: WebViewId,
        expected_sessions: usize,
        close: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let before = self.controller.read().snapshot();
        if before.active_view != Some(source)
            || self.controller.read().renderer_state(source) != RendererAvailability::Unresponsive
            || before.engine_sessions != expected_sessions + usize::from(close)
            || before.tabs.get(neighbor_index).and_then(|tab| tab.web_view)
                != Some(expected_neighbor)
            || self
                .performance
                .begin(operation_id, BenchmarkKind::UnresponsiveChrome)
                .is_err()
        {
            return false;
        }

        let chrome_before = self.chrome_focused(window);
        let changed = if close {
            self.close_active_tab(window, cx).is_some()
        } else {
            self.select_tab(neighbor_index, cx).is_some()
        };
        let after = self.controller.read().snapshot();
        if !changed
            || after.active_view != Some(expected_neighbor)
            || after.engine_sessions != expected_sessions
            || self.chrome_focused(window) != chrome_before
        {
            self.performance.cancel();
            return false;
        }
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            after.active_view,
            self.sidebar_visible,
            chrome_before,
            mode,
        );
        self.performance
            .expect_state(after.active_view, self.sidebar_visible, chrome_before, mode);
        self.commit_benchmark_sync(window);
        true
    }

    pub fn benchmark_probe_tick(
        &mut self,
        operation_id: BenchmarkOperationId,
        view: WebViewId,
        expected_sessions: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let before = self.controller.read().snapshot();
        let state_before = self.controller.read().renderer_state(view);
        let focus_before = self.chrome_focused(window);
        if before.active_view != Some(view)
            || before.engine_sessions != expected_sessions
            || self
                .performance
                .begin(operation_id, BenchmarkKind::ProbeTick)
                .is_err()
        {
            return false;
        }
        self.pump_responsiveness_cadence(window, cx);
        let after = self.controller.read().snapshot();
        if after.active_view != Some(view)
            || after.engine_sessions != expected_sessions
            || self.controller.read().renderer_state(view) != state_before
            || self.chrome_focused(window) != focus_before
        {
            self.performance.cancel();
            return false;
        }
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            after.active_view,
            self.sidebar_visible,
            focus_before,
            mode,
        );
        self.performance
            .expect_state(after.active_view, self.sidebar_visible, focus_before, mode);
        self.commit_benchmark_sync(window);
        true
    }
}
