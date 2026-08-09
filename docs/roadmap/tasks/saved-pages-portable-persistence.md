# Task packet — B1 portable Saved Pages contract and persistence

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[Phase 7 persistence](../phases/07-persistence.md) ·
[portable persistence core](persistence-core.md) ·
[interaction contract](../../product/browser-interactions.md) ·
Source decision: `/Users/patwoz/dev/Personal/macos/bbr-saved-pages-bookmarks-plan.md`.

Activation condition: the verified delivery base and repository/schema territory must be writer-free as recorded in [Current delivery](../../delivery/current.md). This is the
canonical detail owner for Saved Page identity, validation, canonical-URL
duplicate behavior, flat collections, repository operations, and the next
development schema shape. Higher roadmap levels contain links and roll-ups only.

## Outcome

`bbr-domain` and `bbr-persistence` represent a Saved Page as one durable global
URL record with no `TabId`, `WebViewId`, native session, Space, Profile, cookie,
credential, history, snapshot, or favicon-blob ownership. Saving the same
canonical navigable URL is idempotent, repository reopen preserves the record,
and a development schema mismatch discards the old store and is immediately
rewritable without migration or compatibility code.

## Product boundary

| Object | Identity and owner | Lifetime | Opening behavior |
|---|---|---|---|
| Pinned tab | one concrete `TabId` in one Space | closing the tab removes it | activates the same live tab |
| Favorite | global presentation of one concrete live `TabId` | closing the tab removes it | activates the same live tab through its owner |
| Saved Page | stable `BookmarkId` plus canonical URL; global installation data | survives tab/space/window close and restart | B2 opens one fresh regular tab in the current Space/Profile |

A Saved Page is independent after capture. Saving a pinned or Favorite tab
copies only the current navigable URL and title into the record; no identity
link to that tab remains.

## Scope

- Add portable `BookmarkId`, `SavedPage`, and optional flat `CollectionId`
  values with validation, deterministic ordering, and privacy-safe `Debug`.
- Model a record with canonical navigable URL, captured title, optional user
  label, created/updated timestamps, optional flat collection membership, and
  deterministic sort key. No native or app types.
- Establish one canonical-URL equality seam shared by save, lookup, remove,
  and search. Reuse or extract the existing navigation/command URL authority;
  do not invent bookmark-only rewriting. If current normalization cannot give
  deterministic equality without changing navigable meaning, stop for the
  smallest explicit URL decision rather than adding a second normalizer.
- Repository operations: idempotent save, get by id/canonical URL, list,
  deterministic search, update captured title/user label, remove, create/
  rename/remove a flat collection, and move a record into or out of one flat
  collection.
- Duplicate semantics: at most one active record per canonical URL. Re-saving
  returns the existing `BookmarkId`; captured title updates only when explicitly
  requested or when no user label exists. Repeated remove is a no-op success.
- Add the Saved Pages tables/rows to the one repository authority and the next
  development schema version (`v3` if `v2` remains current when B1 starts).
  Per Patrick's standing rule, mismatched stamp or shape is discarded and the
  empty current schema is recreated; no migration, upgrade, downgrade,
  compatibility read, or fallback path.
- Add checked-in current-schema, prior-schema, corrupt-record, duplicate-URL,
  and collection-integrity fixtures. Repository reads remain deterministic and
  never mutate a matching valid store.
- Document the portable and persistence APIs. App bootstrap, command-provider
  composition, Cmd-D, menus, tabs, and UI remain B2/B3.

## Non-goals

- No app/sidebar/Library/Palette/menu/Portal rendering and no real-app command.
- No tab creation, selection, pinning, Favorite mutation, navigation, profile
  switch, native session, cookie/data-store, or favicon fetch.
- No cloud/mobile/browser sync, offline copy, page snapshot/archive, reading
  list, tags, bookmarklets, nested collections, smart collections, or import/
  export.
- No migration or backward compatibility. No preservation of development data
  across the schema bump.
- No new dependency unless existing URL and SQLite facilities are proven
  insufficient.

## Parallel-lane contract

```text
Base condition:       verified main after the current F2 integration kernel
                      and after every active repository/schema writer settles
Depends on:           one integrated AppStore/repository authority; current
                      schema truth; free bbr-domain/bbr-persistence territory
Provides/API:         portable Saved Page/collection values; canonical URL
                      equality seam; repository CRUD/search/list contracts;
                      next development schema shape and fixtures
Exclusive paths:      focused bbr-domain Saved Page modules/tests;
                      focused bbr-persistence schema/repository/fixture/tests;
                      this packet
Shared integration points: schema version/validation registry, repository DTO
                      aggregate, Cargo manifests only if unavoidable, Phase 7/8
                      and ROADMAP roll-ups
Integration order:    current F2/F3 repository work → B1 → B2; B3 is UI-only
                      but consumes the integrated B1/B2 contracts
```

