//! Repository row methods for the app-owned global stores (Phase 7,
//! command-history provider slice): the single settings row and the
//! command-history collection. Focused here so `repository.rs` stays close
//! to its pre-slice size; the `Repository` API surface is unchanged.
//!
//! Both groups follow the same store disposition as every other write/read
//! path: a mismatched or corrupt store is discarded and recreated empty on
//! write (`setup::prepare_store`), while the standalone read-only loads
//! refuse a mismatched store with a typed `SchemaMismatch` and leave it
//! untouched. Row validation (settings invariants, history URL/identity/
//! counter bounds) runs before any write.

use std::collections::HashSet;

use rusqlite::{params, Connection, OpenFlags};

use crate::repository::to_i64;
use crate::repository_load::{load_command_history, load_settings};
use crate::{
    schema, setup, validation, CommandHistoryEntry, PersistenceError, Repository, SearchSettings,
};

impl Repository {
    /// Loads the single global settings row with the restore recovery
    /// applied: an incoherent persisted record (custom engine without a
    /// valid template, or a template without the custom engine) falls back
    /// to the deterministic defaults, exactly like the full restore path.
    /// The startup restore ([`Self::initialize_and_restore`]) surfaces the
    /// typed [`crate::RecoveryWarning::Settings`] for that case; this
    /// standalone read exists for the settings-store bridge and documents
    /// the same recovery. A mismatched store is refused with a typed
    /// [`PersistenceError::SchemaMismatch`] and left untouched.
    pub fn load_settings(&self) -> Result<SearchSettings, PersistenceError> {
        let connection = Connection::open_with_flags(
            self.path(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(schema::storage)?;
        connection
            .pragma_update(None, "query_only", true)
            .map_err(schema::storage)?;
        let version = schema::user_version(&connection)?;
        if version != schema::SCHEMA_VERSION {
            return Err(PersistenceError::SchemaMismatch {
                found: version,
                supported: schema::SCHEMA_VERSION,
            });
        }
        schema::validate_schema(&connection)?;
        schema::check_foreign_keys(&connection)?;
        load_settings(&connection, &mut Vec::new())
    }

    /// Transactionally upserts the single global settings row (id 1).
    ///
    /// Runs the same store disposition as the other write paths, so writing
    /// to a discarded or legacy store is always safe. The write validates
    /// the settings invariants (custom engine requires a valid `%s`
    /// template; a template without the custom engine is rejected) and
    /// keeps the "exactly one settings row" invariant: any prior row is
    /// replaced, never duplicated.
    pub fn store_settings(&self, settings: &SearchSettings) -> Result<(), PersistenceError> {
        validation::validate_settings(settings).map_err(PersistenceError::InvalidSnapshot)?;
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        transaction
            .execute(
                "INSERT OR REPLACE INTO settings(id, search_engine, custom_search_template) \
                 VALUES (1, ?, ?)",
                params![settings.engine.as_str(), settings.custom_template],
            )
            .map_err(schema::storage)?;
        transaction.commit().map_err(schema::storage)?;
        Ok(())
    }

    /// Loads the visited command-history entries in persisted corpus order
    /// (read-only; a mismatched store is refused untouched).
    pub fn load_command_history(&self) -> Result<Vec<CommandHistoryEntry>, PersistenceError> {
        let connection = Connection::open_with_flags(
            self.path(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(schema::storage)?;
        connection
            .pragma_update(None, "query_only", true)
            .map_err(schema::storage)?;
        let version = schema::user_version(&connection)?;
        if version != schema::SCHEMA_VERSION {
            return Err(PersistenceError::SchemaMismatch {
                found: version,
                supported: schema::SCHEMA_VERSION,
            });
        }
        schema::validate_schema(&connection)?;
        schema::check_foreign_keys(&connection)?;
        load_command_history(&connection, &mut HashSet::new())
    }

    /// Transactionally replaces the complete command-history collection.
    ///
    /// The whole collection is rewritten in one transaction with dense
    /// position keys preserving `entries` order, so a later restore reads
    /// back exactly the caller's order (the deterministic corpus order the
    /// app's provider owns). Validation rejects an invalid entry (bad URL,
    /// non-positive identity, duplicate identity, out-of-range counters)
    /// before any write; a mismatched store is discarded and recreated
    /// first, like every write path.
    pub fn store_command_history(
        &self,
        entries: &[CommandHistoryEntry],
    ) -> Result<(), PersistenceError> {
        validation::validate_command_history(entries).map_err(PersistenceError::InvalidSnapshot)?;
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        transaction
            .execute("DELETE FROM command_history", [])
            .map_err(schema::storage)?;
        for (position, entry) in entries.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO command_history(id, title, url, visit_count, last_visited, \
                     position) VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        to_i64(entry.id),
                        entry.title,
                        entry.url,
                        to_i64(entry.visit_count),
                        to_i64(entry.last_visited),
                        position as i64
                    ],
                )
                .map_err(schema::storage)?;
        }
        transaction.commit().map_err(schema::storage)?;
        Ok(())
    }
}
