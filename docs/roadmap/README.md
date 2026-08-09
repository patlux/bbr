# Roadmap maintenance

This directory turns [ROADMAP.md](../../ROADMAP.md) into a drill-down planning system.

## Hierarchy and execution pointer

```text
ROADMAP.md
  ├── Current delivery → ../delivery/current.md
  └── phases/<phase>.md
        └── tracks/<workstream>.md
              └── tasks/<implementation-packet>.md
```

- **Roadmap:** the sole stable repository entry point, product-level sequence,
  phase roll-ups, and exactly one committed `Current delivery` pointer.
- **Current delivery:** the sole owner of volatile execution state — verified
  base SHA, active/frozen lanes, worktrees/writers, next gates, blockers, and
  serial integration order. Historical ledgers remain evidence, never a second
  current authority.
- **Phase:** independently valuable product outcome and phase acceptance.
- **Track:** one durable concern spanning one or more phases.
- **Task packet:** bounded, long-lived scope and acceptance. It names dependency
  conditions and exclusive/shared surfaces, but not volatile run ids, current
  owners, temporary worktrees, or "today's" base SHA.

Task packets follow [the reusable template](templates/task-packet.md), the
[development workflow](../development-workflow.md), and—when multiple worktrees
are active—the [parallel-delivery contract](../parallel-delivery.md). Existing packets may retain
their current shape, but every newly activated or substantially rewritten packet
must state outcome, scope, non-goals, constraints, evidence, completion gates,
iteration policy, and blocked stop condition.

## Canonical ownership

A concrete checkbox must have exactly one canonical owner, normally a track or task file. Higher-level checkboxes are roll-ups and are checked only when every required linked acceptance item is complete.

Do not copy the same detailed checklist into multiple files. Link to it.

## Checkbox rules

- `[x]` means implementation exists and verification evidence was run.
- `[ ]` means not yet proven, even if code is partially present.
- Add `(deferred)` only when explicitly moved to a later linked phase.
- Do not check an item merely because a lower-level API exists; check the user-visible or contract outcome stated by the item.
- When completing an item, update its canonical file in the same commit as the implementation whenever practical.
- A browser feature remains `[ ]` until automated real-app E2E coverage passes. Unit, fake-engine, and contract tests are additional evidence, not substitutes.

## Status and authority

- Phase/track/task checkboxes are acceptance evidence, not an executable queue.
- A mixed checkbox is split into directly proven and still-open outcomes rather
  than left stale or mass-checked from commit history.
- Executable statuses use the current-delivery vocabulary (`planned`, `ready`,
  `running`, `review`, `remediation`, `frozen`, `blocked`, `integrating`,
  `integrated`, `closed`). Dependencies and integration priority are separate
  fields.
- Runtime changes update `docs/delivery/current.md`; durable scope or acceptance
  changes update the canonical task/track/phase owner. Do not copy the same
  state into both.

## Evidence

Prefer short evidence references next to completed groups:

```text
Evidence: `just verify`, `just e2e-smoke`, commit `abc1234`.
```

Avoid volatile timings unless performance is the acceptance criterion. Implemented-fact details belong in `ARCHITECTURE.md` or `E2E_ARCHITECTURE.md`, not in every roadmap file.

## Adding work

1. Place the item under the nearest existing track.
2. Create a task packet from [the template](templates/task-packet.md) when implementation needs sequencing, explicit non-goals, or several commits.
3. Bound the packet to one independently valuable vertical slice; split phase-wide work before implementation.
4. For a parallel-capable packet, record the durable base **condition**,
   dependencies, provided API, exclusive paths, shared integration points,
   evidence, and merge order. Record the exact activation SHA, runtime owner,
   branch, and worktree only in Current delivery.
5. Add a new track only for a durable concern.
6. Add a new phase only for a distinct user-visible or architectural milestone.
7. Link upward so every task is discoverable from `ROADMAP.md`.

## Regular review cadence

At the start or end of a meaningful implementation group:

1. Read `ROADMAP.md`, Current delivery, and the owning phase/task packet.
2. Confirm one writer exclusively owns the target branch/worktree in Current
   delivery.
3. Update completed canonical checkboxes.
4. Update the one Current-delivery row with status, exact base, owner, worktree,
   next gate, blockers, and integration order; never create a second Current
   focus list in a phase/track/task file.
5. Confirm phase acceptance still matches the product.
6. Run unit/contract tests plus automated real-app E2E and required optimized
   performance evidence for every affected browser feature before marking work
   complete.
7. Record detailed evidence/timing in the preserved delivery ledger or evidence
   report while keeping concise runtime state in Current delivery.

`just roadmap` validates the complete tracked first-party Markdown control
graph. It checks Git-backed regular-file modes; repository-durable local links
and heading fragments; exactly one current-delivery pointer and marker;
section-bound control metadata, operator rows, fixed statuses, unique queue
IDs, final base/rollback SHA shapes; reachability of every task packet from
`ROADMAP.md`; and exactly-once current task-packet headings outside fenced code
for new or materially edited packets. Exact grandfathered legacy packets may
remain until edited; any byte change requires the current sections. The normal
gate rejects unresolved `PENDING_*` cutover metadata. Index and HEAD audits are
available through `scripts/check_roadmap.py --source index|head`; the
preparation-only `--allow-pending-current` switch must never be used by `just
roadmap` or `just verify`.
