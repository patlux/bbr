//! Testkit-only find-in-page benchmark drivers (task packet
//! `find-in-page-portal-bar`).
//!
//! `benchmark_find_open` measures the production `Cmd-F` path (the exact
//! seam the `FindInPage` action routes to): the sample commits only from a
//! render of the one `NativeAboveWeb` portal window that drew the find bar
//! with the field focused over the same active view (fast-but-wrong guard —
//! a render that omitted the bar or changed the view/session can never
//! commit). `benchmark_find_query_result` and `benchmark_find_step` measure
//! the production query/step paths (`find_set_query` / `find_next` /
//! `find_previous`): the sample commits only when WebKit's content process
//! reported the armed count/ordinal through the portable contract and the
//! bar re-rendered it — the completion path schedules the render-inclusive
//! commit, so a pending, stale, or mismatched report rejects in the app.
//! Fixture/network time is excluded by construction: the deterministic page
//! is already loaded before any sample begins.
//!
//! Opening, stepping, and closing never create, destroy, move, or reselect a
//! web session (the portable adapter binds the retained view once at open);
//! every driver arms the unchanged active view, sidebar, find-focus domain,
//! and inactive command-bar mode, so the probe rejects any sample that
//! touches them.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use gpui::{App, AsyncApp, Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production `Cmd-F` find-bar open: the sample starts at
    /// the exact seam the `FindInPage` action routes to (`find_open`, which
    /// presents the bar through the one native portal host, focuses the
    /// find field, and captures the origin focus domain) and commits only
    /// from a render of the portal window that drew the bar open with the
    /// field focused. The active view/sidebar are unchanged and the find
    /// field owns focus (chrome domain released); the armed expectations
    /// reject any render that omits the bar or touches the session.
    pub fn benchmark_find_open(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::FindOpen)
            .is_err()
        {
            return false;
        }
        // Production Cmd-F entry: the exact seam the `FindInPage` action
        // routes to (`window_seam` → `find_open`).
        self.find_open(window, cx);
        let active_view = self.controller.snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        // Fast-but-wrong guard: the same active view stays selected, the
        // sidebar stays visible, the find field owns focus (chrome released),
        // and no command surface is active. Refresh the probe from the
        // authoritative post-mutation state so the presentation commit sees
        // the real state (the find-open render has not painted yet).
        self.performance
            .observe_render_state(active_view, self.sidebar_visible, false, mode);
        self.performance
            .expect_state(active_view, self.sidebar_visible, false, mode);
        self.performance.expect_find_open();
        // Render-inclusive commit: draw the portal window (the find bar
        // renders there) so the render records the drawn find state; the
        // probe then commits only if the armed open/focused identity and
        // browser state match.
        self.schedule_find_render_commit(cx);
        true
    }

    /// Benchmark the production find-query path (`find_set_query`, the exact
    /// mutation typing/backspace route to): the sample starts when the query
    /// enters the production path and commits only when WebKit's content
    /// process reported the armed count/ordinal through the portable
    /// contract and the bar re-rendered it. A pending, stale, or mismatched
    /// report rejects in the app (fast-but-wrong guard).
    pub fn benchmark_find_query_result(
        &mut self,
        operation_id: BenchmarkOperationId,
        query: &str,
        expected_count: usize,
        expected_ordinal: Option<usize>,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::FindQueryResult)
            .is_err()
        {
            return false;
        }
        // Production query entry: the exact mutation the key handler and the
        // testkit `FindSetQuery` request route to (`find_set_query` → the
        // portable session start → the native adapter → the content-world
        // strategy). The completion lands asynchronously on a later update.
        self.find_set_query(query, cx);
        let active_view = self.controller.snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        // The query never changes view/sidebar/focus/mode: the find field
        // keeps focus, the bar stays over the same active view.
        self.performance
            .observe_render_state(active_view, self.sidebar_visible, false, mode);
        self.performance
            .expect_state(active_view, self.sidebar_visible, false, mode);
        self.performance
            .expect_find_report(query, expected_count, expected_ordinal);
        // The completion path schedules the render-inclusive commit when the
        // report lands (see `find.rs::on_find_completion`).
        true
    }

    /// Benchmark the production find step (`find_next`/`find_previous`, the
    /// exact seams Enter/Shift-Enter and the bar buttons route to): the
    /// sample starts at the step entry and commits only when WebKit reported
    /// the armed ordinal and the bar re-rendered it.
    pub fn benchmark_find_step(
        &mut self,
        operation_id: BenchmarkOperationId,
        next: bool,
        expected_count: usize,
        expected_ordinal: Option<usize>,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::FindStep)
            .is_err()
        {
            return false;
        }
        let query = self.find.state.query.clone();
        // Production step entry (the exact `find_next`/`find_previous`
        // seams); the report lands asynchronously like the query path.
        if next {
            self.find_next(cx);
        } else {
            self.find_previous(cx);
        }
        let active_view = self.controller.snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        self.performance
            .observe_render_state(active_view, self.sidebar_visible, false, mode);
        self.performance
            .expect_state(active_view, self.sidebar_visible, false, mode);
        self.performance
            .expect_find_report(&query, expected_count, expected_ordinal);
        true
    }

    /// Schedule the render-inclusive commit for a pending find benchmark: a
    /// bounded forced draw of the portal window (the find bar renders in the
    /// one `NativeAboveWeb` host, so the main-window draw used by the other
    /// command benchmarks cannot present it), followed by the explicit
    /// commit through the app-entity contract. The draw runs on the main
    /// thread after this update completes, outside the mutable `BbrApp`
    /// lease (the same reentrancy rule as `dispatch_portal`).
    pub(crate) fn schedule_find_render_commit(&self, cx: &mut Context<Self>) {
        if !matches!(
            self.performance.pending_kind(),
            Some(
                BenchmarkKind::FindOpen | BenchmarkKind::FindQueryResult | BenchmarkKind::FindStep
            )
        ) {
            return;
        }
        let Some(portal) = self.portal_frame_handle() else {
            return;
        };
        let weak = cx.weak_entity();
        cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
            async_cx.update(|cx| {
                // Draw the portal window: `PortalRoot` renders the find bar
                // from the app's live `FindSurface` and records the drawn
                // find state into the probe (the render milestone). A render
                // that omitted the bar records nothing and can never commit.
                let _ = portal.update(cx, |_root, window, cx| {
                    let app: &mut App = cx;
                    let _ = window.draw(app);
                });
                // Post-render commit through the app-entity contract: the
                // explicit path validates the armed identity + browser state
                // and cancels truthfully when the render did not present the
                // bar or the report (no observation → no commit).
                let _: bool = weak
                    .update(cx, |this, _cx| this.commit_find_benchmark())
                    .unwrap_or(false);
            });
        })
        .detach();
    }

    /// Commit the pending find sample from the render milestone: the probe
    /// compares the armed open/report identity against the observation the
    /// portal render recorded and the browser-state expectations against the
    /// state observed at the mutation site. Returns false and cancels when
    /// the render omitted the bar or the identity/state was wrong — a
    /// truthful rejection, never a fast-but-wrong pass.
    fn commit_find_benchmark(&self) -> bool {
        self.performance.commit_current_presentation();
        if matches!(
            self.performance.pending_kind(),
            Some(
                BenchmarkKind::FindOpen | BenchmarkKind::FindQueryResult | BenchmarkKind::FindStep
            )
        ) {
            self.performance.cancel();
            return false;
        }
        true
    }

    /// Record the find state the portal render actually drew for the find
    /// bar (the render milestone; `PortalRoot` calls this after consuming
    /// the app's live `FindSurface`). The probe commits find samples only
    /// from this observation.
    #[cfg(feature = "testkit")]
    pub(crate) fn observe_portal_find_render(&self) {
        let s = &self.find.state;
        self.performance.observe_find_state(
            s.open,
            s.focused,
            &s.query,
            s.match_count,
            s.current_match,
            s.pending,
            s.stale,
        );
    }
}
