# Phase 7 — Persistence and crash recovery

Parent: [Roadmap](../../../ROADMAP.md) · Quality: [Performance/release track](../tracks/performance-release.md)

## Outcome

Browser organization restores safely after normal quit, force quit, and schema upgrades without eagerly creating every web session.

## Parallel lane

After the [parallel-delivery foundation](../tasks/parallel-delivery-foundation.md)
lands, activate persistence core as a bounded lane rooted at that clean commit.
`crates/bbr-persistence/**` plus checked-in migration fixtures are its exclusive
surface; domain DTOs, app startup/shutdown composition, future migration
numbering, and command-history provider integration remain serial integration
points. The serial [SQLite dependency contract](../tasks/persistence-sqlite-contract.md)
is complete. The reviewed [portable persistence-core packet](../tasks/persistence-core.md)
records schema version 1 and the exact post-promotion implementation anchor
`aed5bc77d416f69b913876e3977020d99abffe26`. It is ready for lane settlement
and remains writer-free.

## Checklist

- [ ] Add `bbr-persistence` with SQLite repository interfaces and versioned migrations. *(The dependency-free crate landing zone exists with the parallel-delivery foundation; repository/schema behavior remains open.)*
- [ ] Persist profiles, spaces, tabs, pin/order state, active tabs, and recently closed tabs.
- [ ] Persist window geometry, sidebar state, and split layouts.
- [ ] Persist history and per-site settings/zoom.
- [ ] Write structural changes transactionally.
- [ ] Debounce transient title/favicon/history metadata writes.
- [ ] Restore browser chrome first and hydrate only visible sessions.
- [ ] Recover optional corrupt session state without deleting profile browsing data.
- [ ] Add checked-in migration fixtures and force-quit recovery tests.
- [ ] Add first-run, empty, migration-error, and recovery UI states.

## Phase exit

- [ ] Force quit during navigation or tab reorder restores a valid model.
- [ ] Startup does not instantiate every persisted WKWebView.
