//! Schema-v3 Saved Pages table shape (B1 packet
//! `saved-pages-portable-persistence`): the expected `collections` and
//! `saved_pages` tables, the strict `saved_pages.collection_id ->
//! collections(id)` foreign-key expectation, and the child-first DELETE
//! order. Split out of `schema_tables.rs` so the v2 structural validator
//! keeps its pre-B1 size; `schema_tables.rs` chains these definitions into
//! its one validation pass.

use crate::schema_tables::ExpectedColumn;

/// The schema-v3 Saved Pages tables, chained after the v2 tables by
/// [`crate::schema_tables::validate_schema`].
pub(crate) const SAVED_PAGES_TABLES: &[(&str, &[ExpectedColumn])] = &[
    (
        "collections",
        &[
            ExpectedColumn {
                name: "id",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "title",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "position",
                not_null: true,
                primary_key: false,
            },
        ],
    ),
    (
        "saved_pages",
        &[
            ExpectedColumn {
                name: "id",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "canonical_url",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "captured_title",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "user_label",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "created_at",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "updated_at",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "collection_id",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "position",
                not_null: true,
                primary_key: false,
            },
        ],
    ),
];

/// The schema-v3 Saved Pages foreign-key expectation: a strict reference
/// from `saved_pages.collection_id` into `collections.id`.
pub(crate) fn saved_pages_foreign_keys() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![("collections", "collection_id", "id")]
}

/// The child-first DELETE order for the v3 Saved Pages tables, appended by
/// [`crate::schema_tables::deleted_rows`]: `saved_pages` (which references
/// `collections`) must be emptied before `collections` because the write
/// path enables foreign keys.
pub(crate) const SAVED_PAGES_DELETE_TABLES: &[&str] = &["saved_pages", "collections"];
