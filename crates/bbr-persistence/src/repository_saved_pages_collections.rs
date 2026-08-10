//! Collection-group writes of the global Saved Pages store (schema v3, B1
//! packet `saved-pages-portable-persistence`): whole-set flat collection
//! replacement (create/rename/remove) and record-membership moves. Split out
//! of the pre-split module so each contract group stays below the ~500-line
//! budget: the record-lifecycle writes live in `repository_saved_pages` and
//! the read/load/validation surface in `repository_saved_pages_rows`.
//!
//! Membership is strict: every persisted `saved_pages.collection_id` must
//! resolve in the current collection set (declared foreign key plus typed
//! validation), so a dangling reference can never be written and a
//! collection that is still referenced cannot be removed in one step —
//! collection removal is an explicit move-out-then-remove.

use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension};

use crate::repository::to_i64;
use crate::repository_saved_pages_rows::{load_collections, load_saved_pages};
use crate::validation_saved_pages::{validate_collections, validate_saved_pages};
use crate::{
    schema, setup, validation, CollectionRecord, InvalidSnapshotIssue, PersistenceError,
    Repository, SavedPageRecord, WriteStats,
};

impl Repository {
    /// Atomically replaces the complete flat collection set (create, rename,
    /// and remove through one whole-set write).
    ///
    /// Every Saved Page record's membership is validated against the new
    /// set before any write: removing a collection that is still referenced
    /// rejects whole (typed [`InvalidSnapshotIssue::UnknownCollection`]) and
    /// leaves the store byte-identical — a dangling membership can never be
    /// created. Collection removal is therefore an explicit two-step
    /// operation: move the member records out first, then remove the
    /// collection.
    pub fn replace_collections(
        &self,
        collections: &[CollectionRecord],
    ) -> Result<WriteStats, PersistenceError> {
        validate_collections(collections).map_err(PersistenceError::InvalidSnapshot)?;
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        // The combined contract: every persisted Saved Page reference must
        // resolve in the new set, exactly like `replace_snapshot`.
        let pages = load_saved_pages(
            &connection,
            &mut HashSet::new(),
            &load_collections(&connection, &mut HashSet::new())?,
        )?;
        validate_saved_pages(&pages, collections).map_err(PersistenceError::InvalidSnapshot)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        // Id-preserving rewrites (rename) delete and re-insert the same
        // collection row while member records still reference it. Deferring
        // foreign-key enforcement to commit lets that atomic rewrite pass;
        // a genuine removal of a referenced collection is already rejected
        // by the typed validation above (the deferred check is a safety
        // net, never the rejection authority).
        transaction
            .pragma_update(None, "defer_foreign_keys", true)
            .map_err(schema::storage)?;
        let deleted_rows = transaction
            .execute("DELETE FROM collections", [])
            .map_err(schema::storage)? as u64;
        let mut inserted_rows = 0_u64;
        for (position, collection) in collections.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO collections(id, title, position) VALUES (?, ?, ?)",
                    params![to_i64(collection.id), collection.title, position as i64],
                )
                .map_err(schema::storage)?;
            inserted_rows += 1;
        }
        transaction.commit().map_err(schema::storage)?;
        Ok(WriteStats {
            committed_transactions: 1,
            sql_statements: 1 + inserted_rows,
            deleted_rows,
            inserted_rows,
        })
    }

    /// Validates and atomically replaces **both** persisted Saved Page
    /// tables (the records and the flat collections they may reference) in
    /// one transaction.
    ///
    /// This is the whole-Saved-Pages write path the B3 Library attaches to
    /// the bounded nonblocking writer: the in-memory provider is the single
    /// runtime authority, and each accepted mutation replaces the complete
    /// persisted Saved Page state in one transaction, so a removed record
    /// or collection can never return from the store and a collection
    /// membership can never dangle. The combined validation runs before any
    /// write — a duplicate canonical URL, duplicate identity, dangling
    /// collection reference, or invalid collection set rejects whole and
    /// leaves the store byte-identical. The records arrive in the
    /// provider's deterministic presentation order and the collections in
    /// their declaration order (`position` = index for both tables).
    ///
    /// The B3 writer needs this combined write because collection and page
    /// mutations are batched together: two separate `replace_*` calls would
    /// have to run in opposite orders depending on whether a page references
    /// a newly created collection (collections first) or a persisted page
    /// still references a removed collection (pages first) — an unresolvable
    /// ordering constraint. One transaction with the combined contract is
    /// the only order-independent answer, and it mirrors the existing
    /// `replace_snapshot` whole-store write for the Saved Pages tables.
    pub fn replace_saved_snapshot(
        &self,
        pages: &[SavedPageRecord],
        collections: &[CollectionRecord],
    ) -> Result<WriteStats, PersistenceError> {
        validate_collections(collections).map_err(PersistenceError::InvalidSnapshot)?;
        validate_saved_pages(pages, collections).map_err(PersistenceError::InvalidSnapshot)?;
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        // Delete the referencing rows first: `saved_pages` rows carry the
        // strict `collection_id` foreign key, so the collections table must
        // be emptied only after every referencing row is gone (the same
        // order `replace_snapshot` uses).
        let deleted_pages = transaction
            .execute("DELETE FROM saved_pages", [])
            .map_err(schema::storage)? as u64;
        let deleted_collections = transaction
            .execute("DELETE FROM collections", [])
            .map_err(schema::storage)? as u64;
        let mut inserted_rows = 0_u64;
        for (position, collection) in collections.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO collections(id, title, position) VALUES (?, ?, ?)",
                    params![to_i64(collection.id), collection.title, position as i64],
                )
                .map_err(schema::storage)?;
            inserted_rows += 1;
        }
        for (position, page) in pages.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO saved_pages(id, canonical_url, captured_title, user_label, \
                     created_at, updated_at, collection_id, position) VALUES (?, ?, ?, ?, ?, ?, \
                     ?, ?)",
                    params![
                        to_i64(page.id),
                        page.canonical_url,
                        page.captured_title,
                        page.user_label,
                        to_i64(page.created_at),
                        to_i64(page.updated_at),
                        page.collection_id.map(to_i64),
                        position as i64
                    ],
                )
                .map_err(schema::storage)?;
            inserted_rows += 1;
        }
        transaction.commit().map_err(schema::storage)?;
        Ok(WriteStats {
            committed_transactions: 1,
            sql_statements: 1 + inserted_rows,
            deleted_rows: deleted_collections + deleted_pages,
            inserted_rows,
        })
    }

    /// Moves one Saved Page record into or out of one flat collection in one
    /// transaction.
    ///
    /// Unknown identities and unknown collection targets reject whole before
    /// any write; `None` clears the membership. The record's position (and
    /// therefore its persisted order) is untouched.
    pub fn move_saved_page_collection(
        &self,
        id: u64,
        collection: Option<u64>,
    ) -> Result<(), PersistenceError> {
        if id == 0 || id > validation::MAX_SQLITE_ID {
            return Err(PersistenceError::InvalidSnapshot(
                InvalidSnapshotIssue::InvalidId {
                    kind: "saved page",
                    id,
                },
            ));
        }
        if let Some(collection) = collection {
            if collection == 0 || collection > validation::MAX_SQLITE_ID {
                return Err(PersistenceError::InvalidSnapshot(
                    InvalidSnapshotIssue::InvalidId {
                        kind: "collection",
                        id: collection,
                    },
                ));
            }
        }
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        let exists: Option<i64> = transaction
            .query_row(
                "SELECT id FROM saved_pages WHERE id = ?1",
                [to_i64(id)],
                |row| row.get(0),
            )
            .optional()
            .map_err(schema::storage)?;
        if exists.is_none() {
            return Err(PersistenceError::InvalidSnapshot(
                InvalidSnapshotIssue::UnknownBookmark { id },
            ));
        }
        if let Some(collection) = collection {
            let known: Option<i64> = transaction
                .query_row(
                    "SELECT id FROM collections WHERE id = ?1",
                    [to_i64(collection)],
                    |row| row.get(0),
                )
                .optional()
                .map_err(schema::storage)?;
            if known.is_none() {
                return Err(PersistenceError::InvalidSnapshot(
                    InvalidSnapshotIssue::UnknownCollection { id: collection },
                ));
            }
        }
        transaction
            .execute(
                "UPDATE saved_pages SET collection_id = ?1 WHERE id = ?2",
                params![collection.map(to_i64), to_i64(id)],
            )
            .map_err(schema::storage)?;
        transaction.commit().map_err(schema::storage)?;
        Ok(())
    }
}
