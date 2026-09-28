//! Space-appearance commands on the [`super::BrowserController`]
//! (portable appearance contract): read/validate the curated theme +
//! identity record of a space beside the browser model. Split from
//! `controller.rs` so the gateway module stays under the ~500-line budget.

use bbr_domain::{AppearanceError, SpaceAppearance, SpaceIdentity, SpacePaper, SpaceTheme};

use super::BrowserController;

impl BrowserController {
    /// The effective appearance of `space`: the explicit record when one was
    /// set, otherwise the deterministic default derived from the owning
    /// profile's identity hue (theme = the profile's hue family, identity =
    /// the default curated glyph). `None` when the space does not exist.
    pub fn space_appearance(&self, space: bbr_domain::SpaceId) -> Option<SpaceAppearance> {
        self.appearances.effective(&self.browser, space)
    }

    /// Set the curated theme token and validated identity of `space`
    /// atomically (both validated against the same space; unknown spaces
    /// reject whole before any mutation).
    pub fn set_space_appearance(
        &mut self,
        space: bbr_domain::SpaceId,
        theme: SpaceTheme,
        identity: SpaceIdentity,
    ) -> Result<(), AppearanceError> {
        self.appearances
            .set_appearance(&self.browser, space, theme, identity)?;
        self.check_invariants();
        Ok(())
    }

    /// Set only the curated theme token of `space` (the identity stays the
    /// derived default or the existing explicit record's).
    pub fn set_space_theme(
        &mut self,
        space: bbr_domain::SpaceId,
        theme: SpaceTheme,
    ) -> Result<(), AppearanceError> {
        self.appearances.set_theme(&self.browser, space, theme)?;
        self.check_invariants();
        Ok(())
    }

    /// Set only the curated paper (tint intensity + static grain) of
    /// `space` (theme and identity stay as they are).
    pub fn set_space_paper(
        &mut self,
        space: bbr_domain::SpaceId,
        paper: SpacePaper,
    ) -> Result<(), AppearanceError> {
        self.appearances.set_paper(&self.browser, space, paper)?;
        self.check_invariants();
        Ok(())
    }

    /// Restore a persisted appearance after a restore: a record is kept only
    /// when it differs from the derived default (defaults never create
    /// records). Unknown spaces reject whole.
    pub fn restore_space_appearance(
        &mut self,
        appearance: SpaceAppearance,
    ) -> Result<(), AppearanceError> {
        self.appearances.restore(&self.browser, appearance)?;
        self.check_invariants();
        Ok(())
    }

    /// Set only the validated identity of `space` (the theme stays the
    /// derived default or the existing explicit record's).
    pub fn set_space_identity(
        &mut self,
        space: bbr_domain::SpaceId,
        identity: SpaceIdentity,
    ) -> Result<(), AppearanceError> {
        self.appearances
            .set_identity(&self.browser, space, identity)?;
        self.check_invariants();
        Ok(())
    }
}
