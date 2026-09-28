# Task packet — Blueprint D09: Peek presentation

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.17, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 12 of 24. Paper boards: G-8 · Peek preview, PR-04 · Open links — foreground, background, Peek, external, pop-up (file
“BBR — Browser Shell”).

## Outcome

The origin dims under a 9 % wash and the preview sits in an inset frame under its bar (entrance polish).

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.17 in the same group and flip the
  D09 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: The Peek bar spans the card top over the preview.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] `PeekFrame` geometry across S/N/XL density (`card::tests::peek_frame_insets_the_bar_and_the_preview_inside_the_card`), `covered_pane_layout` under an open preview (controller_peek_tests), Gallery story 50 renders the production wash (`just gallery-capture 50`).

### Real-app E2E

- [x] `just e2e-peek` — `peek_sits_inset_under_its_bar_over_a_washed_origin` proves the drawn bar/preview/card rects, the native preview slot equal to the inset frame, the `bbr-peek-wash`/`bbr-peek-frame` ids and the full-card slot after Escape; the existing Peek flows stay green.

### Performance

- [ ] No new interaction: the existing `peek_request_to_present` scenario measures this presentation (its commit frame now carries the wash); execution deferred (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-peek
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D09 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                The origin dims under wash.active and the preview sits in an inset frame under its inset bar
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    `PeekFrame` geometry across S/N/XL density (`card::tests::peek_frame_insets_the_bar_and_the_preview_inside_the_card`), `covered_pane_layout` under an open preview (controller_peek_tests), Gallery story 50 renders the production wash (`just gallery-capture 50`).
Real-app E2E:           `just e2e-peek` — `peek_sits_inset_under_its_bar_over_a_washed_origin` proves the drawn bar/preview/card rects, the native preview slot equal to the inset frame, the `bbr-peek-wash`/`bbr-peek-frame` ids and the full-card slot after Escape; the existing Peek flows stay green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.17 surface/motion/theme/ids, §19 D09 row
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
