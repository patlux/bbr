# Phase 7 — Persistence and crash recovery

Parent: [Roadmap](../../../ROADMAP.md) · Quality: [Performance/release track](../tracks/performance-release.md)

## Outcome

Browser organization restores safely after normal quit, force quit, and schema upgrades without eagerly creating every web session.

## Parallel lane

The bounded [portable persistence-core packet](../tasks/persistence-core.md)
landed on `main` at `3a3c7f1f51759aa68ecf84b519b0c7acd3826889` after independent review and
focused correctness remediation. It owns schema version 1, repository DTOs,
transactional complete-snapshot writes, read-only restore/recovery, checked-in
migration fixtures, and dense repository distributions. The former
`parallel/persistence-core` lane is settled and writer-free. Domain mapping,
app startup/shutdown composition, future migration numbering, lazy native
session hydration, force-quit E2E, and command-history provider integration
remain serial integration points.

## Checklist

- [x] Add `bbr-persistence` with SQLite repository interfaces and versioned migrations. *(Schema v1, strict validation, typed errors, and fixture-backed `0 -> 1` migration are integrated.)*
- [x] Persist profiles, spaces, tabs, pin/order state, active tabs, and recently closed tabs.
- [x] Persist window geometry, sidebar state, and split layouts.
- [ ] Persist history and per-site settings/zoom.
- [x] Write structural changes transactionally.
- [ ] Debounce transient title/favicon/history metadata writes.
- [ ] Restore browser chrome first and hydrate only visible sessions.
- [x] Recover optional corrupt session state without deleting profile browsing data. *(Repository projection returns ordered typed warnings without mutating persisted state.)*
- [ ] Add checked-in migration fixtures and force-quit recovery tests. *(Migration/recovery fixtures are integrated; real-app force-quit recovery remains in the composition slice.)*
- [ ] Add first-run, empty, migration-error, and recovery UI states.

## Phase exit

- [ ] Force quit during navigation or tab reorder restores a valid model.
- [ ] Startup does not instantiate every persisted WKWebView.