No other writer may own schema numbering, repository validation, or Saved Page
portable identities while B1 runs.

## Constraints

- Global installation scope is explicit: records carry no `ProfileId` and no
  implicit profile filter. Opening context is not persisted in B1.
- URLs must be navigable through the existing product navigation contract;
  malformed, dangerous, or unsupported schemes reject whole.
- One repository/SQLite authority; no second store, writer, connection owner,
  or render-time query path.
- Transactions reject invalid records or collection references whole. Same-
  version corruption is typed, logged by later app composition, discarded under
  the development rule, and immediately rewritable.
- Portable crates import no GPUI, AppKit, WebKit, filesystem UI, or native
  pointers. `#![forbid(unsafe_code)]` remains.
- Files stay below roughly 500 lines. Tests and validators are not weakened.

## Evidence

### Unit and contract

- [x] Identifier, URL, title/label, timestamp, sort-key, and collection
      validation cover boundaries and privacy-safe debug output.
- [x] Canonical duplicate tests prove repeated save returns the same
      `BookmarkId`, creates no second row, and follows the settled title/label
      update rule.
- [x] Ordering/search are deterministic for equal titles, URLs, timestamps,
      and collection membership; global ownership has no profile dimension.
- [x] Update/remove/collection moves are idempotent where specified and reject
      unknown or invalid references whole.

### Persistence

- [x] Current-schema round-trip preserves every field and deterministic order;
      matching valid reads produce no writes.
- [x] Prior/mismatched schema and same-version structural corruption are
      discarded under the development rule and the repository is immediately
      rewritable; no migration/compatibility branch exists.
- [x] Duplicate canonical URLs and invalid collection references cannot enter
      the committed store; rollback leaves the complete prior state intact.
- [x] A repository reopen proves Saved Pages survive process-independent store
      close/reopen without any tab/session object.

### Real-app E2E

- [ ] Not applicable to B1: it exposes no browser feature. B2 owns Cmd-D,
      restart, Palette, menu, and fresh-tab production E2E.

### Performance

- [x] Repository list/search/write distributions over at least 1,000 records
      are retained as test evidence without claiming UI or product latency.
      B2/B3 own optimized presentation budgets.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-domain -p bbr-persistence
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-domain --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-persistence --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo test -p bbr-persistence --release \
  --test repository_benchmark -- --ignored --nocapture
nix develop -c just verify
nix develop -c just roadmap
git diff --check
```

No GUI gate is required because B1 changes no user-visible browser path.

## B1 decisions (recorded, not silently changed later)

- **Supported schemes:** exactly `http`, `https`, and `about` (BBR's
  product-navigation schemes). `javascript:`, `data:`, `file:`, `mailto:`,
  and every other scheme reject whole as unsupported/dangerous.
- **Credential-free by construction:** userinfo in the authority
  (`https://user:pass@host`) rejects whole, so a record can never embed
  credentials and its `Debug` output is privacy-safe.
- **Protocol-relative input** (`//host`) rejects (no scheme after
  defaulting). NUL/whitespace/control bytes reject.
- **Canonical seam:** `bbr-domain::saved_pages::canonical_url` reuses
  `bbr_web::normalize_url` for scheme defaulting, then ASCII-case-folds only
  scheme and host (percent-escape sequences in the host preserved
  byte-for-byte). Path, query, fragment, explicit port, trailing slash,
  `www.`, and percent-encoding are preserved byte-for-byte; no default-port
  elision and no bookmark-only rewrite. Persistence stores the key opaquely
  (bounded `valid_url` only) and enforces uniqueness via the `UNIQUE` column
  plus write-side validation.
- **Deterministic sort key:** `(updated_at, id)` ascending
  ([`SavedPage::sort_key`]); equal timestamps tie-break by id. The domain
  exposes `deterministic_order`; persistence preserves caller order in dense
  `position` keys.
- **Duplicate semantics:** re-save returns the existing `BookmarkId`, never
  creates a second row, preserves `created_at`, never touches a user label,
  and updates the captured title only when no user label exists and the title
  actually changed (a byte-identical re-save performs no write). The stored
  `updated_at` can never regress below the stored `created_at`: a stale
  re-save/update timestamp is clamped up to the existing record's
  `created_at` (smallest consistent contract), so the loader-enforced
  monotonic invariant is unbreakable through the public API and the next
  write never discards data.
