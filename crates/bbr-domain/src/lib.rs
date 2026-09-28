//! # bbr-domain — portable browser core model
//!
//! Owning model for the browser's persistent objects: [`Profile`], [`Space`],
//! [`Tab`], and the two-pane [`Split`] layout, plus the [`Browser`] container
//! that enforces their invariants.
//!
//! This crate is **fully portable**: no GPUI, no macOS frameworks, no
//! SQLite, no `unsafe`, and no native pointers. Runtime engine state
//! (live WKWebViews) is deliberately kept elsewhere, keyed by a separate
//! `WebViewId` (`bbr-web`); the domain only carries stable page identities.
//!
//! Ownership rules (PLAN.md §3):
//!
//! - A [`Profile`] is a browsing identity backed by an isolated website
//!   data store. Profile identity reuses [`bbr_web::ProfileId`], the same
//!   type the engine adapters map to a persistent data store.
//! - A [`Space`] belongs to exactly one profile.
//! - A [`Tab`] belongs to exactly one space (its profile is derived from
//!   the space, never stored on the tab).
//! - A [`Split`] references two tabs of the same space.
//!
//! ## Invariants enforced by [`Browser`]
//!
//! 1. Every space references an existing profile.
//! 2. Every tab referenced by a space exists and points back at that space.
//! 3. Pinned tabs always precede unpinned tabs in a space's ordering.
//! 4. The active tab of a space is a member of that space (or absent when
//!    the space is empty).
//! 5. A split's two tabs belong to the space and differ from each other.
//! 6. Recently closed entries are not members of their space.
//!
//! Mutating operations preserve all invariants by construction; the
//! invariant checker is public and unit-tested so tampering is detectable.

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

mod browser;
mod error;
mod folders;
mod invariants;
mod lifecycle;
mod model;
mod saved_pages;
mod space_appearance;
mod space_lifecycle;
mod split;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_dense;
#[cfg(test)]
mod tests_folders;
#[cfg(test)]
mod tests_layout;
#[cfg(test)]
mod tests_restore;
#[cfg(test)]
mod tests_saved_pages;
#[cfg(test)]
mod tests_space_lifecycle;

pub use browser::Browser;
pub use error::{DomainError, InvariantViolation};
pub use folders::{
    Favorite, FavoriteToggle, Folder, FolderError, FolderId, FolderInvariantViolation, FolderStore,
    TabFacts,
};
pub use lifecycle::{LifecycleError, TabLifecycle};
pub use model::{ClosedTab, Profile, ProfileHue, Space, Tab};
pub use saved_pages::{
    canonical_url, deterministic_order, validate_collection, BookmarkId, CanonicalUrl, Collection,
    CollectionId, SavedPage, SavedPageError, MAX_CAPTURED_TITLE_LEN, MAX_COLLECTION_TITLE_LEN,
    MAX_USER_LABEL_LEN,
};
pub use space_appearance::{
    AppearanceError, AppearanceInvariantViolation, BbrGlyph, IdentityKind, SpaceAppearance,
    SpaceAppearanceStore, SpaceEmoji, SpaceIdentity, SpacePaper, SpaceTheme, TintIntensity,
    ALLOWED_EMOJI,
};
pub use space_lifecycle::{DeletedSpace, SpaceMove};
pub use split::Split;

pub use bbr_web::ProfileId;

/// Stable identifier of a workspace attached to one profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpaceId(pub u64);

/// Stable identifier of a persistent page identity inside one space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TabId(pub u64);
