# Phase 7 — Persistence and crash recovery

Parent: [Roadmap](../../../ROADMAP.md) · Quality: [Performance/release track](../tracks/performance-release.md)

## Outcome

Browser organization restores safely after normal quit, force quit, and schema upgrades without eagerly creating every web session.

## Checklist

- [ ] Add `bbr-persistence` with SQLite repository interfaces and versioned migrations.
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
