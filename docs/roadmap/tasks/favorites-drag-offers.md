# Task packet — Favorites grid and drag offers

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §10.9, §11.2, §11.10, §17.16.

Specification input: the design-branch DESIGN.md §10.9/§11.2/§11.10 text at
`design/browser-shell-paper@414c97c7` and the Paper boards “Spec · Rail widths
& fill” and “Spec · Drag in flight”. Patrick adopted drift ledger rows 6, 7
and 8 on 2026-09-27.

## Outcome

Favorites use a fixed four-column grid whose tile width derives from the rail
and whose height is fixed per density. An empty Favorites region shows one
dashed drop band that names the region and the gesture. Once a tab drag passes
the 4 pt activation distance, Favorites offer a drop cell and the organized
area offers a “Pin tab here” ghost row; exactly one live target exists per
region; the lifted row is a drag proxy.

## Scope

- DESIGN.md §10.9/§11.2/§11.10 contract from the design branch.
- `grid_columns() = 4`, derived tile width, `favorites_region_height` as a pure
  function of density and count; drop-band geometry committed once at
  activation.
- Empty band, drop cell, ghost row, one-live-target rule, drag proxy, with
  typed ids `bbr-favorites-empty`, `bbr-favorites-drop-cell`,
  `bbr-pinned-drop-row`, `bbr-tab-drag-proxy`, and the existing indexed
  insertion tick `bbr-drag-tick-<n>` (the design branch's
  `bbr-drag-insertion-tick`; the indexed id is the unchanged §10.9
  contract).
- Real-app E2E for every state and the no-reflow guarantee.

Scope decisions made during implementation (2026-09-27):

- The ghost row is offered only to an unpinned source; a pinned tab is
  never offered pinning. Its growth (one pitch) and the drop cell's extra
  row are committed once at activation through one shared function
  (`activation_growth`); the resolver subtracts that growth from the pointer
  travel, so slot targets stay on the rendered rows.
- The whole populated Favorites region resolves to the drop cell (append).
  Per-tile insert-before targets need an insert-at-index favorite
  capability, which is a non-goal here.
- The former whole-region accent outline was removed: it added a 2 pt
  border when targeted and so reflowed the region mid-drag.

## Non-goals

- No drop-onto-folder semantics beyond the folder membership invariant.
- No new favorite capability, reorder animation, or keyboard drag path change.

## Parallel-lane contract

```text
Base condition:          folder membership invariant integrated
Depends on:              folder-pinned-invariant; address-field-space-identity
Provides/API:            favorites grid geometry, drag offer presentation
Exclusive paths:         bbr-ui favorites/drag components, app drag resolver
Shared integration points: testkit snapshot, e2e_shell_organize, e2e_shell_sidebar, DESIGN.md §10.9/§11.10
Integration order:       after folder-pinned-invariant
```

## Constraints

- A plain click never moves the rail; offers appear only after 4 pt of travel.
- The drag resolver and the render derive geometry from one function.
- Semantic assertions only; no pixel comparison.

## Evidence

### Unit and contract

- [x] Grid geometry for 0–9 favorites at S/N/XL and 200–400 pt rails.
- [x] Offer lifecycle: appear at activation, commit growth once, remove on
      drop/cancel/Escape.

### Real-app E2E

- [x] `just e2e-shell-organize` proves the offers, the one-live-target rule,
      pinning via the ghost row and favoriting via the drop cell and band.
- [x] `just e2e-shell-sidebar` proves grid geometry across widths and the
      empty band at zero favorites.

### Performance

- [ ] Drag-related scenarios keep their budgets; execution follows the active
      performance override.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-organize
just e2e-shell-sidebar
just e2e-shell-dense
just e2e-perf
just e2e
```

## Iteration policy

Geometry function and tests first, then the empty band, then the drag offers,
then the proxy; one real-app suite after each step.

## Blocked stop condition

If committing offer growth once at activation breaks the dense scroll math,
stop with the snapshot evidence rather than letting the rail reflow mid-drag.

## Delivery ledger

```text
Outcome:        four-column grid with rail-derived tiles, zero-favorites
                drop band and glyph slot, drag offers (drop cell, "Pin tab
                here" ghost row, drag proxy) with one live target and growth
                committed once at activation.
Commits:        see `git log --grep "favorites grid"` (feat(ui) commit of
                2026-09-27).
Functional evidence: bbr-ui favorites/affordance/drag_offer unit tests
                (0–9 favorites × S/N/XL × 200–400 pt; offer growth equals the
                region-height difference); bbr drag_state/drag_target tests
                (activation keeps the source slot, ghost-row hit, growth-
                corrected slots).
Real-app E2E:   e2e_shell_drag_offers (new; ghost-row pin, drop-cell favorite,
                no reflow while moving, measured region == pure height,
                micro-drag release is a no-op), u2_favorites_drag (band at
                rest, plain press never moves the region, drop cell replaces
                the rest slot, Escape removes every offer), sidebar
                composition/footer/favorites, organize/dense_edge/drag_chip
                with the activation move. Full `just e2e` minus e2e-perf,
                e2e-peek and e2e-gallery green on 2026-09-27; after review
                remediation organize/sidebar/dense/hover re-run green.
Regression proofs: the removed whole-region outline and the old activation
                resolution were re-inserted temporarily; the new E2E failed
                on both (region reflow; slot 1 instead of the source slot).
Performance evidence: not run — performance execution deferred again by
                Patrick (2026-09-27) until the end of the queue.
Review:         independent read-only review (deepseek-v4-flash via the
                homelab opencode-go route): two minor findings (activation
                frame resolved against pre-growth rects; stale
                favorites_region_height), both remediated and re-verified.
Roadmap/documentation updates: DESIGN §10.9/§11.2/§11.10/§17.16.
Open blockers and non-goals: per-tile insert-before targets need an
                insert-at-index favorite capability (non-goal here).
Worktree state: main, serial, no lane worktree.
```
