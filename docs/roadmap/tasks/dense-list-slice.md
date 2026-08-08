# Task packet — Dense-500 list slice (lifecycle, bounded-window render, recency)

Parents: [Phase 4 — UI shell](../phases/04-ui-shell.md) ·
[Performance/release track](../tracks/performance-release.md) · Product:
[DESIGN.md §11.6](../../../DESIGN.md) ·
[performance contract](../../product/performance-contract.md)

## Objective

Make the vertical tab shell genuinely dense: 500 tab identities with exactly
50 live/warm native sessions (450 suspended) through the production lifecycle
policy, a bounded-window render that never lays out the whole collection,
deterministic recency-preserving selection, and gate-enforced scroll/select
frame-time scenarios so a dense render regression fails the benchmark.

## Scope

- **Lifecycle bounds:** seed and keep 500 identities / 50 live sessions /
  450 suspended with a concurrent-session peak that never approaches 500
  WKWebViews (each transient session is destroyed before the next open).
- **Bounded windowed rendering:** the regular region renders only the window
  around the production scroll offset (≤ `DENSE_WINDOW_ROWS` rows) plus
  spacers; the scrollbar/wheel listener behave as if every row were laid out.
- **Recency model:** the keep set is pinned + recently used identities;
  suspended selection recreates exactly one session; warm selection reuses
  the existing WebViewId with no reload.
- **Gate wiring:** `dense_scroll` / `dense_select` scenarios in the
  interaction matrix with contract budgets in the baseline seed, sample-count
  asserts in the full gate, and honest scroll-seam docs (the seam mutates the
  tracked `ScrollHandle`, not a synthetic `ScrollWheelEvent` — GPUI 0.2.2
  exposes no public wheel dispatch).

## Evidence

- `just e2e-shell-dense` (+ `e2e_shell_dense_edge`): 500/50 lifecycle, bounded
  render + deterministic anchor, keyboard traversal, warm/suspended selection
  recreation, edge states, dense drag tick.
- `just e2e-perf` / `just e2e-perf-full`: `dense_scroll` / `dense_select`
  scenarios, 30 samples each, against the 50/100 ms warm budgets (the perf
  gate seeds the S-density 230/20 collection; the 500/50 lifecycle itself is
  proven by the functional dense E2E, and the measured 500/50 frame-time
  acceptance stays on the canonical runner below — a perf-gate re-seed to
  500/50 was measured and reverted because the perf matrix's prior WebKit
  churn kills the active tab's process mid-loop on the dev host).
- Unit/contract: dense geometry (`dense.rs`), dense lifecycle
  (`controller_dense_tests.rs`, `tests_dense.rs`), baseline seeding guard.

## Non-goals (explicit)

- **Persisted-restore acceptance** — the dense state is in-memory; restoring
  500 persisted tabs through the persistence repository is deliberately open.
- **Measured frame-time evidence on the canonical runner** — the phase-4
  acceptance "sidebar remains responsive with 500 persisted tabs and 50
  live/warm sessions, with measured frame-time evidence" stays open until the
  canonical benchmark machine runs the full gate and updates the baseline.
- **Drag auto-scroll** — a drag whose target row is off-window shows no tick
  (bounded-window rendering); auto-scroll during drag is a separate feature.
