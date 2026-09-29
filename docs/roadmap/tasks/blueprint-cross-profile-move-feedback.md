# Task packet — Blueprint D07: Cross-profile move feedback

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.3 §12 §10.9, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 8 of 24. Paper boards: P-7 · Move a tab across profiles, PR-08 · Profiles — create, assign, move a tab (file
“BBR — Browser Shell”).

**Unblocked 2026-09-29:** the D08 toast and the hit-region Portal landed.

## Outcome

The drag names the destination profile before the drop; after it a toast says the page reloaded in that profile, with Undo.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.3 §12 §10.9 in the same group and flip the
  D07 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: A cross-profile move recreates the page silently.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] `BrowserController::move_tab_id` moves a tab by identity from a space that is not active and recreates it in the source store (`controller_move_tests`); the toast's profile lead and the drag label are production `bbr-ui` components.

### Real-app E2E

- [x] `just e2e-shell-organize` (drag-chip suite) — `cross_profile_drop_names_the_destination_and_undo_moves_the_tab_back`: over the Work chip the label reads 'Move to Clients · Profile Work'; the drop recreates the page and the toast names Work with Undo; Undo moves the tab back (recreated again) and removes the toast; the existing cross-profile drag stays green.

### Performance

- [ ] No perf scenario: execution deferred (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-organize
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D07 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                The drag names the destination profile before a cross-profile drop; a toast with Undo follows the drop
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    `BrowserController::move_tab_id` moves a tab by identity from a space that is not active and recreates it in the source store (`controller_move_tests`); the toast's profile lead and the drag label are production `bbr-ui` components.
Real-app E2E:           `just e2e-shell-organize` (drag-chip suite) — `cross_profile_drop_names_the_destination_and_undo_moves_the_tab_back`: over the Work chip the label reads 'Move to Clients · Profile Work'; the drop recreates the page and the toast names Work with Undo; Undo moves the tab back (recreated again) and removes the toast; the existing cross-profile drag stays green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §12 cross-profile move feedback bullet, §19 D07 row
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
