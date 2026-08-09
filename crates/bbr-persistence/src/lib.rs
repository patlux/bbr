#![forbid(unsafe_code)]

//! Portable, versioned SQLite persistence for BBR's structural browser state.
//!
//! Schema v3 carries no migrations and no backward compatibility (product
//! decision 2026-08-08): the store owns a version stamp in SQLite's
//! `user_version`, and any store whose stamp or structure does not match the
//! current schema is discarded and recreated empty. That is a normal,
//! logged, non-fatal startup path ([`StoreLoad`] records the disposition);
//! it is never a panic and never a partial load. Schema v3 adds the global
//! Saved Pages tables (`saved_pages`, `collections`) on top of v2's
//! appearance/settings/downloads/command-history shape.
//!
//! The public API owns its DTOs and typed errors; it exposes no SQLite
//! connection, row, value, or error type. URL validation is deliberately
//! bounded rather than a full parser: required URLs must be non-empty and must
//! contain no ASCII whitespace, control, or NUL bytes. The canonical Saved
//! Page URL rule lives in `bbr-domain`; the repository stores the canonical
//! key opaquely and enforces uniqueness.

mod error;
mod fields;
mod load_helpers;
mod model;
mod repository;
mod repository_browser;
mod repository_load;
mod repository_rows;
mod repository_saved_pages;
mod repository_saved_pages_collections;
mod repository_saved_pages_rows;
mod schema;
mod schema_tables;
mod schema_tables_saved_pages;
mod setup;
mod validation;
mod validation_saved_pages;

pub use error::{CorruptionIssue, InvalidSnapshotIssue, PersistenceError, SchemaIssue};
pub use fields::{
    CollectionRecord, CommandHistoryEntry, DownloadOutcome, DownloadRecord, GlyphName,
    SavedPageRecord, SavedPageSaveOutcome, SavedPageUpdate, SearchEngineId, SearchSettings,
    SpaceGlyph, SpaceTint, StoreDiscardReason, StoreLoad,
};
pub use model::{
    BrowserState, Profile, ProfileHue, RecentlyClosedTab, RecoveryWarning, Repository,
    RestoreOutcome, SidebarState, Snapshot, Space, Split, Tab, WindowGeometry, WindowState,
    WriteStats,
};
