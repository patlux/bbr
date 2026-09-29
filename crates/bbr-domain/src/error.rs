//! Typed domain-operation and invariant errors.

use crate::{ProfileId, SpaceId, TabId, TabLifecycle};

/// Errors returned by browser-domain operations.
///
/// Invalid input is rejected before mutation; structural corruption is
/// reported separately through [`InvariantViolation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    UnknownProfile(ProfileId),
    UnknownSpace(SpaceId),
    UnknownTab(TabId),
    TabNotInSpace {
        tab: TabId,
        space: SpaceId,
    },
    IndexOutOfBounds {
        index: usize,
        len: usize,
    },
    ReorderAcrossPinnedBoundary,
    /// A new regular (unpinned) tab cannot be inserted inside the pinned
    /// region (`index < pinned_count`); the background-link child path must
    /// land in the regular region.
    InsertionAcrossPinnedBoundary {
        index: usize,
        pinned: usize,
    },
    SplitRequiresDistinctTabs,
    SplitAlreadyExists(SpaceId),
    /// Blueprint D23: a split-group operation was refused.
    Tiling(crate::tiling::TilingError),
    NothingToReopen(SpaceId),
    IllegalLifecycleTransition {
        tab: TabId,
        from: TabLifecycle,
        to: TabLifecycle,
    },
    LifecycleVisibilityConflict {
        tab: TabId,
        requested: TabLifecycle,
    },
    /// An id-preserving restore tried to import an identity whose id is
    /// already owned by another profile, space, tab, or recently-closed
    /// entry (the domain identity contract is global uniqueness across
    /// kinds), or an id that is not a positive persisted value.
    RestoreCollision(u64),
    /// Blueprint D05: the last space of a profile cannot be deleted.
    LastSpaceOfProfile(SpaceId),
    /// Blueprint D05: a space move past the edge of its profile group.
    SpaceMoveOutOfRange(SpaceId),
    /// Blueprint D06: the last profile cannot be deleted.
    LastProfile(ProfileId),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownProfile(id) => write!(f, "unknown profile {id:?}"),
            Self::UnknownSpace(id) => write!(f, "unknown space {id:?}"),
            Self::UnknownTab(id) => write!(f, "unknown tab {id:?}"),
            Self::TabNotInSpace { tab, space } => {
                write!(f, "tab {tab:?} does not belong to space {space:?}")
            }
            Self::IndexOutOfBounds { index, len } => {
                write!(f, "index {index} is out of bounds for a list of {len} tabs")
            }
            Self::ReorderAcrossPinnedBoundary => {
                write!(f, "cannot reorder a tab across the pinned boundary")
            }
            Self::InsertionAcrossPinnedBoundary { index, pinned } => write!(
                f,
                "cannot insert a regular tab at index {index}: the pinned region holds {pinned} tabs"
            ),
            Self::SplitRequiresDistinctTabs => write!(f, "a split requires two distinct tabs"),
            Self::SplitAlreadyExists(id) => write!(f, "space {id:?} already has a split"),
            Self::Tiling(error) => write!(f, "split layout: {error:?}"),
            Self::NothingToReopen(id) => write!(f, "space {id:?} has no tabs to reopen"),
            Self::IllegalLifecycleTransition { tab, from, to } => {
                write!(f, "tab {tab:?} cannot transition from {from:?} to {to:?}")
            }
            Self::LifecycleVisibilityConflict { tab, requested } => write!(
                f,
                "tab {tab:?} cannot enter {requested:?} while its visibility disagrees"
            ),
            Self::RestoreCollision(id) => {
                write!(f, "restore rejected identity {id}: id already owned")
            }
            Self::LastSpaceOfProfile(id) => {
                write!(f, "space {id:?} is the last space of its profile")
            }
            Self::SpaceMoveOutOfRange(id) => {
                write!(f, "space {id:?} has no neighbor in that direction")
            }
            Self::LastProfile(id) => write!(f, "the last profile {id:?} cannot be deleted"),
        }
    }
}

impl std::error::Error for DomainError {}

/// A structural invariant violation found by `Browser::check_invariants`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantViolation(pub String);

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "domain invariant violated: {}", self.0)
    }
}

impl std::error::Error for InvariantViolation {}
