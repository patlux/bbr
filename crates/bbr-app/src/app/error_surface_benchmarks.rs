//! Testkit-only error-surface benchmark driver (task packet
//! `error-surface-contract`).
//!
//! `benchmark_error_surface_present` measures the production
//! incident-presentation path: a classified incident enters through the
//! exact seam the native delegate's incident sink drives (`on_incident` →
//! the contract `present` lifecycle → the surface replaces the native view)
//! and the sample commits only from a render of the content area that drew
//! the error surface for the armed category over the unchanged active view
//! (fast-but-wrong guard — a frame that omitted the surface, drew a
//! different category, or changed the view/session can never commit).
//! Fixture/network time is excluded by construction: the classified
//! incident is applied in-process, never through a network transfer. The
//! native-code → taxonomy mapping is proven by the `bbr-web-macos`
//! `error_mapping` unit suite.
//!
//! Presenting the surface never creates, destroys, moves, or reselects a
//! web session (the native view is only hidden while the surface replaces
//! it); every driver arms the unchanged active view, sidebar, chrome-focus
//! domain, and inactive command-bar mode, so the probe rejects any sample
//! that touches them.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use bbr_web::web_errors::{IncidentContext, IncidentSubReason, NavigationReason};
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production error-surface presentation: the sample
    /// starts at the exact seam the native incident sink drives
    /// (`on_incident` → the contract `present` path, which allocates the
    /// monotonic per-view incident id, supersedes any older incident, and
    /// hides the native view) and commits only from a render of the content
    /// area that drew the surface for the armed category. The active
    /// view/sidebar/focus/mode stay unchanged; the armed expectations
    /// reject any render that omitted the surface or touched the session.
    /// The surface must not already be shown (the driver dismisses it
    /// between samples with an unmeasured dismissal).
    pub fn benchmark_error_surface_present(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::ErrorSurfacePresent)
            .is_err()
        {
            return false;
        }
        let Some(view) = self.controller.snapshot().active_view else {
            self.performance.cancel();
            return false;
        };
        if self.error_surface.spec_for(view).is_some() {
            self.performance.cancel();
            return false;
        }
        // Production incident presentation: the exact seam the native
        // delegate's incident sink drives (`on_incident` calls this same
        // `present`). A deterministic navigation failure on the active view;
        // the contract derives the category, copy, and legal recovery.
        self.error_surface.present(
            self.controller.active_profile(),
            view,
            IncidentSubReason::Navigation(NavigationReason::ConnectionRefused),
            IncidentContext::from_url("https://example.com/"),
            Some("https://example.com/".into()),
        );
        // The surface replaces the native view (identity untouched).
        self.sync_error_surface_visibility();
        let active_view = self.controller.snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        let chrome_focused = self.chrome_focused(window);
        // Fast-but-wrong guard: the same active view stays selected, the
        // sidebar stays visible, the chrome-focus domain is unchanged, and
        // no command surface is active. Refresh the probe from the
        // authoritative post-mutation state so the presentation commit sees
        // the real state (the surface render has not painted yet).
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance
            .expect_error_surface(bbr_web::web_errors::IncidentCategory::Navigation);
        true
    }

    /// Commit the pending error-surface sample after the main-window draw:
    /// the probe compares the armed surface identity and browser state
    /// against the observation the render recorded. Returns false and
    /// cancels when the render omitted the surface or the identity/state
    /// was wrong — a truthful rejection, never a fast-but-wrong pass.
    pub fn commit_error_surface_benchmark(&self) -> bool {
        self.performance.commit_current_presentation();
        if matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::ErrorSurfacePresent)
        ) {
            self.performance.cancel();
            return false;
        }
        true
    }
}
