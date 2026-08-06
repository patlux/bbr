use rusqlite::{Connection, Transaction};

use crate::{PersistenceError, SchemaIssue};

pub(crate) const SCHEMA_VERSION: u32 = 1;

const CREATE_SCHEMA_V1: &str = r#"
CREATE TABLE profiles (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    hue TEXT NOT NULL,
    position INTEGER NOT NULL
);
CREATE TABLE spaces (
    id INTEGER PRIMARY KEY NOT NULL,
    profile_id INTEGER NOT NULL REFERENCES profiles(id),
    title TEXT NOT NULL,
    position INTEGER NOT NULL,
    active_tab_id INTEGER
);
CREATE TABLE tabs (
    id INTEGER PRIMARY KEY NOT NULL,
    space_id INTEGER NOT NULL REFERENCES spaces(id),
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    position INTEGER NOT NULL,
    pinned INTEGER NOT NULL
);
CREATE TABLE recently_closed (
    tab_id INTEGER PRIMARY KEY NOT NULL,
    space_id INTEGER NOT NULL REFERENCES spaces(id),
    title TEXT,
    url TEXT NOT NULL,
    pinned INTEGER NOT NULL,
    former_position INTEGER NOT NULL,
    position INTEGER NOT NULL
);
CREATE TABLE splits (
    space_id INTEGER PRIMARY KEY NOT NULL,
    primary_tab_id INTEGER NOT NULL,
    secondary_tab_id INTEGER NOT NULL
);
CREATE TABLE windows (
    id INTEGER PRIMARY KEY NOT NULL,
    position INTEGER NOT NULL,
    x REAL,
    y REAL,
    width REAL,
    height REAL,
    sidebar_width REAL NOT NULL,
    sidebar_collapsed INTEGER NOT NULL,
    last_expanded_width REAL NOT NULL
);
"#;

struct ExpectedColumn {
    name: &'static str,
    not_null: bool,
    primary_key: bool,
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
];

pub(crate) fn initialize_or_migrate(connection: &mut Connection) -> Result<(), PersistenceError> {
    let version = user_version(connection)?;
    match version {
        SCHEMA_VERSION => validate_schema(connection),
        0 => migrate_zero_to_one(connection),
        found if found > SCHEMA_VERSION => Err(PersistenceError::IncompatibleSchema {
            found,
            supported: SCHEMA_VERSION,
        }),
        found => Err(PersistenceError::Schema(SchemaIssue::MissingTable(
            format!("unsupported schema version {found}"),
        ))),
    }
}

pub(crate) fn validate_current(connection: &Connection) -> Result<(), PersistenceError> {
    let version = user_version(connection)?;
    if version > SCHEMA_VERSION {
        return Err(PersistenceError::IncompatibleSchema {
            found: version,
            supported: SCHEMA_VERSION,
        });
    }
    if version != SCHEMA_VERSION {
        return Err(PersistenceError::Schema(SchemaIssue::MissingTable(
            format!("schema version {SCHEMA_VERSION}"),
        )));
    }
    validate_schema(connection)
}

fn migrate_zero_to_one(connection: &mut Connection) -> Result<(), PersistenceError> {
    let transaction = connection
        .transaction()
        .map_err(|error| PersistenceError::Migration(error.to_string()))?;
    let result = (|| {
        transaction
            .execute_batch(CREATE_SCHEMA_V1)
            .map_err(|error| PersistenceError::Migration(error.to_string()))?;
        transaction
            .pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|error| PersistenceError::Migration(error.to_string()))?;
        validate_schema(&transaction)?;
        Ok(())
    })();
    match result {
        Ok(()) => transaction
            .commit()
            .map_err(|error| PersistenceError::Migration(error.to_string())),
        Err(error) => Err(error),
    }
}

pub(crate) fn validate_schema(connection: &Connection) -> Result<(), PersistenceError> {
    for (table, expected_columns) in TABLES {
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
    for table in [
        "recently_closed",
        "splits",
        "tabs",
        "spaces",
        "profiles",
        "windows",
    ] {
        total += transaction
            .execute(&format!("DELETE FROM {table}"), [])
            .map_err(storage)? as u64;
    }
    Ok(total)
}

pub(crate) fn storage(error: rusqlite::Error) -> PersistenceError {
    PersistenceError::Storage(error.to_string())
}
