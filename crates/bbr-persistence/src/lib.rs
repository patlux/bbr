#![forbid(unsafe_code)]

//! Portable, versioned SQLite persistence for BBR's structural browser state.
//!
//! Schema v2 carries no migrations and no backward compatibility (product
//! decision 2026-08-08): the store owns a version stamp in SQLite's
//! `user_version`, and any store whose stamp or structure does not match the
//! current schema is discarded and recreated empty. That is a normal,
//! logged, non-fatal startup path ([`StoreLoad`] records the disposition);
//! it is never a panic and never a partial load.
//!
//! The public API owns its DTOs and typed errors; it exposes no SQLite
//! connection, row, value, or error type. URL validation is deliberately
//! bounded rather than a full parser: required URLs must be non-empty and must
//! contain no ASCII whitespace, control, or NUL bytes.

mod error;
mod fields;
mod load_helpers;
mod model;
mod repository;
mod repository_browser;
mod repository_load;
mod repository_rows;
mod schema;
mod schema_tables;
mod setup;
mod validation;

pub use error::{CorruptionIssue, InvalidSnapshotIssue, PersistenceError, SchemaIssue};
pub use fields::{
    CommandHistoryEntry, DownloadOutcome, DownloadRecord, GlyphName, SearchEngineId,
    SearchSettings, SpaceGlyph, SpaceTint, StoreDiscardReason, StoreLoad,
};
pub use model::{
    BrowserState, Profile, ProfileHue, RecentlyClosedTab, RecoveryWarning, Repository,
    RestoreOutcome, SidebarState, Snapshot, Space, Split, Tab, WindowGeometry, WindowState,
    WriteStats,
};
