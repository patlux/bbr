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

- [ ] Identifier, URL, title/label, timestamp, sort-key, and collection
      validation cover boundaries and privacy-safe debug output.
- [ ] Canonical duplicate tests prove repeated save returns the same
      `BookmarkId`, creates no second row, and follows the settled title/label
      update rule.
- [ ] Ordering/search are deterministic for equal titles, URLs, timestamps,
      and collection membership; global ownership has no profile dimension.
- [ ] Update/remove/collection moves are idempotent where specified and reject
      unknown or invalid references whole.

### Persistence

- [ ] Current-schema round-trip preserves every field and deterministic order;
      matching valid reads produce no writes.
- [ ] Prior/mismatched schema and same-version structural corruption are
      discarded under the development rule and the repository is immediately
      rewritable; no migration/compatibility branch exists.
- [ ] Duplicate canonical URLs and invalid collection references cannot enter
      the committed store; rollback leaves the complete prior state intact.
- [ ] A repository reopen proves Saved Pages survive process-independent store
      close/reopen without any tab/session object.

### Real-app E2E

- [ ] Not applicable to B1: it exposes no browser feature. B2 owns Cmd-D,
      restart, Palette, menu, and fresh-tab production E2E.

### Performance

- [ ] Repository list/search/write distributions over at least 1,000 records
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
Outcome:
Commits:
Functional evidence:
Persistence evidence:
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```
