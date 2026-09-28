-- Schema v4 current-schema fixture (B1 Saved Pages): the complete v4 shape
-- with representative rows, including a flat collection and member records
-- and a non-default space paper (`rich`, grain on).
-- `restore_read_only` must load it as `Loaded` with zero writes (the
-- inventory byte-identity test proves it).
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
INSERT INTO profiles VALUES (1, 'Personal', 'cobalt', 0);
INSERT INTO spaces VALUES (10, 1, 'Main', 'pine', 'emoji', '🚀', 0, 100, 'rich', 1);
INSERT INTO tabs VALUES (100, 10, 'Pinned', 'https://example.test/pinned', 0, 1);
INSERT INTO tabs VALUES (101, 10, 'Regular', 'https://example.test/regular', 1, 0);
INSERT INTO recently_closed VALUES (900, 10, NULL, 'https://closed.test/', 0, 2, 0);
INSERT INTO splits VALUES (10, 100, 101);
INSERT INTO windows VALUES (50, 0, -100.0, 20.0, 1080.0, 700.0, 260.0, 0, 260.0);
INSERT INTO settings VALUES (1, 'custom', 'https://search.example.test/?q=%s');
INSERT INTO downloads VALUES (700, 'succeeded', 4096, '/tmp/report.pdf', 1000, 0, 'https://example.test/report.pdf', 'report.pdf', NULL, 1);
INSERT INTO downloads VALUES (701, 'cancelled', 12, NULL, 2000, 1, 'https://example.test/archive.zip', 'archive.zip', NULL, 2);
INSERT INTO command_history VALUES (800, 'Example', 'https://example.test/', 3, 2000, 0);
-- Flat collections and global Saved Page records (canonical keys are
-- duplicate-free; membership resolves; position keys are dense).
INSERT INTO collections VALUES (1, 'Work', 0);
INSERT INTO collections VALUES (2, 'Reading', 1);
INSERT INTO saved_pages VALUES (1000, 'https://example.test/pinned', 'Pinned', NULL, 100, 100, 1, 0);
INSERT INTO saved_pages VALUES (1001, 'https://example.test/regular', 'Regular', 'My label', 200, 300, 1, 1);
INSERT INTO saved_pages VALUES (1002, 'https://archive.test/a', 'Archive', NULL, 400, 400, NULL, 2);
