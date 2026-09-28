# Task packet — Blueprint D01: Footer chips start after Downloads

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.3 §11.4 §18.3, §19.

Status: implemented 2026-09-28 (DESIGN §19); position 1 of 24. Paper boards: S-7 · Footer strip, W-1 · Shell — standard (file
“BBR — Browser Shell”).

## Outcome

Chips start right after Downloads and scroll from there, as in the Paper reference; DESIGN text follows.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.3 §11.4 §18.3 in the same group and flip the
  D01 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: Chips centre in the strip viewport while they fit (DESIGN §11.4, §18.3).
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] Gallery footer-geometry test rewritten to the D01 rule (first chip at the strip's leading edge, free space trailing); bbr-ui lib 201.

### Real-app E2E

- [x] `just e2e-shell-spaces` — new `strip::fitting_chips_start_at_the_strip_leading_edge` (typed strip viewport/offset/chip bounds), all 23 green; `just e2e-gallery` 48/48.

### Performance

- [ ] Performance: no new interaction (layout only); unpassed — deferred.

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

If Patrick rejects or changes the D01 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                The footer chips start right after Downloads whether they fit or overflow; the v2 centring is withdrawn.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    Gallery footer-geometry test rewritten to the D01 rule (first chip at the strip's leading edge, free space trailing); bbr-ui lib 201.
Real-app E2E:           `just e2e-shell-spaces` — new `strip::fitting_chips_start_at_the_strip_leading_edge` (typed strip viewport/offset/chip bounds), all 23 green; `just e2e-gallery` 48/48.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.4 strip text and §18.3 footer line
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
