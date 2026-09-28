# Task packet — Blueprint D06: Create, edit and delete profiles

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §12, §19.

Status: adopted 2026-09-28 (DESIGN §19); implementation position 14 of 24. Paper boards: P-6 · Profiles — create & edit, L-4 · Settings — Profiles & spaces, PR-08 · Profiles — create, assign, move a tab (file
“BBR — Browser Shell”).

## Outcome

A New Profile sheet (name, hue, first space) and Settings › Profiles with edit and a counted delete of the data store.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §12 in the same group and flip the
  D06 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: Profiles exist in the domain; there is no UI to create, rename or delete them.
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

- [ ] `just e2e-profiles` covers the flow drawn on the boards through the
      production path with semantic ids and typed snapshots.

### Performance

- [ ] Add the optimized presentation scenario for the new user-visible interaction, or record why none applies (performance contract).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-profiles
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D06 proposal, or the target conflicts
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
