//! Testkit-only web-content gesture benchmarks (N1 packet
//! `web-content-navigation-gestures`).
//!
//! `benchmark_web_history_swipe_back` and `benchmark_child_tab_swipe_return`
//! drive the **real production content-gesture path** — the exact method the
//! native content-gesture bridge reporter pump calls
//! (`BbrApp::on_content_gesture_commit`) — and commit the sample only when
//! the correct semantic state is presented (fast-but-wrong guard inside the
//! app; the driver re-asserts the typed snapshot afterwards). External
//! page/network latency stays outside BBR timings: the fixtures are
//! deterministic and already loaded before every sample, and the sample
//! spans the commit → controller decision → engine dispatch →
//! presentation-commit path, never the async WebKit navigation.

use bbr_browser::ControllerEvent;
use bbr_web::{ContentGestureCommit, ContentGestureDirection, WebViewId};
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark one committed logical Back content gesture against the
    /// exact bound `view` of a deterministic already-loaded two-entry
    /// history (N1 packet `web-content-navigation-gestures`,
    /// `web_history_swipe_back` scenario). The sample starts immediately
    /// before the production gesture execution and commits only when:
    ///
    /// - the bound view is the presented active view with real
    ///   `can_go_back == true` before the commit (the deterministic
    ///   already-loaded history entry);
    /// - the controller classified the commit as **exactly one** history
    ///   dispatch (`GestureHistoryRouted`), never a no-op and never a tab
    ///   mutation; and
    /// - the presentation (same active view, focus domain, command mode,
    ///   geometry) is committed from the authoritative post-commit state.
    ///
    /// The driver additionally waits for the real WebKit URL/history state
    /// (title of the previous entry, `can_go_back == false`) before
    /// accepting the sample — a dispatch that never navigated is rejected
    /// there.
    pub fn benchmark_web_history_swipe_back(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        view: WebViewId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::WebHistorySwipeBack,
            )
            .is_err()
        {
            return false;
        }
        let before = self.controller.read().snapshot();
        if before.active_view != Some(view) || !before.nav.can_go_back {
            self.performance.cancel();
            return false;
        }
        // The exact production method the native gesture-bridge pump calls.
        let outcome = self.on_content_gesture_commit(
            ContentGestureCommit {
                view,
                direction: ContentGestureDirection::Back,
            },
            cx,
        );
        if !matches!(outcome, Some(ControllerEvent::GestureHistoryRouted { .. })) {
            self.performance.cancel();
            return false;
        }
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
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance.commit_current_presentation();
        if self.performance.presentation_pending()() {
            self.performance.cancel();
            return false;
        }
        true
    }

    /// Benchmark one committed Back-at-settled-child-root opener return
    /// (N1 packet `web-content-navigation-gestures`,
    /// `child_tab_swipe_return` scenario). The sample starts immediately
    /// before the production gesture execution and commits only when:
    ///
    /// - the child view is the presented active view at its settled root
    ///   (`can_go_back == false`) before the commit;
    /// - the controller ran the atomic **opener return** transaction
    ///   (`OpenerReturn`), never a plain history dispatch or a no-op; and
    /// - the exact warm opener's view is the presented active view with the
    ///   unchanged focus/command/geometry state (the child session is gone
    ///   and the opener session reused — the driver re-asserts the typed
    ///   tabs/sessions/opener identity afterwards).
    pub fn benchmark_child_tab_swipe_return(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        child_view: WebViewId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::ChildTabSwipeReturn,
            )
            .is_err()
        {
            return false;
        }
        let before = self.controller.read().snapshot();
        if before.active_view != Some(child_view) || before.nav.can_go_back {
            self.performance.cancel();
            return false;
        }
        // The exact production method the native gesture-bridge pump calls.
        let outcome = self.on_content_gesture_commit(
            ContentGestureCommit {
                view: child_view,
                direction: ContentGestureDirection::Back,
            },
            cx,
        );
        let presented_view = match outcome {
            Some(ControllerEvent::OpenerReturn { presented_view, .. }) => presented_view,
            _ => {
                self.performance.cancel();
                return false;
            }
        };
        // Fast-but-wrong guard: the child's view is gone from the live
        // presentation and the opener's view is the presented active view.
        let snapshot = self.controller.read().snapshot();
        if snapshot.active_view != Some(presented_view) {
            self.performance.cancel();
            return false;
        }
        let chrome_focused = self.chrome_focused(window);
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            Some(presented_view),
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance.expect_state(
            Some(presented_view),
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance.commit_current_presentation();
        if self.performance.presentation_pending()() {
            self.performance.cancel();
            return false;
        }
        true
    }
}
