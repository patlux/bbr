//! Profile rename, colour and deletion on the [`Browser`] (blueprint D06,
//! DESIGN.md §12): the Settings › Profiles authorities.
//!
//! Deleting a profile removes its spaces and every tab in them — their
//! data store goes with the profile, so nothing is kept to reopen — and the
//! last profile cannot be deleted.

use crate::{Browser, DomainError, ProfileHue, ProfileId, SpaceId, TabId};

/// The outcome of [`Browser::delete_profile`]: the spaces and tabs that
/// went, and the first space of the first remaining profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedProfile {
    pub profile: ProfileId,
    pub spaces: Vec<SpaceId>,
    pub tabs: Vec<TabId>,
    pub fallback: SpaceId,
}

impl Browser {
    /// Rename `profile` (a trimmed, non-empty title; an empty one is
    /// ignored so a profile never loses its name).
    pub fn rename_profile(&mut self, profile: ProfileId, title: &str) -> Result<(), DomainError> {
        let entry = self
            .profiles
            .iter_mut()
            .find(|candidate| candidate.id == profile)
            .ok_or(DomainError::UnknownProfile(profile))?;
        let title = title.trim();
        if !title.is_empty() {
            entry.title = title.to_string();
        }
        Ok(())
    }

    /// Give `profile` another identity hue.
    pub fn set_profile_hue(
        &mut self,
        profile: ProfileId,
        hue: ProfileHue,
    ) -> Result<(), DomainError> {
        self.profiles
            .iter_mut()
            .find(|candidate| candidate.id == profile)
            .ok_or(DomainError::UnknownProfile(profile))?
            .hue = hue;
        Ok(())
    }

    /// Whether `profile` can be deleted (it exists and is not the last).
    pub fn can_delete_profile(&self, profile: ProfileId) -> bool {
        self.profiles.len() > 1
            && self
                .profiles
                .iter()
                .any(|candidate| candidate.id == profile)
    }

    /// Delete `profile` with its spaces and their tabs (recently-closed
    /// entries included). Unknown profiles and the last profile are rejected
    /// whole. The caller owns the sessions of the removed tabs and the
    /// active-space choice.
    pub fn delete_profile(&mut self, profile: ProfileId) -> Result<DeletedProfile, DomainError> {
        if !self
            .profiles
            .iter()
            .any(|candidate| candidate.id == profile)
        {
            return Err(DomainError::UnknownProfile(profile));
        }
        if self.profiles.len() < 2 {
            return Err(DomainError::LastProfile(profile));
        }
        let fallback_profile = self
            .profiles
            .iter()
            .find(|candidate| candidate.id != profile)
            .map(|candidate| candidate.id)
            .expect("another profile exists");
        let fallback = self
            .spaces
            .iter()
            .find(|space| space.profile == fallback_profile)
            .map(|space| space.id)
            .ok_or(DomainError::UnknownProfile(fallback_profile))?;
        let spaces: Vec<SpaceId> = self
            .spaces
            .iter()
            .filter(|space| space.profile == profile)
            .map(|space| space.id)
            .collect();
        let tabs: Vec<TabId> = self
            .tabs
            .iter()
            .filter(|tab| spaces.contains(&tab.space))
            .map(|tab| tab.id)
            .collect();
        self.tabs.retain(|tab| !spaces.contains(&tab.space));
        self.spaces.retain(|space| space.profile != profile);
        self.profiles.retain(|candidate| candidate.id != profile);
        Ok(DeletedProfile {
            profile,
            spaces,
            tabs,
            fallback,
        })
    }
}
