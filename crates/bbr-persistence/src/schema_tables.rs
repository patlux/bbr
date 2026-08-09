//! Expected table/column/foreign-key shape of the v2 schema, and the
//! structural validator that enforces it.

use rusqlite::{Connection, Transaction};

use crate::schema_tables_saved_pages::{
    saved_pages_foreign_keys, SAVED_PAGES_DELETE_TABLES, SAVED_PAGES_TABLES,
};
use crate::{PersistenceError, SchemaIssue};

pub(crate) struct ExpectedColumn {
    pub(crate) name: &'static str,
    pub(crate) not_null: bool,
    pub(crate) primary_key: bool,
}

const TABLES: &[(&str, &[ExpectedColumn])] = &[
    (
        "profiles",
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
                name: "hue",
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
        "spaces",
        &[
            ExpectedColumn {
                name: "id",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "profile_id",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "title",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "tint",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "glyph_kind",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "glyph",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "position",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "active_tab_id",
                not_null: false,
                primary_key: false,
            },
        ],
    ),
    (
        "tabs",
        &[
            ExpectedColumn {
                name: "id",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "space_id",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "title",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "url",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "position",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "pinned",
                not_null: true,
                primary_key: false,
            },
        ],
    ),
    (
        "recently_closed",
        &[
            ExpectedColumn {
                name: "tab_id",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "space_id",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "title",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "url",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "pinned",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "former_position",
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
        "splits",
        &[
            ExpectedColumn {
                name: "space_id",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "primary_tab_id",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "secondary_tab_id",
                not_null: true,
                primary_key: false,
            },
        ],
    ),
    (
        "windows",
        &[
            ExpectedColumn {
                name: "id",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "position",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "x",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "y",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "width",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "height",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "sidebar_width",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "sidebar_collapsed",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "last_expanded_width",
                not_null: true,
                primary_key: false,
            },
        ],
    ),
    (
        "settings",
        &[
            ExpectedColumn {
                name: "id",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "search_engine",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "custom_search_template",
                not_null: false,
                primary_key: false,
            },
        ],
    ),
    (
        "downloads",
        &[
            ExpectedColumn {
                name: "id",
                not_null: true,
                primary_key: true,
            },
            ExpectedColumn {
                name: "outcome",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "size",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "destination",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "terminal_at",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "position",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "url",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "filename",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "retry_of",
                not_null: false,
                primary_key: false,
            },
            ExpectedColumn {
                name: "attempt",
                not_null: true,
                primary_key: false,
            },
        ],
    ),
    (
        "command_history",
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
                name: "url",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "visit_count",
                not_null: true,
                primary_key: false,
            },
            ExpectedColumn {
                name: "last_visited",
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
];

pub(crate) fn validate_schema(connection: &Connection) -> Result<(), PersistenceError> {
    // The schema-v3 Saved Pages tables are chained after the v2 tables so
    // the whole expected shape is validated in one pass (B1-owned shape in
    // `schema_tables_saved_pages`).
    for (table, expected_columns) in TABLES.iter().chain(SAVED_PAGES_TABLES) {
        validate_columns(connection, table, expected_columns)?;
        validate_foreign_keys(connection, table)?;
    }
    Ok(())
}

fn validate_columns(
    connection: &Connection,
    table: &str,
    expected: &[ExpectedColumn],
) -> Result<(), PersistenceError> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, i64>(3)? != 0,
                row.get::<_, i64>(5)? != 0,
            ))
        })
        .map_err(storage)?;
    let actual = rows.collect::<Result<Vec<_>, _>>().map_err(storage)?;
    if actual.is_empty() {
        return Err(PersistenceError::Schema(SchemaIssue::MissingTable(
            table.to_owned(),
        )));
    }
    if actual.len() != expected.len()
        || actual
            .iter()
            .zip(expected)
            .any(|((name, _, _), column)| name != column.name)
    {
        return Err(PersistenceError::Schema(SchemaIssue::UnexpectedColumns {
            table: table.to_owned(),
        }));
    }
    for ((_, not_null, primary_key), column) in actual.iter().zip(expected) {
        if *not_null != column.not_null || *primary_key != column.primary_key {
            return Err(PersistenceError::Schema(SchemaIssue::InvalidColumnShape {
                table: table.to_owned(),
                column: column.name.to_owned(),
            }));
        }
    }
    Ok(())
}

fn validate_foreign_keys(connection: &Connection, table: &str) -> Result<(), PersistenceError> {
    let expected: Vec<(&str, &str, &str)> = match table {
        "spaces" => vec![("profiles", "profile_id", "id")],
        "tabs" => vec![("spaces", "space_id", "id")],
        "recently_closed" => vec![("spaces", "space_id", "id")],
        "saved_pages" => saved_pages_foreign_keys(),
        _ => Vec::new(),
    };
    let mut statement = connection
        .prepare(&format!("PRAGMA foreign_key_list({table})"))
        .map_err(storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(storage)?;
    let mut actual = rows.collect::<Result<Vec<_>, _>>().map_err(storage)?;
    actual.sort();
    let mut expected = expected
        .into_iter()
        .map(|(owner, from, to)| (owner.to_owned(), from.to_owned(), to.to_owned()))
        .collect::<Vec<_>>();
    expected.sort();
    if actual != expected {
        return Err(PersistenceError::Schema(
            SchemaIssue::UnexpectedForeignKeys {
                table: table.to_owned(),
            },
        ));
    }
    Ok(())
}

pub(crate) fn check_foreign_keys(connection: &Connection) -> Result<(), PersistenceError> {
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .map_err(storage)?;
    if count == 0 {
        Ok(())
    } else {
        Err(PersistenceError::Corruption(
            crate::CorruptionIssue::ForeignKeyViolation,
        ))
    }
}

pub(crate) fn user_version(connection: &Connection) -> Result<u32, PersistenceError> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(storage)?;
    u32::try_from(version).map_err(|_| PersistenceError::Storage("negative user_version".into()))
}

pub(crate) fn deleted_rows(transaction: &Transaction<'_>) -> Result<u64, PersistenceError> {
    let mut total = 0;
    // The schema-v3 Saved Pages DELETE order is appended after the v2 tables
    // (child `saved_pages` before parent `collections`).
    for table in [
        "recently_closed",
        "splits",
        "tabs",
        "spaces",
        "profiles",
        "windows",
        "settings",
        "downloads",
        "command_history",
    ]
    .iter()
    .chain(SAVED_PAGES_DELETE_TABLES)
    {
        total += transaction
            .execute(&format!("DELETE FROM {table}"), [])
            .map_err(storage)? as u64;
    }
    Ok(total)
}

pub(crate) fn storage(error: rusqlite::Error) -> PersistenceError {
    PersistenceError::Storage(error.to_string())
}
