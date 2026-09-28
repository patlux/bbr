# Task packet — Blueprint D21: Keyboard entry into the rail

Parents: [Phase 9](../phases/09-quality-release.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §9, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 10 of 24. Paper boards: PR-19 · Keyboard focus — chrome ↔ page, B-2 · Fields, toggles & focus (file
“BBR — Browser Shell”).

## Outcome

⌥⌘S moves keyboard focus to the selected rail row; ↵ selects and returns focus to the page.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §9 in the same group and flip the
  D21 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: The rail is reached through the chrome focus order only.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] FocusRail action bound to alt-cmd-s routes to BbrApp::focus_rail (closes the command portal, shows a hidden rail, focuses the tab list on the selected row); the existing tab-list keyboard (↑/↓, Return → page) is reused.

### Real-app E2E

- [x] `just e2e-shell-tabs` — rail_focus::option_command_s_moves_focus_into_the_rail_and_return_goes_back_to_the_page (page focus → ⌥⌘S chrome focus on the selected row → ↑ selects row 0 → Return returns focus to the page with sessions intact; with the rail hidden ⌥⌘S shows it).

### Performance

- [ ] Performance scenario: none added — a focus move on the existing tab-list path; measurement is deferred (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D21 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                ⌥⌘S moves keyboard focus into the rail on the selected row; Return selects and returns to the page.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    FocusRail action bound to alt-cmd-s routes to BbrApp::focus_rail (closes the command portal, shows a hidden rail, focuses the tab list on the selected row); the existing tab-list keyboard (↑/↓, Return → page) is reused.
Real-app E2E:           `just e2e-shell-tabs` — rail_focus::option_command_s_moves_focus_into_the_rail_and_return_goes_back_to_the_page (page focus → ⌥⌘S chrome focus on the selected row → ↑ selects row 0 → Return returns focus to the page with sessions intact; with the rail hidden ⌥⌘S shows it).
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §9 keyboard entry into the rail, §19 D21 implemented
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
