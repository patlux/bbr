//! Schema-v5 tab organization table shape (task packet
//! `closed-pins-and-favorites`): the expected `folders`, `folder_members`
//! and `favorites` tables, their strict foreign keys, and the child-first
//! DELETE order. Split out of `schema_tables.rs` like the Saved Pages
//! shape; `schema_tables.rs` chains these definitions into its one
//! validation pass.

use crate::schema_tables::ExpectedColumn;

const fn column(name: &'static str, not_null: bool, primary_key: bool) -> ExpectedColumn {
    ExpectedColumn {
        name,
        not_null,
        primary_key,
    }
}

/// The schema-v5 organization tables, chained after the other tables by
/// [`crate::schema_tables::validate_schema`].
pub(crate) const ORGANIZATION_TABLES: &[(&str, &[ExpectedColumn])] = &[
    (
        "folders",
        &[
            column("id", true, true),
            column("space_id", true, false),
            column("title", true, false),
            column("position", true, false),
        ],
    ),
    (
        "folder_members",
        &[
            column("tab_id", true, true),
            column("folder_id", true, false),
            column("position", true, false),
        ],
    ),
    (
        "favorites",
        &[
            column("tab_id", true, true),
            column("position", true, false),
        ],
    ),
];

/// The strict foreign keys of the organization tables (none for any other
/// table): a folder belongs to a space, a member is a tab of a folder, and
/// a favorite is a tab.
pub(crate) fn organization_foreign_keys(
    table: &str,
) -> Vec<(&'static str, &'static str, &'static str)> {
    match table {
        "folders" => vec![("spaces", "space_id", "id")],
        "folder_members" => vec![("tabs", "tab_id", "id"), ("folders", "folder_id", "id")],
        "favorites" => vec![("tabs", "tab_id", "id")],
        _ => Vec::new(),
    }
}

/// The child-first DELETE order for the organization tables: they
/// reference `tabs`/`spaces`, so the write paths (foreign keys on) empty
/// them before the browser tables.
pub(crate) const ORGANIZATION_DELETE_TABLES: &[&str] = &["favorites", "folder_members", "folders"];
