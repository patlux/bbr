-- Schema v4 duplicate-canonical-URL fixture: the version stamp matches the
-- current schema (4) and the shape validates, but two `saved_pages` rows
-- carry the same canonical URL key. The fixture's `saved_pages` table
-- deliberately omits the UNIQUE constraint so the duplicate rows can be
-- inserted; the at-most-one-active-record-per-key invariant is enforced by
-- the loader, which refuses the store as `Corrupt` read-only and the write
-- path discards it under the development rule.
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
CREATE TABLE saved_pages (id INTEGER PRIMARY KEY NOT NULL, canonical_url TEXT NOT NULL, captured_title TEXT NOT NULL, user_label TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, collection_id INTEGER REFERENCES collections(id), position INTEGER NOT NULL);
PRAGMA user_version = 4;
INSERT INTO settings VALUES (1, 'duckduckgo', NULL);
INSERT INTO collections VALUES (1, 'Work', 0);
INSERT INTO saved_pages VALUES (1000, 'https://example.test/dupe', 'First', NULL, 100, 100, NULL, 0);
INSERT INTO saved_pages VALUES (1001, 'https://example.test/dupe', 'Second', NULL, 200, 200, NULL, 1);
