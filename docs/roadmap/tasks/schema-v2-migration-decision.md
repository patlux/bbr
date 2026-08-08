# Schema v2 — decision record and implementation (no migrations)

Status: **closed and implemented** (lane `parallel/schema-v2`, base `a24321d`).

The migration question this document was opened to answer is **closed** by the
binding product decision of 2026-08-08: BBR is in development, so on any schema
change the existing data is deleted. There are no migrations and no backward
compatibility. Instead of migration paths, the store carries a version stamp,
and a store whose stamp does not match the current schema is discarded and
recreated empty. Deleting a developer's local browsing state is acceptable;
silently loading data that does not match the code is not.

This document records the implemented design, why the migration question is
closed, and the evidence. It supersedes the earlier draft analysis (which
explored a `0 → 1 → 2` migration chain, rollback fixtures, backfill, and
`MigrationRequired` errors); all of that is obsolete and was deliberately not
implemented.

## 1. The version stamp and discard-on-mismatch

- The store carries the current schema version in SQLite's
  `PRAGMA user_version` (`SCHEMA_VERSION = 2` in `schema.rs`).
- `setup::prepare_store` decides the disposition of every store before any
  read or write:
  - stamp matches → validate structure (tables/columns/foreign keys) →
    `StoreLoad::Loaded`;
  - stamp matches but structure is invalid → discard and recreate empty →
    `StoreLoad::Discarded { Corrupt }`;
  - stamp mismatch with existing data → discard and recreate empty →
    `StoreLoad::Discarded { Unstamped | VersionMismatch { found } }`;
  - stamp mismatch on a fresh empty database → create the v2 schema →
    `StoreLoad::Loaded` (a genuine first run discards nothing).
- Data-level corruption (invalid order keys, duplicate identities) is only
  detectable while restoring, so `initialize_and_restore` runs a second
  discard pass when restore reports `Corruption`: the write path always
  returns a complete snapshot — **never a panic, never a partial load, never
  a fatal error for a mismatched or corrupt store**.
- Discard is in-place and transactional: every user table and view is
  dropped, the v2 schema is recreated, the stamp is pinned, and the fresh
  schema is validated before commit. Foreign-key enforcement is disabled for
  the drop only (rusqlite enables it by default) and stays enabled for
  normal writes.
- The disposition is part of `RestoreOutcome::store`. There is no logging
  dependency in the portable crate; the `StoreLoad` value is the log surface
  the app prints on startup. `restore_read_only` (the no-write path) refuses
  a mismatched store with a typed `PersistenceError::SchemaMismatch { found,
  supported }` and leaves it byte-for-byte untouched; it never discards.

## 2. The v2 shape (four field groups)

```text
spaces           + tint TEXT NOT NULL, glyph_kind TEXT NOT NULL, glyph TEXT NOT NULL
settings         new single-row table (id 1): search_engine, custom_search_template
downloads        new table: id, outcome, size, destination, terminal_at, position
command_history  new table: id, title, url, visit_count, last_visited, position
```

- **Space appearance**: one curated tint identity (`spaces.tint`, closed
  nine-token palette mirroring the delivered space-appearance contract) and
  one curated glyph identity (`spaces.glyph_kind` ∈ `emoji|glyph` plus
  `spaces.glyph`: an allowlisted emoji scalar or a closed BBR icon-glyph
  name). Identity data, not free-form styling: unknown tokens are corruption,
  never a partial fallback.
- **Settings**: one global row (`id = 1`) with a closed search-engine id
  (`duckduckgo|google|bing|ecosia|kagi|custom`) and an optional custom search
  template. The template is validated against the delivered search-settings
  contract's rules (non-empty, absolute `http(s)://`, exactly one `%s`,
  valid `%XX` sequences) and must be present exactly when the engine is
  `custom`. The row is seeded with the deterministic DuckDuckGo defaults at
  schema creation, so "exactly one settings row" is a restore invariant.
- **Downloads history**: terminal attempts surviving a restart — attempt
  identity, terminal outcome (`succeeded|failed|cancelled`), received size,
  resolved destination path (`NULL` when the attempt ended before a
  destination was decided), and the deterministic monotonic terminal
  timestamp. Dense per-table order keys keep the history order stable across
  restarts.
