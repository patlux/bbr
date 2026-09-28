# Task packet — Blueprint D22: Full-screen card rules under test

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §2, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 11 of 24. Paper boards: W-5 · Shell — full screen (file
“BBR — Browser Shell”).

## Outcome

Real-app evidence that full screen is full-bleed (inset 0, radius 0, no shadow) and the titlebar row keeps its controls without traffic lights.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §2 in the same group and flip the
  D22 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: Full-screen rules are specified in DESIGN §2 but have no dedicated evidence.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] SafeArea::full_screen (no traffic-light clearance, ≥ 40 pt row) unit test in bbr-ui; the app's safe area switches to it while the window is in full screen; CardInsets full-bleed rule unchanged.

### Real-app E2E

- [x] `just e2e-shell-titlebar-navigation` — full_screen::full_screen_is_full_bleed_and_the_titlebar_keeps_its_controls (full screen: inset 0, radius 0, no shadow, no chassis band, zero traffic-light clearance, row ≥ 40 pt, all titlebar controls present; windowed again: the clearance and the card shadow return). The existing sidebar content-card scenario keeps covering the native clip radius.

### Performance

- [ ] Performance scenario: none added — evidence-only delta; measurement is deferred (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-titlebar-navigation
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D22 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                Full screen is proven full-bleed and the titlebar keeps its controls without traffic-light clearance.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    SafeArea::full_screen (no traffic-light clearance, ≥ 40 pt row) unit test in bbr-ui; the app's safe area switches to it while the window is in full screen; CardInsets full-bleed rule unchanged.
Real-app E2E:           `just e2e-shell-titlebar-navigation` — full_screen::full_screen_is_full_bleed_and_the_titlebar_keeps_its_controls (full screen: inset 0, radius 0, no shadow, no chassis band, zero traffic-light clearance, row ≥ 40 pt, all titlebar controls present; windowed again: the clearance and the card shadow return). The existing sidebar content-card scenario keeps covering the native clip radius.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §2 full-screen rule, §19 D22 implemented
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
