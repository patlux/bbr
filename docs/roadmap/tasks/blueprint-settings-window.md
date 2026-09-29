# Task packet — Blueprint D16: Settings window

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §3.4 §6 §8 §9, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 15 of 24. Paper boards: L-3 · Settings — General, L-4 · Settings — Profiles & spaces, L-5 · Settings — Search, L-6 · Settings — Privacy & site data, L-7 · Settings — Downloads, L-8 · Settings — Appearance, L-9 · Settings — Shortcuts, PR-23 · Change settings (file
“BBR — Browser Shell”).

## Outcome

⌘, opens a macOS preferences window with General, Profiles, Search, Privacy, Downloads, Appearance and Shortcuts panes; changes apply immediately.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §3.4 §6 §8 §9 in the same group and flip the
  D16 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: No settings window; values live in contracts and defaults.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] preferences (round trip, tolerance, atomic save, Follow), settings (keycaps order, every listed shortcut bound), bbr_ui settings pane keys/ids.

### Real-app E2E

- [x] `just e2e-shell-settings` (⌘, opens General; appearance/contrast/density/paper apply at once; pop-up open/dismiss; search engine; shortcut keycaps; Peek off opens a tab; preferences.txt; relaunch restores).

### Performance

- [ ] Performance scenario for opening Settings: deferred with the other blueprint deltas (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-smoke
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D16 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                Settings window with seven panes; rows only where behaviour exists (DESIGN §11.18); Profiles and Privacy grow with D06/D10/D11.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    preferences (round trip, tolerance, atomic save, Follow), settings (keycaps order, every listed shortcut bound), bbr_ui settings pane keys/ids.
Real-app E2E:           just e2e-shell-settings (⌘, opens General; appearance/contrast/density/paper apply at once; pop-up open/dismiss; search engine; shortcut keycaps; Peek off opens a tab; preferences.txt; relaunch restores).
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §8 §11.18 §19 D16; ARCHITECTURE app preferences; justfile e2e-shell-settings
Open blockers and non-goals: omitted board rows (DESIGN §11.18), the visual comparison with the boards (no screen capture here)
Worktree state:         main, clean after commit
```
