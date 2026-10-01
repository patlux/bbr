//! Repository browser-state row methods (Phase 7, warm-restore slice): the
//! focused structural write that replaces only the browser tables —
//! profiles, spaces, tabs, recently-closed entries, splits, windows, and the
//! schema-v5 folders/favorites —
//! while leaving the global settings, download, and command-history rows
//! untouched. Focused here so `repository.rs` and `repository_rows.rs` keep
//! their pre-slice size; the `Repository` API surface grows by exactly one
//! method.
//!
//! The write follows the same store disposition as every other write path: a
//! mismatched or corrupt store is discarded and recreated empty on write
//! (`setup::prepare_store`). Validation reuses the complete-snapshot write
//! contract ([`validation::validate_snapshot`] over a synthesized snapshot),
//! so the exact same global-identity uniqueness, pinned-prefix, member
//! active-tab, distinct-member split, bounded-URL, window-geometry, and
//! sidebar invariants that protect `replace_snapshot` protect this write.

use rusqlite::Connection;

use crate::repository_browser_rows::{insert_browser_rows, BROWSER_DELETE_TABLES};
use crate::{
    schema, setup, validation, BrowserState, PersistenceError, Repository, SearchSettings,
    Snapshot, WriteStats,
};

impl Repository {
    /// Transactionally replaces the complete browser structural state
    /// (profiles, spaces, tabs, recently-closed, splits, windows) without
    /// touching the settings, download, or command-history tables.
    ///
    /// The whole browser state is rewritten in one transaction with dense
    /// per-table position keys preserving the caller's order, so a later
    /// restore reads back exactly the persisted browser model. The store
    /// disposition (discard and recreate empty on stamp/shape mismatch) and
    /// the validation contract are identical to
    /// [`Self::replace_snapshot`](crate::Repository::replace_snapshot).
    pub fn store_browser_state(
        &self,
        state: &BrowserState,
    ) -> Result<WriteStats, PersistenceError> {
        // Validate through the complete-snapshot contract: browser tables
        // share the same invariants as the full snapshot write, and the
        // global stores contribute nothing here (defaults/empty collections
        // are trivially valid and never written by this method).
        let full = Snapshot {
            profiles: state.profiles.clone(),
            windows: state.windows.clone(),
            settings: SearchSettings::default(),
            downloads: Vec::new(),
            command_history: Vec::new(),
            // Schema v3: the global Saved Pages tables are never part of a
            // browser-state write (isolation is asserted in the saved-pages
            // repository tests); empty defaults keep the shared snapshot
            // contract valid.
            saved_pages: Vec::new(),
            collections: Vec::new(),
            favorites: state.favorites.clone(),
        };
        validation::validate_snapshot(&full).map_err(PersistenceError::InvalidSnapshot)?;
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        let mut deleted_rows = 0_u64;
        // Child-first deletion of the browser tables only: the global
        // settings/download/command-history rows survive this write.
        for table in BROWSER_DELETE_TABLES {
            deleted_rows += transaction
                .execute(&format!("DELETE FROM {table}"), [])
                .map_err(schema::storage)? as u64;
        }
        let inserted_rows = insert_browser_rows(
            &transaction,
            &state.profiles,
            &state.windows,
            &state.favorites,
        )?;
        transaction.commit().map_err(schema::storage)?;
        Ok(WriteStats {
            committed_transactions: 1,
            sql_statements: BROWSER_DELETE_TABLES.len() as u64 + inserted_rows,
            deleted_rows,
            inserted_rows,
        })
    }
}
