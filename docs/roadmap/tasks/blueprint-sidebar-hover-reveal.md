# Task packet — Blueprint D02: Hover reveal of a hidden rail

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §2 §11.4 §11.14, §19.

Status: adopted 2026-09-28 (DESIGN §19); implementation position 9 of 24. Paper boards: W-3 · Shell — sidebar hidden ⌘B & hover reveal, V-6 · Motion — transitions (file
“BBR — Browser Shell”).

**Blocked 2026-09-29 (implementation stop):** a rail that floats over the
card sits above the native WKWebView, so only the one native Portal may draw
it (§13). The Portal window is a non-activating panel that never becomes key
and captures every click in the window while shown. The rail's address field
needs text input and focus, its rows drag (GPUI drags are window-local), and
its 34 callbacks are bound to the main window. Running them from the Portal
would take one of two things: a second rail host window that can become key,
or a click-only floating rail. Both are product/architecture decisions for
Patrick. The 300 ms dwell detection itself is possible: the collapsed card
keeps a 10 pt GPUI chassis strip at the leading edge.

## Outcome

A 300 ms dwell at the left edge reveals the rail as a floating panel over the card (elev:2), without reflowing the card; leaving or Escape hides it.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §2 §11.4 §11.14 in the same group and flip the
  D02 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: ⌘B hides the rail; only ⌘B or the toggle bring it back.
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

- [ ] `just e2e-shell-sidebar` covers the flow drawn on the boards through the
      production path with semantic ids and typed snapshots.

### Performance

- [ ] Extend `just e2e-perf` with the presentation scenario for this interaction (performance contract).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D02 proposal, or the target conflicts
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
