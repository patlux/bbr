# Track — Performance, energy, and release safety

Parent phases: [Phase 0](../phases/00-foundation.md), [Phase 7](../phases/07-persistence.md), [Phase 9](../phases/09-quality-release.md)

## Implemented foundation

- [x] Bounded local build/test/E2E recipes with process cleanup.
- [x] Native WKWebView content avoids screenshot/texture composition.
- [x] Native slot geometry updates only on layout changes.
- [x] Lifecycle model distinguishes active, warm, suspended, and discarded tabs.

## Performance measurements

- [ ] Cold and warm startup to usable browser controls.
- [ ] Time to first visible page.
- [ ] Tab/space switch latency for active, warm, suspended, and discarded states.
- [ ] Command-bar first-result and full-search latency.
- [ ] Sidebar frame time with 500 tabs.
- [ ] Idle CPU, wakeups, and energy impact.
- [ ] Memory per active/warm tab and total memory under pressure.
- [ ] Suspend/resume latency and state fidelity.
- [ ] Persistence write volume and startup hydration cost.

## Policies and budgets

- [ ] Define target budgets before Phase 9 tuning.
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
