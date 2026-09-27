# Task packet — v2 S0: contract and delivery control

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §6, §17.17, §18.

## Outcome

DESIGN.md records the adopted v2 target (§18) with decisions D1–D11, the
`ease:decelerate` correction lands in §6, every slice has a packet reachable
from ROADMAP.md, and the current-delivery document pauses RA-MEDIA and names
this program as the active serial work.

## Scope

- DESIGN.md §6 (D11), §17.17 and §18 (decisions, token/radius/type/motion
  targets, slice status table).
- Program packet plus eleven slice packets (S1–S11) and this one.
- ROADMAP.md, Phase 4, the UI design-system track and
  `docs/delivery/current.md` links and rows.

## Non-goals

- No code, token or test change; the numbered DESIGN sections keep describing
  the running app until each slice lands.

## Parallel-lane contract

Not parallelized.

## Constraints

- One current-delivery pointer; RA-MEDIA keeps its packets and evidence and
  resumes after the program (its portable contract is integrated).
- `just roadmap` stays green.

## Evidence

### Unit and contract

- [x] `just roadmap` green (all v2 packets reachable, required sections).

### Real-app E2E

- [x] Not applicable: documentation-only slice.

### Performance

- [x] Not applicable: no runtime change.

## Completion gates

```bash
just roadmap
just verify
```

## Iteration policy

Write the contract once, run `just roadmap`, fix link or section findings,
commit.

## Blocked stop condition

If the current-delivery graph cannot represent a paused slice, stop with the
checker output rather than deleting RA-MEDIA state.

## Delivery ledger

```text
Outcome:        DESIGN §18 + packets + delivery rows
Commits:        docs(design): adopt the v2 visual refresh contract (2026-09-28)
Functional evidence: just roadmap, just verify
Real-app E2E:   not applicable
Performance evidence: not applicable
Worktree state: main, serial
```
