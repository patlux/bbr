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
- [x] Add bounded `just e2e-perf` and `just e2e-perf-full` recipes. *(`e2e-perf-full` currently aliases the short gate.)*
- [ ] Enforce absolute budgets plus >15% p95 regression detection on a dedicated canonical macOS runner.
- [ ] Store raw samples, p50/p95/max, machine/build metadata, and failure artifacts.
- [x] Make every new user-visible interaction add or extend its performance E2E scenario in the same implementation group.

## Performance measurements

- [ ] Cold and warm startup to first usable sidebar + selected native content presentation. *(Fresh-profile startup implemented; persisted warm startup remains.)*
- [ ] Time to deterministic local fixture readiness, measured separately from shell presentation.
- [ ] Tab/space switch latency for active, warm, suspended, and discarded states.
- [x] Active-tab close to correct neighbor presentation.
- [ ] Sidebar open, close/toggle, resize, and restoration latency. *(Toggle implemented; resize/restoration remain.)*
- [ ] Command-bar first-result and full-search latency.
- [ ] Sidebar frame time with 500 tabs.
- [ ] Idle CPU, wakeups, and energy impact.
- [ ] Memory per active/warm tab and total memory under pressure.
- [ ] Suspend/resume latency and state fidelity.
- [ ] Persistence write volume and startup hydration cost.

## Policies and budgets

- [x] Define initial product budgets before broad Phase-4 UI implementation.
- [ ] Validate and enforce the initial budgets through the performance E2E foundation. *(Absolute implemented-scenario gates are enforced; canonical baseline regression policy remains.)*
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
