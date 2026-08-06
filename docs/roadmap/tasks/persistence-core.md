# Task packet — portable persistence core

Parents: [Phase 7](../phases/07-persistence.md) · [SQLite dependency contract](persistence-sqlite-contract.md) · [Performance contract](../../product/performance-contract.md) · [Parallel-delivery contract](../../parallel-delivery.md).

## Outcome

`bbr-persistence` provides a versioned SQLite repository with checked-in
migration fixtures and transactional structural-state round trips for browser
organization, without importing GPUI, native web engines, or app composition.
It restores a large persisted model as portable data while leaving native
session hydration to the later serial startup integration.

## Scope

- Repository-owned persistent DTOs and typed errors for profiles, spaces, tabs,
  pinned/order state, active tabs, recently closed entries, window/sidebar
  state, and the existing two-pane split composition.
- A SQLite connection/repository implementation in `crates/bbr-persistence/**`
  using the reserved schema version `1`, one crate-owned migration registry,
  and strict schema-version validation.
- Transactional writes for complete structural snapshots/mutations and
  deterministic ordered reads; foreign keys and checked invariants prevent
  orphan profile, space, tab, split, and recently-closed references.
- Checked-in schema-v0, schema-v1, corrupt-optional, invalid-structural,
  newer-schema, and failed-migration fixtures under the crate.
- Repository restore tests for an empty first run and a dense model of at least
  500 tab identities; no WKWebView creation or app process involved.
- Typed recovery warnings for optional presentation/session state, following
  the recovery matrix below without mutating the database during restore.
- Measured repository read/write evidence and write-count instrumentation for
  structural operations. Persisted warm-startup product acceptance remains
  owned by later app composition.
- Truthful task-packet and crate documentation updates. Phase 7 and the root
  roadmap remain serial integration-owned.

## Non-goals

- No app startup/shutdown wiring, force-kill process harness, GPUI recovery UI,
  WebKit profile-store mutation, or native-session hydration.
- No history/search provider exposed to `bbr-command` in this packet; that is a
  serial shared integration contract after both cores settle.
- No metadata debounce scheduler, favicon blobs, per-site settings/zoom, or
  broad future schema beyond the bounded structural model.
- No changes to `bbr-domain`, `bbr-browser`, `bbr-app`, `bbr-command`, `bbr-web`,
  testkit protocol, root workspace manifests, `Cargo.lock`, `justfile`, Phase 7,
  `ROADMAP.md`, or performance baselines from this lane. If another dependency
  or shared DTO contract is required, stop for serial integration rather than
  editing it independently.

## Parallel-lane contract

```text
Base commit: ba273b90436de5617f61347ef61892828ae1c89a
Prepared packet commit: 9bb0a3b7ca6b66999d7d453a9ab141317def078f
Depends on: parallel-delivery foundation and completed SQLite contract
  5bf1fd9 + 88b538d + ba273b9
Provided dependency: rusqlite =0.40.1, default-features=false, bundled
Provides/API: SQLite repository, persistent DTOs/errors/recovery warnings,
  schema-v1 migration and fixture contracts
Reserved migration sequence: 0 -> 1 only; future version allocation remains
  serial integration-owned
Exclusive paths: crates/bbr-persistence/**; this task packet
Shared integration points: domain persistence DTO mapping, future migration
  numbering, app startup/shutdown, command history provider, root Cargo/Cargo.lock,
  Phase 7 and ROADMAP.md
Integration order: land repository/migrations before app restore/hydration and
  before command-history provider composition; shared surfaces integrate serially
```

## Schema and migration contract

- `PRAGMA user_version = 1` is the only current schema version reserved for this
  packet. The lane may implement only migration `0 -> 1` in one ordered,
  crate-owned registry; later migration numbers require a new serial contract.
- Version `0` means an empty pre-product SQLite database with no BBR tables.
  Opening it creates the complete version-1 schema transactionally.
- Opening version `1` validates required tables, columns, foreign keys, and
  repository invariants before returning data.
- A schema version greater than `1` returns a typed incompatibility error and
  performs no SQL writes, downgrade, deletion, or best-effort interpretation.
- Each migration and its `user_version` update execute in the same transaction.
  Any statement, validation, or injected failure rolls back schema and data.
- Tests use temporary copies of checked-in fixtures and never a developer or
  production database.

## Recovery matrix

Repository restore separates required browsing data from optional window/session
presentation state. It returns either a complete portable snapshot plus ordered,
typed warnings or a typed fatal error. Restore itself is read-only: repairs
exist only in the returned projection until a later explicit repository write
commits them transactionally.

| Persisted state | Classification and required behavior |
|---|---|
| Profiles, spaces, tabs, profile/space ownership, tab ordering and pinned state | Required structural graph. Missing owners, duplicate identities/order keys, invalid pin regions, malformed required values, or broken foreign keys return a typed fatal corruption error; database remains byte-for-byte logically unchanged. |
| Recently closed entries | Browsing data. Invalid identity, owning-space reference, URL, or required former position is fatal and unchanged; an absent optional title restores as the documented empty-title fallback without dropping the entry. |
| Active-tab reference per space | Optional session state. A missing/non-member tab clears only the returned active reference and emits a typed warning; persisted row remains unchanged during restore. |
| Two-pane split composition | Optional session state. Missing, identical, or cross-space panes remove only the returned split, retain both valid tab identities, and emit a typed warning; persisted rows remain unchanged during restore. |
| Window geometry | Optional presentation state. Non-finite, non-positive, or out-of-contract geometry resets only the returned geometry to typed defaults and emits a warning; no other state changes. |
| Sidebar width/collapsed state | Optional presentation state. Invalid width resets only width to the documented default while retaining a valid collapsed flag; malformed required encoding returns a typed warning and the complete sidebar default. |
| Unsupported schema version greater than 1 | Typed incompatibility error; no migration, repair, downgrade, or deletion. |
| Migration or post-migration validation failure | Fatal typed migration error; entire migration transaction, data, and `user_version` roll back. |