- **Search:** repository `search_saved_pages` is read-only, case-insensitive,
  literal-substring over title/label/canonical URL, deterministic in
  persisted order. Matching is one small Rust Unicode-lowercase authority
  mirroring `bbr-domain::SavedPage::matches` — no SQL pattern is built, so
  `%`/`_`/`\` in the query are literal by construction and non-ASCII
  (umlaut) queries fold exactly like the domain predicate; SQLite `LIKE`
  folds ASCII case only and is never used.
- **Collection membership is strict:** `saved_pages.collection_id` is a
  declared foreign key into `collections`. A dangling reference is structural
  corruption (discarded under the development rule). Removing a collection
  that is still referenced rejects whole — collection removal is an explicit
  two-step move-out-then-remove.
- **Schema v3:** adds `collections` and `saved_pages` tables (canonical key
  `UNIQUE`, strict collection FK, dense `position`); stamp bump plus discard
  only, no migration/backcompat path anywhere.

## Delivery sequence

1. Confirm repository/schema territory is free and record the verified base.
2. Add portable identity/validation and canonical equality tests.
3. Add the next-schema tables, DTO mapping, repository operations, and fixtures.
4. Run repository distributions and all non-GUI gates.
5. One independent read-only review; one bounded remediation only for concrete
   findings; focused recheck; then serial integration before B2.

## Iteration policy

Start with one valid record round-trip and one idempotent duplicate save. Add
collections only after the single-record contract is stable. Diagnose failures
as URL equality, validation, transaction, schema disposition, ordering, or
fixture truth. Do not add app composition or compatibility code to clear a
repository blocker.

## Blocked stop condition

Stop with the preserved branch and exact evidence if schema territory is not
free, the existing URL authority cannot define canonical equality without a
product decision, or repository integration would create a second store/writer.
Report the conflicting owner, attempted seam, smallest decision/settlement
needed, and do not start B2.

## Delivery ledger

```text
Outcome: portable bbr-domain Saved Page contract (BookmarkId, CollectionId,
  CanonicalUrl seam, SavedPage/Collection values, validation, deterministic
  ordering, privacy-safe Debug) plus the schema-v3 bbr-persistence repository
  (idempotent save, get by id/URL, deterministic list/search, update,
  idempotent remove, whole-set collection replace, membership moves),
  checked-in fixtures, reopen proof, and the >=1,000-record distribution.
Commits: none (writer leaves the coherent uncommitted tree; the Orchestrator
  commits and integrates the lane per the parallel-delivery contract).
Functional evidence: 117 bbr-domain tests + 43 bbr-persistence tests green
  (validation boundaries, canonical equality/duplicates, deterministic
  order/search, idempotent update/remove/moves, whole-write rejection with
  byte-identical inventory, reopen, browser-write isolation).
Persistence evidence: schema v3 bump discards prior/corrupt/duplicate/
  dangling-ref stores and is immediately rewritable (fixtures schema_v3,
  schema_v2, newer_schema v4, schema_v3_corrupt, schema_v3_duplicate_url,
  schema_v3_collection_integrity); matching valid reads produce zero writes
  (byte-identical inventory on schema_v3).
Performance evidence: release repository_benchmark, 4 operations; new
  saved_pages_write_list_search over 1,000 records + 20 collections
  (write 1,032 statements / 1,021 rows; list + literal search assertions);
  dense_structural_write 519 statements / 508 rows. Artifact retained under
  target/bbr-persistence-artifacts/repository-benchmark.txt.
Roadmap/documentation updates: this packet (evidence boxes + decisions),
  Phase 7 wording, ARCHITECTURE.md implemented-facts section. ROADMAP.md and
  docs/delivery/current.md untouched (serial integration-owned).
Open blockers and non-goals: none in scope; B2 owns app composition, Cmd-D,
  Palette provider, menus, and fresh-tab E2E. No UI/command/app work landed.
Worktree state: uncommitted changes in the exclusive B1 worktree on
  parallel/saved-pages-b1 over clean base e9f5d30214e7efc46586cb3ae11c6472e0b1801c.

Planning/read-only: ~0.5 h (documents, source map, contract model)
Implementation: ~4 h (domain contract, schema v3, repository, fixtures, tests)
Test execution: ~1 h (iterative targeted runs + gates)
Review: pending (independent read-only reviewer follows)
Remediation: pending
Waiting/retries: none
Total elapsed: ~5.5 h (labeled estimate; not wall-clock measured)
Human intervention: none
```
