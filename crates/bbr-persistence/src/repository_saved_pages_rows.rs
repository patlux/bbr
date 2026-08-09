//! Read surface of the global Saved Pages store (schema v3, B1 packet
//! `saved-pages-portable-persistence`): the read-only row methods
//! (`load_saved_pages`, `load_collections`, `saved_page_by_id`,
//! `saved_page_by_url`, `search_saved_pages`), the loaders, and the one
//! shared per-row data-validation helper. Split out of
//! `repository_saved_pages.rs` so the record-lifecycle writes stay in a
//! focused module below the ~500-line budget; this is the read/validation
//! group of the same contract.
//!
//! Every read follows the same store disposition as every other read/write
//! path: a mismatched or corrupt store is refused read-only with a typed
//! [`PersistenceError::SchemaMismatch`] and left untouched. The canonical-URL
//! **rule** lives in `bbr-domain` (reusing `bbr_web::normalize_url`); this
//! module treats the key as opaque and enforces the at-most-one-active-record-
//! per-key invariant on the exact key.
//!
//! The list/restore load and the single-row reads share exactly one
//! row-data validation authority ([`validate_saved_page_row`]), so an
//! externally corrupt same-version row (invalid timestamps, over-long
//! title/label, unresolvable collection reference, or a duplicate canonical
//! key) is refused consistently across the whole read surface (review F3).

use std::collections::HashSet;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};

use crate::load_helpers::{invalid_value, persisted_id, require_id, require_order};
use crate::repository::to_i64;
use crate::{
    schema, validation, CollectionRecord, CorruptionIssue, PersistenceError, Repository,
    SavedPageRecord,
};

/// Opens the store read-only at the current schema (refusing a mismatched
/// store untouched) and runs `load` against it. Shared by every saved-pages
/// read so the read-determinism contract is enforced in one place.
fn with_read_only_connection<T>(
    path: &std::path::Path,
    load: impl FnOnce(&Connection) -> Result<T, PersistenceError>,
) -> Result<T, PersistenceError> {
    let connection = Connection::open_with_flags(
        path,
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
    load(&connection)
}

impl Repository {
    /// Loads every Saved Page record in persisted order (read-only).
    pub fn load_saved_pages(&self) -> Result<Vec<SavedPageRecord>, PersistenceError> {
        with_read_only_connection(self.path(), |connection| {
            load_saved_pages(
                connection,
                &mut HashSet::new(),
                &load_collections(connection, &mut HashSet::new())?,
            )
        })
    }

    /// Loads every flat collection in persisted order (read-only).
    pub fn load_collections(&self) -> Result<Vec<CollectionRecord>, PersistenceError> {
        with_read_only_connection(self.path(), |connection| {
            load_collections(connection, &mut HashSet::new())
        })
    }

    /// Reads one Saved Page record by stable identity (read-only).
    ///
    /// The row is validated through the same row-data invariants as the
    /// list/restore load, so an externally corrupt row is refused here too
    /// (review F3).
    pub fn saved_page_by_id(&self, id: u64) -> Result<Option<SavedPageRecord>, PersistenceError> {
        with_read_only_connection(self.path(), |connection| saved_page_by_id(connection, id))
    }

    /// Reads one Saved Page record by its canonical URL key (read-only).
    ///
    /// The key is compared byte-for-byte against the persisted canonical
    /// key: callers must pass a key produced by the `bbr-domain` canonical
    /// seam (persistence never re-canonicalizes). The row is validated
    /// through the same row-data invariants as the list/restore load
    /// (review F3).
    pub fn saved_page_by_url(
        &self,
        canonical_url: &str,
    ) -> Result<Option<SavedPageRecord>, PersistenceError> {
        with_read_only_connection(self.path(), |connection| {
            saved_page_by_url(connection, canonical_url)
        })
    }

    /// Deterministic case-insensitive search over title, user label, and
    /// canonical URL (read-only). Results keep persisted order; an empty
    /// query returns the full list. Searches are a repository read — the
    /// app's ranking consumes one loaded snapshot and never queries from
    /// render.
    pub fn search_saved_pages(
        &self,
        query: &str,
    ) -> Result<Vec<SavedPageRecord>, PersistenceError> {
        with_read_only_connection(self.path(), |connection| {
            search_saved_pages(connection, query)
        })
    }
}

fn saved_page_by_id(
    connection: &Connection,
    id: u64,
) -> Result<Option<SavedPageRecord>, PersistenceError> {
    if id == 0 || id > validation::MAX_SQLITE_ID {
        return Ok(None);
    }
    let collection_ids = collection_ids(connection)?;
    let row = connection
        .query_row(
            "SELECT id, canonical_url, captured_title, user_label, created_at, updated_at, \
             collection_id FROM saved_pages WHERE id = ?1",
            [to_i64(id)],
            saved_page_row_raw,
        )
        .optional()
        .map_err(schema::storage)?;
    let Some(row) = row else {
        return Ok(None);
    };
    // A second row sharing the canonical key is the same corruption the
    // list load detects (the checked-in duplicate fixture omits the UNIQUE
    // constraint); the single-row read must refuse it, never serve it
    // (review F3).
    let duplicate: Option<i64> = connection
        .query_row(
            "SELECT id FROM saved_pages WHERE canonical_url = ?1 AND id != ?2 \
             ORDER BY position LIMIT 1",
            params![&row.1, to_i64(id)],
            |dup| dup.get(0),
        )
        .optional()
        .map_err(schema::storage)?;
    if let Some(duplicate) = duplicate {
        return Err(PersistenceError::Corruption(
            CorruptionIssue::DuplicateCanonicalUrl {
                id: duplicate as u64,
            },
        ));
    }
    Ok(Some(validate_saved_page_row(row, &collection_ids)?))
}

fn saved_page_by_url(
    connection: &Connection,
    canonical_url: &str,
) -> Result<Option<SavedPageRecord>, PersistenceError> {
    if canonical_url.is_empty() {
        return Ok(None);
    }
    let collection_ids = collection_ids(connection)?;
    let row = connection
        .query_row(
            "SELECT id, canonical_url, captured_title, user_label, created_at, updated_at, \
             collection_id FROM saved_pages WHERE canonical_url = ?1 ORDER BY position",
            [canonical_url],
            saved_page_row_raw,
        )
        .optional()
        .map_err(schema::storage)?;
    let Some(row) = row else {
        return Ok(None);
    };
    // Duplicate canonical keys are corruption; refuse like the list load.
    let duplicate: Option<i64> = connection
        .query_row(
            "SELECT id FROM saved_pages WHERE canonical_url = ?1 ORDER BY position \
             LIMIT 1 OFFSET 1",
            [&row.1],
            |dup| dup.get(0),
        )
        .optional()
        .map_err(schema::storage)?;
    if let Some(duplicate) = duplicate {
        return Err(PersistenceError::Corruption(
            CorruptionIssue::DuplicateCanonicalUrl {
                id: duplicate as u64,
            },
        ));
    }
    Ok(Some(validate_saved_page_row(row, &collection_ids)?))
}

/// The persisted collection-id set for membership validation (review F3:
/// single-row reads resolve collection references against the same loaded
/// set as the list load).
fn collection_ids(connection: &Connection) -> Result<HashSet<u64>, PersistenceError> {
    load_collections(connection, &mut HashSet::new())
        .map(|collections| collections.iter().map(|collection| collection.id).collect())
}

/// One persisted saved-pages row in raw column order (ids and timestamps
/// stay in `i64` until they are range-validated).
type SavedPageRow = (i64, String, String, Option<String>, i64, i64, Option<i64>);

/// Reads one persisted saved-pages row as raw values so the shared
/// [`validate_saved_page_row`] helper can apply the row-data invariants.
fn saved_page_row_raw(row: &rusqlite::Row<'_>) -> rusqlite::Result<SavedPageRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
    ))
}

