//! Testkit-only Saved Page benchmark drivers (B2 packet
//! `saved-pages-actions-command`).
//!
//! `benchmark_saved_page_search_first_result` measures the production
//! command-text path against the seeded ≥1,000-record Saved Page provider:
//! Cmd-L, replace-selection typing, the real ranking debounce, and a render
//! of the result surface committing the sample — the expected first result
//! is a ranked Saved Page row (`saved_page:<id>`), and the fast-but-wrong
//! guard rejects a tab/fallback row or a stale generation. Same warm
//! command-type class (50/100) as `command_bar_first_result`.
//!
//! `benchmark_saved_page_open` measures the production open of a Saved Page
//! result: the record is revalidated in the in-memory provider, then
//! exactly one fresh regular tab/session opens in the currently active
//! Space/Profile through the existing NewTab authority — the
//! session-recreation class (100/150), like `command_activate_new_tab` and
//! `tab_reopen`. The sample commits only after the caller-specific
//! validation (exactly one new tab and live session, a fresh active view,
//! surface closed in web focus) passes.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production command-text path against the Saved Page
    /// provider (B2 packet `saved-pages-actions-command`,
    /// `saved_page_search_first_result`): the identical production
    /// command-text entry body as `benchmark_command_bar_first_result`
    /// (Cmd-L, replace-selection typing, the real ranking debounce, and the
    /// render-inclusive commit whose predicate enforces the exact
    /// generation, first/selected keys, state, mode, and focus), measured
    /// against the seeded ≥1,000-record provider. A fast-but-wrong first
    /// result (a tab or fallback row instead of the ranked Saved Page
    /// record) rejects the sample.
    pub fn benchmark_saved_page_search_first_result(
        &mut self,
        operation_id: BenchmarkOperationId,
        text: &str,
        expected_first_key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.benchmark_command_entry(
            operation_id,
            BenchmarkKind::SavedPageSearchFirstResult,
            text,
            expected_first_key,
            window,
            cx,
        )
    }

    /// Benchmark the production open of a Saved Page result (B2 packet
    /// `saved-pages-actions-command`, `saved_page_open`): the record is
    /// revalidated in the in-memory provider, then exactly one fresh regular
    /// tab/session opens in the currently active Space/Profile through the
    /// existing NewTab authority — the session-recreation class. The generic
    /// activation commit is suppressed while this benchmark is pending; the
    /// sample commits only after the caller-specific validation (exactly
    /// one new tab and live session, a fresh active view, surface closed in
    /// web focus) passes. A stale/deleted `BookmarkId` rejects truthfully.
    pub fn benchmark_saved_page_open(
        &mut self,
        operation_id: BenchmarkOperationId,
        bookmark_id: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let before = self.controller.snapshot();
        if self
            .performance
            .begin(operation_id, BenchmarkKind::SavedPageOpen)
            .is_err()
        {
            return false;
        }
        // Production open path: the one app-owned command authority
        // revalidates the record and opens exactly one fresh regular tab in
        // the currently active Space/Profile (never a silent selection of an
        // existing matching tab).
        let command = self.saved_pages;
        let opened = command.open(self, bbr_command::id::BookmarkId(bookmark_id), window, cx);
        let after = self.controller.snapshot();
        // Fast-but-wrong guard: exactly one new tab and one new live
        // session, a fresh active view (never the pre-activation view), and
        // the surface closed in web focus. A stale/deleted record or a
        // failed open rejects truthfully.
        if !opened
            || after.tabs.len() != before.tabs.len() + 1
            || after.engine_sessions != before.engine_sessions + 1
            || after.active_view == before.active_view
            || self.command_bar_mode != crate::command_bar::CommandBarMode::Inactive
        {
            self.performance.cancel();
            return false;
        }
        // The validation passed: arm the presented state and commit the
        // sample (the probe observes the same state synchronously).
        self.performance.expect_state(
            after.active_view,
            self.sidebar_visible,
            false,
            crate::command_bar::CommandBarMode::Inactive.into(),
        );
        self.commit_benchmark_sync(window);
        true
    }
}
