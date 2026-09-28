# Task packet — Blueprint D08: Toasts and uniform surface transitions

Parents: [Phase 9](../phases/09-quality-release.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §6 §18.4 §18.5, §19.

Status: adopted 2026-09-28 (DESIGN §19); implementation position 7 of 24. Paper boards: B-7 · Feedback — progress, attention, banners, G-7 · Page zoom, V-6 · Motion — transitions, PR-12 · Save a page & Library (file
“BBR — Browser Shell”).

**Blocked 2026-09-29 (implementation stop):** a toast at the bottom centre
of the card overlaps web content, so it must render in the native Portal
(§13). The Portal window captures every click in the browser window while it
is shown, so a 4 s toast would block the page for 4 s. The toast needs a
hit-region Portal mode first: the panel lets clicks through outside the
toast rect, for example by toggling `ignoresMouseEvents` from a local
mouse-moved monitor. That mode also lets the D09 Peek preview take pointer
input through its hole. Real-pointer evidence for it needs an unlocked
screen. D07's reload toast depends on this.

## Outcome

One toast (4 s, Undo, bottom-centre of the card), a transient zoom readout, and the V-6 table as the rule for every appearing surface.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §6 §18.4 §18.5 in the same group and flip the
  D08 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: No toast component; zoom has no readout; sheets fade only where they are Portal surfaces.
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

- [ ] `just e2e-shell-zoom` covers the flow drawn on the boards through the
      production path with semantic ids and typed snapshots.

### Performance

- [ ] Extend `just e2e-perf` with the presentation scenario for this interaction (performance contract).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-zoom
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D08 proposal, or the target conflicts
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
