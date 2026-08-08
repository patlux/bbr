//! Testkit-only zoom benchmark driver (queue item 11, part A).
//!
//! `benchmark_zoom_step` measures the production `Cmd-Plus` path (the exact
//! seam the `ZoomIn` action routes to): the lifecycle change, the
//! persistence write, and the native `pageZoom` application on the active
//! view. The sample commits synchronously — zoom is a main-thread property
//! write, so no async completion is involved — but only from the app's
//! observed zoom state at the armed level, applied to the view, with no
//! pending write (fast-but-wrong guard: a no-op step, an unapplied level,
//! or a stale persistence state rejects in the app and the driver reports
//! failure).
//!
//! The active view/sidebar/focus/mode are unchanged by zooming (no tab,
//! session, profile, space, or session identity is touched); the armed
//! browser-state expectations reject any sample that mutates them.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production `Cmd-Plus` zoom step: the sample starts at
    /// the exact seam the `ZoomIn` key binding routes to (`zoom_in`) and
    /// commits only from the observed zoom state at `level` — the level
    /// actually applied to the active view with no pending persistence
    /// write — plus the unchanged browser state (active view, sidebar,
    /// focus domain, command mode).
    pub fn benchmark_zoom_step(
        &mut self,
        operation_id: BenchmarkOperationId,
        level: u16,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::ZoomStep)
            .is_err()
        {
            return false;
        }
        // Arm the zoom expectation first: a fresh arm clears any prior
        // observation, so a stale level can never satisfy the sample.
        self.performance.expect_zoom(level);
        // Production Cmd-Plus entry: the exact seam the `ZoomIn` action
        // routes to (`window_seam` → `zoom_in`). The mutation records the
        // observed zoom state (level/applied/pending) on the way out.
        self.zoom_in(cx);
        let active_view = self.controller.snapshot().active_view;
        let chrome_focused = self.chrome_focused(window);
        let mode = self.command_bar_mode.into();
        // Fast-but-wrong guard: the same active view stays selected, the
        // sidebar stays visible, and the focus domain + command mode are
        // unchanged. Refresh the probe from the authoritative post-mutation
        // state and commit synchronously (the benchmark harness has no
        // display link; the semantic check inside `commit_current_presentation`
        // still rejects a wrong zoom state).
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance.commit_current_presentation();
        if matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::ZoomStep)
        ) {
            // Truthful rejection: the observed state did not carry the armed
            // level (no-op step, unapplied level, or pending write).
            self.performance.cancel();
            return false;
        }
        true
    }
}
