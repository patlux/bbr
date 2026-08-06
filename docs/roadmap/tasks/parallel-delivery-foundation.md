# Task packet — parallel delivery foundation

Parents: [Roadmap](../../../ROADMAP.md) · [Development workflow](../../development-workflow.md) · [Roadmap maintenance](../README.md) · [Testing track](../tracks/testing.md) · [Parallel-delivery contract](../../parallel-delivery.md).

## Outcome

BBR can prepare independently valuable roadmap work in sibling Git worktrees
without ambiguous writer ownership, accidental shared-contract divergence, or
concurrent AppKit/WKWebView GUI gates. A clean, verified foundation commit is
the only valid base for new lanes, and Phase 5 command core plus Phase 7
persistence core have portable, dependency-free landing zones.

## Scope

- Document active execution lanes, their dependencies, exclusive surfaces,
  shared integration points, readiness, and integration order.
- Define the serial integration kernel and the worktree lifecycle, including
  settlement, handoff, integration-wave verification, and stale-lane handling.
- Add a host-wide advisory GUI lock shared by every worktree through Git's
  common directory; wrap every bounded real-app/performance recipe with it.
- Add contract tests for command exit forwarding, cross-process serialization,
  timeout behavior, and holder diagnostics.
- Add empty portable workspace crates `bbr-command` and `bbr-persistence` as
  lane landing zones without product behavior or speculative dependencies.
- Update the canonical roadmap and workflow documents, then record the clean
  verified foundation commit in this delivery ledger before feature worktrees
  are created.

## Non-goals

- No command parsing/ranking, SQLite schema, repository implementation, split
  behavior, WebKit capability, or other product feature.
- No `bbr-ui` extraction: the current GPUI shell and shared testkit/snapshot
  surfaces need a separate bounded architecture packet before moving.
- No new feature worktrees or writers in this packet. Lane activation follows
  only after the foundation is reviewed, committed, and green.
- No mutation, rebase, reset, or replacement of the preserved
  `/Users/patwoz/dev/Personal/macos/bbr-url-input` worktree.
- No remote creation, push, PR, CI runner provisioning, or canonical benchmark
  acceptance.

## Constraints

- Start only after Pi session `019fd6aa-c03a-7313` and workflow
  `bbr-phase4-cross-context-identity-mshncwou-95feng` settle terminally; preserve
  their clean `main` result as the predecessor anchor.
- One writer per branch/worktree; reviewers are read-only. Shared contracts
  have one integration owner at a time.
- `bbr-command` and `bbr-persistence` stay portable and `#![forbid(unsafe_code)]`;
  no GPUI, macOS, WebKit, or cross-lane dependency is introduced.
- GUI suites remain sequential across all sibling worktrees. The lock is
  advisory, bounded, stale-process safe, and cannot weaken each command's
  existing timeout/process cleanup.
- No test, validator, budget, sample count, roadmap acceptance, or architecture
  boundary is weakened.

## Evidence

### Unit and contract

- [x] `python3 -m unittest scripts/test_with_gui_lock.py -v` proves exit-code
      forwarding, serialization, bounded timeout, and holder diagnostics.
- [x] Workspace checks compile and test the portable landing-zone crates.
- [x] Roadmap validation covers this packet and the linked parallel contract.

### Real-app E2E

- [x] `just e2e-smoke` proves a wrapped GUI command acquires the host-wide gate
      and still launches the real BBR app with real WKWebView behavior.
- [x] Canonical `just e2e` remains green with every constituent recipe wrapped;
      no GUI suite runs concurrently inside the gate.

### Performance

- [x] `just e2e-perf` remains green through the same lock with unchanged
      performance budgets, samples, milestones, and baseline evaluation.
- [x] No new user-visible interaction exists; no new benchmark scenario is
      required.

## Completion gates

```bash
python3 -m unittest scripts/test_with_gui_lock.py -v
python3 -m py_compile scripts/with_gui_lock.py scripts/test_with_gui_lock.py
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-smoke
just e2e-perf
just e2e
just roadmap
```

GUI recipes use the host-wide lock and remain sequential.

## Delivery sequence

1. One exclusive writer implements and verifies the bounded foundation.
2. One independent read-only reviewer maps every lane/lock/worktree requirement
   to source, docs, tests, and command evidence.
3. Run one remediation writer only for concrete findings, then focused recheck.
4. Commit the reviewed foundation and record its exact commit below.
5. Only then may separate bounded packets activate new lane worktrees.

## Iteration policy

First establish the documented ownership and serial-kernel contract, then the
smallest executable GUI-lock seam, then portable crate landing zones. Use the
wrapper's deterministic contract tests before running a real GUI suite. After
each failure, identify whether the mechanism is lock-key derivation, process
settlement, command forwarding, just recipe composition, or unrelated product
behavior. Do not broaden into feature implementation.

## Blocked stop condition

Stop with preserved Git/worktree state, workflow settlement evidence, attempted
checks, exact blocker, and required next input if the predecessor writer is not
terminal, ownership becomes ambiguous, the existing URL-input worktree would
need mutation, no clean anchor can be established, or the host cannot provide a
bounded advisory-lock mechanism without weakening existing GUI gates.

## Delivery ledger

```text
Outcome: Worktree lanes, serial integration kernel, portable command/
         persistence landing zones, and one host-wide GUI gate.
Commits: c7a9cd2 baseline-reproduced domain lint fix; 03b9e8b parallel-
         delivery foundation; this ledger follow-up.
Predecessor anchor: 20628ff on clean main after terminal remediation + recheck.
Foundation anchor: 03b9e8b (implementation); the ledger follow-up is the exact
         branch point for newly activated lane worktrees.
Functional evidence: just verify; testkit Clippy; GUI-lock unit contracts and
         py_compile; shared Git-common-dir lock key verified from main and the
         preserved URL-input worktree.
Real-app E2E: just e2e-smoke; canonical just e2e, every recipe reporting the
         same /tmp/bbr-gui-gate-6ff732a0fdc0bb09.lock key.
Performance evidence: just e2e-perf and canonical perf scenario green with
         unchanged budgets/samples; no new user-visible interaction.
Roadmap/documentation updates: ROADMAP lanes/current focus, parallel-delivery
         contract, workflow/roadmap/template/phase/testing/architecture docs.
Open blockers and non-goals: no feature implementation; split and WebKit lanes
         still require serial contract packets; no remote/canonical runner;
         preserved parallel/url-input is not activated or mutated.
Worktree state: main; foundation files ready for coherent commit.

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```
