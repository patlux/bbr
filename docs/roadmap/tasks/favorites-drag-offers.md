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
  `bbr-pinned-drop-row`, `bbr-tab-drag-proxy`, `bbr-drag-insertion-tick`.
- Real-app E2E for every state and the no-reflow guarantee.

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

- [ ] Grid geometry for 0–9 favorites at S/N/XL and 200–400 pt rails.
- [ ] Offer lifecycle: appear at activation, commit growth once, remove on
      drop/cancel/Escape.

### Real-app E2E

- [ ] `just e2e-shell-organize` proves the offers, the one-live-target rule,
      pinning via the ghost row and favoriting via the drop cell and band.
- [ ] `just e2e-shell-sidebar` proves grid geometry across widths and the
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
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:
```
