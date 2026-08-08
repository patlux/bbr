//! The history-rich command-first-result benchmark driver (Phase 7,
//! command-history provider slice; queue item 16) and the shared
//! command-text benchmark body it reuses.
//!
//! `benchmark_command_type` (in `command_benchmarks.rs`) and
//! `benchmark_command_bar_first_result` share the exact production
//! command-text path — Cmd-L, replace-selection typing, the real ranking
//! debounce, and a render-inclusive commit whose predicate enforces the
//! exact generation, first/selected keys, typed state, mode, and focus
//! (fast-but-wrong guard). The two kinds differ only in the expected
//! identity they arm: `CommandType` expects a tab/fallback row, while
//! `CommandBarFirstResult` expects the ranked history row `history:<id>`
//! against the seeded 500-tab + 10k-history corpus. This module owns the
//! shared entry body and its render-inclusive commit so both kinds follow
//! the identical presentation-milestone path without conflating identities.

use bbr_testkit::protocol::BenchmarkKind;
use gpui::{App, AsyncApp, Context, Window};

use super::BbrApp;

/// True for the two kinds that share the command-text entry body and the
/// render-presentation commit: the sample may only leave from a render of
/// the result surface carrying the armed command identity. The guard is
/// kind-inclusive so both paths use the identical schedule/commit logic
/// without conflating their expected identities.
pub(crate) fn is_command_text_kind(kind: BenchmarkKind) -> bool {
    matches!(
        kind,
        BenchmarkKind::CommandType | BenchmarkKind::CommandBarFirstResult
    )
}

impl BbrApp {
    /// The Phase-7 history-rich twin of `benchmark_command_type` (queue
    /// item 16, `command_bar_first_result`): the identical production
    /// command-text path — Cmd-L, replace-selection typing, the real
    /// ranking debounce, and the render-inclusive commit whose predicate
    /// enforces the exact generation, first/selected keys, state, mode, and
    /// focus — measured against the 500-tab + 10k-history corpus the
    /// scenario seeds. A fast-but-wrong first result (a tab or fallback row
    /// instead of the ranked history record) rejects the sample.
    pub fn benchmark_command_bar_first_result(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        text: &str,
        expected_first_key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.benchmark_command_entry(
            operation_id,
            BenchmarkKind::CommandBarFirstResult,
            text,
            expected_first_key,
            window,
            cx,
        )
    }

    /// The shared production command-text benchmark body (Cmd-L selects the
    /// active URL, replace-selection typing enters `text` through the exact
    /// mutation the key handler runs, and the real ranking debounce expires
    /// and publishes the current generation). The sample commits only from a
    /// subsequent render of the result surface (the probe predicate
    /// enforces generation/result identity against the identity the render
    /// drew), so a stale generation, wrong first row, wrong state, mode, or
    /// focus rejects the measurement in the app — and a render that omitted
    /// or delayed the palette can never pass. Timing begins at the
    /// production input entry; the deterministic flush is never used inside
    /// this measured scenario. `pub(crate)`: both command-text kinds
    /// (`CommandType` in `command_benchmarks.rs` and `CommandBarFirstResult`
    /// here) run the identical entry body.
    pub(crate) fn benchmark_command_entry(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        kind: BenchmarkKind,
        text: &str,
        expected_first_key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.performance.begin(operation_id, kind).is_err() {
            return false;
        }
        // Production entry: the Cmd-L path plus replace-selection typing
        // (the exact mutation `on_chrome_key_down` runs), which restarts the
        // real cancellable debounce.
        self.activate_command_bar(window, cx);
        self.command_text.replace_selection(text);
        self.on_command_text_changed(cx);
        // Arm the exact rendered-presentation identity now — the debounce
        // is still 30 ms away, so no publication can land before arming:
        // the generation the entry just scheduled, the correct first
        // result, the selected row, the typed state, mode, and focus. The
        // commit predicate only accepts the render carrying this identity.
        let expected_generation = self.command_ranking.generation;
        let active_view = self.controller.snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        self.performance
            .expect_state(active_view, self.sidebar_visible, true, mode);
        self.performance.expect_command_presentation(
            expected_generation,
            expected_first_key,
            expected_first_key,
            bbr_testkit::protocol::CommandResultState::Results,
        );
        // Observe the (unchanged) presented state at arm time: typing never
        // changes the view/sidebar/focus/mode, so the render is compared
        // against the state the surface actually presents.
        self.performance
            .observe_render_state(active_view, self.sidebar_visible, true, mode);
        true
    }

    /// The ranking publication arrived: when a command-text benchmark
    /// (`CommandType` or `CommandBarFirstResult`) is pending, the sample
    /// must commit only from a subsequent render of the result surface —
    /// the ranking callback itself never emits it. This schedules a bounded
    /// forced draw (the same testkit pattern the render-dependent dense
    /// scenarios use) so the palette is actually rendered in the hidden
    /// harness and the probe commits at the presentation milestone. The
    /// draw runs on the main thread after this update completes, through
    /// the untyped window path (the entity lease is free then).
    pub(crate) fn schedule_command_text_render_commit(&self, cx: &mut Context<Self>) {
        let pending = self.performance.pending_kind();
        if !pending.is_some_and(is_command_text_kind) {
            return;
        }
        let Some(window) = cx.windows().into_iter().next() else {
            return;
        };
        let weak = cx.weak_entity();
        cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
            async_cx.update(|cx| {
                // Render the published result surface: the render
                // observes the command identity after constructing the
                // palette and refreshes the browser-state observation;
                // the slot paint then commits the sample when the armed
                // identity and state match (fast-but-wrong guard).
                let _ = window.update(cx, |_root, window, cx| {
                    let app: &mut App = cx;
                    let _ = window.draw(app);
                });
                // Post-render commit through the app-entity contract (the
                // window root is now `gpui_component::Root`, so the commit
                // no longer downcasts the root): the explicit path
                // validates the same identity + state and cancels
                // truthfully when the render omitted the palette (no
                // observation → no commit), so a broken palette render
                // fails the sample instead of silently passing.
                let _: bool = weak
                    .update(cx, |this, _cx| this.commit_command_text_benchmark())
                    .unwrap_or(false);
            });
        })
        .detach();
    }

    /// Commit the command-text sample (`CommandType` or
    /// `CommandBarFirstResult`) from the render milestone: the probe
    /// compares the armed command identity against the observation the
    /// render recorded after constructing the palette, and the
    /// browser-state expectations against the state the render observed.
    /// Returns false and cancels when the render omitted the palette or the
    /// identity/state was wrong — a truthful rejection, never a
    /// fast-but-wrong pass. The pending-kind check is kind-inclusive: a
    /// still-pending sample of either command-text kind means the
    /// presentation did not match, while a committed sample has no pending
    /// kind left.
    fn commit_command_text_benchmark(&self) -> bool {
        self.performance.commit_current_presentation();
        if self
            .performance
            .pending_kind()
            .is_some_and(is_command_text_kind)
        {
            self.performance.cancel();
            return false;
        }
        true
    }
}
