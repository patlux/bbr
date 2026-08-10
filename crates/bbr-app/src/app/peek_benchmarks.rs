//! Testkit-only transient Peek benchmark drivers (Link Preview / Peek
//! packet).
//!
//! `benchmark_peek_request_to_present` measures the production eligible
//! new-window → Peek path: the sample starts inside the app when the request
//! enters `BbrApp::on_new_window_request` (the exact seam the native
//! coordinator delivery pump and the typed testkit injection share) and
//! commits only from a render of the one `NativeAboveWeb` portal window that
//! drew the Peek bar for the preview session raised above its warm origin
//! (fast-but-wrong guard — a render that omitted the bar, changed the
//! origin, or failed the native z-order can never commit). The preview
//! session is the fresh native-session class (p95 ≤ 100 ms; max ≤ 150 ms);
//! fixture/network time is excluded by construction (the deterministic local
//! target is loaded by the harness before the block, and the sample commits
//! at presentation, not at page ready).
//!
//! `benchmark_peek_dismiss_to_origin` and `benchmark_peek_promote_to_tab`
//! measure the production dismissal/promotion paths (`peek_close` /
//! `peek_promote`): the samples commit from the authoritative post-mutation
//! browser state (the warm origin active/focused again; the SAME preview
//! `WebViewId` as a regular tab, active and focused, no reload) — the warm
//! interaction class (p95 ≤ 50 ms; max ≤ 100 ms). Neither path creates,
//! recreates, or reloads a native session.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId, CommandBarMode};
use gpui::{App, AsyncApp, Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production eligible new-window → Peek presentation
    /// (`peek_request_to_present`): the sample starts when the request
    /// enters `on_new_window_request` and commits only from a portal render
    /// that drew the Peek bar for the preview session raised above the warm
    /// origin (the in-app probe owns the fast-but-wrong guard). `url` is
    /// the deterministic local preview target fixture. The covered origin
    /// stays the active view and the preview owns native focus while open.
    pub fn benchmark_peek_request_to_present(
        &mut self,
        operation_id: BenchmarkOperationId,
        url: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::PeekRequestToPresent)
            .is_err()
        {
            return false;
        }
        // Build the production delivery exactly as the native coordinator
        // decides it for an eligible direct activation: default intent from
        // the current active view, default (empty) window features — the
        // portable `peek_eligible`/`decision_for_intent` policy decides
        // Preview. The request URL passes verbatim (never normalized).
        let Some(source) = self.controller.snapshot().active_view else {
            self.performance.cancel();
            return false;
        };
        let source_profile = self.controller.snapshot().active_profile;
        let request = bbr_web::NewWindowRequest {
            source,
            source_profile,
            url,
            user_initiated: true,
            target: bbr_web::NewWindowTarget::NewWindow,
            features: bbr_web::NewWindowFeatures::default(),
        };
        let decision = bbr_web::decision_for_intent(&request, bbr_web::NewWindowIntent::Default);
        let delivery = bbr_web::NewWindowDelivery {
            outcome: bbr_web::decide(&request, decision),
            request,
        };
        let Some(bbr_browser::ControllerEvent::PreviewOpened { view, source, .. }) =
            self.on_new_window_request(delivery, window, cx)
        else {
            // The request was not decided to Preview (or failed) — a
            // truthful rejection, never a fast-but-wrong sample.
            self.performance.cancel();
            return false;
        };
        // Arm the exact Peek presentation the sample must commit from and
        // the browser state the preview open must preserve (the covered
        // origin stays the active view; the preview owns focus — chrome
        // released, command surface inactive).
        self.performance.expect_peek_presented(view, source);
        let mode: CommandBarMode = self.command_bar_mode.into();
        self.performance
            .observe_render_state(Some(source), self.sidebar_visible, false, mode);
        self.performance
            .expect_state(Some(source), self.sidebar_visible, false, mode);
        // Render-inclusive commit: draw the portal window (the Peek bar
        // renders there) so the render records the drawn Peek state; the
        // probe then commits only if the armed presentation identity and
        // browser state match.
        self.schedule_peek_render_commit(cx);
        true
    }

    /// Benchmark the production Peek dismissal path
    /// (`peek_dismiss_to_origin`): the sample starts when `peek_close`
    /// destroys the preview session and commits only when the exact warm
    /// origin is again the active, focused view with the Peek bar gone.
    pub fn benchmark_peek_dismiss_to_origin(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::PeekDismissToOrigin)
            .is_err()
        {
            return false;
        }
        let Some(source) = self.controller.snapshot().preview.map(|p| p.source) else {
            self.performance.cancel();
            return false;
        };
        // Production dismissal entry (the exact `peek_close` seam the Escape
        // key and the close control route to).
        if !self.peek_close(window, cx) {
            self.performance.cancel();
            return false;
        }
        let snapshot = self.controller.snapshot();
        if snapshot.preview.is_some() || snapshot.active_view != Some(source) {
            // The preview is gone but the origin is not the active view: a
            // truthful rejection (the warm origin must be presented again).
            self.performance.cancel();
            return false;
        }
        let mode: CommandBarMode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            snapshot.active_view,
            self.sidebar_visible,
            false,
            mode,
        );
        self.performance
            .expect_state(snapshot.active_view, self.sidebar_visible, false, mode);
        self.commit_benchmark_sync(window);
        true
    }

    /// Benchmark the production Peek promotion path
    /// (`peek_promote_to_tab`): the sample starts when `peek_promote`
    /// adopts the preview session and commits only when the SAME preview
    /// `WebViewId` is a regular tab in the source space/profile, active and
    /// focused, with the preview state gone — no reload, no recreation.
    pub fn benchmark_peek_promote_to_tab(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::PeekPromoteToTab)
            .is_err()
        {
            return false;
        }
        let Some(preview_view) = self.controller.snapshot().preview.map(|p| p.view) else {
            self.performance.cancel();
            return false;
        };
        // Production promotion entry (the exact `peek_promote` seam the
        // "Open as Tab" control routes to).
        if !self.peek_promote(window, cx) {
            self.performance.cancel();
            return false;
        }
        let snapshot = self.controller.snapshot();
        let promoted = snapshot
            .tabs
            .iter()
            .any(|tab| tab.web_view == Some(preview_view));
        if !promoted || snapshot.preview.is_some() || snapshot.active_view != Some(preview_view) {
            // The SAME view must be a regular tab, active and focused: a
            // truthful rejection.
            self.performance.cancel();
            return false;
        }
        let mode: CommandBarMode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            snapshot.active_view,
            self.sidebar_visible,
            false,
            mode,
        );
        self.performance
            .expect_state(snapshot.active_view, self.sidebar_visible, false, mode);
        self.commit_benchmark_sync(window);
        true
    }

    /// Schedule the render-inclusive commit for a pending
    /// `peek_request_to_present` sample: a bounded forced draw of the portal
    /// window (the Peek bar renders in the one `NativeAboveWeb` host, so the
    /// main-window draw used by the other command benchmarks cannot present
    /// it), followed by the explicit commit through the app-entity contract.
    /// The draw runs on the main thread after this update completes, outside
    /// the mutable `BbrApp` lease (the same reentrancy rule as
    /// `dispatch_portal` and the find benchmarks).
    pub(crate) fn schedule_peek_render_commit(&self, cx: &mut Context<Self>) {
        if !matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::PeekRequestToPresent)
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
                // Draw the portal window: `PortalRoot` renders the Peek bar
                // from the app's live Peek surface and records the drawn
                // Peek state into the probe (the render milestone). A render
                // that omitted the bar records nothing and can never commit.
                let _ = portal.update(cx, |_root, window, cx| {
                    let app: &mut App = cx;
                    let _ = window.draw(app);
                });
                // Post-render commit through the app-entity contract: the
                // explicit path validates the armed identity + browser state
                // and cancels truthfully when the render did not present the
                // Peek bar or the z-order (no observation → no commit).
                let _: bool = weak
                    .update(cx, |this, _cx| this.commit_peek_benchmark())
                    .unwrap_or(false);
            });
        })
        .detach();
    }

    /// Commit the pending `peek_request_to_present` sample from the render
    /// milestone: the probe compares the armed Peek presentation identity
    /// (preview session drawn, raised above the armed origin) against the
    /// observation the portal render recorded and the browser-state
    /// expectations against the state observed at the mutation site. Returns
    /// false and cancels when the render omitted the bar or the
    /// identity/state was wrong — a truthful rejection, never a
    /// fast-but-wrong pass.
    fn commit_peek_benchmark(&self) -> bool {
        self.performance.commit_current_presentation();
        if matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::PeekRequestToPresent)
        ) {
            self.performance.cancel();
            return false;
        }
        true
    }

    /// Record the Peek state the portal render actually drew for the Peek
    /// bar (the render milestone; `PortalRoot` calls this after consuming
    /// the app's live Peek surface). The probe commits
    /// `peek_request_to_present` samples only from this observation,
    /// including the truthful native z-order of the preview above its warm
    /// origin.
    #[cfg(feature = "testkit")]
    pub(crate) fn observe_portal_peek_render(&self) {
        let presented = self.peek_presented();
        let (view, source) = match self.controller.snapshot().preview {
            Some(preview) => (Some(preview.view), Some(preview.source)),
            None => (None, None),
        };
        let raised_above_origin = {
            #[cfg(target_os = "macos")]
            {
                match (self.find_engine.as_ref(), view, source) {
                    (Some(engine), Some(preview_view), Some(source_view)) => {
                        let engine = engine.borrow();
                        match (
                            engine.subview_index(preview_view),
                            engine.subview_index(source_view),
                        ) {
                            (Some(preview_index), Some(source_index)) => {
                                preview_index > source_index
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = (view, source);
                false
            }
        };
        self.performance
            .observe_peek_state(presented, view, source, raised_above_origin);
    }
}
