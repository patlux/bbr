# Task packet — downloads sidebar panel

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Testing track](../tracks/testing.md).

Implementation state: the original sidebar panel is integrated (`78f3782`),
and the settled persistence/action follow-up is integrated at `cd1bc65`. The
surface consumes the L6 portable downloads contract
(`crates/bbr-web/src/downloads`); DESIGN §11.12 and §17.7–§17.12 are
implementation facts rather than queued decisions.

## Outcome

The fixed footer `[Downloads]` control (DESIGN §11.4) opens an anchored
popover (`bbr-downloads-panel`) confined to the sidebar column: an **Active**
group of non-terminal attempts and a **History** group of terminal attempts,
rendered from the L6 machine states (`Waiting`, `Running`, `Succeeded`,
`Failed`, `Cancelled`), with per-row status glyph, progress/size/status
secondary line, and exactly one contextual primary action (Cancel / Retry /
Reveal-in-Finder), a bounded 6-row window, and an empty hint. The panel
consumes the controller-owned history built on the portable L6 contract,
never invents transitions, and stays ordinary GPUI sidebar chrome — no
native-above-web Portal host.

## Scope

- Implement the macOS download adapter and controller-owned download/history
  store around the L6 attempt state machine.
- Compose the footer `[Downloads]` control into the anchored popover per
  DESIGN §11.12: Active/History groups with `type:badge` labels, status
  glyphs, secondary-line progress (known and unknown length) / size / status
  copy, contextual action, empty hint, `dur:fine` opacity open/close, and
  Escape / outside-press / re-activation dismissal.
- Implement bounded rendering: max 6 visible rows with §11.6 overscan and
  scrollbar, and deterministic group ordering (Active by ascending
  `DownloadId`, History most-recent-terminal first) so row index `<n>` is
  stable for E2E.
- Define destination policy/save-panel behavior and typed native events
  without leaking filesystem/AppKit types into `bbr-web`.
- Add deterministic fixture-server download routes: known length, unknown
  length, slow progress, failure, cancellation race, retry, and zero bytes.
- Expose typed testkit commands/snapshots/events (`download_rows` with
  `machine_state`, `group`, `progress_fraction`, `visual_state`, `action`,
  `action_enabled`) and stable `bbr-download-*` ids; add the bounded
  `just e2e-shell-downloads` recipe to the justfile and the canonical
  `just e2e` sequence.
- Document implemented architecture after evidence.

## Non-goals

- No Portal host for the panel: it is sidebar chrome confined to the sidebar
  column and stays `GpuiDeferred`/ordinary GPUI; the Portal host remains
  reserved for chrome that renders above web content.
- No cloud download manager, background daemon, sync, or cross-window history.
- No auto-open of completed files and no open-on-double-click: completed
  files open only through the per-row Reveal-in-Finder action (settled
  §17.12).
- No multi-select or batch actions in v1.
- No save panel / explicit destination picker: the silent suggested-
  destination policy is settled (§17.9); an explicit destination stays a
  later non-breaking option.
- No migration or backward compatibility: a legacy same-v2 downloads shape
  is discarded and immediately rewritable (settled 2026-08-08).
- No weakening of L6 identity, monotonic progress, terminal, cancellation, or
  retry invariants; no production network fixture dependency.

## Parallel-lane contract

Not parallelized. Adapter events, app controller state, sidebar/footer
composition, testkit wire, history policy, and performance baseline are shared
integration surfaces.

## Constraints

- L6 remains the portable lifecycle authority; the UI renders exactly
  `Waiting`, `Running`, `Succeeded`, `Failed { failure }`, `Cancelled` and
  never invents transitions. Retry exists on `Failed` and restart on
  `Cancelled` (settled §17.8) — both link a fresh attempt via `retry_of`;
  the panel labels the same machine path differently by state.
- Cancellation is request/confirmation based; the row stays active until the
  engine's terminal event lands, and the first terminal event wins — no
  optimistic state change.
- Reveal-in-Finder is enabled only for an existing resolved terminal path and
  returns a typed failure otherwise (DESIGN §10.8 `disabled` with reason
  tooltip).
- Filesystem/save-panel/AppKit code stays in macOS/app layers, never `bbr-web`.
- Geometry comes only from settled DESIGN tokens (§8/§11.12); no new sizes.
- Ids follow the `bbr-download-*` convention (§11.12/§15).
- No sleeps or pixel assertions in E2E; typed events, stable ids,
  deterministic fixtures, explicit deadlines.
- Settled post-MVP decisions (DESIGN §17.7–§17.12, 2026-08-08): an explicit
  Clear panel action empties the controller history and the repository
  durably (§17.7); a cancelled row offers Restart **and** Delete (§17.8);
  silent suggested destination (§17.9); terminal history persists through
  restarts with no migration (§17.10); the footer badge counts only
  in-flight attempts (§17.11); completed files open only via
  Reveal-in-Finder (§17.12).
