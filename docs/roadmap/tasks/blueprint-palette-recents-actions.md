# Task packet — Blueprint D04: ⌘T recents and the > actions mode

Parents: [Phase 5](../phases/05-command-bar.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.4, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 6 of 24. Paper boards: C-1 · ⌘T — empty with recents, C-4 · ⌘T — actions, L-9 · Settings — Shortcuts (file
“BBR — Browser Shell”).

## Outcome

Empty ⌘T lists recent tabs, history and Reopen Closed Tab; typing > switches to ACTIONS with the shortcut in the kind column.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.4 in the same group and flip the
  D04 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: The empty palette waits for input; actions are result rows without a mode.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] bbr-command recents (recent tabs by last_used excluding the current tab, deduplicated recent history, Reopen Closed Tab only when available) and the > prefix → Mode::Action; rank test updated for the new empty ⌘T contract; bbr-ui kind hint shows an action's shortcut and the > prefix is not the match needle; BrowserController::can_reopen_tab.

### Real-app E2E

- [x] `just e2e-shell-command` — recents::empty_new_tab_offers_recents_and_the_angle_bracket_switches_to_actions (empty ⌘T lists the previous tab first and never the one on screen, ⌘T ↵ switches back, Reopen Closed Tab appears last after a close with ⇧⌘T, > lists only actions with the ACTIONS chip, '> reopen' + Return reopens, an unmatched > query is Empty and Return opens nothing); saved-pages and tabs suites green.

### Performance

- [ ] Performance scenario: none added — recents rank through the measured debounce/rank/render path; measurement is deferred (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-command
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D04 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                Empty ⌘T offers recents and Reopen Closed Tab; a leading > switches the palette to actions with shortcuts.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    bbr-command recents (recent tabs by last_used excluding the current tab, deduplicated recent history, Reopen Closed Tab only when available) and the > prefix → Mode::Action; rank test updated for the new empty ⌘T contract; bbr-ui kind hint shows an action's shortcut and the > prefix is not the match needle; BrowserController::can_reopen_tab.
Real-app E2E:           `just e2e-shell-command` — recents::empty_new_tab_offers_recents_and_the_angle_bracket_switches_to_actions (empty ⌘T lists the previous tab first and never the one on screen, ⌘T ↵ switches back, Reopen Closed Tab appears last after a close with ⇧⌘T, > lists only actions with the ACTIONS chip, '> reopen' + Return reopens, an unmatched > query is Empty and Return opens nothing); saved-pages and tabs suites green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.4 recents + actions mode + kind column, §19 D04 implemented
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
