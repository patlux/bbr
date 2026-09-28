"""Generate one task packet per blueprint delta (W7)."""
import sys
sys.path.insert(0, "/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint")
from deltas import DELTAS, EXISTING
from inventory import BOARDS

REPO = "/Users/patwoz/dev/Personal/macos/bbr"
PHASE_TITLE = {"04-ui-shell": "Phase 4", "05-command-bar": "Phase 5", "06-split-view": "Phase 6", "07-persistence": "Phase 7",
               "08-daily-driver": "Phase 8", "09-quality-release": "Phase 9"}


def board_list(codes):
    return ", ".join(f"{c} · {BOARDS[c][1]}" for c in codes.split())


for did, slug, title, boards, today, target, phase, refs, suite, perf in DELTAS:
    if slug is None:
        continue
    text = f"""# Task packet — Blueprint {did}: {title}

Parents: [{PHASE_TITLE[phase]}](../phases/{phase}.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) {refs}, §19.

Status: proposed. Blueprint delta {did}; it starts only after Patrick adopts
the {did} row in DESIGN §19. Paper boards: {board_list(boards)} (file
“BBR — Browser Shell”).

## Outcome

{target}

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md {refs} in the same group and flip the
  {did} row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: {today}
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [ ] Portable model / controller / component tests for the new behaviour
      and its states.

### Real-app E2E

- [ ] `just {suite}` covers the flow drawn on the boards through the
      production path with semantic ids and typed snapshots.

### Performance

- [ ] {"Extend `just e2e-perf` with the presentation scenario for this interaction (performance contract)." if perf else "Add the optimized presentation scenario for the new user-visible interaction, or record why none applies (performance contract)."}

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just {suite}
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the {did} proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:
```
"""
    open(f"{REPO}/docs/roadmap/tasks/{slug}.md", "w").write(text)
    print("wrote", slug)
