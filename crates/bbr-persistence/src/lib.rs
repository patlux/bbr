#![forbid(unsafe_code)]

//! Portable, versioned SQLite persistence for BBR's structural browser state.
//!
//! The public API owns its DTOs and typed errors; it exposes no SQLite
//! connection, row, value, or error type. URL validation is deliberately
//! bounded rather than a full parser: required URLs must be non-empty and must
//! contain no ASCII whitespace, control, or NUL bytes.

mod error;
mod model;
mod repository;
mod repository_load;
mod schema;
mod validation;

pub use error::{CorruptionIssue, InvalidSnapshotIssue, PersistenceError, SchemaIssue};
pub use model::{
    Profile, ProfileHue, RecentlyClosedTab, RecoveryWarning, Repository, RestoreOutcome,
    SidebarState, Snapshot, Space, Split, Tab, WindowGeometry, WindowState, WriteStats,
};
