//! Per-Space appearance records, the companion store, and its errors.

use std::collections::HashSet;

use crate::{Browser, SpaceId, SpaceIdentity, SpaceTheme};

/// The complete per-Space appearance record: one curated theme token and
/// one validated identity.
///
/// The record is the **explicit override**: when no record exists, the
/// effective appearance is derived deterministically from the space's
/// owning profile identity hue (see [`SpaceAppearanceStore::effective`]).
/// Defaults never create records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceAppearance {
    /// The space this appearance belongs to.
    pub space: SpaceId,
    /// Curated theme-token palette entry.
    pub theme: SpaceTheme,
    /// Validated emoji-or-glyph identity.
    pub identity: SpaceIdentity,
}

impl SpaceAppearance {
    /// The canonical fallback appearance: default theme and default
    /// identity (used as the base when an explicit record is first created).
    pub fn default_for(space: SpaceId) -> Self {
        Self {
            space,
            theme: SpaceTheme::default(),
            identity: SpaceIdentity::default(),
        }
    }

    /// The appearance derived from a space's owning profile identity hue:
    /// the theme maps 1:1 from the hue ([`SpaceTheme::from_profile_hue`]),
    /// the identity stays the default glyph. This is what
    /// [`SpaceAppearanceStore::effective`] returns for spaces without an
    /// explicit record, which is how existing identity hues "go into" the
    /// theming mechanism without changing hue assignment.
    pub fn derived(space: SpaceId, hue: crate::ProfileHue) -> Self {
        Self {
            space,
            theme: SpaceTheme::from_profile_hue(hue),
            identity: SpaceIdentity::default(),
        }
    }
}

/// Errors returned by space-appearance operations.
///
/// Invalid input is rejected before any mutation, so a rejected operation
/// never partially changes the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppearanceError {
    /// The space does not exist in the live browser.
    UnknownSpace(SpaceId),
    /// The string is not a canonical member of the curated emoji allowlist.
    InvalidEmoji(String),
}

impl std::fmt::Display for AppearanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSpace(id) => write!(f, "unknown space {id:?}"),
            Self::InvalidEmoji(s) => {
                write!(f, "{s:?} is not in the curated emoji allowlist")
            }
        }
    }
}

impl std::error::Error for AppearanceError {}

/// A structural invariant violation found by
/// [`SpaceAppearanceStore::check_invariants`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppearanceInvariantViolation(pub String);

impl std::fmt::Display for AppearanceInvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "space-appearance invariant violated: {}", self.0)
    }
}

impl std::error::Error for AppearanceInvariantViolation {}

/// Companion store keeping per-Space appearance **beside** the
/// [`Browser`], following the [`FolderStore`](crate::FolderStore) pattern:
///
/// - [`Browser`] is the canonical owner of profiles/spaces/tabs and must
///   not be edited by an appearance lane, so appearance state lives beside
///   the browser and the orchestration layer owns both.
/// - The store validates every mutation against the browser's public read
///   API: appearance operations on an unknown space reject **whole** before
///   any mutation.
///
/// # Invariants
///
/// 1. Every recorded appearance references an existing space.
/// 2. Each space has at most one recorded appearance.
/// 3. Every identity is valid by construction ([`SpaceEmoji`](crate::SpaceEmoji) and
///    [`BbrGlyph`](crate::BbrGlyph) are closed, validated types) and the
///    theme is always a curated token.
///
/// Records keep deterministic assignment order. The invariant checker is
/// public and unit-tested so tampering is detectable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceAppearanceStore {
    /// Appearance records in deterministic assignment order.
    /// `pub(crate)` so the invariant checker's tamper-detection tests can
    /// forge violations (mutations themselves forbid them).
    pub(crate) appearances: Vec<SpaceAppearance>,
}

impl Default for SpaceAppearanceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SpaceAppearanceStore {
    pub fn new() -> Self {
        Self {
            appearances: Vec::new(),
        }
    }

    /// The explicit record for `space`, if one was set.
    pub fn appearance(&self, space: SpaceId) -> Option<&SpaceAppearance> {
        self.appearances.iter().find(|record| record.space == space)
    }

