# Task packet — <bounded vertical slice>

Parents: link the owning phase, durable track, and product contract here.

## Outcome

<One independently valuable end state, phrased as observable product or
architecture behavior.>

## Scope

- <Smallest production-path capability required for the outcome.>
- <Required lower-level seam or harness extension.>
- <Required documentation/roadmap update.>

## Non-goals

- <Adjacent capability intentionally excluded.>
- <Later-phase behavior and its canonical owner.>

## Parallel-lane contract

Complete this section when the packet may run in a sibling worktree; otherwise
state `Not parallelized`.

```text
Base condition:          <durable dependency/integration condition; exact runtime SHA lives in Current delivery>
Depends on:
Provides/API:
Exclusive paths:
Shared integration points:
Integration order:
```

Do not put active run ids, current owners, temporary branch/worktree paths, or
"today's" SHA in this packet. Current delivery owns those volatile facts.

## Constraints

- <Architecture and dependency boundaries.>
- <Security, isolation, native-platform, input, or compatibility invariants.>
- <No weakened tests, budgets, validators, or acceptance criteria.>

## Evidence

### Unit and contract

- [ ] <Portable/domain/controller/component evidence.>

### Real-app E2E

- [ ] <Production GPUI path and real WKWebView behavior where applicable.>
- [ ] <Semantic IDs, typed events/snapshots, deterministic fixtures, deadlines.>

### Performance

- [ ] <Optimized production-path presentation scenario and semantic commit.>
- [ ] <Absolute budget and canonical-baseline requirement, or explicit reason
      performance coverage is not applicable.>

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-<affected-suite>
just e2e-perf          # when user-visible interaction or timing changed
just e2e
```

Add every other affected bounded suite. GUI suites run sequentially.

## Delivery sequence

1. One exclusive writer implements and verifies the bounded slice.
2. One independent read-only reviewer maps requirements to evidence.
3. Run one remediation writer only for concrete findings, followed by a focused
   recheck.
4. Reserve a separate final audit for phase closeout or high-risk boundary work.

## Iteration policy

Start with the smallest end-to-end production path. After each failing check,
identify the mechanism before changing the implementation. Prefer targeted
feedback while iterating; run the canonical sequential gate once after the
complete slice is stable. Do not broaden scope to solve unrelated findings.

## Blocked stop condition

If no defensible path remains, stop with:

- evidence gathered;
- approaches attempted;
- preserved worktree/branch state and current owner;
- exact blocker;
- smallest environment, contract decision, or user input required next.

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

Use measured timing only when runtime/command timestamps support it; otherwise
label values as estimates.
