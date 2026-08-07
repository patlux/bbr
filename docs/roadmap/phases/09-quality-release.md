# Phase 9 — Accessibility, polish, performance, and release hardening

Parent: [Roadmap](../../../ROADMAP.md) · Tracks: [UI design](../tracks/ui-design-system.md), [Accessibility](../tracks/accessibility.md), [Performance/release](../tracks/performance-release.md), [Testing](../tracks/testing.md)

## Outcome

The complete alpha feature set receives a deliberate product-wide quality pass and becomes safe to distribute.

## Dedicated visual polish pass

- [ ] Audit every surface against `DESIGN.md` and eliminate visual drift.
- [ ] Refine typography, spacing rhythm, alignment, icon optical balance, materials, and hierarchy.
- [ ] Refine purposeful transitions and micro-interactions; honor Reduce Motion.
- [ ] Verify light, dark, increased-contrast, narrow, wide, empty, dense, loading, and error variants.
- [ ] Remove placeholder copy, glyphs, debug surfaces, and inconsistent terminology.
- [ ] Add onboarding and first-run activation only after the real workflow exists.

## Accessibility and input

- [ ] Complete the [Accessibility track](../tracks/accessibility.md).
- [ ] Run VoiceOver workflows for browser controls and WKWebView hand-off.
- [ ] Verify all actions by keyboard and pointer.

## Performance and energy

- [ ] Complete the [Performance/release track](../tracks/performance-release.md).
- [ ] Tighten and harden the performance budgets established before Phase 4; do not introduce the first benchmark harness here.
- [ ] Measure cold/warm startup, command latency, sidebar frame time, idle CPU/wakeups, memory per lifecycle state, and suspend/resume latency.
- [ ] Compare representative workflows with Safari and the Swift reference.

## Release safety

- [ ] Complete the deferred [native Portal host hardening](../tasks/native-portal-hardening.md) for focus/key routing, multi-window ownership, display lifecycle, teardown, and packaged-app behavior.
- [ ] Reject test-only flags in non-testkit builds.
- [ ] Verify release binaries contain no test socket startup path.
- [ ] Build signed/notarized `.app` artifacts and test clean-machine startup.
- [ ] Add a small Appium Mac2 blackbox suite for menus, Accessibility, real input, window behavior, and packaged-app launch.
- [ ] Define crash reporting, privacy, data retention, update, and rollback policy.

## Phase exit

- [ ] Release candidate meets functional, accessibility, visual, performance, security, and packaging gates.
