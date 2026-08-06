# Task packet — portable persistence core

Parents: [Phase 7](../phases/07-persistence.md) · [Performance contract](../../product/performance-contract.md) · [Parallel-delivery contract](../../parallel-delivery.md).

## Outcome

`bbr-persistence` provides a versioned SQLite repository with checked-in
migration fixtures and transactional structural-state round trips for browser
organization, without importing GPUI, native web engines, or app composition.
It can restore a large persisted model as data while leaving native session
hydration to the later serial startup integration.

## Scope

- Repository-owned persistent DTOs and typed errors for profiles, spaces, tabs,
  pinned/order state, active tabs, recently closed entries, window/sidebar
  state, and two-pane split composition where the existing domain contract is
  stable enough to serialize.
- A SQLite connection/repository implementation in `crates/bbr-persistence/**`
  with one versioned migration sequence and schema-version validation.
- Transactional writes for structural mutations and deterministic ordered
  reads; foreign keys/invariants prevent orphan profile/space/tab references.
- Checked-in empty/current/previous-version/corrupt-optional fixtures under the
  crate, with migration and rollback tests.
- Repository-level restore tests for an empty first run and a dense model of at
  least 500 tab identities; no WKWebView creation or app process involved.
- Measured repository read/write evidence and write-count instrumentation for
  structural operations. Persisted warm-startup product acceptance remains
  owned by later app composition.
- Truthful Phase-7 and crate documentation updates.

## Non-goals

- No app startup/shutdown wiring, force-kill process harness, GPUI recovery UI,
  WebKit profile-store mutation, or native-session hydration.
- No history/search provider exposed to `bbr-command` in this packet; that is a
  serial shared integration contract after both cores settle.
- No metadata debounce scheduler, favicon blobs, per-site settings/zoom, or
  broad future schema beyond the bounded structural model.
- No changes to `bbr-domain`, `bbr-browser`, `bbr-app`, `bbr-command`, `bbr-web`,
  testkit protocol, root workspace manifests, `Cargo.lock`, `justfile`, or
  `ROADMAP.md` current-focus roll-up from this lane. If a dependency or shared
  DTO contract is required, stop for serial integration rather than editing it
  independently.

## Parallel-lane contract

```text
Base commit: 9352ded94075d3c82f90c682fb7ff248d0ee2fe6
Depends on: parallel-delivery foundation only
Provides/API: SQLite repository, persistent DTOs/errors, migration and fixture contracts
Exclusive paths: crates/bbr-persistence/**; this task packet; Phase-7 detail only
Shared integration points: domain persistence DTO mapping, migration numbering,
  app startup/shutdown, command history provider, root Cargo/Cargo.lock
Integration order: land repository/migrations before app restore/hydration and
  before command-history provider composition; shared surfaces integrate serially
```

## Constraints

- `#![forbid(unsafe_code)]`; no GPUI, AppKit/WebKit, test socket, or platform
  dependency.
- Prefer the PLAN-selected maintained SQLite approach (`rusqlite`) only after
  root dependency integration is assigned to the serial owner; do not hand-edit
  shared manifests from the lane without that handoff.
- Migrations are forward-only, deterministic, transactional, and covered by
  fixtures. Tests must never alter a developer database.
- Structural state writes are atomic; invalid references fail whole.
- Repository restore creates data only. A passing repository test cannot claim
  the Phase-7 product exit of chrome-first startup or lazy WKWebView hydration.
- Do not modify or inspect sibling worktrees; one writer owns this branch.

## Evidence

### Unit and contract

- [ ] Empty database migrates to the current schema and round-trips the bounded
      structural model with stable ordering and identities.
- [ ] Previous-version fixtures migrate; a failed migration rolls back without
      partial schema/data changes.
- [ ] Foreign-key/invariant violations and corrupt optional session data produce
      typed, bounded recovery outcomes rather than deleting profile data.
- [ ] Dense 500-tab data restores as identities with no engine/native types.

### Real-app E2E

- [ ] Not applicable to this repository-core packet: it creates no app startup
      behavior. The later Phase-7 composition packet owns normal quit,
      force-quit, chrome-first restore, lazy WKWebView hydration, recovery UI,
      typed testkit events, and production E2E.

### Performance

- [ ] Optimized repository benchmark records repeated migration/current-schema
      load and transactional structural-write samples for the dense fixture.
- [ ] No product startup budget is claimed or changed; later real-app persisted
      startup must measure the production path under the performance contract.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-persistence
cargo clippy -p bbr-persistence --all-targets -- -D warnings
cargo test -p bbr-persistence --release
just roadmap
```

No GUI gate is required because the lane is portable and changes no browser
surface. The integration lane runs `just verify` after landing.

## Delivery sequence

1. One writer owns `/Users/patwoz/dev/Personal/macos/bbr-persistence-core` on
   `parallel/persistence-core`.
2. If the SQLite dependency is absent, stop for a serial root-manifest contract
   commit; resume from the resulting clean anchor rather than editing root
   manifests independently.
3. Implement repository/schema/fixtures and verify only the portable core.
4. One independent read-only reviewer maps Phase-7 repository and migration
   requirements to schema, tests, fixtures, and benchmark evidence.
5. Remediate concrete findings, commit coherent work, and hand the settled
   branch to the serial integration owner.

## Iteration policy

Start with schema-version metadata and an empty migration, then one minimal
structural round trip, then fixtures and failure rollback, then dense restore
and measured evidence. Keep every migration independently testable. When a
shared domain or dependency contract is missing, stop and request the smallest
serial foundation instead of creating a compatibility copy.

## Blocked stop condition

Stop with the preserved branch/worktree, schema/fixture evidence, attempted
approaches, and exact serial contract needed if root dependency integration,
domain DTO ownership, migration numbering, split serialization, or corrupt
state policy cannot be resolved within `crates/bbr-persistence/**`. Do not
mutate sibling/root shared surfaces to bypass the blocker.

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
