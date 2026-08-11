# Task packet — Web-content gesture real-app revalidation

Parents: [Phase 8](../phases/08-daily-driver.md) · [Web-content navigation gestures](web-content-navigation-gestures.md) · [daily-driver delivery](../../delivery/2026-08-12-daily-driver-delivery.md).

## Outcome

The current integrated BBR application demonstrably performs two-finger Back/Forward over real webpage content through the production WKWebView input path. If the path has regressed since its original 9/9 acceptance, the smallest causal production fix restores it without changing the accepted gesture or opener-return contract.

## Scope

- Rebind the existing gesture E2E to the exact current Daily-driver activation base.
- Inspect the production native input, event pump, controller action, and real session history chain only if current evidence fails.
- Add a regression test for any newly found integration defect before fixing it.
- Preserve the existing nine scenario families and performance code/guards.
- Record whether the lane closes as evidence-only or produces a remediation candidate.

## Non-goals

- No new gesture semantics, thresholds, animation, sidebar swipe changes, titlebar behavior, background-tab policy, or private platform input handling.
- No performance execution, benchmark retry, sample generation, or baseline change.
- No synthetic direct-controller command presented as gesture evidence.

## Parallel-lane contract

```text
Base condition:          committed Daily-driver delivery authority and clean common activation base
Depends on:              integrated N1 gesture implementation and real-app harness
Provides/API:            current-base evidence or the smallest regression fix; no new public product API expected
Exclusive paths:         existing focused gesture E2E/fixture modules; production gesture modules only after a failing regression is reproduced
Shared integration points: native input bridge, event pump, testkit request/event envelope, justfile only if the existing recipe is broken
Integration order:       evidence-only may close without merge; any remediation integrates after DD-LINK
```

## Constraints

- A green existing suite on the exact current base closes the functional concern; do not churn product code for manual reassurance.
- A failure must be classified and reproduced before mutation.
- Gesture over web content targets the exact pane; sidebar, Portal, palette, menu, and find surfaces retain input ownership.
- Public APIs only; no global event tap or weakened threshold/focus/history assertions.

## Evidence

### Unit and contract

- [ ] Existing gesture decision, provenance, and atomic-return tests remain green on the current base.
- [ ] Any regression fix adds a causal test that fails for the observed mechanism before the fix and passes afterward.

### Real-app E2E

- [ ] `just e2e-web-navigation-gestures` passes all nine existing scenarios on the exact activation base or exact reviewed remediation tree.
- [ ] Back and Forward change the real current WKWebView history and preserve TabId/WebViewId/session identity.
- [ ] Sidebar/Portal/command/menu hit-testing and child Back-at-root remain distinct.

### Performance

- [ ] Existing production performance code and guards remain byte-preserved unless a causal product fix requires corresponding milestone maintenance.
- [ ] Execution remains unexecuted/unpassed under Patrick's override.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-browser -p bbr-web -p bbr-web-macos
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-web-navigation-gestures
just roadmap
git diff --check
```

## Delivery sequence

1. Run the exact bounded current-base functional suite once under the repository GUI lock.
2. If green, record evidence and close without product mutation.
3. If red, reproduce the smallest causal defect, add regression coverage, and fix only that mechanism.
4. Independent read-only review is required only for a product remediation candidate; evidence-only closeout receives Orchestrator readback.

## Iteration policy

One current-base run only before diagnosis. Do not retry an unchanged failure. Use event-ring, app stderr, typed snapshots, and source ownership to classify native mapping, hit testing, pump dispatch, history availability, provenance, focus, or harness drift.

## Blocked stop condition

If the exact production gesture cannot be driven because of a host/input limitation, stop with the log, current tree, event-ring evidence, and smallest manual or harness input needed. Do not substitute a direct WebAction and claim the gesture works.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```
