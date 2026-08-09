//! Store lifecycle: the schema version stamp and discard-on-mismatch.
//!
//! Schema v3 has **no migrations and no backward compatibility** (product
//! decision 2026-08-08): the store carries the current schema version in
//! `PRAGMA user_version`, and any store whose stamp or structure does not
//! match the current schema is discarded and recreated empty. This is a
//! normal, logged, non-fatal startup path — never a panic and never a
//! partial load. [`crate::StoreLoad`] records what happened so the caller
//! can log it.

use rusqlite::Connection;

use crate::{schema, PersistenceError, StoreDiscardReason, StoreLoad};

/// Decides how to bring a store up to the current schema and returns the
/// disposition ([`StoreLoad`]).
///
/// - stamp matches → validate structure; valid → `Loaded`, invalid → discard
///   and recreate, `Discarded { Corrupt }`;
/// - stamp mismatch with existing data → discard and recreate, `Discarded`
///   with `Unstamped` or `VersionMismatch`;
/// - stamp mismatch on a fresh empty database → create creates the v3 schema,
///   `Loaded` (a genuine first run discards nothing).
///
/// Every write path runs this before any read or write, so a discarded
/// store is immediately usable and rewritable.
pub(crate) fn prepare_store(connection: &mut Connection) -> Result<StoreLoad, PersistenceError> {
    let version = schema::user_version(connection)?;
    if version == schema::SCHEMA_VERSION {
        return match schema::validate_all(connection) {
            Ok(()) => Ok(StoreLoad::Loaded),
            Err(error) => {
                discard_and_recreate(connection)?;
                Ok(StoreLoad::Discarded {
                    reason: StoreDiscardReason::Corrupt {
                        detail: error.to_string(),
                    },
                })
            }
        };
    }
    if is_empty(connection)? {
        create_schema(connection)?;
        return Ok(StoreLoad::Loaded);
    }
    discard_and_recreate(connection)?;
    let reason = if version == 0 {
        StoreDiscardReason::Unstamped
    } else {
        StoreDiscardReason::VersionMismatch { found: version }
    };
    Ok(StoreLoad::Discarded { reason })
}

/// Creates the complete v3 schema and pins the version stamp in one
/// transaction. The DDL lives in `schema.rs` beside its validator.
pub(crate) fn create_schema(connection: &mut Connection) -> Result<(), PersistenceError> {
    let transaction = connection.transaction().map_err(schema::storage)?;
    transaction
        .execute_batch(schema::CREATE_SCHEMA_V3)
        .map_err(schema::storage)?;
    transaction
        .pragma_update(None, "user_version", schema::SCHEMA_VERSION)
        .map_err(schema::storage)?;
    let result = schema::validate_all(&transaction);
    match result {
        Ok(()) => transaction.commit().map_err(schema::storage),
        Err(error) => Err(error),
    }
}

/// Drops every user table and view, then recreates the v3 schema from
/// scratch in one transaction. Used only to discard a store whose stamp or
/// structure does not match the current schema.
pub(crate) fn discard_and_recreate(connection: &mut Connection) -> Result<(), PersistenceError> {
    // Legacy stores may reference each other arbitrarily; the drop must not
    // be blocked by foreign-key enforcement (which rusqlite enables by
    // default). Enforcement is per-connection and re-enabled by the write
    // path, so this cannot leak into later writes.
    connection
        .pragma_update(None, "foreign_keys", false)
        .map_err(schema::storage)?;
    let transaction = connection.transaction().map_err(schema::storage)?;
    let objects = {
        let mut statement = transaction
            .prepare(
                "SELECT type, name FROM sqlite_schema \
                 WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'",
            )
            .map_err(schema::storage)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(schema::storage)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(schema::storage)?
    };
    for (kind, name) in objects {
        let quoted = name.replace('"', "\"\"");
        transaction
            .execute(&format!("DROP {kind} IF EXISTS \"{quoted}\""), [])
            .map_err(schema::storage)?;
    }
    transaction
        .execute_batch(schema::CREATE_SCHEMA_V3)
        .map_err(schema::storage)?;
    transaction
        .pragma_update(None, "user_version", schema::SCHEMA_VERSION)
        .map_err(schema::storage)?;
    let result = schema::validate_all(&transaction);
    match result {
        Ok(()) => transaction.commit().map_err(schema::storage),
        Err(error) => Err(error),
    }
}

/// True when the database contains no user objects: the fresh-empty case.
pub(crate) fn is_empty(connection: &Connection) -> Result<bool, PersistenceError> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema \
             WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(schema::storage)?;
    Ok(count == 0)
}
