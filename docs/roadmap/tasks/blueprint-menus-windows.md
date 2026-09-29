# Task packet — Blueprint D19: Menu bar, Dock menu and window lifecycle

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.13, §19.

Status: adopted 2026-09-28 (DESIGN §19); implementation position 17 of 24. Paper boards: L-13 · Menu bar & Dock, L-14 · Windows — new, close, quit, W-9 · Shell — several windows, PR-21 · Windows — new, move, close (file
“BBR — Browser Shell”).

**First slice implemented 2026-09-29 (menus):** the menu bar (BBR, File,
View, History, Spaces, Tab, Window) and the Dock menu with the spaces
(`crates/bbr-app/src/menus.rs`). Every item dispatches its shortcut's
action; Quit BBR (⌘Q) flushes and exits; Hide ⌘H and Minimize ⌘M are new.
Evidence: `menus` unit tests; `e2e_shell_menus` (in `just e2e-shell-spaces`)
invokes items through the NSMenuItem action dispatch, checks that the menus
follow a new space and the active space, and quits cleanly. Still open in
this packet: ⇧⌘W window close with a reopen toast, New Window, Duplicate
Tab, Select Next/Previous Tab and dragging tabs between windows. BBR has no
window or tab commands for these yet.

## Outcome

Spaces and Tab menus matching the ⌘T actions, a Dock menu with spaces, ⇧⌘T for windows with a reopen toast, and tab moves between windows by drag.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.13 in the same group and flip the
  D19 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: The menu bar carries the basic items; closing a window has no reopen affordance.
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

- [ ] `just e2e-shell-new-window` covers the flow drawn on the boards through the
      production path with semantic ids and typed snapshots.

### Performance

- [ ] Add the optimized presentation scenario for the new user-visible interaction, or record why none applies (performance contract).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-new-window
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D19 proposal, or the target conflicts
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
