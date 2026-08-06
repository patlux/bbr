# Development workflow

This is BBR's reusable delivery contract for product and architecture work. It
optimizes for short feedback loops without weakening real-app E2E, performance,
architecture, or roadmap acceptance.

Repository rules in [AGENTS.md](../AGENTS.md) remain authoritative. Roadmap
ownership rules live in [docs/roadmap/README.md](roadmap/README.md).

## Default unit of delivery: a bounded vertical slice

Start implementation only after the active task packet states:

- **Outcome:** one independently valuable end state.
- **Scope:** the smallest production path that delivers it end to end.
- **Non-goals:** adjacent behavior explicitly excluded.
- **Constraints:** architecture, security, input, platform, and compatibility
  boundaries that must remain true.
- **Evidence:** unit/contract tests, real-app E2E, and optimized performance
  scenarios required for acceptance.
- **Completion gates:** exact bounded commands that prove the slice.
- **Blocked stop condition:** evidence to report when no defensible path remains.

Use [the task-packet template](roadmap/templates/task-packet.md). Split work that
cannot be reviewed and verified as one coherent outcome. Do not start with a
phase-wide implementation prompt when several independently useful slices exist.

## Default lifecycle

```text
Task packet → one writer → independent read-only review
                                  │
                                  ├─ no concrete findings → complete
                                  └─ findings → one remediation writer → focused recheck
```

### Writer

Exactly one writer owns a branch/worktree at a time. The writer:

- implements the bounded slice through the production path;
- adds the required lower-level, real-app E2E, and performance evidence;
- keeps roadmap and implemented-fact documentation truthful;
- runs targeted checks while iterating and completion gates when stable;
- leaves a concise delivery ledger.

### Independent reviewer

The reviewer is read-only and reports:

1. blockers;
2. high-, medium-, and low-severity findings;
3. requirement-to-evidence gaps;
4. a verdict: complete, remediable, or externally blocked.

The reviewer must inspect actual source, tests, documentation, and verification
evidence. It must not create speculative cleanup work or repeat the writer's
file inventory.

### Remediation

Start remediation only for concrete findings. Use one writer, preserve the
original scope, add regression evidence where applicable, and run a focused
recheck. A green review does not justify an automatic remediation pass.

### Final audit

Do not run a separate final-audit agent for every ordinary slice. Reserve a full
independent audit for:

- phase closeout;
- architecture, security, profile-isolation, or native-boundary changes;
- disputed acceptance or performance evidence;
- recovery from ambiguous ownership or incomplete workflow output.

## Test cadence

Quality requirements remain unchanged; cadence avoids redundant full gates.
GUI suites always run sequentially.

### During implementation

Use the smallest checks that provide useful feedback:

```bash
cargo test -p <affected-crate> <targeted-test>
cargo clippy -p <affected-crate> --all-targets -- -D warnings
just e2e-<affected-suite>
```

Run affected optimized performance scenarios whenever a user-visible
interaction changes. A browser behavior is never accepted only from a fake
engine, unit test, direct-controller shortcut, or debug timing.

### Before an intermediate commit

An intermediate commit inside one exclusively owned slice may use targeted
unit/contract, Clippy, E2E, and performance checks appropriate to that commit.
The commit must remain internally coherent and must not claim roadmap
completion.

### Before slice completion or handoff

Run:

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-<affected-suite>
just e2e-perf          # when user-visible interaction or timing changed
just e2e               # once after the complete slice is stable
```

Also run every other affected bounded suite. Mark roadmap acceptance complete
only after these gates pass.

### At phase closeout

Run one clean closeout sequence:

```bash
just verify
just e2e
just e2e-perf-full
just roadmap
```

Use the canonical performance runner for claims that require its accepted
baseline. If infrastructure is missing, leave the claim open and identify the
external dependency rather than adding test-only substitutes or weakening the
contract.

## Ownership and recovery

- One branch/worktree has at most one active writer.
- A reviewer does not acquire write ownership.
- Do not inspect or mutate another writer's isolated worktree.
- Transfer ownership only after explicit settlement, stop, or user-authorized
  handoff that names the worktree, branch, preserved diff, and new owner.
- Before starting a recovery writer, verify the previous writer is no longer
  active and preserve its diff without reset, clean, or stash.
- A workflow that has not persisted a child is not automatically idle. Use real
  session activity when available, but require an explicit terminal state
  before replacement.
- Do not repeatedly restart an identical dispatch failure. Stop with run state,
  activity evidence, preserved worktree state, and required next action.

A future automated ownership lease may encode these rules, but a stale static
ownership file must never be treated as stronger evidence than actual process,
workflow, session, branch, and worktree state.

## Delivery ledger and metrics

Every completed slice records:

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:
```

When workflow timestamps are available, also record:

```text
Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```

Treat these as measured values only when the runtime exposes sufficient event
and command timestamps. Otherwise label them estimates; never invent precision.
Review the metrics after several slices to identify repeated full gates,
workflow retries, oversized prompts, flaky tests, or unnecessary review stages.

## Phase closeout

A phase closes only when:

1. each phase-owned outcome maps to direct implementation and test evidence;
2. all required real-app and performance gates pass;
3. architecture and roadmap documents state only implemented facts;
4. deferred work has an explicit canonical owner in a later phase/task;
5. the worktree is clean or every remaining change has explicit ownership;
6. one final independent audit finds no unsupported completion claim.

The audit must distinguish product blockers from external infrastructure and
future-phase work. Do not keep adding local code to compensate for an absent
canonical runner, persistence repository, platform API, or packaging harness.