- **Command history**: visited entries with the exact shape the later ranked
  history provider needs (`bbr-command`'s `HistoryEntry`): stable identity,
  display title, normalized URL, visit count, and a monotonic recency marker
  (larger = more recent).

## 3. Ownership: bbr-domain owns identities, bbr-persistence owns storage

`bbr-persistence` deliberately does **not** depend on `bbr-domain` (existing
invariant, `ProfileHue` precedent): the closed identity sets above are
mirrored in the persistence DTOs as the repository's **persisted contract**.
The semantic owners of those identities and their validation invariants are
the portable feature contracts that arrive via serial integration:

- space appearance → the delivered space-appearance contract
  (`bbr-domain` `space_appearance/**`);
- search settings → the delivered search-settings contract
  (`bbr-command` `search_settings/**`);
- downloads → the delivered downloads capability contract
  (`bbr-web` `downloads/**`);
- command history → the command corpus's `HistoryEntry`
  (`bbr-command`).

This slice therefore makes **no changes to `bbr-domain`**: adding duplicate
identity types now would collide with those parked contracts (whose
`lib.rs` hunks are separately owned) or duplicate state permanently. The
later integration lanes map their feature types onto these persisted
contracts; persistence re-validates on write and rejects unknown tokens on
restore (belt and suspenders, mirroring the existing URL/hue strictness).

## 4. What was deliberately not done

- No `0 → 1 → 2` migration chain, no backfill, no rollback fixtures, no
  `MigrationRequired` errors, no downgrade guards. All removed; the schema
  v0/v1 fixtures now serve the discard tests (`Unstamped` / `VersionMismatch`).
- No new dependency crates; no changes outside `crates/bbr-persistence/**`
  and this document.
- No changes to `bbr-domain`, `bbr-app`, `bbr-browser`, `bbr-web`,
  `bbr-command`, `bbr-testkit`, `justfile`, `ROADMAP.md`, or phase docs —
  those remain serial-integration owned.
- The performance benchmark keeps its three operations (renamed to the v2
  semantics); no new latency claims.

## 5. Tests and evidence

`crates/bbr-persistence/tests/repository_contract.rs` (13 tests):

- matching stamp loads without writes (`schema_v2` fixture, inventory equal);
- mismatched stamp (v1 legacy) discards on the write path, yields an empty
  store, is refused untouched by the read-only path, and is **immediately
  rewritable** with a full v2 round-trip (the discard-then-rewrite case);
- newer stamp (v3) discards / is refused untouched;
- unstamped legacy store (data without `user_version`) discards;
- structurally corrupt store is fatal read-only (no writes) and discards on
  the write path;
- optional corruption (dangling active tab/split, invalid geometry/sidebar,
  invalid custom template) projects all ordered warnings without writes and
  keeps valid download/history rows;
- a full v2 round-trip preserves every new field (both glyph kinds, both
  tint kinds, custom settings, downloads with and without destination,
  history);
- invalid writes reject the whole snapshot for every new field kind;
- existing identity-collision, hue, write-count, and dense-500 evidence
  updated to the v2 shape.

Fixtures: `schema_v2.sql`, `corrupt_optional.sql` and `invalid_structural.sql`
reworked to the v2 shape, `newer_schema.sql` repointed to version 3,
`discard_unstamped.sql` (replaces the obsolete `failed_migration.sql`).

## 6. Delivery ledger (schema v2 lane)

| Item | Result |
|---|---|
| Stamp + discard | `schema.rs` (`SCHEMA_VERSION = 2`), new `setup.rs` (`prepare_store`), `StoreLoad`/`StoreDiscardReason`, second discard pass on restore-corruption |
| v2 shape | `model.rs` DTOs + SQL for appearance/settings/downloads/command-history |
| Read-only path | typed `SchemaMismatch`, no writes |
| Tests | 13 contract tests green; domain untouched (67 tests green) |
| Files | `lib.rs` 36, `schema.rs` 101, `schema_tables.rs` 454, `setup.rs` 129, `fields.rs` 363, `model.rs` 230, `error.rs` 195, `validation.rs` 344, `repository.rs` 397, `repository_load.rs` 428, `load_helpers.rs` 126, `repository_contract.rs` 705, `repository_benchmark.rs` 426 |
| Gates | `cargo fmt --all -- --check`, `cargo clippy -p bbr-persistence --all-targets -- -D warnings`, `cargo clippy -p bbr-domain --all-targets -- -D warnings`, `cargo test -p bbr-persistence`, `cargo test -p bbr-domain`, `nix develop -c just verify` |
