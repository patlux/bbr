# Task packet — downloads sidebar panel

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Testing track](../tracks/testing.md).

Status: **post-MVP; consumes the integrated L6 portable downloads contract.**

## Outcome

The sidebar footer's Downloads control opens a coherent GPUI panel showing
active, completed, failed, cancelled, and retrying downloads with destination,
progress, retry/cancel, and “Show in Finder” actions. The UI consumes one
controller-owned history built on the portable L6 contract and does not require
the native-above-web Portal host.

## Scope

- Implement the macOS download adapter and controller-owned download/history
  store around the L6 attempt state machine.
- Compose the existing footer Downloads control into a GPUI sidebar panel with
  active progress, terminal history, empty/loading/error states, cancel, retry,
  clear-history policy, and reveal-in-Finder action.
- Define destination policy/save-panel behavior and typed native events without
  leaking filesystem/AppKit types into `bbr-web`.
- Add deterministic fixture-server download routes including known length,
  unknown length, slow progress, failure, cancellation race, retry, and zero
  bytes.
- Expose typed testkit commands/snapshots/events and stable `bbr-download-*`
  ids; document implemented architecture after evidence.

## Non-goals

- No Portal host for the panel; it is sidebar chrome and stays
  `GpuiDeferred`/ordinary GPUI unless a later design explicitly overlaps web
  content.
- No cloud download manager, background daemon, sync, or cross-window history.
- No weakening of L6 identity, monotonic progress, terminal, cancellation, or
  retry invariants.
- No production network fixture dependency.

## Parallel-lane contract

Not parallelized. Adapter events, app controller state, sidebar/footer
composition, testkit wire, persistence/history policy, and performance baseline
are shared integration surfaces.

## Constraints

- L6 remains the portable lifecycle authority; adapters cannot invent invalid
  transitions or reuse attempt ids.
- Filesystem/save-panel/AppKit code stays in macOS/app layers, never `bbr-web`.
- History records terminal paths/timestamps outside the stateless L6 machine.
- Cancellation is request/confirmation based; first terminal event wins.
- Retry creates a fresh attempt id linked to the failed attempt.
- Reveal-in-Finder is enabled only for an existing resolved terminal path and
  returns a typed failure otherwise.
- Keep files below roughly 500 lines; no new dependency unless existing
  persistence/native APIs are proven insufficient.

## Evidence

### Unit and contract

- [ ] Adapter/controller tests cover every L6 event/rejection, history ordering,
      destination decisions, cancel races, retry linkage, reveal eligibility,
      and persistence policy.
- [ ] Component/semantic tests cover empty, active, complete, failed,
      cancelled, and retrying rows plus footer/panel focus.

### Real-app E2E

- [ ] `just e2e-shell-downloads` downloads deterministic fixture content in the
      real app and proves progress, success/path, failure, cancel, retry, zero
      bytes, panel dismissal, and Show-in-Finder request semantics.
- [ ] Opening/operating the panel preserves active tab/WebView/session/profile/
      space identity and uses stable typed ids/events without sleeps.

### Performance

- [ ] Add `downloads_panel_open`, `download_progress_commit`, and required row
      action scenarios with at least 30 accepted samples where the action is
      user-visible.
- [ ] Exclude fixture/network latency from application presentation timings.

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
2. Add the smallest sidebar panel and typed snapshot contract.
3. Add cancel/retry/reveal and complete empty/error states.
4. Add real-app E2E, optimized performance evidence, review, remediation, and
   recheck.

## Iteration policy

First prove one known-length successful download end to end. Then add failure,
cancel, retry, unknown/zero length, history, and Finder reveal. Diagnose failures
as L6 transition, adapter mapping, destination/filesystem policy, panel state,
or fixture synchronization before changing code.

## Blocked stop condition

If the pinned WebKit download API cannot expose the required destination or
progress lifecycle, stop with L6 intact and the footer control non-destructive.
Report APIs tried, events observed, file-policy constraints, and the smallest
adapter/product decision required. Do not bypass L6 invariants or fabricate
progress.

## Delivery ledger

```text
Outcome: post-MVP packet documented
Commits: planning only
Functional evidence: pending
Real-app E2E: pending
Performance evidence: pending
Roadmap/documentation updates: task packet + Phase 8/track links
Open blockers and non-goals: GPUI sidebar panel; no Portal requirement
Worktree state: no implementation lane active
```
