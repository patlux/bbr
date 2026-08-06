# Task packet — persistence SQLite dependency contract

Parents: [Phase 7](../phases/07-persistence.md) · [Portable persistence core](../../../crates/bbr-persistence/src/lib.rs) · [Parallel-delivery contract](../../parallel-delivery.md).

## Outcome

The serial integration lane provides one reproducible SQLite dependency contract
for `bbr-persistence`, so its isolated repository writer can implement schema,
migrations, fixtures, and transactions without independently changing the root
workspace or relying on a host-specific SQLite installation.

## Scope

- Pin the current maintained `rusqlite` release in workspace dependencies.
- Disable default features and enable only `bundled`, giving Darwin/Linux builds
  the same SQLite source and modern bindings without optional SQLCipher,
  extension, time, UUID, URL, or serialization features.
- Connect only `crates/bbr-persistence` to the workspace dependency.
- Verify the entire workspace resolves, checks, tests, and passes Clippy.
- Update Phase 7 and the persistence crate documentation with the dependency
  boundary and handoff instructions.

## Non-goals

- No schema, SQL, repository API, migration, fixture, DTO, app composition,
  history provider, or persistence behavior.
- No changes in the command-core, URL-input, app, domain, browser, web, testkit,
  performance, or GUI surfaces.
- No system SQLite, SQLCipher, loadable extensions, async wrapper, ORM, or
  speculative feature set.

## Parallel-lane contract

```text
Base commit: 26e528cb91c5f63fa385925f1a2b23cd6741b6b5
Depends on: parallel-delivery foundation
Provides/API: workspace rusqlite 0.40.1 with default-features=false, bundled
Exclusive paths: root Cargo.toml/Cargo.lock, crates/bbr-persistence/Cargo.toml,
  this packet, Phase-7 dependency note
Shared integration points: persistence-core lane only
Integration order: land on main, merge main into the clean persistence-core
  branch, update its exact implementation base, then start its writer
```

## Constraints

- Root dependency versions remain serial integration-owned.
- `bbr-persistence` remains `#![forbid(unsafe_code)]`; all FFI stays inside the
  maintained dependency.
- Reproducibility takes precedence over a smaller system-linked build.
- Do not start the persistence writer until its branch includes this contract.
- No weakening of workspace checks or platform evaluation.

## Evidence

### Unit and contract

- [x] Cargo metadata resolves `rusqlite 0.40.1` for `bbr-persistence` only,
      with exactly `bundled` and `modern_sqlite` enabled on `rusqlite`.
- [x] Workspace check/test/Clippy compile the bundled SQLite path.

### Real-app E2E

- [x] Not applicable: no product behavior or native browser surface changes.

### Performance

- [x] Not applicable: no repository behavior exists yet; the persistence-core
      packet owns measured dense repository evidence.

## Completion gates

```bash
cargo metadata --format-version 1
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
just roadmap
```

## Iteration policy

Use the smallest feature set that compiles the future repository contract. If
the bundled build fails, identify the compiler/linker mechanism before changing
features; do not fall back to host SQLite without reproducibility evidence.

## Blocked stop condition

Stop with Cargo resolution/build evidence and the exact platform blocker if
`rusqlite` cannot compile on the supported Darwin/Linux workspace. Do not start
repository implementation against an unresolved dependency contract.

## Delivery ledger

```text
Outcome: Reproducible workspace SQLite dependency for the persistence lane.
Commit: recorded after this implementation commit.
Dependency evidence: cargo metadata resolves rusqlite 0.40.1 with features
  bundled + modern_sqlite only; cargo tree confirms bundled libsqlite3-sys.
Workspace evidence: just verify green (fmt/check/test/clippy/roadmap/scripts).
Roadmap/documentation updates: Phase 7 dependency order, this packet, crate docs.
Open blockers/non-goals: repository/schema/migrations remain in persistence-core;
  no real-app/performance claim; no system SQLite or optional feature growth.
Worktree state: clean main after commit; persistence-core remains writer-free
  until it incorporates this serial contract.
```
