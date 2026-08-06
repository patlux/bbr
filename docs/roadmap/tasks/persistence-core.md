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
  deterministic ordered reads; structural foreign keys protect ownership,
  while projection validation handles active-tab and split soft references.
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
Canonical contract base: aed5bc77d416f69b913876e3977020d99abffe26
Settled lane foundation: b83bd3acf3aeccd235ea21a0a24c18b1b1924147
Historical prepared packet: 9bb0a3b7ca6b66999d7d453a9ab141317def078f
  (created from 9352ded; superseded by the canonical promoted contract)
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
- Opening version `1` validates required tables, columns, structural foreign
  keys (`profile -> space`, `space -> tab`, and `recently closed -> space`
  only), and repository invariants before returning data. Active-tab and split-
  pane ids are deliberate soft references validated during restore projection.
- A schema version greater than `1` returns a typed incompatibility error and
  performs no SQL writes, downgrade, deletion, or best-effort interpretation.
- Each migration and its `user_version` update execute in the same transaction.
  Any statement, validation, or injected failure rolls back schema and data.
- Tests use temporary copies of checked-in fixtures and never a developer or
  production database.

## Recovery matrix

Repository restore separates required browsing data from optional window/session
presentation state. It returns either a complete portable snapshot plus ordered,
typed warnings or a typed fatal error. Warnings sort by owning window/space id,
then the fixed variant order `ActiveTab`, `Split`, `WindowGeometry`, `Sidebar`.
Nonmigrating restore opens with `SQLITE_OPEN_READ_ONLY` and enables
`PRAGMA query_only = ON`; repairs exist only in the returned projection until a
later explicit repository write commits them transactionally.

| Persisted state | Classification and required behavior |
|---|---|
| Profiles, spaces, tabs, profile/space ownership, tab ordering and pinned state | Required structural graph with database-enforced ownership foreign keys. Missing owners, duplicate identities/order keys, invalid pin regions, malformed required values, or structural foreign-key failures return a typed fatal corruption error; persisted state remains unchanged. |
| Recently closed entries | Browsing data with a structural foreign key to its owning space. Invalid identity, owner, URL, or required former position is fatal and unchanged; an absent optional title restores as the empty string without dropping the entry. |
| Active-tab reference per space | Recoverable soft reference, deliberately not a database foreign key. A missing/non-member tab clears only the returned active reference and emits `ActiveTab`; persisted state remains unchanged. |
| Two-pane split composition | Recoverable soft references, deliberately not database foreign keys. Missing, identical, or cross-space panes remove only the returned split, retain every valid tab identity, and emit `Split`; persisted state remains unchanged. |
| Window geometry | Optional presentation state. `x` and `y` accept any finite signed value, including negative multi-display coordinates; `width` and `height` must be finite and positive. Any invalid component clears only the returned geometry to `None`, so later app composition centers the canonical 1080 x 700 pt default, and emits `WindowGeometry`. |
| Sidebar width/collapsed state | Optional presentation state. Width and last-expanded width must each be finite within 200..=360 pt, and collapsed encoding must be exactly 0 or 1. Any invalid component returns `{ width: 260, collapsed: false, last_expanded_width: 260 }` and emits `Sidebar`. |
| Unsupported schema version greater than 1 | Typed incompatibility error; no migration, repair, downgrade, or deletion. |
| Migration or post-migration validation failure | Fatal typed migration error; entire migration transaction, data, and `user_version` roll back. |

Fixtures and tests must assert the exact returned projection and warning order.
Before and after every fatal or nonmigrating recovery path they compare the main
database and `-wal`/`-shm` existence plus bytes, `user_version`, and logical
schema/table contents. No sidecar may appear or disappear. Migration tests
instead assert the committed result or the complete pre-migration file inventory
and logical state after rollback.

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

- [x] Empty/schema-v0 databases migrate to schema v1 and round-trip the bounded
      structural model with stable ordering and identities.
