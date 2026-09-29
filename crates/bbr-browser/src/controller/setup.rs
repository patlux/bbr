//! Profile/space setup helpers on the [`super::BrowserController`]
//! (multi-profile tests and composition): typed profile/space creation,
//! activation, and the active-profile accessor. Split from `controller.rs`
//! so the gateway module stays under the ~500-line budget.

use bbr_domain::{SpaceId, SpaceMove};
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
        // The destination's visible set is shown now: a session-less
        // (restored/suspended) visible tab wakes, and `sync_views` below
        // recreates exactly its session.
        self.browser.wake_visible(space)?;
        // A preview belongs to its source space; switching away ends it.
        self.close_preview_if_stale();
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::SpaceActivated { space })
    }

    /// Blueprint D05 (space menu Move Left/Right): swap `space` with its
    /// same-profile neighbor in display order. Tabs, selection and
    /// sessions stay untouched; a move past the group's edge is rejected.
    pub fn move_space(
        &mut self,
        space: SpaceId,
        direction: SpaceMove,
    ) -> Result<ControllerEvent, ControllerError> {
        self.browser.move_space(space, direction)?;
        self.check_invariants();
        let to = self
            .browser
            .spaces()
            .position(|candidate| candidate.id == space)
            .expect("moved space exists");
        Ok(ControllerEvent::SpaceMoved { space, to })
    }

    /// Blueprint D05 (space menu Delete Space…): delete `space`. Every
    /// session of its tabs is destroyed, the tabs close onto the fallback
    /// space's recently-closed stack (Reopen Closed Tab is the undo), its
    /// appearance record is dropped, and — when it was active — the
    /// fallback becomes the active space. The last space of a profile is
    /// rejected whole.
    pub fn delete_space(&mut self, space: SpaceId) -> Result<ControllerEvent, ControllerError> {
        if !self.browser.can_delete_space(space) {
            // Validate before any session teardown (unknown or last space).
            self.browser.delete_space(space)?;
        }
        let members = self
            .browser
            .space(space)
            .map(|state| state.tabs.clone())
            .unwrap_or_default();
        for tab in &members {
            self.destroy_session_of(*tab);
            self.provenance.invalidate_for(*tab);
        }
        self.appearances
            .reset(&self.browser, space)
            .expect("the space exists until deletion");
        let deleted = self.browser.delete_space(space)?;
        let activated = self.active_space == space;
        if activated {
            self.active_space = deleted.fallback;
            self.browser.wake_visible(deleted.fallback)?;
        }
        self.close_preview_if_stale();
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::SpaceDeleted {
            space,
            fallback: deleted.fallback,
            closed_tabs: deleted.closed.len(),
            activated,
        })
    }

    /// Blueprint D06: rename `profile` (an empty title is ignored).
    pub fn rename_profile(
        &mut self,
        profile: ProfileId,
        title: &str,
    ) -> Result<(), ControllerError> {
        self.browser.rename_profile(profile, title)?;
        Ok(())
    }

    /// Blueprint D06: give `profile` another identity hue.
    pub fn set_profile_hue(
        &mut self,
        profile: ProfileId,
        hue: bbr_domain::ProfileHue,
    ) -> Result<(), ControllerError> {
        self.browser.set_profile_hue(profile, hue)?;
        Ok(())
    }

    pub fn can_delete_profile(&self, profile: ProfileId) -> bool {
        self.browser.can_delete_profile(profile)
    }

    /// Blueprint D06 (Settings › Profiles › Delete…): delete `profile` with
    /// its spaces and tabs. Every session of those tabs is destroyed and
    /// their appearance records dropped; when the active space belonged to
    /// the profile, the first space of the first remaining profile becomes
    /// active. The last profile is rejected whole. The caller removes the
    /// profile's website data store.
    pub fn delete_profile(
        &mut self,
        profile: ProfileId,
    ) -> Result<ControllerEvent, ControllerError> {
        if !self.browser.can_delete_profile(profile) {
            self.browser.delete_profile(profile)?;
        }
        let spaces: Vec<SpaceId> = self
            .browser
            .spaces()
            .filter(|space| space.profile == profile)
            .map(|space| space.id)
            .collect();
        for space in &spaces {
            let members = self
                .browser
                .space(*space)
                .map(|state| state.tabs.clone())
                .unwrap_or_default();
            for tab in &members {
                self.destroy_session_of(*tab);
                self.provenance.invalidate_for(*tab);
            }
            let _ = self.appearances.reset(&self.browser, *space);
        }
        let deleted = self.browser.delete_profile(profile)?;
        let activated = spaces.contains(&self.active_space);
        if activated {
            self.active_space = deleted.fallback;
            self.browser.wake_visible(deleted.fallback)?;
        }
        self.close_preview_if_stale();
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::ProfileDeleted {
            profile,
            fallback: deleted.fallback,
            spaces: deleted.spaces.len(),
            closed_tabs: deleted.tabs.len(),
            activated,
        })
    }

    /// Whether `space` can move `direction` / be deleted (the space menu's
    /// availability).
    pub fn can_move_space(&self, space: SpaceId, direction: SpaceMove) -> bool {
        self.browser.can_move_space(space, direction)
    }

    pub fn can_delete_space(&self, space: SpaceId) -> bool {
        self.browser.can_delete_space(space)
    }

    /// Whether Reopen Closed Tab has something to reopen in the active space
    /// (blueprint D04: the empty ⌘T offers the command only then).
    pub fn can_reopen_tab(&self) -> bool {
        self.browser
            .space(self.active_space)
            .is_some_and(|space| !space.recently_closed.is_empty())
    }

    /// The profile backing the active space.
    pub fn active_profile(&self) -> ProfileId {
        self.browser
            .space(self.active_space)
            .expect("active space exists")
            .profile
    }
}
