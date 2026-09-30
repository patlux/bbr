# Task packet — Closed pinned tabs and Favorites that leave the list

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Browser orchestration track](../tracks/browser-orchestration.md) · [interaction contract](../../product/browser-interactions.md) · [DESIGN.md](../../../DESIGN.md) §10.12, §11.2, §11.10, §11.11.

## Outcome

Pinned tabs and Favorites are kept places, not open pages (Patrick,
2026-09-30). Closing a pinned tab leaves its row in place with its icon and
title greyed out; selecting it opens the URL it was pinned at. A tab dragged
into Favorites leaves the tab list: the tile *is* the tab. Closing a Favorite
greys its tile; activating it opens the URL again. The unplanned row pin
control (a hollow ring) is removed.

## Scope

- Remove the row pin control; pinning stays on `Cmd-Shift-P`, the Tab menu
  and the Pinned ghost-row drag offer.
- Domain: `TabLifecycle::Closed` (no native session; only a selection
  reopens it) and `Tab::pinned_page` (URL and title captured when the tab
  becomes pinned — directly, by folder grouping or by favoriting).
  Closing a pinned tab closes it in place: session destroyed, URL/title reset
  to the pinned page, no recently-closed record. Neighbor selection never
  lands on a closed tab. Unpinning a closed tab leaves an unloaded
  (suspended) regular tab.
- Organization: `favorite => pinned` and `favorite => no folder`.
  Favoriting pins a regular tab in the same transition (reported as an
  ordinary pin, persisted like folder grouping) and leaves its folder;
  grouping a Favorite ends the favorite; unpinning a Favorite ends it;
  Remove from Favorites leaves a loose pinned tab.
- Presentation: the loose pinned block, drag slots, keyboard traversal and
  snapshots exclude Favorites like grouped folder members; closed rows,
  closed folder members and closed Favorite tiles render the §10.12 closed
  state; closed rows offer no close control and `Close Tab` is disabled for
  them.
- Typed evidence: `lifecycle == "closed"`, `LifecycleCounts::closed`,
  `FavoriteTileSnapshot::closed`, and the semantic row/tile value `closed`.

## Non-goals

- Persisting the closed state or the pinned page across a relaunch: the
  store has no migrations and a schema bump discards it. After a relaunch a
  closed pinned tab restores as an asleep pinned tab at its pinned URL
  (closing resets the URL), and its pinned page is re-captured from that
  URL. Favorites and folders remain session-local as before (Phase 7).
- `Cmd-Shift-T` restoring the last page of a closed pinned tab (the tab is
  still there; its row reopens the pinned URL).
- A separate "Reset to pinned URL" command, per-tile insert positions,
  dragging a tile back into the list, and the `asleep` visual (§10 names it,
  no producer paints it yet).
- A `closed` family in the Gallery tab-row, folders and favorites matrices
  (the production render is proven by the real-app E2E and a direct GPUI
  capture; the Gallery stories still show open rows and tiles only).

## Parallel-lane contract

Not parallelized: serial on `main` at Patrick's instruction (2026-09-30).

## Constraints

- `TabId`, `SpaceId`, `ProfileId`, profile isolation and the pinned-first
  ordering invariant stay domain-owned; the UI presents `Closed` but never
  decides it.
- A closed tab owns no native session and never counts against the session
  budget; only a selection (pointer, keyboard, Favorite activation, command
  result) recreates it.
- No weakened assertion, invariant, budget or E2E step.

## Evidence

### Unit and contract

- [x] Domain: closing a pinned tab keeps it at its index as `Closed` with the
      pinned page's URL/title and no recently-closed entry; neighbor selection
      skips closed tabs; selection reopens; unpinning a closed tab suspends it;
      invariants reject a closed unpinned or visible tab.
- [x] Organization: favoriting pins and ungroups atomically; grouping a
      Favorite and unpinning it end the favorite; closing keeps the tile;
      invariants reject an unpinned or grouped Favorite.
- [x] Controller: a closed tab has no session and reopens at the pinned URL.

### Real-app E2E

- [x] `just e2e-shell-organize` (`e2e_shell_closed_pins`): a pinned row closes
      in place, greys out without a close control, keeps its session count at
      zero and reopens the pinned URL on a real click; a Favorite leaves the
      list, its tile greys when closed and reopens on activation; Remove from
      Favorites returns a loose pinned row.
- [x] Every affected existing suite stays green with the new semantics.

### Performance

- [ ] Closing a pinned tab and activating a closed row/tile are user-visible
      interactions; their optimized scenarios are deferred with the rest of
      performance execution (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-organize
just e2e-shell-tab-menu
just e2e-shell-sidebar
just e2e-shell-hover
just e2e-gallery
just e2e                # without e2e-perf while performance is deferred
```

## Delivery sequence

1. Remove the row pin control (`3fd97e37`).
2. Closed pinned tabs, then Favorites leaving the list, each with its tests.
3. DESIGN, interaction contract, Paper board, delivery ledger.

## Iteration policy

Start from the domain transition, then the controller, then presentation.
After each failing check identify the mechanism before changing code. Do not
broaden scope into persistence or reopen history.

## Blocked stop condition

Stop with the evidence, attempts, worktree state and the exact decision
needed if a product rule turns out to contradict an existing contract that
this packet does not name.

## Delivery ledger

```text
Outcome:            closed pinned rows, Favorites as their tile only, closed
                    tiles, selected tile edge; row pin control removed
Commits:            3fd97e37 (pin control); the closed-pins/favorites commit
                    on top of e817633
Functional evidence: unit tests of bbr-domain (tests_closed_pins,
                    tests_folders), bbr-browser (controller_closed_pin_tests,
                    organization, tab menu), bbr-ui, bbr-gallery, bbr bins;
                    just verify; testkit clippy
Real-app E2E:       e2e_shell_closed_pins 2/2 (shown key window, real ⌘W,
                    real clicks on row and tile); every `just e2e` suite
                    without e2e-perf, plus e2e-peek and e2e-gallery, green
                    (the mbx cache GC removed target/ once mid-run; the
                    affected suites reran green after mbx restored it).
                    Rewritten to the new contract: Sidebar favorites,
                    favorites drag and folders, drag-offer drop cell, dense
                    edge empty space, Tab-menu favorite
Visual:             direct GPUI capture (visual-capture) light and dark:
                    grey closed row and tile, accent edge on the selected
                    tile; Paper PR-24, PR-05 step 3, B-3 closed row
Performance evidence: deferred (performance execution override)
Roadmap/documentation updates: DESIGN §10.12/§11.2/§11.10/§11.11/§15,
                    browser-interactions, ARCHITECTURE, Phase 4, ROADMAP,
                    Current delivery
Open blockers and non-goals: persistence of the closed state and of
                    Favorites (schema); Gallery closed family
Worktree state:     main, serial, clean after the commit

Planning/read-only: estimate ~1 h (Paper survey, contracts, packet)
Implementation:     estimate ~3 h
Test execution:     measured ~1.5 h of E2E wall time across two runs
Review:             self-review against the packet; no independent
                    reviewer (subagents were not requested)
Human intervention: two decisions (grey icon+title, pinned URL) and the
                    hand-over of main
```
