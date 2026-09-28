# Task packet — Blueprint D14: Back / Forward history menu

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.14 §11.13, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 4 of 24. Paper boards: W-7 · Shell — titlebar navigation states, M-3 · Page menu & web menus (file
“BBR — Browser Shell”).

## Outcome

Long-press or secondary click on Back/Forward lists the session history in the menu anatomy.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.14 §11.13 in the same group and flip the
  D14 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: Back and Forward act one step; no history list.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] bbr-web SessionHistory/HistoryEntry + WebAction::GoToHistory contract; WKBackForwardList read/jump in bbr-web-macos session_history; BrowserController::session_history/go_to_history over the fake (controller::history tests); history rows, bound and offsets (app nav_history tests).

### Real-app E2E

- [x] `just e2e-shell-titlebar-navigation` — history::back_and_forward_list_the_session_history_and_jump_in_one_step over a real three-entry WKWebView history (right-click Back lists [two, one], row 2 jumps two steps, Forward lists [two, three], Escape closes, a 500 ms hold opens without stepping, a short press/release steps once); sidebar, context-menu, portal, navigation-gesture and gallery suites green.

### Performance

- [ ] Performance scenario: none added — the menu reuses the measured page-menu portal path; measurement is deferred with the rest of the performance evidence (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-titlebar-navigation
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D14 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                Back/Forward list the session history on secondary click or a 500 ms hold; a row jumps straight to its entry.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    bbr-web SessionHistory/HistoryEntry + WebAction::GoToHistory contract; WKBackForwardList read/jump in bbr-web-macos session_history; BrowserController::session_history/go_to_history over the fake (controller::history tests); history rows, bound and offsets (app nav_history tests).
Real-app E2E:           `just e2e-shell-titlebar-navigation` — history::back_and_forward_list_the_session_history_and_jump_in_one_step over a real three-entry WKWebView history (right-click Back lists [two, one], row 2 jumps two steps, Forward lists [two, three], Escape closes, a 500 ms hold opens without stepping, a short press/release steps once); sidebar, context-menu, portal, navigation-gesture and gallery suites green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.14 history menu + ids, §11.13 ownership/family ids, §19 D14 implemented
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
