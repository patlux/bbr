//! Pointer-state benchmark drivers (testkit-only).
//!
//! The §10.2 hover and §10.3 press presentation measurements run through the
//! exact app methods the row element handlers call (`set_row_hover` /
//! `set_row_pressed`), so the sample measures the production pointer path a
//! real mouse drives — never a snapshot mutation. Each sample commits
//! synchronously from the authoritative post-mutation state and only when
//! the derived render state actually shows the pointer wash, so a
//! fast-but-wrong sample (state derivation broken, stale row index, missing
//! notify) is rejected in the app before it leaves. These are
//! warm-interaction class (transient indication, next-frame presentation),
//! like the other shell interaction benchmarks.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use gpui::{Context, Window};

use bbr_ui::state::TabVisualState;

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production pointer hover entry on a visible tab row:
    /// `set_row_hover` (the exact app method the row's `on_hover` element
    /// listener calls) renders the §10.2 hover wash and reveals the close
    /// control. The sample commits only when the row's derived state is
    /// `Hover` (a broken hover path cancels instead of committing
    /// fast-but-wrong).
    pub fn benchmark_hover_tab(
        &mut self,
        operation_id: BenchmarkOperationId,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::TabHover)
            .is_err()
        {
            return false;
        }
        self.set_row_hover(index, true, cx);
        if self.row_state_of(index) != Some(TabVisualState::Hover) {
            self.performance.cancel();
            return false;
        }
        self.commit_pointer_benchmark(window);
        true
    }

    /// Benchmark the production pointer press on a visible tab row:
    /// `set_row_pressed` (the exact app method the row's mouse-down listener
    /// calls) renders the §10.3 active wash. The sample commits only when
    /// the row's derived state is `Active`.
    pub fn benchmark_press_tab(
        &mut self,
        operation_id: BenchmarkOperationId,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::TabPress)
            .is_err()
        {
            return false;
        }
        self.set_row_pressed(index, cx);
        if self.row_state_of(index) != Some(TabVisualState::Active) {
            self.performance.cancel();
            return false;
        }
        self.commit_pointer_benchmark(window);
        true
    }

    /// The composed §10 state of `index` from the same `row_visuals` source
    /// the render and the snapshot use, so the benchmark guard cannot drift
    /// from the shell.
    fn row_state_of(&self, index: usize) -> Option<TabVisualState> {
        let browser = self.controller.read().snapshot();
        let rows = self.row_visuals(&browser.tabs, self.tab_drag.source());
        rows.iter()
            .find(|row| row.index == index)
            .map(|row| row.state)
    }

    /// Commit a pointer-state sample synchronously from authoritative
    /// post-mutation state (active view, focus, mode), like the other
    /// interaction benchmarks.
    fn commit_pointer_benchmark(&self, window: &Window) {
        let active_view = self.controller.read().snapshot().active_view;
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