- Keep files below roughly 500 lines; no new dependency unless existing
  persistence/native APIs are proven insufficient.

## Evidence

### Unit and contract

- [x] Adapter/controller tests cover every L6 event/rejection, history
      ordering, destination decisions, cancel races, retry linkage, reveal
      eligibility, and zero-byte success.
- [x] Component/semantic tests cover empty, active (known and unknown
      length), complete, failed, and cancelled rows, group transitions,
      bounded-window scroll, and footer/panel focus and dismissal.

### Real-app E2E

- [x] `just e2e-shell-downloads` (a bounded suite in the canonical `just e2e`
      sequence) downloads deterministic fixture content in the real app and
      proves progress (known and unknown length), success/path, failure,
      cancel, retry, zero bytes, bounded 6-row scroll, dismissal (Escape /
      outside press / re-activation), and Reveal-in-Finder request
      semantics — plus the settled post-MVP decisions: cancelled Restart
      with a fresh `retry_of` attempt, the active-only footer badge, and
      terminal-history restore, durable Clear, and durable row Delete
      across real process restarts on the same profile directory.
- [x] Opening/operating the panel preserves active tab/WebView/session/
      profile/space identity, asserts the `bbr-download-*` ids and
      `download_rows` snapshot fields, and uses typed events without sleeps.

### Performance

- [x] Add `downloads_panel_open`, `download_progress_commit`, and required row
      action scenarios with at least 30 accepted samples where the action is
      user-visible, asserting the DESIGN §15 presentation-commit gate:
      `download_row_retry`, `download_row_restart`, `download_row_delete`,
      and `downloads_clear` (30 samples each).
- [x] Exclude fixture/network latency from application presentation timings.

## Completion gates

```bash
just verify
cargo clippy -p bbr-web-macos --all-targets -- -D warnings
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-shell-downloads
just e2e-perf
just e2e
just roadmap
```

## Delivery sequence

1. Implement adapter/controller lifecycle with deterministic fixture tests.
2. Add the smallest anchored popover and typed snapshot contract.
3. Add cancel/retry/reveal and complete empty/error states with bounded
   rendering.
4. Add real-app E2E, optimized performance evidence, review, remediation, and
   recheck.

## Iteration policy

First prove one known-length successful download end to end in the real app.
Then add failure, cancel, retry, unknown/zero length, history, and Finder
reveal. Diagnose failures as L6 transition, adapter mapping,
destination/filesystem policy, panel state, or fixture synchronization before
changing code.

## Blocked stop condition

If the pinned WebKit download API cannot expose the required destination or
progress lifecycle, stop with L6 intact and the footer control
non-destructive. Report APIs tried, events observed, file-policy constraints,
and the smallest adapter/product decision required. Do not bypass L6
invariants or fabricate progress.

## Delivery ledger

```text
Outcome: post-MVP downloads follow-up integrated as one real vertical slice
  on the post-F2c AppStore form (persistence restore at startup, durable
  Clear/Delete, cancelled Restart, active-only badge, Reveal-only).
Commits: original panel delivery recorded at `78f3782`; durable serial
  persistence/action integration `cd1bc65`
Functional evidence: bbr-web 331, bbr-browser 144, bbr-persistence 22,
  bbr-app unit 256 — all green; fmt/clippy -D warnings green; just verify
  green; independent final verdict REVIEW_PASS
Real-app E2E: e2e-shell-downloads 11/11 green (incl. restart persistence,
  durable Clear/Delete after restart, cancelled Restart with fresh retry_of,
  active-only badge, identity)
Performance evidence: downloads_panel_open / download_progress_commit /
  download_row_retry / download_row_restart / download_row_delete /
  downloads_clear each 30 accepted samples, p95 ≈ 0.5–1.4 ms against the
  unchanged 50/100 ms warm budgets. Full-gate attempts also observed
  unrelated single-sample startup/command-max host-load overshoots; the
  unchanged repeat passed with zero regressions. Budgets and samples stayed
  unchanged
Roadmap/documentation updates: DESIGN §11.12 + §17.7–§17.12 resolved;
  packet rewritten to the settled decisions; evidence checkboxes ticked
Open blockers and non-goals: no F2b-specific blocker. The canonical `just
  e2e` aggregate remains globally unavailable at the missing shutdown target,
  and the two separately tracked Portal base failures remain outside this
  slice; see the post-MVP delivery ledger. Product non-goals remain above
Worktree state: integrated on `main`; historical lane/worktree state remains in
  the preserved delivery ledger, not in the current execution queue
```
