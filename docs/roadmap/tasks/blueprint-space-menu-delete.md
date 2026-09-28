# Task packet — Blueprint D05: Space menu and delete confirmation

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.3 §11.13, §19.

Status: adopted 2026-09-28 (DESIGN §19); implementation position 3 of 24. Paper boards: P-4 · Space menu & delete, PR-07 · Spaces — create, style, switch, delete (file
“BBR — Browser Shell”).

## Outcome

Secondary click on a chip opens Edit / New Tab in Space / Move / Delete Space…; delete asks once with counts in a sheet on the card.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.3 §11.13 in the same group and flip the
  D05 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: Spaces have no chip menu and no delete surface.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [ ] Portable model / controller / component tests for the new behaviour
      and its states.

### Real-app E2E

- [ ] `just e2e-shell-spaces` covers the flow drawn on the boards through the
      production path with semantic ids and typed snapshots.

### Performance

- [ ] Add the optimized presentation scenario for the new user-visible interaction, or record why none applies (performance contract).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-spaces
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D05 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

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
```
