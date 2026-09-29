# Task packet — Blueprint D10: History and Clear Browsing Data

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.16, §19.

Status: implemented 2026-09-30 (DESIGN §19); position 13 of 24. Paper boards: M-6 · History, M-7 · Clear browsing data, PR-22 · History & clear data (file
“BBR — Browser Shell”).

## Outcome

⌘Y History in the Library anatomy, per profile, grouped by day, with recently closed; a per-profile Clear Browsing Data sheet.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.16 in the same group and flip the
  D10 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: No history surface and no clear-data control.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] Unit/contract: `app::history_page::{time,model,clear}` (local days/titles/times, day groups, range/search, row limit, closed tabs, clear ranges and options, toast text), `store::history_ops` (wall-clock stamps, remove/clear/count), domain `reopen_closed_tab_restores_that_entry_and_keeps_the_rest`, controller `history_reopens_an_older_closed_tab_by_identity`, registry catalog (actions 17/18), shortcut list bound.

### Real-app E2E

- [x] `just e2e-history` (`e2e_history.rs`, real WKWebView): Today/Earlier groups, search, range, keyboard remove, Recently closed reopen, ⌘T action, History menu, Settings › Privacy, ⌘⇧⌫ sheet with Escape order, Return clears visits and the profile's cookie (fixture echo reads none); `e2e_shell_titlebar_navigation` Show Full History; saved-pages-library, settings and command suites green.

### Performance

- [ ] Extend `just e2e-perf` with the History open and Clear Data presentation scenarios (deferred).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-history
just e2e-shell-saved-pages-library
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D10 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                ⌘Y History as a Library page (global visits by local day, Recently closed), Clear Browsing Data sheet (range, visits, cookies/site data, caches, downloads and Saved Pages with All time), Settings › Privacy entry, History menu and ⌘T actions, Show Full History.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    Unit/contract: `app::history_page::{time,model,clear}` (local days/titles/times, day groups, range/search, row limit, closed tabs, clear ranges and options, toast text), `store::history_ops` (wall-clock stamps, remove/clear/count), domain `reopen_closed_tab_restores_that_entry_and_keeps_the_rest`, controller `history_reopens_an_older_closed_tab_by_identity`, registry catalog (actions 17/18), shortcut list bound.
Real-app E2E:           `just e2e-history` (`e2e_history.rs`, real WKWebView): Today/Earlier groups, search, range, keyboard remove, Recently closed reopen, ⌘T action, History menu, Settings › Privacy, ⌘⇧⌫ sheet with Escape order, Return clears visits and the profile's cookie (fixture echo reads none); `e2e_shell_titlebar_navigation` Show Full History; saved-pages-library, settings and command suites green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.16 History + Clear Browsing Data, §11.14 Show Full History, §11.13 menu bar, §11.18 Privacy, §19 D10 row; answer 5 deviation recorded.
Open blockers and non-goals: visits carry no profile (answer 5's per-profile filter needs a store schema change, which discards the store); the performance scenarios are deferred
Worktree state:         main, clean after commit
```
