# Task packet — Drag pinned tabs and Favorites back out

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Browser orchestration track](../tracks/browser-orchestration.md) · [interaction contract](../../product/browser-interactions.md) · [DESIGN.md](../../../DESIGN.md) §10.9, §11.2, §11.10. Follows [closed pinned tabs and Favorites](closed-pins-and-favorites.md), whose non-goal "dragging a tile back into the list" this packet delivers.

## Outcome

Patrick (2026-10-01): "es sollte möglich sein Favoriten oder angepinnte Tabs
auch wieder per Drag & Drop zurück zu schieben." A pinned row dragged below
the New Tab row is unpinned and lands at the regular slot under the pointer.
A Favorite tile is draggable: dropped among the regular tabs it is unpinned
there (which ends the favorite); dropped on a loose pinned row or on the
"Pin tab here" ghost row it stays pinned and leaves Favorites as a loose
row; dropped on a folder row it is grouped (which ends the favorite); a
space chip moves it like any tab.

## Scope

- Domain: `Browser::unpin_tab_to(space, tab, position)` unpins and places
  the tab inside the regular region in one transition (validated before any
  mutation; `DomainError::NotPinned`). Controller: `unpin_tab_to(index,
  position)` reports `TabUnpinned`.
- Drag resolution: a new `DragTarget::Unpin { slot }`. A pinned row over the
  regular region (below the New Tab row, recorded by its rect anchor)
  resolves the regular slot from the rendered row rects. A Favorite tile's
  drag (`DragOrigin::Favorite`) resolves every target at the pointer's
  absolute position: its own grid is no target, then ghost row, folders,
  regular region, loose pinned rows, chips.
- Pointer path: a tile press activates the tile (unchanged) and arms a drag
  of its tab; the 4 pt activation, the proxy, Escape and the drop are the
  row drag's. A tile drag offers the ghost row but no drop cell, and never
  previews a split or another window.
- Presentation: the tick renders in the region its target belongs to
  (`TabsModel::tick_regular`), including the regular region's end; the
  snapshot reports `drag_unpin_slot` and `drag_from_favorite`; the testkit
  seam `BeginFavoriteDrag` presses a tile through the production handler.
- DESIGN §10.9/§11.2/§11.10 and the interaction contract.

## Non-goals

- Dragging a regular row into the pinned rows at a position (the ghost row
  stays the pointer pin path), dragging folder members out of their folder,
  reordering tiles inside the grid, splitting with or moving a Favorite to
  another window by drag.
- A `dragging` placeholder look for the pressed tile (the proxy carries the
  drag).

## Parallel-lane contract

Not parallelized: serial on `main`.

## Constraints

- Every drop goes through one existing or new single-transition authority;
  `TabId`, the live session and the selection are untouched; a closed
  pinned row becomes an unloaded regular tab (the unpin rule).
- Drag targets are hit through recorded rects, never a parallel layout
  model; nothing reflows while the pointer moves.
- No weakened assertion, invariant, budget or E2E step. The former
  "pinned → regular crossing is rejected" E2E step encoded the old
  contract and is replaced by the unpin drop plus a still-rejected
  regular → pinned crossing.

## Evidence

### Unit and contract

- [x] Domain `tests_unpin_to`: every position, empty regular region, closed
      row → suspended with its reopen record dropped, rejection of an
      unpinned tab and a position past the end without mutation.
- [x] Controller `unpinning_to_a_position_keeps_the_session_and_lands_at_the_drop`.
- [x] Resolver `drag_target_back_tests` (pinned row → `Unpin` slots, New Tab
      row and missing anchor stay pinned-region targets, regular rows never
      unpin; Favorite: own grid none, loose slots, folder, ghost row,
      regular, chip, outside the rail) and the drag-state Favorite origin.

### Real-app E2E

- [x] `e2e_shell_organize`: a pinned row dropped on the first regular row
      unpins there with its session; a regular row dragged over the pinned
      rows is still no drop.
- [x] `e2e_shell_drag_back` (in `just e2e-shell-organize`): a plain tile
      press only activates; a tile drag offers proxy and ghost row but no
      drop cell; its own tile is no target; dropped before a regular row it
      unpins there (favorite ends, session kept); dropped on a loose pinned
      row or the ghost row it becomes a loose pinned row; Escape cancels.
- [x] `e2e_real_input::favorites`: a real AppKit drag of a tile to the end
      of the tab list.

### Performance

- [ ] A drag-unpin scenario in the optimized matrix: deferred with the
      performance execution override (Patrick, 2026-09-27); the existing
      drag scenarios keep their budgets.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-organize
just e2e-shell-sidebar
just e2e-shell-hover
just e2e-shell-dense
just e2e-windows
just e2e-split
just e2e-real-input
just e2e-gallery
just e2e-perf          # deferred (performance override)
```

## Iteration policy

Domain transition first, then the resolver with unit tests, then the
pointer path and presentation, then the real-app suites.

## Blocked stop condition

Stop with the evidence and the exact decision needed if a drop target
would need a second authority for pin, favorite or folder membership.

## Delivery ledger

```text
Outcome:            pinned rows and Favorite tiles drag back out of the
                    pinned area (unpin at the drop, or a loose pinned row)
Functional evidence: bbr-domain tests_unpin_to; bbr-browser
                    controller_closed_pin_tests; bbr drag_target/drag_state
                    tests; testkit clippy
Real-app E2E:       e2e-shell-organize (8 binaries incl. new
                    e2e_shell_drag_back), e2e-shell-sidebar 21/21,
                    e2e-shell-hover, e2e-shell-dense, e2e-windows 9/9,
                    e2e-split 3/3, e2e-real-input 7/7, e2e-gallery 48/48 on
                    the host (Cargo through mbx)
Performance:        not run (override); scenario open above
Review:             self-review against this packet (no subagents requested)
Worktree state:     main, serial
```
