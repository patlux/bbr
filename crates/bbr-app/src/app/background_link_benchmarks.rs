//! Testkit-only background-link benchmark driver (DD-LINK packet
//! `background-link-tabs`).
//!
//! `benchmark_background_link_open` measures the production
//! Command-primary/middle-click → background-child path: the sample starts
//! inside the app when the request enters `BbrApp::on_new_window_request`
//! (the exact seam the native coordinator delivery pump and the typed
//! testkit injection share) and commits only from the authoritative
//! post-mutation browser state that proves the child opened in the
//! background — the exact opener is still the active, web-focused view, the
//! new child is a live session at the expected index immediately after the
//! opener, and the child is NOT the active/foreground pane (fast-but-wrong
//! guard: any selection move, focus bounce, or foreground activation
//! rejects the sample). The child session is the fresh native-session class
//! (p95 ≤ 100 ms; max ≤ 150 ms), same as `peek_request_to_present`; fixture
//! page-load time is excluded by construction (the sample commits at
//! insertion, not at page ready).

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId, CommandBarMode};
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production background-link open
    /// (`background_link_open`): the sample starts when the request enters
    /// `on_new_window_request` and commits only when the exact opener is
    /// still the active, web-focused view and the new child is a live,
    /// hidden session at the grouped insertion index directly after the
    /// opener — never a foreground activation.
    pub fn benchmark_background_link_open(
        &mut self,
        operation_id: BenchmarkOperationId,
        url: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::BackgroundLinkOpen)
            .is_err()
        {
            return false;
        }
        // Build the production delivery exactly as the native coordinator
        // decides it for a Command-primary / middle click: default intent,
        // background input disposition, from the current active view. The
        // request URL passes verbatim (never normalized).
        let Some(opener) = self.controller.read().snapshot().active_view else {
            self.performance.cancel();
            return false;
        };
        let opener_profile = self.controller.read().snapshot().active_profile;
        let request = bbr_web::NewWindowRequest {
            source: opener,
            source_profile: opener_profile,
            url,
            user_initiated: true,
            target: bbr_web::NewWindowTarget::NewWindow,
            input: bbr_web::NewWindowInput::Background,
            features: bbr_web::NewWindowFeatures::default(),
        };
        let decision = bbr_web::decision_for_intent(&request, bbr_web::NewWindowIntent::Default);
        let delivery = bbr_web::NewWindowDelivery {
            outcome: bbr_web::decide(&request, decision),
            request,
        };
        let Some(bbr_browser::ControllerEvent::NewWindowOpened {
            view: child, index, ..
        }) = self.on_new_window_request(delivery, window, cx)
        else {
            // The request was not decided to a background child (or failed)
            // — a truthful rejection, never a fast-but-wrong sample.
            self.performance.cancel();
            return false;
        };
        // The authoritative browser state after the mutation: the opener
        // stays selected and web-focused, and the child is a live hidden
        // session at the insertion index directly after the opener — the
        // semantic fast-but-wrong guard (a foreground activation, a
        // selection move, or a missing/misplaced session rejects).
        let snapshot = self.controller.read().snapshot();
        let child_ok = snapshot
            .tabs
            .get(index)
            .is_some_and(|tab| tab.web_view == Some(child) && !tab.active)
            && snapshot.active_view == Some(opener);
        if !child_ok {
            self.performance.cancel();
            return false;
        }
        let mode: CommandBarMode = self.command_bar_mode.into();
        self.performance
            .expect_state(snapshot.active_view, self.sidebar_visible, false, mode);
        self.commit_benchmark_sync(window);
        true
    }
}
