# Task packet — Closed pinned tabs and Favorites that leave the list

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Browser orchestration track](../tracks/browser-orchestration.md) · [interaction contract](../../product/browser-interactions.md) · [DESIGN.md](../../../DESIGN.md) §10.12, §11.2, §11.10, §11.11.

## Outcome

Pinned tabs and Favorites are kept places, not open pages (Patrick,
2026-09-30). Closing a pinned tab leaves its row in place with its icon and
title greyed out; selecting it opens the URL it was pinned at. A tab dragged
into Favorites leaves the tab list: the tile *is* the tab. Closing a Favorite
greys its tile; activating it opens the URL again. The unplanned row pin
control (a hollow ring) is removed. Follow-up (Patrick, 2026-09-30, "dann
fix alle"): `⇧⌘T` brings back the page a closed pinned tab showed, and the
closed state, the pinned page, Favorites and folders survive a relaunch.

## Scope

- Remove the row pin control; pinning stays on `Cmd-Shift-P`, the Tab menu
  and the Pinned ghost-row drag offer.
- Domain: `TabLifecycle::Closed` (no native session; only a selection
  reopens it) and `Tab::pinned_page` (URL and title captured when the tab
  becomes pinned — directly, by folder grouping or by favoriting).
  Closing a pinned tab closes it in place: session destroyed, URL/title reset
  to the pinned page, and an in-place recently-closed record keeps the page
  it closed on. `⇧⌘T`/History reopen that page in the same row, in close
  order with regular closes; selecting, unpinning or moving the row drops
  the record. Neighbor selection never lands on a closed tab. Unpinning a
  closed tab leaves an unloaded (suspended) regular tab.
- Persistence (schema v5, no migration): `tabs.pinned_url/pinned_title/
  closed`, `recently_closed.in_place`, and the `folders`, `folder_members`
  and `favorites` tables; the restore intent carries the kept places, a
  closed tab never hydrates a session, and the organization restores with
  its folder ids after the controller. Folder, membership and favorite
  commands persist like any structural mutation.
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
- Gallery: `closed` families in the Tab Row, Favorites and Folders matrices
  (plus the selected tile and member), with the rejected combinations
  "closed regular row" and "selected closed row/tile/member".
- Performance: `pinned_tab_close` (warm 50/100), `closed_tab_reopen` and
  `closed_row_select` (session recreation 100/150).

## Non-goals

- A separate "Reset to pinned URL" command, per-tile insert positions,
  dragging a tile back into the list (delivered later by
  [dragging back out](drag-back-out-of-pins.md)), and the `asleep` visual (§10 names it,
  no producer paints it yet).
- Persisting the folder disclosure (expanded/collapsed) state; it stays
  session-local presentation.

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

- [x] Domain: closing a pinned tab records the page it closed on; `⇧⌘T` and
      History reopen it in the same row in close order; selection, unpin,
      move and space deletion drop or convert the record; restore imports the
      kept place and the record; invariants reject a stray record.
- [x] Persistence v5: kept places and the organization round-trip through
      both writes, invalid ones reject whole, a broken store restores as
      corruption, a selected closed row as a typed warning; v3/v4 stores are
      discarded.
- [x] Hydration/controller: the intent carries pinned page, closed state
      and in-place records; a closed row hydrates nothing; `⇧⌘T` after a
      restore reopens the persisted page; the organization restores with its
      ids and exports reconciled.

### Real-app E2E

- [x] `just e2e-shell-organize` (`e2e_shell_closed_pins`): a pinned row closes
      in place, greys out without a close control, keeps its session count at
      zero and reopens the pinned URL on a real click; a Favorite leaves the
      list, its tile greys when closed and reopens on activation; Remove from
      Favorites returns a loose pinned row.
- [x] `e2e_shell_closed_pins::cmd_shift_t_reopens_a_closed_pinned_row_at_the_page_it_closed_on`:
      a real `⇧⌘T` reopens the closed pinned row at the page it closed on.
- [x] `just e2e-shell-persistence` (`kept_places`, `folder_pins`): after a
      graceful restart the closed row stays closed without a session, the
      navigated pinned tab resets to its persisted pinned page on `⌘W`, `⇧⌘T`
      reopens first that row and then the row closed before the restart at
      their pages, and Favorites and folders (with ids) restore; folders
      also survive a force quit.
- [x] `just e2e-gallery`: the closed and selected families render with
      their ids; closed rows and members draw no close control.
- [x] Every affected existing suite stays green with the new semantics:
      every `just e2e` suite except `e2e-perf` on the host (Organize and
      Real input green on a rerun after the user's activity took the key
      window), and the same suites in a throwaway Tart clone of
      `macos-27-golden-gate` (NAT, `--no-graphics`, host-built binaries,
      3200×1800 display; `e2e_windows` needs the wide display, one
      titlebar reload title race passed on its rerun).

### Performance

- [x] The optimized matrix measures `pinned_tab_close` (60 samples),
      `closed_tab_reopen` and `closed_row_select` (30 each); in the Tart
      clone (report-only, non-canonical) p95/max were 2.36/3.07 ms,
      24.85/25.14 ms and 26.13/26.38 ms against 50/100 and 100/150 ms.
- [ ] `just e2e-perf` green as a whole: blocked by the pre-existing
      renderer-responsiveness block (`b6ffee5`, never executed under the
      performance override; it expects two sessions after the dense block
      left twenty) and, on the loaded host and in the VM, the 750 ms
      startup ceiling. Neither is touched here.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-organize
just e2e-shell-tab-menu
just e2e-shell-sidebar
just e2e-shell-hover
just e2e-shell-persistence
just e2e-gallery
just e2e-perf
just e2e
```

## Delivery sequence

1. Remove the row pin control (`3fd97e37`).
2. Closed pinned tabs, then Favorites leaving the list, each with its tests
   (`4c7c471`).
3. DESIGN, interaction contract, Paper board, delivery ledger.
4. Follow-up: in-place reopen records, schema v5 persistence of kept places
   and the organization, Gallery closed families, performance scenarios,
   independent review.

## Iteration policy

Start from the domain transition, then the controller, then presentation.
After each failing check identify the mechanism before changing code.

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
Performance evidence: deferred (performance execution override); see
                    the follow-up ledger below
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

### Follow-up (2026-10-01, "dann fix alle")

```text
Outcome:            ⇧⌘T/History reopen a closed pinned row at the page it
                    closed on; schema v5 persists closed state, pinned page,
                    reopen records, Favorites and folders; Gallery closed
                    families; three performance scenarios
Functional evidence: unit/contract tests of bbr-domain (tests_closed_pins,
                    tests_folders), bbr-persistence (repository_kept_places,
                    contract fixtures v5), bbr-browser (restore_tests,
                    controller_closed_pin_tests), bbr bins (store mapping);
                    just verify; testkit clippy
Real-app E2E:       e2e_shell_closed_pins 3/3 (new real ⇧⌘T), e2e_shell_
                    persistence kept_places + folder_pins (restart and force
                    quit), every `just e2e` suite except e2e-perf on the host
                    and in a Tart clone; e2e-gallery 48/48
Performance:        scenarios measured in the Tart clone, within budget;
                    whole gate blocked by pre-existing blocks (see Evidence)
Review:             independent read-only review
                    (homelab-llm/opencode-go/deepseek-v4-flash:high): no
                    blocker; remediated: Favorite activation persists, a
                    stale split group or a crash report never reopens a
                    closed row, stale v4 comment; documented: an in-place
                    close counts for the closed-window ⇧⌘T rule. Not changed:
                    moving a closed row opens it (packet rule)
Also fixed:         the perf helper `loose_pinned` now excludes Favorites
                    (the Favorites-leave-the-list rule broke the dense scroll
                    guard; perf had not run since)
Worktree state:     main, serial
```

