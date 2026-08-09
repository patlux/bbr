//! Schema-v3 Saved Pages validation (B1 packet `saved-pages-portable-persistence`):
//! the flat-collection set and the global saved-page set. Split out of
//! `validation.rs` so the shared snapshot validator keeps its pre-B1 size;
//! this is the B1-owned validation group of the same write/restore contract.
//!
//! Both validators mirror exactly what restore refuses, so a rejected write
//! can never surface as corruption on the next load: a duplicate canonical
//! URL or a dangling collection reference can never enter the committed
//! store, and the whole write rejects before any mutation.

use std::collections::HashSet;

use crate::validation::{
    valid_url, validate_id, MAX_CAPTURED_TITLE_LEN, MAX_COLLECTION_TITLE_LEN, MAX_SQLITE_ID,
    MAX_USER_LABEL_LEN,
};
use crate::{CollectionRecord, InvalidSnapshotIssue, SavedPageRecord};

/// Validates the flat collection set (schema v3): unique, in-range ids and
/// bounded, non-empty titles. The whole collection write rejects before any
/// mutation on the first invalid record.
pub(crate) fn validate_collections(
    collections: &[CollectionRecord],
) -> Result<(), InvalidSnapshotIssue> {
    let mut ids = HashSet::new();
    for collection in collections {
        validate_id("collection", collection.id)?;
        if !ids.insert(collection.id) {
            return Err(InvalidSnapshotIssue::DuplicateIdentity { id: collection.id });
        }
        if collection.title.is_empty() {
            return Err(InvalidSnapshotIssue::InvalidCollection {
                id: collection.id,
                detail: "empty collection title".into(),
            });
        }
        if collection.title.len() > MAX_COLLECTION_TITLE_LEN {
            return Err(InvalidSnapshotIssue::InvalidCollection {
                id: collection.id,
                detail: format!("collection title exceeds {MAX_COLLECTION_TITLE_LEN} bytes"),
            });
        }
        if collection
            .title
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return Err(InvalidSnapshotIssue::InvalidCollection {
                id: collection.id,
                detail: "collection title contains control bytes".into(),
            });
        }
    }
    Ok(())
}

/// Validates the global saved-page set (schema v3) against the collection
/// set it references: unique ids, unique canonical URLs, bounded URL/title/
/// label shape, in-range monotonic timestamps, and collection references
/// that name a known collection (or are absent). The whole write rejects
/// before any mutation, so a duplicate canonical URL or a dangling
/// collection reference can never enter the committed store.
pub(crate) fn validate_saved_pages(
    pages: &[SavedPageRecord],
    collections: &[CollectionRecord],
) -> Result<(), InvalidSnapshotIssue> {
    let collection_ids = collections.iter().map(|c| c.id).collect::<HashSet<_>>();
    let mut ids = HashSet::new();
    let mut urls = HashSet::new();
    for page in pages {
        validate_id("saved page", page.id)?;
        if !ids.insert(page.id) {
            return Err(InvalidSnapshotIssue::DuplicateIdentity { id: page.id });
        }
        // The key is opaque here: bounded URL shape only. Canonicality is
        // the `bbr-domain` seam's rule; persistence enforces the
        // at-most-one-active-record-per-key invariant on the exact key.
        if !valid_url(&page.canonical_url) {
            return Err(InvalidSnapshotIssue::InvalidSavedPage {
                id: page.id,
                detail: "invalid canonical URL".into(),
            });
        }
        if !urls.insert(page.canonical_url.clone()) {
            return Err(InvalidSnapshotIssue::DuplicateCanonicalUrl { id: page.id });
        }
        if page.captured_title.len() > MAX_CAPTURED_TITLE_LEN {
            return Err(InvalidSnapshotIssue::InvalidSavedPage {
                id: page.id,
                detail: format!("captured title exceeds {MAX_CAPTURED_TITLE_LEN} bytes"),
            });
        }
        if page
            .captured_title
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return Err(InvalidSnapshotIssue::InvalidSavedPage {
                id: page.id,
                detail: "captured title contains control bytes".into(),
            });
        }
        if let Some(label) = &page.user_label {
            if label.is_empty() {
                return Err(InvalidSnapshotIssue::InvalidSavedPage {
                    id: page.id,
                    detail: "empty user label".into(),
                });
            }
            if label.len() > MAX_USER_LABEL_LEN {
                return Err(InvalidSnapshotIssue::InvalidSavedPage {
                    id: page.id,
                    detail: format!("user label exceeds {MAX_USER_LABEL_LEN} bytes"),
                });
            }
            if label
                .bytes()
                .any(|byte| byte == 0 || byte.is_ascii_control())
            {
                return Err(InvalidSnapshotIssue::InvalidSavedPage {
                    id: page.id,
                    detail: "user label contains control bytes".into(),
                });
            }
        }
        if page.created_at > MAX_SQLITE_ID || page.updated_at > MAX_SQLITE_ID {
            return Err(InvalidSnapshotIssue::InvalidSavedPage {
                id: page.id,
                detail: "timestamp exceeds the SQLite integer range".into(),
            });
        }
        if page.updated_at < page.created_at {
            return Err(InvalidSnapshotIssue::InvalidSavedPage {
                id: page.id,
                detail: "updated_at precedes created_at".into(),
            });
        }
        if let Some(collection_id) = page.collection_id {
            if !collection_ids.contains(&collection_id) {
                return Err(InvalidSnapshotIssue::UnknownCollection { id: collection_id });
            }
        }
    }
    Ok(())
}
