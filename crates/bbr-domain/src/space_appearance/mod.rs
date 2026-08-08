//! Per-Space appearance: curated theme tokens and validated emoji-or-glyph
//! identity (product spec `bbr-spaces-theming-settings-spec.md` §1).
//!
//! This module is the portable `bbr-domain` contract for space theming. It
//! defines **types, defaults, and invariants only**: it does not attach
//! fields to [`Space`](crate::Space), does not touch persistence, and never
//! renders anything. Concrete surface/tint colors, glyph paths, and emoji
//! presentation are later UI slices that resolve this contract's stable
//! names.
//!
//! - [`SpaceTheme`] — the curated stable palette. v1 offers no free color
//!   picker; the six DESIGN §12 profile identity hues are a canonical subset
//!   ([`SpaceTheme::from_profile_hue`]), so existing round-robin profile
//!   identity "goes into" this mechanism without changing assignment.
//! - [`SpaceEmoji`] / [`BbrGlyph`] / [`SpaceIdentity`] — validated emoji
//!   **or** BBR icon-glyph identity. Exactly one of the two variants is
//!   representable, and every value is closed and validated by construction:
//!   emoji must be a member of the curated [`ALLOWED_EMOJI`] allowlist,
//!   glyphs are a fixed catalog of stable names.
//! - [`SpaceAppearance`] / [`SpaceAppearanceStore`] — the explicit
//!   per-space appearance record and its companion store beside the
//!   [`Browser`](crate::Browser) (the [`FolderStore`](crate::FolderStore)
//!   pattern). Effective appearance derives deterministic defaults from the
//!   owning profile's identity hue when no explicit record exists; defaults
//!   never create records, and reset removes an override.
//!
//! Invariants are enforced at the store boundary (unknown spaces reject
//! whole) and detectable through [`SpaceAppearanceStore::check_invariants`],
//! which is public and unit-tested so tampering is detectable.

mod identity;
mod palette;
mod store;

pub use identity::{BbrGlyph, IdentityKind, SpaceEmoji, SpaceIdentity, ALLOWED_EMOJI};
pub use palette::SpaceTheme;
pub use store::{
    AppearanceError, AppearanceInvariantViolation, SpaceAppearance, SpaceAppearanceStore,
};

#[cfg(test)]
mod tests;
