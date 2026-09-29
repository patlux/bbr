# Task packet — Blueprint D15: First run and onboarding

Parents: [Phase 9](../phases/09-quality-release.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.8, §19.

Status: implemented 2026-09-30 (DESIGN §19); position 21 of 24. Paper boards: L-1 · First run — welcome, L-2 · First run — import, default browser, first space, PR-01 · First run & onboarding (file
“BBR — Browser Shell”).

## Outcome

A welcome page in the first tab and three skippable steps: import, default browser, first space.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.8 in the same group and flip the
  D15 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: A first launch opens the default empty shell.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] Unit/contract: `import` counts and spans, Chrome bookmarks JSON, `store::history_ops` import merge, `plist_bookmarks` XML plist through Foundation, `onboarding` toast copy, preferences welcomed/external links round trip.

### Real-app E2E

- [x] `just e2e-first-run` (`e2e_first_run.rs`): a fresh launch without an address shows the welcome in the empty space (ids, no empty pane), the import step reads a fixture home (Chrome profile with Local State name, Bookmarks, History; Safari Bookmarks.plist and History.db) with counts, Import lands 3 Saved Pages and 2 history entries with the toast, default browser skipped, typed first-space name, the welcome never returns on relaunch; Get Started opens the palette.

### Performance

- [ ] Extend `just e2e-perf` with the welcome presentation scenario (deferred).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-first-run
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

If Patrick rejects or changes the D15 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                First-run welcome in the empty space, three skippable steps (import from Safari and Chrome, default browser, first space name).
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    Unit/contract: `import` counts and spans, Chrome bookmarks JSON, `store::history_ops` import merge, `plist_bookmarks` XML plist through Foundation, `onboarding` toast copy, preferences welcomed/external links round trip.
Real-app E2E:           `just e2e-first-run` (`e2e_first_run.rs`): a fresh launch without an address shows the welcome in the empty space (ids, no empty pane), the import step reads a fixture home (Chrome profile with Local State name, Bookmarks, History; Safari Bookmarks.plist and History.db) with counts, Import lands 3 Saved Pages and 2 history entries with the toast, default browser skipped, typed first-space name, the welcome never returns on relaunch; Get Started opens the palette.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.8 First run, §19 D15 row.
Open blockers and non-goals: the welcome is the empty space's card, not a first tab (BBR has no web session for internal pages); import progress shows as a toast when done, not in a Downloads-style panel; reading the real Safari library needs Full Disk Access and is checked by hand; performance scenario deferred
Worktree state:         main, clean after commit
```