/// Deterministic search over one loaded snapshot in persisted order using
/// the one small Rust Unicode-lowercase literal-substring authority — the
/// repository seam's mirror of `bbr-domain::SavedPage::matches` (the
/// persistence crate must not import `bbr-domain`). No SQL pattern is
/// built, so `%`/`_`/`\` in the query are literal by construction,
/// multi-byte UTF-8 can never be corrupted, and case-folding matches the
/// domain predicate for non-ASCII (umlaut) queries — SQLite `LIKE` folds
/// ASCII case only and would diverge (review F1).
fn search_saved_pages(
    connection: &Connection,
    query: &str,
) -> Result<Vec<SavedPageRecord>, PersistenceError> {
    let collections = load_collections(connection, &mut HashSet::new())?;
    let mut pages = load_saved_pages(connection, &mut HashSet::new(), &collections)?;
    if query.is_empty() {
        return Ok(pages);
    }
    let needle = query.to_lowercase();
    pages.retain(|page| {
        page.captured_title.to_lowercase().contains(&needle)
            || page
                .user_label
                .as_deref()
                .is_some_and(|label| label.to_lowercase().contains(&needle))
            || page.canonical_url.to_lowercase().contains(&needle)
    });
    Ok(pages)
}

/// Loads the flat Saved Page collections in persisted order (schema v3).
///
/// A persisted collection must satisfy the same bounded shape the write path
/// enforces (unique in-range id, non-empty bounded title, dense position);
/// anything else is corruption and discards the store under the development
/// rule.
pub(crate) fn load_collections(
    connection: &Connection,
    identities: &mut HashSet<u64>,
) -> Result<Vec<CollectionRecord>, PersistenceError> {
    let mut statement = connection
        .prepare("SELECT id, title, position FROM collections ORDER BY position")
        .map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut result = Vec::new();
    for (expected, row) in rows.enumerate() {
        let (id, title, position) = row.map_err(schema::storage)?;
        require_order("collections", expected, position)?;
        let id = require_id("collections", id, identities)?;
        if title.is_empty()
            || title.len() > validation::MAX_COLLECTION_TITLE_LEN
            || title
                .bytes()
                .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return Err(invalid_value(
                "collections",
                format!("collection {id} has an invalid title"),
            ));
        }
        result.push(CollectionRecord { id, title });
    }
    Ok(result)
}

