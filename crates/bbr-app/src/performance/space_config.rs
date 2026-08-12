//! Space-configuration surface presentation identity for the presentation
//! probe (packet `space-creation-appearance-ui.md`, DESIGN.md §11.3).
//!
//! The `space_create_present` / `space_config_identity_update` benchmarks
//! measure the production create→activate→configure lifecycle and the
//! exact-SpaceId identity commit, and commit only from a *render* of the
//! configuration surface that carried the armed presentation: the surface
//! drawn open bound to the exact created persistent `SpaceId`
//! (`SpaceCreatePresent`), or the surface drawn closed after committing
//! the identity for the exact bound `SpaceId` with the bound space still
//! active and the same warm session (`SpaceConfigIdentityUpdate`).
//!
//! The observation is render-gated — the app's `space_config_surface`
//! render records the surface state it actually drew into the probe on
//! every frame (open/closed plus the bound SpaceId and the drawn
//! theme/identity), so a model mutation alone can never emit a measurement
//! and a frame that omitted the surface can never commit one. A mismatched
//! or stale observation rejects the sample in the app (fast-but-wrong
//! guard).
//!
//! Execution stays unexecuted/unpassed under Patrick's active override;
//! this module gates the presentation commit for when execution resumes.
//! It reaches the probe's private `State`/`Pending` through parent-module
//! privacy, exactly like `super::downloads`.

use bbr_testkit::protocol::TestSpaceId;

use super::{Pending, PerformanceProbe, State};

/// The exact space-configuration presentation a pending sample must commit
/// from.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum SpaceConfigExpectation {
    /// The surface rendered open bound to the exact persistent SpaceId
    /// (the create→activate→configure presentation).
    Open { space: TestSpaceId },
    /// The surface is not rendered (the post-commit presentation; the
    /// commit's exact-SpaceId/identity effect on the chip is re-verified
    /// from the typed snapshot by the driver).
    Closed,
}

/// The space-configuration presentation the sidebar render actually drew on
/// the last frame.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpaceConfigObservation {
    /// Whether the surface rendered (open). Closed frames record `false`
    /// so a post-commit sample can commit from a frame that omitted the
    /// surface.
    pub(super) open: bool,
    /// The bound exact persistent SpaceId (captured at open; the
    /// render-gated exact-SpaceId guard). `None` while closed.
    pub(super) space: Option<TestSpaceId>,
    /// The drawn curated theme name.
    pub(super) theme: String,
    /// The drawn curated identity name.
    pub(super) identity: String,
}

impl PerformanceProbe {
    /// Record the space-configuration presentation the sidebar render
    /// actually drew (render-gated, every frame).
    pub fn observe_space_config(
        &self,
        open: bool,
        space: Option<TestSpaceId>,
        theme: impl Into<String>,
        identity: impl Into<String>,
    ) {
        let mut state = self.inner.borrow_mut();
        state.latest_space_config = Some(SpaceConfigObservation {
            open,
            space,
            theme: theme.into(),
            identity: identity.into(),
        });
    }

    /// Require the pending presentation to have drawn the surface open
    /// bound to the exact persistent `space` (the create→activate→
    /// configure presentation).
    pub fn expect_space_config_open(&self, space: TestSpaceId) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.space_config = Some(SpaceConfigExpectation::Open { space });
        }
    }

    /// Require the pending presentation to have drawn the surface closed
    /// (the post-commit presentation).
    pub fn expect_space_config_closed(&self) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.space_config = Some(SpaceConfigExpectation::Closed);
        }
    }
}

/// The pending sample's surface expectation against the latest observation.
pub(super) fn space_config_matches(pending: &Pending, state: &State) -> bool {
    let Some(expected) = pending.space_config.as_ref() else {
        return true;
    };
    let Some(observed) = state.latest_space_config.as_ref() else {
        return false;
    };
    match expected {
        SpaceConfigExpectation::Open { space } => observed.open && observed.space == Some(*space),
        SpaceConfigExpectation::Closed => !observed.open,
    }
}
