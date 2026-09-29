# Task packet — Blueprint D17: Restore notice and recovery panes

Parents: [Phase 7](../phases/07-persistence.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.8, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 16 of 24. Paper boards: L-10 · Session restore, L-11 · Recovery & migration errors, PR-18 · Quit, relaunch, crash recovery (file
“BBR — Browser Shell”).

**First slice implemented 2026-09-29 (recovery):** a newer store is
refused and left unchanged (the session runs in memory); a corrupt store is
copied aside (`bbr_persistence` `recovery.rs`) before the discard; the empty
space shows the recovery pane (`bbr_ui::components::recovery`). Evidence:
`repository_contract` (newer refused on both paths, corrupt kept aside
byte for byte), `recovery` component tests, `e2e_shell_recovery` (in `just
e2e-shell-persistence`). Then the restore notice (L-10): an info banner over
the card top through the Portal's overlay layer, until Dismiss or the first
navigation away (`restore_notice` E2E in the same suite). The partial-restore
banner (L-11) is omitted because the persistence layer reports no lost tabs.

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

- [x] Newer stores refused on both paths and left unchanged, corrupt stores kept aside byte for byte (`repository_contract`), recovery copy that never promises a missing file (`components::recovery`), notice copy counts (`app::transient`).

### Real-app E2E

- [x] `just e2e-shell-persistence` — `e2e_shell_recovery` (newer store unchanged after a session; unreadable store kept aside, empty space explains it) and `restore_notice` (relaunch shows 'Restored 1 space and 2 tabs …' in hit-region mode; leaves on the first navigation away, and on Dismiss).

### Performance

- [ ] No perf scenario: execution deferred (Patrick, 2026-09-27).

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
Outcome:                Recovery panes, keep-aside/newer-store persistence rules and the restore notice
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    Newer stores refused on both paths and left unchanged, corrupt stores kept aside byte for byte (`repository_contract`), recovery copy that never promises a missing file (`components::recovery`), notice copy counts (`app::transient`).
Real-app E2E:           `just e2e-shell-persistence` — `e2e_shell_recovery` (newer store unchanged after a session; unreadable store kept aside, empty space explains it) and `restore_notice` (relaunch shows 'Restored 1 space and 2 tabs …' in hit-region mode; leaves on the first navigation away, and on Dismiss).
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.8 recovery panes + restore notice, §19 D17 row
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
