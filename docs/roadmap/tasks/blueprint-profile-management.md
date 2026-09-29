# Task packet — Blueprint D06: Create, edit and delete profiles

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §12, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 14 of 24. Paper boards: P-6 · Profiles — create & edit, L-4 · Settings — Profiles & spaces, PR-08 · Profiles — create, assign, move a tab (file
“BBR — Browser Shell”).

## Outcome

A New Profile sheet (name, hue, first space) and Settings › Profiles with edit and a counted delete of the data store.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §12 in the same group and flip the
  D06 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: Profiles exist in the domain; there is no UI to create, rename or delete them.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] bbr-domain tests_profile_lifecycle (delete with spaces/tabs, last profile kept, rename/recolour), bbr-browser controller_profile_tests (sessions destroyed, fallback activated), bbr-command registry catalogue (New Profile…, Settings…).

### Real-app E2E

- [x] `just e2e-profiles` (e2e_profile_management: ⌘T › New Profile… opens the sheet; typed name mirrors the first space; swatch; Return creates and activates the empty space; Edit renames/recolours; Delete names what goes and returns to the remaining profile; no Delete for the last profile).

### Performance

- [ ] Performance scenario for the profile sheet: deferred with the other blueprint deltas (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-profiles
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D06 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                Profiles are created, edited and deleted from Settings › Profiles (sheet also from the command bar); the deleted profile's WebKit store is removed.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    bbr-domain tests_profile_lifecycle (delete with spaces/tabs, last profile kept, rename/recolour), bbr-browser controller_profile_tests (sessions destroyed, fallback activated), bbr-command registry catalogue (New Profile…, Settings…).
Real-app E2E:           just e2e-profiles (e2e_profile_management: ⌘T › New Profile… opens the sheet; typed name mirrors the first space; swatch; Return creates and activates the empty space; Edit renames/recolours; Delete names what goes and returns to the remaining profile; no Delete for the last profile).
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.18 §12 §19 D06; justfile e2e-profiles
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