/// Loads the global Saved Page records in persisted order (schema v3).
///
/// The canonical URL key is opaque here (bounded URL shape only); duplicate
/// canonical keys are corruption ([`CorruptionIssue::DuplicateCanonicalUrl`])
/// — the at-most-one-active-record-per-key invariant cannot hold in a valid
/// store. Collection membership is guaranteed to resolve by the structural
/// foreign-key check that runs before restore; the reference is projected
/// onto the loaded collection set only to keep the persisted contract
/// closed. Per-row data validation runs through the one shared
/// [`validate_saved_page_row`] helper, so the list load, restore, and the
/// single-row reads reject exactly the same corruption (review F3).
pub(crate) fn load_saved_pages(
    connection: &Connection,
    identities: &mut HashSet<u64>,
    collections: &[CollectionRecord],
) -> Result<Vec<SavedPageRecord>, PersistenceError> {
    let collection_ids = collections.iter().map(|c| c.id).collect::<HashSet<_>>();
    let mut statement = connection
        .prepare(
            "SELECT id, canonical_url, captured_title, user_label, created_at, updated_at, \
         collection_id, position FROM saved_pages ORDER BY position",
        )
        .map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, i64>(7)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut result = Vec::new();
    let mut urls = HashSet::new();
    for (expected, row) in rows.enumerate() {
        let (
            id,
            canonical_url,
            captured_title,
            user_label,
            created_at,
            updated_at,
            collection_id,
            position,
        ) = row.map_err(schema::storage)?;
        require_order("saved_pages", expected, position)?;
        let id = require_id("saved_pages", id, identities)?;
        if !urls.insert(canonical_url.clone()) {
            return Err(PersistenceError::Corruption(
                CorruptionIssue::DuplicateCanonicalUrl { id },
            ));
        }
        result.push(validate_saved_page_row(
            (
                id as i64,
                canonical_url,
                captured_title,
                user_label,
                created_at,
                updated_at,
                collection_id,
            ),
            &collection_ids,
        )?);
    }
    Ok(result)
}

/// Validates one persisted saved-pages row against the row-data invariants
/// shared by the full list/restore load **and** the single-row reads (review
/// F3): identity range, bounded canonical-URL shape, captured-title/label
/// bounds, in-range monotonic timestamps, and a collection reference that
/// resolves in the loaded collection set. The duplicate-canonical-key
/// invariant is a set-wide check the callers run around this helper.
fn validate_saved_page_row(
    row: SavedPageRow,
    collection_ids: &HashSet<u64>,
) -> Result<SavedPageRecord, PersistenceError> {
    let (id, canonical_url, captured_title, user_label, created_at, updated_at, collection_id) =
        row;
    let id = persisted_id("saved_pages", id)?;
    if !validation::valid_url(&canonical_url) {
        return Err(invalid_value(
            "saved_pages",
            format!("saved page {id} has an invalid canonical URL"),
        ));
    }
    if captured_title.len() > validation::MAX_CAPTURED_TITLE_LEN
        || captured_title
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
    {
        return Err(invalid_value(
            "saved_pages",
            format!("saved page {id} has an invalid captured title"),
        ));
    }
    if let Some(label) = &user_label {
        if label.is_empty()
            || label.len() > validation::MAX_USER_LABEL_LEN
            || label
                .bytes()
                .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return Err(invalid_value(
                "saved_pages",
                format!("saved page {id} has an invalid user label"),
            ));
        }
    }
    if created_at < 0 || updated_at < 0 || updated_at < created_at {
        return Err(invalid_value(
            "saved_pages",
            format!("saved page {id} has invalid timestamps"),
        ));
    }
    let collection_id = match collection_id {
        None => None,
        Some(value) => {
            let value = persisted_id("saved_pages", value)?;
            if !collection_ids.contains(&value) {
                return Err(invalid_value(
                    "saved_pages",
                    format!("saved page {id} references unknown collection {value}"),
                ));
            }
            Some(value)
        }
    };
    Ok(SavedPageRecord {
        id,
        canonical_url,
        captured_title,
        user_label,
        created_at: created_at as u64,
        updated_at: updated_at as u64,
        collection_id,
    })
}
