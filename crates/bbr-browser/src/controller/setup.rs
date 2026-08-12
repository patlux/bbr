//! Profile/space setup helpers on the [`super::BrowserController`]
//! (multi-profile tests and composition): typed profile/space creation,
//! activation, and the active-profile accessor. Split from `controller.rs`
//! so the gateway module stays under the ~500-line budget.

use bbr_domain::SpaceId;
use bbr_web::ProfileId;

use crate::outcome::ControllerEvent;
use crate::ControllerError;

use super::BrowserController;

impl BrowserController {
    pub fn create_profile(&mut self, title: impl Into<String>) -> ProfileId {
        self.browser.create_profile(title)
    }

    /// Create a profile with one initial empty space, ready for activation.
    pub fn create_profile_space(
        &mut self,
        profile_title: impl Into<String>,
        space_title: impl Into<String>,
    ) -> Result<ControllerEvent, ControllerError> {
        let profile = self.browser.create_profile(profile_title);
        let space = self.browser.create_space(profile, space_title)?;
        self.check_invariants();
        Ok(ControllerEvent::ProfileCreated { profile, space })
    }

    pub fn create_space(
        &mut self,
        profile: ProfileId,
        title: impl Into<String>,
    ) -> Result<SpaceId, ControllerError> {
        Ok(self.browser.create_space(profile, title)?)
    }

    /// Create another space in an existing profile with a typed outcome.
    pub fn create_space_event(
        &mut self,
        profile: ProfileId,
        title: impl Into<String>,
    ) -> Result<ControllerEvent, ControllerError> {
        let space = self.browser.create_space(profile, title)?;
        self.check_invariants();
        Ok(ControllerEvent::SpaceCreated { profile, space })
    }

    /// Rename the existing space `space` through the one domain authority
    /// (the Space-creation-and-configuration surface's rename seam).
    /// Rejects unknown spaces whole before any mutation; profile ownership,
    /// tab order, and warm sessions stay untouched. The title change is
    /// immediately readable by the next snapshot — there is no second
    /// title authority.
    pub fn rename_space(
        &mut self,
        space: SpaceId,
        title: impl Into<String>,
    ) -> Result<(), ControllerError> {
        self.browser.rename_space(space, title)?;
        self.check_invariants();
        Ok(())
    }

    /// The current title of `space` (the configuration surface's prefill
    /// source; `None` for unknown spaces).
    pub fn space_title(&self, space: SpaceId) -> Option<String> {
        self.browser.space(space).map(|state| state.title.clone())
    }

    /// Switch the workspace whose tabs are visible and receive commands.
    pub fn activate_space(&mut self, space: SpaceId) -> Result<ControllerEvent, ControllerError> {
        if self.browser.space(space).is_none() {
            return Err(ControllerError::UnknownSpace(space));
        }
        self.active_space = space;
        // A preview belongs to its source space; switching away ends it.
        self.close_preview_if_stale();
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::SpaceActivated { space })
    }

    /// The profile backing the active space.
    pub fn active_profile(&self) -> ProfileId {
        self.browser
            .space(self.active_space)
            .expect("active space exists")
            .profile
    }
}
