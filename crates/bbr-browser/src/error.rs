//! Typed controller errors.
//!
//! Input that is invalid for the current state is rejected before any
//! mutation; structural corruption surfaces separately through invariant
//! checks. Domain errors are wrapped (the controller is the caller of
//! `bbr-domain`), registry errors are surfaced directly, and controller-local
//! errors cover missing focus targets and sessions.

use bbr_domain::{DomainError, SpaceId, TabId};

/// Errors returned by controller operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerError {
    /// The underlying domain rejected the mutation.
    Domain(DomainError),
    /// The session registry rejected the mutation (duplicate/orphan risk).
    Session(crate::SessionError),
    /// No space selected; commands need an active space.
    NoActiveSpace,
    /// The index does not name a tab in the active space.
    IndexOutOfBounds { index: usize, len: usize },
    /// No active tab to target a navigation command at.
    NoActiveTab,
    /// The tab has no live session to route an action or focus to.
    NoSession(TabId),
    /// The target space is unknown.
    UnknownSpace(SpaceId),
}

impl std::fmt::Display for ControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(e) => write!(f, "domain rejected operation: {e}"),
            Self::Session(e) => write!(f, "session registry rejected operation: {e}"),
            Self::NoActiveSpace => write!(f, "no active space selected"),
            Self::IndexOutOfBounds { index, len } => {
                write!(f, "index {index} out of bounds for {len} tabs")
            }
            Self::NoActiveTab => write!(f, "no active tab to target"),
            Self::NoSession(tab) => write!(f, "tab {tab:?} has no live session"),
            Self::UnknownSpace(space) => write!(f, "unknown space {space:?}"),
        }
    }
}

impl std::error::Error for ControllerError {}

impl From<DomainError> for ControllerError {
    fn from(e: DomainError) -> Self {
        Self::Domain(e)
    }
}

impl From<crate::SessionError> for ControllerError {
    fn from(e: crate::SessionError) -> Self {
        Self::Session(e)
    }
}
