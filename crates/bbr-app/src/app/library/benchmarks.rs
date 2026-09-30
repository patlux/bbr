//! B3 Saved Pages Library benchmark drivers (testkit-only, packet
//! `saved-pages-library`).
//!
//! `benchmark_library_search` measures the production Library search entry
//! (the typed `library_set_search` path the keyboard input and the testkit
//! request share) against the seeded ≥1,000-record provider: the bounded
//! window over the exact matching set re-renders and the sample commits
//! only when the rendered model presents the exact armed result count and
//! window (fast-but-wrong guard: a missing/wrong filter, a stale window,
//! or a wrong anchor rejects the sample in the app before it leaves).
//!
//! `benchmark_library_scroll` measures the production Library scroll entry
//! (`library_set_scroll`, the tracked offset the wheel listener drives):
//! the bounded window re-anchors and the sample commits only when the
//! rendered model presents the exact armed window/anchor and preserves the
//! exact multi-select set (fast-but-wrong guard: a no-op scroll, a wrong
//! window/anchor, or a corrupted selection rejects the sample).
//!
//! Both commit through the render-inclusive path: the dispatch draws the
//! main-window frame that renders the new model, then the explicit
//! `commit_library_benchmark` closes the pending sample (the same
//! two-phase shape as the dense and error-surface drivers). The Library is
//! ordinary GPUI chrome — never a second Saved Page authority and never a
//! network/session touch.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production Library search entry: set the query through
    /// the one production method, rebuild the presentation model, and arm
    /// the exact armed result count/window on the probe. A fast-but-wrong
    /// filter (the model does not carry the exact armed count/window)
    /// cancels the sample truthfully. The sample starts in the app
    /// immediately before the mutation; transport and harness scheduling
    /// are excluded.
    #[allow(clippy::too_many_arguments)]
    pub fn benchmark_library_search(
        &mut self,
        operation_id: BenchmarkOperationId,
        text: &str,
        expected_total: usize,
        expected_first: usize,
        expected_last: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::SavedPagesLibrarySearch)
            .is_err()
        {
            return false;
        }
        // The production search entry (the typed request and the keyboard
        // path share this method): sets the query and resets the scroll.
        self.library_set_search(text, cx);
        // Fast-but-wrong guard: the bounded window over the exact matching
        // set must materialize — the armed result count and window.
        let model = self.library_model();
        let (total, first, last, anchor) = (
            model.total,
            model.window.first,
            model.window.last,
            model.window.anchor,
        );
        if (total, first, last) != (expected_total, expected_first, expected_last) {
            self.performance.cancel();
            return false;
        }
        let selected = self.library.selected.clone();
        self.performance
            .expect_library(total, first, last, anchor, selected.clone());
        self.performance
            .observe_library(total, first, last, anchor, selected);
        self.observe_library_browser_state(window);
        true
    }

    /// Benchmark the production Library scroll entry: set the tracked
    /// offset through the one production method, rebuild the presentation
    /// model, and arm the exact armed window/anchor plus the preserved
    /// selection on the probe. A no-op/wrong scroll or a corrupted
    /// selection cancels the sample truthfully.
    #[allow(clippy::too_many_arguments)]
    pub fn benchmark_library_scroll(
        &mut self,
        operation_id: BenchmarkOperationId,
        offset: usize,
        expected_first: usize,
        expected_last: usize,
        expected_anchor: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::SavedPagesLibraryScroll)
            .is_err()
        {
            return false;
        }
        let selected_before = self.library.selected.clone();
        // The production scroll entry (the typed request and the wheel
        // listener share this method): sets the tracked offset.
        self.library_set_scroll(offset, cx);
        let model = self.library_model();
        let (first, last, anchor) = (model.window.first, model.window.last, model.window.anchor);
        if (first, last, anchor) != (expected_first, expected_last, Some(expected_anchor)) {
            self.performance.cancel();
            return false;
        }
        if self.library.selected != selected_before {
            self.performance.cancel();
            return false;
        }
        self.performance
            .expect_library(model.total, first, last, anchor, selected_before.clone());
        self.performance.observe_library(
            model.total,
            first,
            last,
            anchor,
            self.library.selected.clone(),
        );
        self.observe_library_browser_state(window);
        true
    }

    /// Record the generic browser state (active view, sidebar, chrome-focus
    /// domain, command-bar mode) the Library samples commit against, and
    /// arm the expected state from the same authoritative post-mutation
    /// snapshot (the fast-but-wrong guard rejects a sample that touched the
    /// session, sidebar, focus domain, or command surface).
    fn observe_library_browser_state(&self, window: &Window) {
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
    }

    /// Commit the pending Library sample after the main-window draw: the
    /// probe compares the armed Library identity and browser state against
    /// the observation; a mismatch leaves the sample pending and cancels
    /// truthfully (the driver's deadline catches a missing presentation).
    pub fn commit_library_benchmark(&self) -> bool {
        self.performance.commit_current_presentation();
        if matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::SavedPagesLibrarySearch)
                | Some(BenchmarkKind::SavedPagesLibraryScroll)
        ) {
            self.performance.cancel();
            return false;
        }
        true
    }
}
