//! Testkit-only auth benchmark driver (queue item 10).
//!
//! `benchmark_auth_sheet_open` measures the production presentation of the
//! credential sheet (the exact path a challenge arrival drives): the sample
//! commits only from a render of the one `NativeAboveWeb` portal window
//! that drew the sheet with the armed challenge id (fast-but-wrong guard —
//! a frame that omitted the sheet, or drew a different challenge, can never
//! commit). The measured entry is `auth_present_sheet`, so fixture time
//! (the awaiting challenge) is excluded by construction.
//!
//! Redaction: the probe observation carries only the challenge id and
//! masked entered flags — the typed username/password never enter the
//! probe, the measurement, or any report shape.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use gpui::{App, AsyncApp, Context, Window};

use super::auth::AuthSheetSurface;
use super::BbrApp;

impl BbrApp {
    /// Benchmark the production credential-sheet open: the sample starts at
    /// `auth_present_sheet` (the presentation path a challenge arrival
    /// drives) and commits only from a render of the portal window that
    /// drew the sheet presenting the armed challenge id. The awaiting
    /// challenge is the fixture (recorded before the sample, never part of
    /// the measured duration). The active view/sidebar are unchanged and
    /// the sheet owns keyboard focus (chrome released); the armed
    /// expectations reject any render that omits the sheet or touches the
    /// session.
    pub fn benchmark_auth_sheet_open(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::AuthSheetOpen)
            .is_err()
        {
            return false;
        }
        // Production presentation entry (the exact seam `auth_on_challenge`
        // drives after recording the challenge).
        if !self.auth_present_sheet(window, cx) {
            self.performance.cancel();
            return false;
        }
        let Some(challenge) = self.auth.presented().map(|id| id.0) else {
            self.performance.cancel();
            return false;
        };
        let active_view = self.controller.snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        // Fast-but-wrong guard: the same active view stays selected, the
        // sidebar stays visible, the sheet owns focus (chrome released), and
        // no command surface is active. Refresh the probe from the
        // authoritative post-mutation state so the presentation commit sees
        // the real state (the sheet render has not painted yet).
        self.performance
            .observe_render_state(active_view, self.sidebar_visible, false, mode);
        self.performance
            .expect_state(active_view, self.sidebar_visible, false, mode);
        self.performance.expect_auth_sheet(challenge);
        // Render-inclusive commit: draw the portal window (the sheet renders
        // there) so the render records the drawn sheet state; the probe then
        // commits only if the armed challenge identity and browser state
        // match.
        self.schedule_auth_render_commit(cx);
        true
    }

    /// Schedule the render-inclusive commit for a pending auth benchmark: a
    /// bounded forced draw of the portal window (the sheet renders in the
    /// one `NativeAboveWeb` host, so the main-window draw used by the other
    /// command benchmarks cannot present it), followed by the explicit
    /// commit through the app-entity contract. The draw runs on the main
    /// thread after this update completes, outside the mutable `BbrApp`
    /// lease (the same reentrancy rule as `dispatch_portal`).
    pub(crate) fn schedule_auth_render_commit(&self, cx: &mut Context<Self>) {
        if !matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::AuthSheetOpen)
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
                // Draw the portal window: `PortalRoot` renders the sheet
                // from the app's live redacted surface and records the
                // drawn sheet state into the probe (the render milestone). A
                // render that omitted the sheet records nothing and can
                // never commit.
                let _ = portal.update(cx, |_root, window, cx| {
                    let app: &mut App = cx;
                    let _ = window.draw(app);
                });
                // Post-render commit through the app-entity contract: the
                // explicit path validates the armed identity + browser state
                // and cancels truthfully when the render did not present the
                // sheet (no observation → no commit).
                let _: bool = weak
                    .update(cx, |this, _cx| this.commit_auth_benchmark())
                    .unwrap_or(false);
            });
        })
        .detach();
    }

    /// Commit the pending auth sample from the render milestone: the probe
    /// compares the armed sheet identity against the observation the portal
    /// render recorded and the browser-state expectations against the state
    /// observed at the mutation site. Returns false and cancels when the
    /// render omitted the sheet or the identity/state was wrong — a
    /// truthful rejection, never a fast-but-wrong pass.
    fn commit_auth_benchmark(&self) -> bool {
        self.performance.commit_current_presentation();
        if matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::AuthSheetOpen)
        ) {
            self.performance.cancel();
            return false;
        }
        true
    }

    /// Record the redacted sheet state the portal render actually drew for
    /// the credential sheet (the render milestone; `PortalRoot` calls this
    /// after consuming the app's live `AuthSheetSurface`). The probe
    /// commits auth samples only from this observation; credentials never
    /// reach the probe — only the challenge id and masked flags.
    pub(crate) fn observe_portal_auth_render(&self, surface: &AuthSheetSurface) {
        self.performance.observe_auth_sheet(
            true,
            self.auth.presented().map(|id| id.0),
            Some(surface.focused.label()),
            surface.username_entered(),
            surface.password_entered,
        );
    }
}
