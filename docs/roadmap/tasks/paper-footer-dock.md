# Task packet — Paper S-7b: the footer dock in every state

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §9, §11.3, §11.4, §17.11 ·
[Paper fidelity: Downloads panel and rail footer](paper-sidebar-downloads-fidelity.md).

Specification input: Patrick's 2026-10-11 request — the rail footer still did
not look right; design it separately in Paper with every state, rework the
implementation, and test it with visual snapshots. Design: Paper file
"BBR — Browser Shell", page "21 — Sidebar", board "S-7b · Footer — all states
(proposal)" (17 states in Light and Dark, the three densities, six notes).

## Outcome

The footer is one quiet dock: Downloads, the space chips and Plus share one
square and radius, sit 8 pt apart (chips 4 pt), and draw at the titlebar
icon size; hover, press, open and focus states exist for every target; the
in-flight count is a badge that never moves the row; a drag target keeps
its size and shows the destination profile as a tag.

## Scope

- `bbr-ui` footer metrics per density, control states, count badge, chip
  hover/press/focus ring and destination tag; one hover target type.
- App hover state through the production `on_hover` listeners, open states
  from the panel and the menu, the strip focus; testkit
  `HoverFooterControl`/`UnhoverFooterControl` and the `footer` snapshot.
- DESIGN §11.3/§11.4/§11.12/§18 text; real-app E2E; eight new browser
  visual cases in Light and Dark.
- Findings of the capture-versus-Paper check (2026-10-12): the identity
  ink follows the chip state (`ink` active, `accent.ink` drag target,
  `ink.secondary` at rest), and the straight Paper glyphs (Downloads,
  Trash, Plus/Minus, Back/Forward) get Paper's round caps and joins. GPUI
  only reaches lyon's butt/miter stroke defaults from `bbr-ui`'s allowed
  dependencies (`gpui`, `bbr-domain`), so those strokes are filled as a
  non-zero union of round-capped capsules; Plus uses Paper's 5.25/18 arm.
- Paper board corrections to match DESIGN: the drag tag is the §12
  monogram (`WO`, not `WORK`), the active chip carries 8 % of the space
  hue (`#F0F2FD` / `#303954`), note 4 states both.

## Non-goals

- Keyboard traversal into Plus (§9 order): drawn in Paper, reachable once
  the Phase 9 keyboard traversal lands.
- A fade at the clipped strip edges: it would have to match the grain and
  the floating rail; the strip clips as DESIGN §11.4 states.
- Reference acceptance: changed and new references need Patrick's review.
- Round caps for the curved §7 glyphs (identity glyphs, Reload, marks):
  they need their Paper paths first; they keep lyon's butt caps.

## Parallel-lane contract

Not parallelized (single serial writer on `main`, see Current delivery).

## Constraints

- The footer stays ordinary sidebar chrome; the hover state is presentation
  only and never changes product state.
- Changed test expectations state the new Paper values at the same
  strictness (exact geometry, explicit ids).

## Evidence

### Unit and contract

- [x] `bbr-ui`: band metrics per density, glyph sizes, target wire names,
      round-capped capsule geometry, Paper plus/minus paths, chip identity
      ink (238 lib tests); `bbr` bins 379; `bbr-gallery` 153; `bbr-testkit` 35;
      strict Clippy for `bbr` (testkit, visual-capture), `bbr-ui`,
      `bbr-gallery`, `bbr-testkit`.
- [x] Visual inventory contract red (84 cases, missing footer states) then
      green (100 cases); serial visual-test suites.

### Real-app E2E

- [x] `e2e-shell-sidebar`: footer geometry (8 pt lead, 28 pt squares, 10 pt
      insets) and the new dock-state test (hover never moves a target,
      open states, strip focus, unknown target rejects).
- [x] `e2e-shell-spaces`: 4 pt chip gap, overflow scroll-into-view.
- [x] `e2e-shell-downloads`: the count badge id and snapshot while a
      transfer runs, gone at terminal.
- [x] `e2e-shell-organize` (drag to chip), `e2e-shell-hover`, `e2e-gallery`.
      Tart run `/tmp/bbr-e2e-vm/run-20261011-224431-60449`: 19 of 19
      executables, 131 tests, clone and key removed.
- [x] Browser visual capture A/B in Tart (`footer5-20261012`): 100 PNGs,
      A and B byte-identical, cleanup rechecked for both clones, no
      baseline writes, working tree unchanged during the runs. Review: 62
      changed, 10 same, 28 without a reference (the same changed set as
      `paper3`). Side by side with Paper S-7b: positions (Downloads 24,
      first chip 46–74, Plus W−24, 12 pt below the squares), glyph boxes
      (Downloads 26×25 px, Plus 24 px), badge position and colour (Paper
      exports Display P3), tag and rings match.

### Performance

- Presentation-only change; hover state updates notify only on change.
  Performance execution stays deferred (Current delivery).

## Completion gates

```bash
mbx test -p bbr-ui -p bbr-gallery -p bbr-testkit
mbx test -p bbr --features testkit --bins
mbx clippy -p bbr --features testkit --all-targets -- -D warnings
just visual-test
just e2e-vm e2e-shell-sidebar e2e-shell-spaces e2e-shell-downloads e2e-shell-organize e2e-shell-hover e2e-gallery
browser visual capture A/B (Tart)
just roadmap
```

## Iteration policy

Follow the Paper board's computed values; after each failing check find the
mechanism before changing code; keep DESIGN-backed decisions unless Patrick
changes them.

## Blocked stop condition

If no defensible path remains, stop with the evidence gathered, the
approaches attempted, the preserved worktree state, the exact blocker and
the smallest decision or input required next.

## Delivery ledger

```text
Outcome:                footer dock matches Paper S-7b in all captured
                        states (Light/Dark); references await review.
Commits:                none yet (uncommitted, see Current delivery).
Functional evidence:    see Unit and contract.
Real-app E2E:           19/19 executables (run-20261011-224431-60449).
Visual evidence:        reports/visual/downloads-footer5-20261012-20261009-{a,b};
                        failed attempts kept: footer2 (prepare probe
                        deadline), footer3-a (probe deadline), footer4-b
                        (SSH banner timeout; rerun stopped by the source
                        gate after an edit), all cleanups verified.
Performance evidence:   not run (deferred).
Roadmap/documentation updates: DESIGN, this packet, Phase 4.
Open blockers and non-goals: Plus keyboard reachability (Phase 9); reference
                        review by Patrick.
```