Fixtures and tests must assert the exact returned projection, ordered warning
set, unchanged required profile/space/tab/recently-closed rows, and unchanged
on-disk schema/data after fatal or read-only recovery paths.

## Constraints

- `#![forbid(unsafe_code)]`; no GPUI, AppKit/WebKit, test socket, platform
  dependency, async runtime, ORM, or exposed SQLite connection types.
- Use the serially supplied exact `rusqlite` contract; do not change root or
  crate dependency features from the lane.
- Migrations are forward-only, deterministic, transactional, and fixture-backed.
- Structural state writes are atomic; invalid references fail the whole write.
- Repository restore creates data only. A passing repository test cannot claim
  the Phase-7 product exit of chrome-first startup or lazy WKWebView hydration.
- Do not modify or inspect sibling worktrees; one writer owns this branch.

## Evidence

### Unit and contract

- [ ] Empty/schema-v0 databases migrate to schema v1 and round-trip the bounded
      structural model with stable ordering and identities.
- [ ] Checked-in schema-v1 data opens without writes; newer schema is rejected
      unchanged; an injected failed migration rolls back schema, data, and
      `user_version`.
- [ ] Foreign-key/invariant violations fail whole, while each optional-state
      fixture returns the exact projection and ordered typed warnings defined by
      the recovery matrix without deleting browsing data.
- [ ] Dense 500-tab data restores as identities with no engine/native types.
- [ ] Structural writes are transactional and expose deterministic statement /
      transaction counts asserted by normal contract tests.

### Real-app E2E

- [ ] Not applicable to this repository-core packet: it creates no app startup
      behavior. The later Phase-7 composition packet owns normal quit,
      force-quit, chrome-first restore, lazy WKWebView hydration, recovery UI,
      typed testkit events, and production E2E.

### Performance

- [ ] The exact optimized repository benchmark command completes and writes a
      reviewable artifact for schema-v0 migration load, schema-v1 current load,
      and transactional structural write over the dense 500-tab fixture.
- [ ] The artifact contains 10 discarded warm-ups plus at least 100 raw samples
      per operation, p50/p95/max in monotonic nanoseconds, operation and sample
      counts, release profile/package version, target OS/architecture, available
      parallelism, and repository write counts.
- [ ] No product startup threshold is claimed or changed; later real-app
      persisted startup measures production milestones under the performance
      contract.

## Benchmark contract

The lane owns this bounded, explicitly ignored release test:

```bash
cargo test -p bbr-persistence --release \
  --test repository_benchmark -- --ignored --nocapture
```

The test writes plain-text, diffable evidence to:

```text
target/bbr-persistence-artifacts/repository-benchmark.txt
```

Fixture construction and file-copy setup occur outside timed regions. Every
sample validates the restored/written model before it is accepted. The command
fails if an operation, correctness assertion, sample count, artifact write, or
metadata field is missing. It records distributions but introduces no proxy
product-startup latency gate.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-persistence
cargo clippy -p bbr-persistence --all-targets -- -D warnings
cargo test -p bbr-persistence --release \
  --test repository_benchmark -- --ignored --nocapture
just roadmap
```

No GUI gate is required because the lane is portable and changes no browser
surface. The integration lane runs `just verify` after landing.

## Delivery sequence

1. The serial owner promotes and reviews this packet, records its exact clean
   implementation anchor, and updates the writer-free persistence branch to
   include that anchor and the completed SQLite dependency contract.
2. Exactly one writer then owns
   `/Users/patwoz/dev/Personal/macos/bbr-persistence-core` on
   `parallel/persistence-core` and modifies only the exclusive paths.
3. Implement schema metadata and migration `0 -> 1`, then one minimal
   structural round trip, then recovery fixtures, then dense restore and
   measured evidence.
4. One independent read-only reviewer maps the repository, migration, recovery,
   and benchmark contracts to code, fixtures, tests, and artifact evidence.
5. Remediate concrete findings, commit coherent work, and hand the settled
   branch to the serial integration owner.

## Iteration policy

Start with schema-version metadata and an empty migration, then one minimal
structural round trip, then fixtures and failure rollback, then dense restore
and measured evidence. Keep the reserved migration independently testable. When
a shared domain or dependency contract is missing, stop and request the
smallest serial foundation instead of creating a compatibility copy.

## Blocked stop condition

Stop with the preserved branch/worktree, schema/fixture evidence, attempted
approaches, and exact serial contract needed if domain DTO ownership or another
shared boundary cannot be resolved within `crates/bbr-persistence/**`. Do not
mutate sibling or root shared surfaces to bypass the blocker.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E: not applicable; later app-composition packet owns it.
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:
```
