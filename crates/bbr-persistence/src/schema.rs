use rusqlite::Connection;

use crate::PersistenceError;

/// The current persistence schema version stamp.
///
/// The store carries this stamp in SQLite's `PRAGMA user_version`. Schema v3
/// adds the global Saved Pages tables (`saved_pages`, `collections`) on top
/// of v2's space appearance identity, global settings record, terminal
/// download history, and visited command-history entries. There are **no
/// migrations**: a store whose stamp does not match this constant is
/// discarded and recreated empty (see [`crate::setup`]).
pub(crate) const SCHEMA_VERSION: u32 = 3;

pub(crate) const CREATE_SCHEMA_V3: &str = r#"
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
    tint TEXT NOT NULL,
    glyph_kind TEXT NOT NULL,
    glyph TEXT NOT NULL,
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
CREATE TABLE settings (
    id INTEGER PRIMARY KEY NOT NULL,
    search_engine TEXT NOT NULL,
    custom_search_template TEXT
);
CREATE TABLE downloads (
    id INTEGER PRIMARY KEY NOT NULL,
    outcome TEXT NOT NULL,
    size INTEGER NOT NULL,
    destination TEXT,
    terminal_at INTEGER NOT NULL,
    position INTEGER NOT NULL,
    url TEXT NOT NULL,
    filename TEXT NOT NULL,
    retry_of INTEGER,
    attempt INTEGER NOT NULL
);
CREATE TABLE command_history (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    visit_count INTEGER NOT NULL,
    last_visited INTEGER NOT NULL,
    position INTEGER NOT NULL
);
-- The single global settings row is structural state: a fresh store is
-- created with the deterministic defaults (DuckDuckGo, no custom template)
-- so "exactly one settings row" is an invariant restore can rely on.
INSERT INTO settings(id, search_engine, custom_search_template) VALUES (1, 'duckduckgo', NULL);
CREATE TABLE collections (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    position INTEGER NOT NULL
);
CREATE TABLE saved_pages (
    id INTEGER PRIMARY KEY NOT NULL,
    canonical_url TEXT NOT NULL UNIQUE,
    captured_title TEXT NOT NULL,
    user_label TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    collection_id INTEGER REFERENCES collections(id),
    position INTEGER NOT NULL
);


"#;

// The schema-shape validators and version/order helpers live in
// `schema_tables`; re-export them here so every caller keeps using
// `schema::…`.
pub(crate) use crate::schema_tables::{
    check_foreign_keys, deleted_rows, storage, user_version, validate_schema,
};

pub(crate) fn validate_all(connection: &Connection) -> Result<(), PersistenceError> {
    validate_schema(connection)?;
    check_foreign_keys(connection)
}