- [x] Checked-in schema-v1 data opens without writes; newer schema is rejected
      unchanged; an injected failed migration rolls back schema, data, and
      `user_version`.
- [x] Foreign-key/invariant violations fail whole, while each optional-state
      fixture returns the exact projection and ordered typed warnings defined by
      the recovery matrix without deleting browsing data.
- [x] Dense 500-tab data restores as identities with no engine/native types.
- [x] Structural writes are transactional and expose deterministic statement /
      transaction counts asserted by normal contract tests.

### Real-app E2E

- [x] Not applicable to this repository-core packet: it creates no app startup
      behavior. The later Phase-7 composition packet owns normal quit,
      force-quit, chrome-first restore, lazy WKWebView hydration, recovery UI,
      typed testkit events, and production E2E.

### Performance

- [x] The exact optimized repository benchmark command completes and writes a
      reviewable artifact for schema-v0 migration load, schema-v1 current load,
      and transactional structural write over the dense 500-tab fixture.
- [x] The artifact contains 10 discarded warm-ups plus at least 100 raw samples
      per operation, p50/p95/max in monotonic nanoseconds, operation and sample
      counts, release profile/package version, target OS/architecture, available
      parallelism, and repository write counts.
- [x] No product startup threshold is claimed or changed; later real-app
      persisted startup measures production milestones under the performance
      contract.

## Benchmark contract

The lane owns this bounded, explicitly ignored release test:

```bash
cargo test -p bbr-persistence --release \
  --test repository_benchmark -- --ignored --nocapture
```

The test resolves its artifact root as
`$CARGO_TARGET_DIR/bbr-persistence-artifacts` when `CARGO_TARGET_DIR` is set,
otherwise as `<workspace-root>/target/bbr-persistence-artifacts`. It atomically
replaces `repository-benchmark.txt` through a same-directory temporary file and
rename. The header records the resolved absolute artifact path plus a fresh
Unix-epoch-millisecond and process-id run identifier.

Fixture construction and file-copy setup occur outside timed regions. Every
sample validates the restored/written model before it is accepted. The command
fails if an operation, correctness assertion, sample count, raw sample,
artifact replacement, run identity, or metadata field is missing. Its parser
contract rejects missing or duplicate operation blocks and content carrying a
stale run identifier. It records distributions but introduces no proxy product-
startup latency gate.

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
Outcome: Portable schema-v1 SQLite repository with strict migrations/schema
  validation, atomic complete-snapshot writes, read-only restore/recovery, and
  checked-in fixture contracts.
Commits: 53fdab9 feat(persistence): add versioned structural repository;
  970a42d perf(persistence): record dense repository distributions.
Functional evidence: cargo test -p bbr-persistence: 9 contract tests passed;
  cargo clippy -p bbr-persistence --all-targets -- -D warnings passed. Fixtures
  cover schema_v0, schema_v1, corrupt_optional, invalid_structural,
  newer_schema, and failed_migration. Inventory assertions compare main/-wal/-shm
  existence and bytes, user_version, schema, and ordered logical table contents.
Real-app E2E: not applicable; later app-composition packet owns it.
Performance evidence: release artifact
  target/bbr-persistence-artifacts/repository-benchmark.txt, run
  1786042522587-99412; 10 warmups + 100 samples each. migration load
  p50/p95/max 486375/611375/941542 ns; current load
  176375/188458/202666 ns; dense write 1597125/1971917/3283833 ns with exactly
  1 committed transaction, 511 SQL statements, 0 deleted rows, 505 inserted rows.
Roadmap/documentation updates: this packet evidence and delivery ledger only;
  Phase 7 and ROADMAP.md intentionally unchanged. just roadmap passed.
Open blockers and non-goals: no app startup/shutdown, native hydration,
  force-quit E2E, history/provider composition, debounce, or product startup
  latency claim; those remain serial integration/later Phase-7 work.
Worktree state: final gate run and documentation commit recorded in handoff;
  root Cargo.toml and Cargo.lock byte-identical to c8c446b.
```
