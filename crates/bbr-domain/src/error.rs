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
    SplitRequiresDistinctTabs,
    SplitAlreadyExists(SpaceId),
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
            Self::SplitRequiresDistinctTabs => write!(f, "a split requires two distinct tabs"),
            Self::SplitAlreadyExists(id) => write!(f, "space {id:?} already has a split"),
            Self::NothingToReopen(id) => write!(f, "space {id:?} has no tabs to reopen"),
            Self::IllegalLifecycleTransition { tab, from, to } => {
                write!(f, "tab {tab:?} cannot transition from {from:?} to {to:?}")
            }
            Self::LifecycleVisibilityConflict { tab, requested } => write!(
                f,
                "tab {tab:?} cannot enter {requested:?} while its visibility disagrees"
            ),
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
