//! Testkit-only benchmark drivers.
//!
//! Tab-organization drivers (pin, reorder, same-profile move, cross-profile
//! move) plus the interaction benchmarks the performance harness drives
//! (selection, reopen, sidebar resize/restore, page-ready milestones).
//!
//! Each driver begins the in-process presentation probe, runs the production
//! mutation path, and commits synchronously from the authoritative
//! post-mutation snapshot — exactly like the other interaction benchmarks —
//! so a fast-but-wrong sample (wrong view, wrong focus, wrong mode) is
//! rejected in the app before it leaves.

use bbr_browser::{ControllerEvent, SpaceId};
use bbr_web::WebViewId;
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production pin toggle of the active tab: the pinned
    /// region boundary moves, the session and native view stay. Commits
    /// synchronously from the authoritative post-mutation snapshot.
    pub fn benchmark_pin_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, bbr_testkit::protocol::BenchmarkKind::TabPin)
            .is_err()
        {
            return false;
        }
        let outcome = self.pin_active_tab(cx);
        if outcome.is_none() {
            self.performance.cancel();
            return false;
        }
        self.commit_organize_benchmark(window);
        true
    }

    /// Benchmark the production reorder of the active tab by `delta` rows
    /// within its region (the keyboard step the `Cmd-Shift-↑/↓` path drives).
    pub fn benchmark_reorder_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        delta: isize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::TabReorder,
            )
            .is_err()
        {
            return false;
        }
        let outcome = self.reorder_active_tab(delta, cx);
        if outcome.is_none() {
            self.performance.cancel();
            return false;
        }
        self.commit_organize_benchmark(window);
        true
    }

    /// Benchmark the production move of the active tab into `to_space`:
    /// same-profile moves keep the live session (warm class), cross-profile
    /// moves recreate it once in the destination store (session-recreation
    /// class, its own budget). The sample commits with the source space's
    /// repaired selection presented.
    pub fn benchmark_move_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        to_space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let snapshot = self.controller.snapshot();
        if snapshot.active_space == to_space {
            return false;
        }
        let same_profile = snapshot
            .spaces
            .iter()
            .find(|s| s.id == to_space)
            .is_some_and(|s| s.profile == snapshot.active_profile);
        let kind = if same_profile {
            bbr_testkit::protocol::BenchmarkKind::TabMoveSameProfile
        } else {
            bbr_testkit::protocol::BenchmarkKind::TabMoveCrossProfile
        };
        if self.performance.begin(operation_id, kind).is_err() {
            return false;
        }
        let outcome = self.move_active_tab_to_space(to_space, window, cx);
        if outcome.is_none() {
            self.performance.cancel();
            return false;
        }
        self.commit_organize_benchmark(window);
        true
    }

    /// Commit an organize benchmark synchronously from the authoritative
    /// post-mutation snapshot (active view, focus, mode), like the other
    /// interaction benchmarks.
    pub(crate) fn commit_organize_benchmark(&self, window: &Window) {
        let active_view = self.controller.snapshot().active_view;
        let chrome_focused = self.chrome_focused(window);
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_browser_state(active_view, self.sidebar_visible);
        self.performance.commit_current_presentation();
    }

    pub fn benchmark_select_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::WarmTabSwitch,
            )
            .is_err()
        {
            return None;
        }
        let outcome = self.select_tab(index, cx);
        if outcome.is_some() {
            // Mutations only schedule a render; the frame that runs
            // `observe_render_state` with the new active view has not happened
            // yet when this synchronous path commits. Refresh the probe with
            // the authoritative post-mutation controller snapshot so the
            // presentation commit sees the real state and cannot silently drop
            // a sample when the selection changes views.
            let active_view = self.controller.snapshot().active_view;
            let chrome_focused = self.chrome_focused(window);
            let mode = self.command_bar_mode.into();
            self.performance.observe_render_state(
                active_view,
                self.sidebar_visible,
                chrome_focused,
                mode,
            );
            self.performance
                .expect_browser_state(active_view, self.sidebar_visible);
            self.performance.commit_current_presentation();
        } else {
            self.performance.cancel();
        }
        outcome
    }

    /// Benchmark the production pointer-selection path (row mouse-down).
    pub fn benchmark_click_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::TabSelectPointer,
            )
            .is_err()
        {
            return false;
        }
        // The production pointer path, exactly as the row mouse-down listener
        // drives it: chrome focus domain preserved while the controller
        // selects. The benchmark commits synchronously from the authoritative
        // post-mutation snapshot (same rationale as `benchmark_select_tab`).
        self.select_row_via_pointer(index, window, cx);
        let active_view = self.controller.snapshot().active_view;
        let chrome_focused = self.chrome_focused(window);
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_browser_state(active_view, self.sidebar_visible);
        self.performance.commit_current_presentation();
        true
    }

    /// Benchmark the reopen path (production `reopen_tab`).
    pub fn benchmark_reopen_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::TabReopen,
            )
            .is_err()
        {
            return false;
        }
        // Production reopen path (contract: preserved identity, fresh session,
        // focus domain untouched). The benchmark arms and commits synchronously
        // from the authoritative post-mutation snapshot, like the other
        // interaction benchmarks.
        let outcome = self.reopen_tab(window, cx);
        if outcome.is_none() {
            self.performance.cancel();
            return false;
        }
        let active_view = self.controller.snapshot().active_view;
        let chrome_focused = self.chrome_focused(window);
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_browser_state(active_view, self.sidebar_visible);
        self.performance.commit_current_presentation();
        true
    }

    /// Arm the deterministic page-ready milestone for a view/title pair.
    pub fn benchmark_page_ready(
        &self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        expected_view: WebViewId,
        expected_title: impl Into<String>,
    ) -> bool {
        self.performance
            .arm_page_ready(operation_id, expected_view, expected_title);
        true
    }

    /// Begin a benchmark operation of the given kind.
    pub fn begin_benchmark(
        &self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        kind: bbr_testkit::protocol::BenchmarkKind,
    ) -> bool {
        self.performance.begin(operation_id, kind).is_ok()
    }

    /// Cancel the in-flight benchmark operation, if any.
    pub fn cancel_benchmark(&self) {
        self.performance.cancel();
    }

    /// Commit the pending benchmark synchronously from authoritative state.
    pub fn commit_benchmark_sync(&self, window: &Window) {
        // Refresh the probe with the authoritative post-mutation controller
        // state and commit synchronously. The `on_next_frame` path does not
        // reliably fire in the offscreen benchmark harness (startup/page-ready
        // only return through engine callbacks), so a frame-dependent commit
        // silently dropped interaction samples. The semantic state check inside
        // `commit_current_presentation` still rejects fast-but-wrong results.
        let active_view = self.controller.snapshot().active_view;
        let chrome_focused = self.chrome_focused(window);
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        // The stored slot rect is the *pre-mutation* paint when the mutation
        // commits synchronously, so the interaction samples commit from the
        // authoritative geometry (rail width + window bounds) instead: that is
        // exactly the rect the next layout hands the native slot, and the
        // expected-width semantic check rejects wrong geometry.
        let window_width = f32::from(window.bounds().size.width);
        let window_height = f32::from(window.bounds().size.height);
        let sidebar_width = if self.sidebar_visible {
            self.sidebar.effective_width(window_width)
        } else {
            0.0
        };
        self.performance
            .commit_geometry(window_width, sidebar_width, window_height);
    }

    /// Apply a controller mutation result: log errors, broadcast the outcome,
    /// and notify the view. Returns the event when the mutation succeeded.
    /// Benchmark the sidebar toggle through the production path.
    pub fn benchmark_toggle_sidebar(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SidebarToggle,
            )
            .is_err()
        {
            return false;
        }
        // `toggle_sidebar` arms the pending and commits synchronously.
        self.toggle_sidebar(window, cx);
        true
    }

    /// Benchmark the production sidebar resize path: the width mutation the
    /// pointer and keyboard listeners share, measured until the resulting
    /// native geometry commits with the exact expected width. The expected
    /// width is the clamped effective width the rail renders at, so a
    /// fast-but-wrong sample (wrong width or wrong visibility) is rejected.
    pub fn benchmark_resize_sidebar(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        width: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SidebarResize,
            )
            .is_err()
        {
            return false;
        }
        let window_width = f32::from(window.bounds().size.width);
        let expected = crate::sidebar::SidebarGeometry::effective_width_for(width, window_width);
        self.performance
            .expect_browser_state(self.controller.snapshot().active_view, true);
        self.performance.expect_sidebar_width(expected);
        // The production mutation; its synchronous commit observes the armed
        // expectations and rejects a wrong geometry.
        self.resize_sidebar_to(width, window, cx);
        true
    }

    /// Benchmark sidebar restoration: collapse through the production Cmd-B
    /// path (unmeasured setup), then expand through the same path and measure
    /// the presentation that restores the last expanded width.
    pub fn benchmark_restore_sidebar(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        // Unmeasured setup: collapse through the production path with no
        // pending measurement, so the toggle commits nothing.
        if self.sidebar_visible {
            self.toggle_sidebar(window, cx);
        }
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SidebarRestore,
            )
            .is_err()
        {
            return false;
        }
        let window_width = f32::from(window.bounds().size.width);
        let expected = crate::sidebar::SidebarGeometry::effective_width_for(
            self.sidebar.last_expanded(),
            window_width,
        );
        self.performance
            .expect_browser_state(self.controller.snapshot().active_view, true);
        self.performance.expect_sidebar_width(expected);
        // The measured expand; the toggle auto-commits the restored
        // presentation synchronously.
        self.toggle_sidebar(window, cx);
        true
    }
}
