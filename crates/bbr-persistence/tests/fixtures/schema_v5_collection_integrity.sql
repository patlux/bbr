-- Foreign-key enforcement is disabled for this fixture so the dangling
-- reference can be inserted; the repository's structural check refuses the
-- store read-only as corruption (see the test).
PRAGMA foreign_keys = OFF;
-- Schema v4 collection-integrity fixture: the version stamp matches the
-- current schema (4), the shape validates (including the declared
-- `saved_pages.collection_id -> collections(id)` foreign key), but a
-- `saved_pages` row references collection id 999, which does not exist.
-- The fixture does not enable foreign-key enforcement, so the dangling
-- reference can be inserted; the structural foreign-key check then refuses
-- the store read-only as `Corrupt` and the write path discards it. A
-- dangling membership can never enter a committed store.
CREATE TABLE profiles (id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, hue TEXT NOT NULL, position INTEGER NOT NULL);
CREATE TABLE spaces (id INTEGER PRIMARY KEY NOT NULL, profile_id INTEGER NOT NULL REFERENCES profiles(id), title TEXT NOT NULL, tint TEXT NOT NULL, glyph_kind TEXT NOT NULL, glyph TEXT NOT NULL, position INTEGER NOT NULL, active_tab_id INTEGER, tint_intensity TEXT NOT NULL, grain INTEGER NOT NULL);
CREATE TABLE tabs (id INTEGER PRIMARY KEY NOT NULL, space_id INTEGER NOT NULL REFERENCES spaces(id), title TEXT NOT NULL, url TEXT NOT NULL, position INTEGER NOT NULL, pinned INTEGER NOT NULL);
CREATE TABLE recently_closed (tab_id INTEGER PRIMARY KEY NOT NULL, space_id INTEGER NOT NULL REFERENCES spaces(id), title TEXT, url TEXT NOT NULL, pinned INTEGER NOT NULL, former_position INTEGER NOT NULL, position INTEGER NOT NULL);
CREATE TABLE splits (space_id INTEGER PRIMARY KEY NOT NULL, primary_tab_id INTEGER NOT NULL, secondary_tab_id INTEGER NOT NULL);
CREATE TABLE windows (id INTEGER PRIMARY KEY NOT NULL, position INTEGER NOT NULL, x REAL, y REAL, width REAL, height REAL, sidebar_width REAL NOT NULL, sidebar_collapsed INTEGER NOT NULL, last_expanded_width REAL NOT NULL);
CREATE TABLE settings (id INTEGER PRIMARY KEY NOT NULL, search_engine TEXT NOT NULL, custom_search_template TEXT);
CREATE TABLE downloads (id INTEGER PRIMARY KEY NOT NULL, outcome TEXT NOT NULL, size INTEGER NOT NULL, destination TEXT, terminal_at INTEGER NOT NULL, position INTEGER NOT NULL, url TEXT NOT NULL, filename TEXT NOT NULL, retry_of INTEGER, attempt INTEGER NOT NULL);
CREATE TABLE command_history (id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, url TEXT NOT NULL, visit_count INTEGER NOT NULL, last_visited INTEGER NOT NULL, position INTEGER NOT NULL);
CREATE TABLE collections (id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, position INTEGER NOT NULL);
CREATE TABLE saved_pages (id INTEGER PRIMARY KEY NOT NULL, canonical_url TEXT NOT NULL UNIQUE, captured_title TEXT NOT NULL, user_label TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, collection_id INTEGER REFERENCES collections(id), position INTEGER NOT NULL);
PRAGMA user_version = 4;
INSERT INTO settings VALUES (1, 'duckduckgo', NULL);
INSERT INTO collections VALUES (1, 'Work', 0);
INSERT INTO saved_pages VALUES (1000, 'https://example.test/a', 'A', NULL, 100, 100, 1, 0);
INSERT INTO saved_pages VALUES (1001, 'https://example.test/b', 'B', NULL, 200, 200, 999, 1);
