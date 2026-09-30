//! Testkit-only benchmark drivers for the Space-configuration surface
//! (DESIGN.md §11.3, packet `space-creation-appearance-ui.md`).
//!
//! Both drivers run the production app authorities — `commit_new_space` and
//! `open_space_config`/`space_config_select_identity`/`space_config_done` —
//! never a controller or model bypass. They are the measurement seams the
//! pending Space create/present and identity-update scenarios drive; the
//! sample commits only from a render that drew the required surface state
//! (render-gated exact-SpaceId guards).
//!
//! Split from the sibling surface-behavior module (the parent
//! [`super`] file) by concern: the surface state/draft/commit/keyboard/
//! render authorities stay with the app behavior while every benchmark
//! seam lives here, so both files stay under the ~500-line budget.

use bbr_browser::SpaceId;
use gpui::{Context, Window};

use crate::app::BbrApp;

/// §11.3 create→activate→configure benchmark driver (testkit-only; packet
/// `space-creation-appearance-ui.md`). Runs the production `commit_new_space`
/// path — creation commits before configuration, the new Space activates
/// immediately, and its optional name/icon surface opens bound to the exact
/// new persistent SpaceId. The sample commits only from a render that drew
/// the configuration surface open bound to the created SpaceId (render-gated
/// exact-SpaceId guard) with the created chip active — a space that never
/// activates, a surface bound to a stale/other id, or a missing surface
/// rejects the measurement in the app. Warm-presentation class: no tab or
/// session is created or touched.
#[cfg(feature = "testkit")]
impl BbrApp {
    pub fn benchmark_create_space(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SpaceCreatePresent,
            )
            .is_err()
        {
            return false;
        }
        // The production create→activate→configure path (the same path the
        // plus-menu and keyboard creation entries share).
        let outcome = self.commit_new_space(name, window, cx);
        let Some(bbr_browser::ControllerEvent::SpaceCreated { space, .. }) = outcome else {
            self.performance.cancel();
            return false;
        };
        let created = bbr_testkit::protocol::TestSpaceId(space.0);
        let snapshot = self.controller.read().snapshot();
        self.performance.observe_active_space(created);
        self.performance.expect_space(created);
        self.performance.expect_state(
            snapshot.active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            bbr_testkit::protocol::CommandBarMode::Inactive,
        );
        // The render-gated exact-SpaceId guard: the sample commits only
        // from a frame that drew the configuration surface open bound to
        // the exact created SpaceId.
        self.performance.expect_space_config_open(created);
        self.performance.commit_current_presentation();
        true
    }

    /// §11.3 identity-commit benchmark driver (testkit-only; packet
    /// `space-creation-appearance-ui.md`): open the configuration surface
    /// for the exact persistent `space`, select the curated `identity`, and
    /// commit through the production `space_config_done` authority (never a
    /// controller bypass). The sample commits only from a render that drew
    /// the surface closed with `space` still active and the same warm
    /// session — the commit's exact-SpaceId/identity effect on the chip is
    /// re-verified from the typed snapshot by the driver. Warm-presentation
    /// class: no tab or session is created or touched.
    pub fn benchmark_space_config_identity_update(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        space: SpaceId,
        identity: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SpaceConfigIdentityUpdate,
            )
            .is_err()
        {
            return false;
        }
        let Some(resolved) = resolve_space_identity(identity) else {
            self.performance.cancel();
            return false;
        };
        if !self.open_space_config(space, window, cx) {
            self.performance.cancel();
            return false;
        }
        self.space_config_select_identity(resolved, cx);
        self.space_config_done(window, cx);
        let bound = bbr_testkit::protocol::TestSpaceId(space.0);
        let snapshot = self.controller.read().snapshot();
        self.performance.observe_active_space(bound);
        self.performance.expect_space(bound);
        self.performance.expect_state(
            snapshot.active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            bbr_testkit::protocol::CommandBarMode::Inactive,
        );
        // The render-gated guard: the sample commits only from a frame that
        // drew the surface closed (the commit's effect on the exact chip is
        // re-verified from the typed snapshot by the driver).
        self.performance.expect_space_config_closed();
        self.performance.commit_current_presentation();
        true
    }
}

/// Resolve a curated identity string (a validated emoji scalar or a BBR
/// glyph name) to its [`bbr_domain::SpaceIdentity`]; `None` for anything
/// outside the closed curated catalogs (the benchmark seam never accepts a
/// free-form icon value).
#[cfg(feature = "testkit")]
fn resolve_space_identity(identity: &str) -> Option<bbr_domain::SpaceIdentity> {
    bbr_domain::SpaceEmoji::new(identity)
        .map(bbr_domain::SpaceIdentity::Emoji)
        .ok()
        .or_else(|| {
            bbr_domain::BbrGlyph::ALL
                .iter()
                .find(|glyph| glyph.name() == identity)
                .copied()
                .map(bbr_domain::SpaceIdentity::Glyph)
        })
}
