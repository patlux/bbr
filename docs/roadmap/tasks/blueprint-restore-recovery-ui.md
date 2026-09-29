# Task packet — Blueprint D17: Restore notice and recovery panes

Parents: [Phase 7](../phases/07-persistence.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.8, §19.

Status: adopted 2026-09-28 (DESIGN §19); implementation position 16 of 24. Paper boards: L-10 · Session restore, L-11 · Recovery & migration errors, PR-18 · Quit, relaunch, crash recovery (file
“BBR — Browser Shell”).

**First slice implemented 2026-09-29 (recovery):** a newer store is
refused and left unchanged (the session runs in memory); a corrupt store is
copied aside (`bbr_persistence` `recovery.rs`) before the discard; the empty
space shows the recovery pane (`bbr_ui::components::recovery`). Evidence:
`repository_contract` (newer refused on both paths, corrupt kept aside
byte for byte), `recovery` component tests, `e2e_shell_recovery` (in `just
e2e-shell-persistence`). Still open: the restore notice banner ("Restored 3
spaces and 42 tabs …", L-10) and the partial-restore banner (L-11). Both
float over the card top through the Portal's overlay layer, like the D08
toast.

## Outcome

A one-time restore notice, a partial-restore banner, and panes for an unreadable session and a newer data version that never touch profile data.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.8 in the same group and flip the
  D17 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: Restore is silent; unreadable or newer stores fall back without explanation.
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

- [ ] `just e2e-shell-persistence` covers the flow drawn on the boards through the
      production path with semantic ids and typed snapshots.

### Performance

- [ ] Add the optimized presentation scenario for the new user-visible interaction, or record why none applies (performance contract).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-persistence
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D17 proposal, or the target conflicts
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
