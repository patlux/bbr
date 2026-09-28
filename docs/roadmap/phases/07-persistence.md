# Phase 7 — Persistence and crash recovery

Parent: [Roadmap](../../../ROADMAP.md) · Quality: [Performance/release track](../tracks/performance-release.md)

## Outcome

Browser organization restores safely after normal quit, force quit, and schema upgrades without eagerly creating every web session.

## Parallel lanes

The bounded [portable persistence-core packet](../tasks/persistence-core.md)
landed on `main` at `3a3c7f1f51759aa68ecf84b519b0c7acd3826889` after independent review and
focused correctness remediation. It owns schema version 1, repository DTOs,
transactional complete-snapshot writes, read-only restore/recovery, checked-in
migration fixtures, and dense repository distributions. The former
`parallel/persistence-core` lane is settled and writer-free.

The [schema v2 decision record](../tasks/schema-v2-migration-decision.md) then
closed the migration question for the current schema: **v2, no migrations, no
backward compatibility** — a store whose stamp or structure does not match is
discarded and recreated empty (a normal, logged, non-fatal startup path), and
the v2 shape adds the global settings row, downloads history, and the
command-history table.

The **command-history provider slice** (the F2c lane) integrated the app-owned
composition on top of v2: one `bbr-app -> bbr-persistence` bootstrap restores
the global search settings and the in-memory command-history provider at
startup, records accepted command/navigation activations (dedupe by normalized
URL with monotonic `visit_count`/`last_visited`), and persists them through a
bounded nonblocking capacity-1 writer — ranking and render never touch SQLite.
The provider scale (500 tabs + 10,000 history entries) is enforced by the
`command_bar_first_result` real-app performance scenario. Still open and serial:
per-site settings/zoom, lazy native session hydration, force-quit E2E, warm
restore, and metadata debounce. The [B1 Saved Pages schema/repository packet](../tasks/saved-pages-portable-persistence.md)
landed on `parallel/saved-pages-b1` for serial integration: it adds the
portable Saved Page contract (`bbr-domain::saved_pages`, one shared canonical
URL seam reusing `bbr_web::normalize_url`) and schema **v3** with the global
`saved_pages`/`collections` tables plus repository CRUD/search/collections and
checked-in fixtures, following the standing development rule — schema bump
plus discard, no migration or compatibility path.

## Checklist

- [x] Add `bbr-persistence` with SQLite repository interfaces and versioned migrations. *(Schema v1 with fixture-backed `0 -> 1` migration landed with the persistence-core packet; the current schema is v2 with no migrations and no backward compatibility — see the [schema v2 decision](../tasks/schema-v2-migration-decision.md).)*
- [x] Persist profiles, spaces, tabs, pin/order state, active tabs, and recently closed tabs.
- [x] Persist window geometry, sidebar state, and split layouts.
- [x] Persist command history and global search settings. *(The command-history provider slice: seeded history ranks/activates through the provider-neutral corpus, fresh visits persist across restarts, a mismatched store is discarded and recreated empty with a truthful `StoreLoad` disposition, the global provider is a single authority across profiles, and the persisted custom search engine feeds the command-bar fallback.)*
- [ ] Per-site settings/zoom remain not done. *(Out of scope for the command-history slice; the next bounded slice owns per-site settings/zoom persistence and its real-app E2E.)*
- [x] Write structural changes transactionally.
- [ ] Debounce transient title/favicon/history metadata writes. *(The command-history provider persists through a bounded nonblocking coalesced writer; title/favicon metadata debounce remains open.)*
- [x] Restore browser chrome first and hydrate only visible sessions. *(F3 warm-restore slice: the repository-restored snapshot maps through the portable chrome-first `HydrationPolicy` into id-preserving domain/controller restore; only the active space's visible set hydrates — the dense 500-tab real-app E2E ends with exactly one live session (two for an active split) and every other identity session-less until activation.)*
- [x] Recover optional corrupt session state without deleting profile browsing data. *(Repository projection returns ordered typed warnings without mutating persisted state.)*
- [x] Add checked-in migration fixtures and force-quit recovery tests. *(Migration/recovery fixtures are integrated; real-app force-quit recovery E2E `e2e_shell_persistence` proves only the last committed repository state returns after a graceful restart and after a SIGKILL force quit — for the open and the tab-reorder mutation classes — never a partial or fast-but-wrong model.)*
- [ ] Add first-run, empty, migration-error, and recovery UI states. *(Targets: [restore and recovery UI](../tasks/blueprint-restore-recovery-ui.md), [first run](../tasks/blueprint-first-run.md).)*

Durable global Saved Page URL records, flat collections, canonical-URL
idempotence, and the next development schema are a Phase-8 extension owned by
[B1](../tasks/saved-pages-portable-persistence.md), not a second Phase-7
checkbox owner. They carry no profile/session/tab ownership and add no
migration/backcompat path.

## Deferred decisions

- **Command-history retention is deferred**: the v1 provider keeps every
  accepted record (dedupe by normalized URL); no retention cap or eviction
  policy is invented here. Options for a later slice include LRU/size bounds,
  time-based pruning, or a pinned-keep set, decided together with the
  per-site settings/zoom work. The 10,000-entry scale exists only as the
  test/performance corpus size, not as a product bound.

## Phase exit

- [x] Force quit during navigation or tab reorder restores a valid model. *(Real-app force-quit E2E: a flushed structural mutation survives a SIGKILL — a flushed tab reorder returns the committed reordered order, proven by `force_quit_after_reorder_recovers_committed_order`; an unflushed mutation is present only when the repository committed it — the app presents exactly the committed state.)*
- [x] Startup does not instantiate every persisted WKWebView. *(Dense 500-tab real-app restore hydrates exactly one session (two for an active split); the optimized `persisted_warm_startup` scenario proves the same visible-set-only bound with 10 accepted samples.)*
