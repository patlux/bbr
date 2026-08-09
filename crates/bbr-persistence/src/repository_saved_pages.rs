//! Record-lifecycle writes of the global Saved Pages store (schema v3, B1
//! packet `saved-pages-portable-persistence`): idempotent save, field
//! updates, and idempotent remove. Split out of the pre-split module so
//! each contract group stays below the ~500-line budget: the read/load/
//! validation surface lives in `repository_saved_pages_rows`, and the
//! collection writes live in `repository_saved_pages_collections`.
//!
//! Every method follows the same store disposition as every other read/write
//! path: a mismatched or corrupt store is discarded and recreated empty on
//! write ([`setup::prepare_store`]). Validation runs before any write; a
//! rejected write leaves the store byte-identical.
//!
//! Two write invariants are enforced here and proven by regression tests:
//! no public mutation can persist `updated_at < created_at` for the stored
//! row (a stale caller timestamp is clamped up to the stored `created_at`,
//! review F2), and an update whose requested title/label values are
//! byte-identical to the stored row performs no write at all (review F4).

use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension};

use crate::load_helpers::persisted_id;
use crate::repository::to_i64;
use crate::repository_saved_pages_rows::load_collections;
use crate::validation_saved_pages::validate_saved_pages;
use crate::{
    schema, setup, validation, CollectionRecord, InvalidSnapshotIssue, PersistenceError,
    Repository, SavedPageRecord, SavedPageSaveOutcome, SavedPageUpdate,
};

