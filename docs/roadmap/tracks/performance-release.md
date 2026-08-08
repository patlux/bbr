# Track — Performance, energy, and release safety

Parent phases: [Phase 0](../phases/00-foundation.md), [Phase 4](../phases/04-ui-shell.md), [Phase 7](../phases/07-persistence.md), [Phase 9](../phases/09-quality-release.md) · Product contract: [performance-contract.md](../../product/performance-contract.md) · Current task: [performance E2E foundation](../tasks/performance-e2e-foundation.md)

## Implemented foundation

- [x] Bounded local build/test/E2E recipes with process cleanup.
- [x] Native WKWebView content avoids screenshot/texture composition.
- [x] Native slot geometry updates only on layout changes.
- [x] Lifecycle model distinguishes active, warm, suspended, and discarded tabs.

## Performance E2E foundation — Phase 4 start

- [x] Add an optimized real-app benchmark build with typed monotonic in-process milestones.
- [x] Require the first usable startup presentation to contain both the complete sidebar and the selected tab's visible native web view.
- [x] Add bounded `just e2e-perf` and `just e2e-perf-full` recipes. *(`e2e-perf-full` runs the repeated cold gate — not an alias.)*
- [x] Enforce absolute budgets plus the >15% p95 regression rule. *(Absolute ceilings always enforced everywhere. The >15% relative rule is implemented and unit-tested but fires only on an explicitly named canonical machine; no dedicated runner is provisioned yet, so `canonical_host_model` is empty and it is currently report-only.)*
- [x] Store raw samples, p50/p95/max, machine/build metadata, and failure artifacts. *(JSON + CSV + event ring + stderr on failure; trend retained on success.)*
- [x] Make every new user-visible interaction add or extend its performance E2E scenario in the same implementation group.

## Performance measurements

- [x] Cold and warm startup to first usable sidebar + selected native content presentation. *(Fresh-profile cold startup is gate-enforced. F3 integrated `persisted_warm_startup` through the real repository with visible-set-only hydration and at least 10 accepted samples; latest integrated report-only evidence was p95/max 220.82 ms with unchanged 500/750 ms budgets. A named canonical-runner distribution remains open separately.)*
- [x] Time to deterministic local fixture readiness, measured separately from shell presentation.
- [x] Tab/space switch latency for active, warm, suspended, and discarded states. *(Warm tab and warm space switching are measured through the production presentation path; the tab-organization slice adds the pin toggle, region-bounded reorder, and same-profile move to the warm class and the cross-profile move to the session-recreation class; the cross-context slice adds `space_switch_cross_profile` — the same warm view-only class as `space_switch` with the §11.3 command-field hint asserted in the driver snapshot and guarded in-app by the presentation probe. Active/suspended/discarded space states remain.)*
- [x] Active-tab close to correct neighbor presentation.
- [x] Sidebar open, close/toggle, resize, and restoration latency. *(Toggle, drag resize, and collapse/restore are measured through the production presentation path; `sidebar_resize`/`sidebar_restore` join the 30-sample interaction matrix with the same 50/100 ms next-frame budget.)*
- [x] Command-bar first-result and full-search latency. *(`command_bar_first_result`
      (Phase 7 history provider: 500 tabs + 10k history, expected row
      `history:1`, 30 samples) and its complete-result-set sibling
      `command_bar_full_search` (the full limit-16 window plus the truthful
      total for a query matching the whole provider) are measured through the
      production command-text path with render-inclusive commits — the
      performance contract's "first result, filtering" interactions now have
      gate-enforced scenarios (queue item 16). Canonical-runner evidence for
      both stays open with the dedicated runner.)*
- [ ] Sidebar frame time with 500 tabs. *(The 500/50/450 lifecycle is proven
      by the functional dense E2E (`just e2e-shell-dense`); the shipped
      `dense_scroll`/`dense_select` 30-sample scenarios prove the
      bounded-window frame path only at the S-density 230/20 collection on
      the report-only dev host — a 500/50 perf-gate re-seed was measured and
      reverted because the perf matrix's prior WebKit churn kills the active
      tab's process mid-loop (`PageCrashed`, 3/3 runs), and neither the
      functional lifecycle proof nor the >=500 command corpus is measured
      frame-time evidence. The phase-4 measured-frame-time acceptance at
      500/50 remains open for the dedicated canonical runner.)*
- [ ] Idle CPU, wakeups, and energy impact.
- [ ] Memory per active/warm tab and total memory under pressure.
- [ ] Suspend/resume latency and state fidelity.
- [ ] Persistence write volume and startup hydration cost.

## Policies and budgets

- [x] Define initial product budgets before broad Phase-4 UI implementation.
- [x] Validate and enforce the initial budgets through the performance E2E foundation. *(Absolute implemented-scenario gates are enforced everywhere; the canonical-only >15% relative rule is implemented and waiting for a named dedicated runner.)*
- [ ] Revisit only with explicit product evidence; never raise a budget solely to make a regression pass.
- [ ] Bound concurrent background loads.
- [ ] Suspend least-recently-used tabs under memory pressure.
- [ ] Ensure pinned tabs are not permanently live by default.
- [ ] Debounce metadata persistence.
- [ ] Avoid animations that continuously invalidate native geometry.
- [ ] Compare equivalent workflows with Safari and the Swift reference.

## Release safety

- [ ] Non-testkit binaries reject test-only flags.
- [ ] Release validation proves no test socket is created.
- [ ] Signed/notarized `.app` build is reproducible.
- [ ] Clean-machine launch, update, rollback, and data migration are tested.
- [ ] Crash reporting and privacy/data-retention policies are documented.
