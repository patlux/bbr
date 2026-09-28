# Task packet — Blueprint D05: Space menu and delete confirmation

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.3 §11.13, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 3 of 24. Paper boards: P-4 · Space menu & delete, PR-07 · Spaces — create, style, switch, delete (file
“BBR — Browser Shell”).

## Outcome

Secondary click on a chip opens Edit / New Tab in Space / Move / Delete Space…; delete asks once with counts in a sheet on the card.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.3 §11.13 in the same group and flip the
  D05 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: Spaces have no chip menu and no delete surface.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] Browser::move_space/delete_space (+ can_*) with the same-profile group rule, last-space rejection and the recently-closed transfer with real former positions (bbr-domain tests_space_lifecycle); BrowserController::move_space/delete_space (sessions destroyed, appearance record dropped, fallback activation) and OrganizationController::delete_space/space_delete_summary (folders and favorite tiles go) in bbr-browser organization_space_tests; space-menu rows/enablement and focus hand-off (bbr-app tab_menu::space); sheet title/body copy and default Cancel focus (bbr-ui surfaces::space_delete); gallery story 49.

### Real-app E2E

- [x] `just e2e-shell-spaces` — space_menu::space_menu_moves_edits_and_opens_new_tabs_without_activating_on_open and space_menu::delete_space_asks_once_and_closes_its_tabs_into_the_fallback (portal sheet over the card, Return on Cancel, Tab to Delete, Escape, confirm → SpaceDeleted, sessions −2, fallback active, Reopen Closed Tab, last space undeletable); `just e2e-shell-persistence` — space_delete::a_deleted_space_stays_deleted_and_its_tabs_stay_reopenable_after_restart; tab-menu, sidebar, portal, context-menu and gallery suites green.

### Performance

- [ ] Performance scenario: none added — menu open reuses the measured tab-menu path and the sheet is a portal presentation like the auth sheet; measurement is deferred with the rest of the performance evidence (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-spaces
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D05 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                Space chips open a space menu (Edit, New Tab in Space, Move Left/Right, Delete Space…); deleting asks once in a portal sheet over the card and closes the space's tabs into the fallback.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    Browser::move_space/delete_space (+ can_*) with the same-profile group rule, last-space rejection and the recently-closed transfer with real former positions (bbr-domain tests_space_lifecycle); BrowserController::move_space/delete_space (sessions destroyed, appearance record dropped, fallback activation) and OrganizationController::delete_space/space_delete_summary (folders and favorite tiles go) in bbr-browser organization_space_tests; space-menu rows/enablement and focus hand-off (bbr-app tab_menu::space); sheet title/body copy and default Cancel focus (bbr-ui surfaces::space_delete); gallery story 49.
Real-app E2E:           `just e2e-shell-spaces` — space_menu::space_menu_moves_edits_and_opens_new_tabs_without_activating_on_open and space_menu::delete_space_asks_once_and_closes_its_tabs_into_the_fallback (portal sheet over the card, Return on Cancel, Tab to Delete, Escape, confirm → SpaceDeleted, sessions −2, fallback active, Reopen Closed Tab, last space undeletable); `just e2e-shell-persistence` — space_delete::a_deleted_space_stays_deleted_and_its_tabs_stay_reopenable_after_restart; tab-menu, sidebar, portal, context-menu and gallery suites green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.3 space menu + delete confirmation + ids, §11.13 source identity/family ids, §19 D05 implemented
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
