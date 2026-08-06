//! Dense-list benchmark drivers (testkit-only).
//!
//! The dense scroll and dense select measurements run through the production
//! paths: the dense scroll moves the tracked `ScrollHandle` the native wheel
//! listener, the keyboard traversal, and the render share (GPUI 0.2.2
//! exposes no public synthetic wheel dispatch — `Window::dispatch_event`
//! returns a crate-private result type — so the seam mutates the exact
//! offset a wheel event would drive), and the dense selection runs the
//! production selection path with instant scroll-into-view. Because GPUI
//! 0.2.2 exposes no public platform-presentation callback, each sample
//! commits after a synchronous production `Window::draw` — the exact
//! render/layout/paint path a display link frame runs — and only when the
//! semantic post-state actually materialized (the scroll anchor moved, or
//! the selected row lies inside the bounded rendered window). A no-op scroll
//! or an off-window selection cancels the sample instead of committing
//! fast-but-wrong. Scanout timing is deliberately not claimed (documented in
//! E2E_ARCHITECTURE.md).

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use gpui::Window;

use super::BbrApp;

impl BbrApp {
    /// Begin the dense-scroll presentation measurement. The anchor recorded
    /// by the last render is the "before" state the commit compares against.
    pub fn begin_dense_scroll_benchmark(&self, operation_id: BenchmarkOperationId) -> bool {
        self.performance
            .begin(operation_id, BenchmarkKind::DenseScroll)
            .is_ok()
    }

    /// Commit the dense-scroll sample after the production wheel dispatch
    /// and its render: the sample only exists when the scroll actually moved
    /// the anchor (a no-op wheel cancels). The presentation commits
    /// synchronously from authoritative post-render state.
    pub fn commit_dense_scroll_benchmark(&self, window: &Window, anchor_before: usize) -> bool {
        if self.dense_window.get().scroll_anchor == anchor_before {
            self.performance.cancel();
            return false;
        }
        self.commit_dense_benchmark(window);
        true
    }

    /// Begin the dense-selection presentation measurement.
    pub fn begin_dense_select_benchmark(&self, operation_id: BenchmarkOperationId) -> bool {
        self.performance
            .begin(operation_id, BenchmarkKind::DenseSelect)
            .is_ok()
    }

    /// Commit the dense-selection sample after the production selection path
    /// and its render: the selected row must lie inside the bounded window
    /// the render actually built (pinned rows are always visible; a
    /// regular-row selection outside the window cancels — the instant
    /// scroll-into-view must have run).
    pub fn commit_dense_select_benchmark(&self, window: &Window, index: usize) -> bool {
        let dense = self.dense_window.get();
        let window_range = dense.first_rendered..dense.first_rendered + dense.rendered_rows;
        let pinned = crate::shell::pinned_count(&self.controller.snapshot().tabs);
        if index >= pinned && !window_range.contains(&index) {
            self.performance.cancel();
            return false;
        }
        self.commit_dense_benchmark(window);
        true
    }

    /// Commit a dense sample synchronously from authoritative post-render
    /// state (active view, focus, mode), like the other interaction
    /// benchmarks. The last draw refreshed the probe's native slot rect, so
    /// the presentation commits against fresh geometry.
    fn commit_dense_benchmark(&self, window: &Window) {
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
}
