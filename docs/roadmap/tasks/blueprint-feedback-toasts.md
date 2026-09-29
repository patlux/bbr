# Task packet — Blueprint D08: Toasts and uniform surface transitions

Parents: [Phase 9](../phases/09-quality-release.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §6 §18.4 §18.5, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 7 of 24. Paper boards: B-7 · Feedback — progress, attention, banners, G-7 · Page zoom, V-6 · Motion — transitions, PR-12 · Save a page & Library (file
“BBR — Browser Shell”).

**Unblocked 2026-09-29 (Patrick: "lets go"):** the Portal gained a
hit-region mode. A local pointer monitor toggles the panel's
`ignoresMouseEvents`, so while no presentation is active it captures the
pointer only over the overlays (`bbr_web_macos::portal_hit`).

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

- [x] Hit-region routing (`portal_hit::portal_captures`), toast/readout geometry (`surfaces::toast` tests), overlay hit rects incl. the leaving toast (`app::transient` tests), Gallery story 51 renders the production toast (three tones) and readout; `Icon::Minus` joins the §7 catalog.

### Real-app E2E

- [x] `just e2e-shell-zoom` — `zoom_readout_and_toast_confirm_without_blocking_the_page`: ⌘+ readout at the card top centre (120×32, 24 pt), its ids and 1.2 s exit; ⌘D toast bottom-centre (40 pt, 24 pt), Undo removes the saved page, 4 s exit; the Portal captures the pointer only over the overlay and lets it through to the page elsewhere; auth (new `bbr-auth-veil`), spaces, context/tab menus, titlebar, sidebar, organize, saved pages, peek, portal (7/7) and gallery suites stay green.

### Performance

- [ ] No perf scenario yet: execution deferred (Patrick, 2026-09-27); the toast/readout presentations need a `toast_present` scenario when perf work resumes.

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
Outcome:                One toast (4 s, Undo, bottom-centre), the zoom readout, veils under blocking sheets and V-6 fades on sheets and menus, over a hit-region Portal that never blocks the page
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    Hit-region routing (`portal_hit::portal_captures`), toast/readout geometry (`surfaces::toast` tests), overlay hit rects incl. the leaving toast (`app::transient` tests), Gallery story 51 renders the production toast (three tones) and readout; `Icon::Minus` joins the §7 catalog.
Real-app E2E:           `just e2e-shell-zoom` — `zoom_readout_and_toast_confirm_without_blocking_the_page`: ⌘+ readout at the card top centre (120×32, 24 pt), its ids and 1.2 s exit; ⌘D toast bottom-centre (40 pt, 24 pt), Undo removes the saved page, 4 s exit; the Portal captures the pointer only over the overlay and lets it through to the page elsewhere; auth (new `bbr-auth-veil`), spaces, context/tab menus, titlebar, sidebar, organize, saved pages, peek, portal (7/7) and gallery suites stay green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §18.4 toast/readout/overlay/veil bullets, §18.5 V-6 rows, §19 D08 row
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