    /// The effective appearance for `space`:
    ///
    /// - the explicit record when present;
    /// - otherwise the deterministic default derived from the owning
    ///   profile's identity hue ([`SpaceAppearance::derived`]): theme =
    ///   the profile's hue family, identity = the default glyph;
    /// - `None` when `space` does not exist.
    ///
    /// Deterministic: identical browser + store state always yields the
    /// same value. Derived defaults never create records.
    pub fn effective(&self, browser: &Browser, space: SpaceId) -> Option<SpaceAppearance> {
        if let Some(record) = self.appearance(space) {
            return Some(record.clone());
        }
        let state = browser.space(space)?;
        let theme = browser
            .profile(state.profile)
            .map_or_else(SpaceTheme::default, |profile| {
                SpaceTheme::from_profile_hue(profile.hue)
            });
        Some(SpaceAppearance {
            space,
            theme,
            identity: SpaceIdentity::default(),
        })
    }

    /// Set the curated theme for `space`. Rejects unknown spaces whole.
    /// When no record exists yet, the record is created from the canonical
    /// defaults and only the theme changes (the derived identity is
    /// preserved).
    pub fn set_theme(
        &mut self,
        browser: &Browser,
        space: SpaceId,
        theme: SpaceTheme,
    ) -> Result<(), AppearanceError> {
        if browser.space(space).is_none() {
            return Err(AppearanceError::UnknownSpace(space));
        }
        self.entry_mut(space).theme = theme;
        Ok(())
    }

    /// Set the identity for `space`. Rejects unknown spaces whole; the
    /// existing theme is preserved.
    pub fn set_identity(
        &mut self,
        browser: &Browser,
        space: SpaceId,
        identity: SpaceIdentity,
    ) -> Result<(), AppearanceError> {
        if browser.space(space).is_none() {
            return Err(AppearanceError::UnknownSpace(space));
        }
        self.entry_mut(space).identity = identity;
        Ok(())
    }

    /// Set theme and identity atomically: both are validated against the
    /// same space, so either the whole record updates or nothing changes.
    pub fn set_appearance(
        &mut self,
        browser: &Browser,
        space: SpaceId,
        theme: SpaceTheme,
        identity: SpaceIdentity,
    ) -> Result<(), AppearanceError> {
        if browser.space(space).is_none() {
            return Err(AppearanceError::UnknownSpace(space));
        }
        let entry = self.entry_mut(space);
        entry.theme = theme;
        entry.identity = identity;
        Ok(())
    }

    /// Remove the explicit record so the effective appearance falls back to
    /// the derived defaults ("reset removes an override"). Resetting a
    /// space without a record is a no-op; the space itself must exist.
    pub fn reset(&mut self, browser: &Browser, space: SpaceId) -> Result<(), AppearanceError> {
        if browser.space(space).is_none() {
            return Err(AppearanceError::UnknownSpace(space));
        }
        self.appearances.retain(|record| record.space != space);
        Ok(())
    }

    /// The explicit record for `space`, creating it from the canonical
    /// defaults when absent. Callers must already have established that
    /// `space` exists (all public mutations validate first).
    fn entry_mut(&mut self, space: SpaceId) -> &mut SpaceAppearance {
        if let Some(index) = self
            .appearances
            .iter()
            .position(|record| record.space == space)
        {
            return &mut self.appearances[index];
        }
        self.appearances.push(SpaceAppearance::default_for(space));
        self.appearances
            .last_mut()
            .expect("appearance pushed above")
    }

    /// Explicit records in deterministic assignment order.
    pub fn iter(&self) -> impl Iterator<Item = &SpaceAppearance> {
        self.appearances.iter()
    }

    /// Number of explicit appearance records.
    pub fn len(&self) -> usize {
        self.appearances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.appearances.is_empty()
    }

    /// Validate every invariant (see module docs) against the live browser.
    pub fn check_invariants(&self, browser: &Browser) -> Result<(), AppearanceInvariantViolation> {
        let fail = |message: String| AppearanceInvariantViolation(message);
        let mut seen: HashSet<SpaceId> = HashSet::new();
        for record in &self.appearances {
            if browser.space(record.space).is_none() {
                return Err(fail(format!(
                    "appearance references unknown space {:?}",
                    record.space
                )));
            }
            if !seen.insert(record.space) {
                return Err(fail(format!(
                    "duplicate appearance for space {:?}",
                    record.space
                )));
            }
        }
        Ok(())
    }
}