impl Repository {
    /// Idempotently saves one Saved Page record.
    ///
    /// Duplicate semantics (B1 packet): saving a canonical URL that already
    /// has an active record returns the **existing** `BookmarkId`, creates
    /// no second row, preserves the original `created_at`, and updates the
    /// captured title only when the existing record has no user label (the
    /// label, when present, is user-owned display truth). A re-save that
    /// changes nothing performs no write at all; a re-save that updates the
    /// captured title also updates `updated_at` from the passed record,
    /// clamped up to the existing record's `created_at` so the monotonic
    /// invariant (`updated_at >= created_at`) can never be broken through
    /// this API. The passed identity is ignored on the duplicate path — the
    /// existing id wins. A passed collection reference must name a known
    /// collection or the whole write rejects before any mutation.
    pub fn store_saved_page(
        &self,
        page: &SavedPageRecord,
    ) -> Result<SavedPageSaveOutcome, PersistenceError> {
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        // Validate the record against the current collection set before any
        // write (the same combined contract `replace_snapshot` enforces).
        let collections = load_collections(&connection, &mut HashSet::new())?;
        validate_single(page, &collections)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        let existing = transaction
            .query_row(
                "SELECT id, captured_title, user_label, created_at, updated_at FROM saved_pages \
                 WHERE canonical_url = ?1",
                [&page.canonical_url],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(schema::storage)?;
        let outcome = match existing {
            Some((
                existing_id,
                existing_title,
                existing_label,
                existing_created_at,
                _existing_updated,
            )) => {
                let existing_id = persisted_id("saved_pages", existing_id)?;
                // Duplicate: the existing identity wins; the user label is
                // never touched by a save; the captured title updates only
                // when no user label exists and the title actually changed.
                if existing_label.is_none() && existing_title != page.captured_title {
                    // The stored `updated_at` must never regress below the
                    // stored `created_at` (the loader-enforced monotonic
                    // invariant): a stale caller timestamp is clamped up to
                    // the existing `created_at` instead of persisting an
                    // invalid pair that restore would refuse (review F2).
                    let updated_at = page.updated_at.max(existing_created_at as u64);
                    transaction
                        .execute(
                            "UPDATE saved_pages SET captured_title = ?1, updated_at = ?2 \
                             WHERE id = ?3",
                            params![page.captured_title, to_i64(updated_at), to_i64(existing_id)],
                        )
                        .map_err(schema::storage)?;
                }
                SavedPageSaveOutcome {
                    id: existing_id,
                    existed: true,
                }
            }
            None => {
                // Insert: the caller's id is the record identity; uniqueness
                // against the store is re-validated (a duplicate id is an
                // allocator bug, not a duplicate URL).
                let taken: Option<i64> = transaction
                    .query_row(
                        "SELECT id FROM saved_pages WHERE id = ?1",
                        [to_i64(page.id)],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(schema::storage)?;
                if taken.is_some() {
                    return Err(PersistenceError::InvalidSnapshot(
                        InvalidSnapshotIssue::DuplicateIdentity { id: page.id },
                    ));
                }
                let position: i64 = transaction
                    .query_row(
                        "SELECT COALESCE(MAX(position), -1) + 1 FROM saved_pages",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(schema::storage)?;
                transaction
                    .execute(
                        "INSERT INTO saved_pages(id, canonical_url, captured_title, user_label, \
                         created_at, updated_at, collection_id, position) VALUES (?, ?, ?, ?, ?, \
                         ?, ?, ?)",
                        params![
                            to_i64(page.id),
                            page.canonical_url,
                            page.captured_title,
                            page.user_label,
                            to_i64(page.created_at),
                            to_i64(page.updated_at),
                            page.collection_id.map(to_i64),
                            position
                        ],
                    )
                    .map_err(schema::storage)?;
                SavedPageSaveOutcome {
                    id: page.id,
                    existed: false,
                }
            }
        };
        transaction.commit().map_err(schema::storage)?;
        Ok(outcome)
    }

    /// Applies a field-level update (captured title and/or user label) to
    /// one Saved Page record in one transaction.
    ///
    /// Unknown identities and invalid text reject whole before any write.
    /// An update whose requested title/label values are byte-identical to
    /// the stored row performs no write at all and does not bump
    /// `updated_at`. A real mutation applies `updated_at` clamped up to the
    /// stored `created_at`, so the monotonic invariant
    /// (`updated_at >= created_at`) can never be broken through this API.
    pub fn update_saved_page(&self, update: &SavedPageUpdate) -> Result<(), PersistenceError> {
        validate_update(update)?;
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        let existing: Option<(String, Option<String>, i64)> = transaction
            .query_row(
                "SELECT captured_title, user_label, created_at FROM saved_pages WHERE id = ?1",
                [to_i64(update.id)],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(schema::storage)?;
        let Some((current_title, current_label, created_at)) = existing else {
            return Err(PersistenceError::InvalidSnapshot(
                InvalidSnapshotIssue::UnknownBookmark { id: update.id },
            ));
        };
        // Assignments are built only for values that actually differ
        // byte-for-byte from the stored row, so a true no-op performs no
        // write and `updated_at` is only applied to a real mutation.
        let mut assignments = Vec::new();
        let mut values: Vec<rusqlite::types::Value> = Vec::new();
        if let Some(title) = &update.captured_title {
            if *title != current_title {
                assignments.push("captured_title = ?");
                values.push(rusqlite::types::Value::Text(title.clone()));
            }
        }
        if let Some(label) = &update.user_label {
            match label {
                Some(label) => {
                    if current_label.as_deref() != Some(label.as_str()) {
                        assignments.push("user_label = ?");
                        values.push(rusqlite::types::Value::Text(label.clone()));
                    }
                }
                None => {
                    if current_label.is_some() {
                        assignments.push("user_label = ?");
                        values.push(rusqlite::types::Value::Null);
                    }
                }
            }
        }
        if assignments.is_empty() {
            // Nothing changed: a no-op update performs no write.
            return Ok(());
        }
        assignments.push("updated_at = ?");
        // Clamp up to the stored `created_at` so the monotonic invariant
        // can never be violated by a stale caller timestamp (review F2).
        values.push(rusqlite::types::Value::Integer(to_i64(
            update.updated_at.max(created_at as u64),
        )));
        values.push(rusqlite::types::Value::Integer(to_i64(update.id)));
        let sql = format!(
            "UPDATE saved_pages SET {} WHERE id = ?",
            assignments.join(", ")
        );
        transaction
            .execute(&sql, rusqlite::params_from_iter(values))
            .map_err(schema::storage)?;
        transaction.commit().map_err(schema::storage)?;
        Ok(())
    }

    /// Removes one Saved Page record; repeated remove of the same (or an
    /// unknown) identity is a no-op success. Remaining records keep their
    /// relative order and their dense position keys are re-compacted in the
    /// same transaction.
    pub fn remove_saved_page(&self, id: u64) -> Result<(), PersistenceError> {
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        transaction
            .execute("DELETE FROM saved_pages WHERE id = ?1", [to_i64(id)])
            .map_err(schema::storage)?;
        // Re-compact the dense position keys in stable relative order: the
        // subquery counts rows with a smaller rowid, which never changes
        // during the UPDATE, so the renumbering is deterministic even though
        // the column being rewritten is the one being compared.
        transaction
            .execute(
                "UPDATE saved_pages SET position = (SELECT COUNT(*) FROM saved_pages s2 \
                 WHERE s2.rowid < saved_pages.rowid)",
                [],
            )
            .map_err(schema::storage)?;
        transaction.commit().map_err(schema::storage)?;
        Ok(())
    }
}

/// Validates one record against the current collection set before an
/// incremental write (the combined contract `validate_saved_pages`
/// enforces for whole-snapshot writes).
fn validate_single(
    page: &SavedPageRecord,
    collections: &[CollectionRecord],
) -> Result<(), PersistenceError> {
    validate_saved_pages(std::slice::from_ref(page), collections)
        .map_err(PersistenceError::InvalidSnapshot)
}

fn validate_update(update: &SavedPageUpdate) -> Result<(), PersistenceError> {
    if update.id == 0 || update.id > validation::MAX_SQLITE_ID {
        return Err(PersistenceError::InvalidSnapshot(
            InvalidSnapshotIssue::InvalidId {
                kind: "saved page",
                id: update.id,
            },
        ));
    }
    if update.updated_at > validation::MAX_SQLITE_ID {
        return Err(PersistenceError::InvalidSnapshot(
            InvalidSnapshotIssue::InvalidSavedPage {
                id: update.id,
                detail: "updated_at exceeds the SQLite integer range".into(),
            },
        ));
    }
    if let Some(title) = &update.captured_title {
        if title.len() > validation::MAX_CAPTURED_TITLE_LEN {
            return Err(PersistenceError::InvalidSnapshot(
                InvalidSnapshotIssue::InvalidSavedPage {
                    id: update.id,
                    detail: format!(
                        "captured title exceeds {} bytes",
                        validation::MAX_CAPTURED_TITLE_LEN
                    ),
                },
            ));
        }
        if title
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return Err(PersistenceError::InvalidSnapshot(
                InvalidSnapshotIssue::InvalidSavedPage {
                    id: update.id,
                    detail: "captured title contains control bytes".into(),
                },
            ));
        }
    }
    if let Some(Some(label)) = &update.user_label {
        if label.is_empty() {
            return Err(PersistenceError::InvalidSnapshot(
                InvalidSnapshotIssue::InvalidSavedPage {
                    id: update.id,
                    detail: "empty user label".into(),
                },
            ));
        }
        if label.len() > validation::MAX_USER_LABEL_LEN {
            return Err(PersistenceError::InvalidSnapshot(
                InvalidSnapshotIssue::InvalidSavedPage {
                    id: update.id,
                    detail: format!(
                        "user label exceeds {} bytes",
                        validation::MAX_USER_LABEL_LEN
                    ),
                },
            ));
        }
        if label
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return Err(PersistenceError::InvalidSnapshot(
                InvalidSnapshotIssue::InvalidSavedPage {
                    id: update.id,
                    detail: "user label contains control bytes".into(),
                },
            ));
        }
    }
    Ok(())
}
