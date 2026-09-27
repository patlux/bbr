# Task packet — v2 S8: drawn empty-state illustrations

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §11.8, §18.1 (D10), §18.5.

## Outcome

The empty-space, crashed and load-error (offline) glass panes show
GPUI-drawn illustrations — floating tab pills around a ⌘ key, a heartbeat
line, packets stopping at a broken link — that play two cycles after the pane
appears and rest on their final frame; Reduce Motion shows the final frame.

## Scope

- `bbr-ui` pane illustrations drawn with paths/quads from tokens.
- Offline variant bound to the existing load-error classification for
  network failures.
- Testkit evidence for the illustration kind and its cycle state.
- DESIGN §11.8; §18.6 row.

## Non-goals

- No copy or action change in the panes; no animation after the two cycles.

## Parallel-lane contract

```text
Base condition:          S6 integrated
Depends on:              v2-motion-core
Provides/API:            pane illustrations
Exclusive paths:         bbr-ui components/pane.rs, surfaces/error.rs
Shared integration points: testkit snapshot, e2e_shell_errors, e2e_shell_states, gallery panes
Integration order:       parallel-capable with S10
```

## Constraints

- Panes never show web content or a screenshot of it (§13).
- Illustrations stop requesting frames after two cycles.

## Evidence

### Unit and contract

- [ ] Illustration frame function is deterministic and ends on the rest frame.

### Real-app E2E

- [ ] Empty space, crash and load-error panes report their illustration and
      reach `rest`; reduced variant starts at `rest`.

### Performance

- [x] Not applicable: panes replace content; no budgeted interaction.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-errors
just e2e-shell-tabs
just e2e-gallery
```

## Iteration policy

Static rest frames first, then the two-cycle motion, then E2E.

## Blocked stop condition

If the load-error classification cannot distinguish network failure, ship the
offline illustration for the generic load-error pane and record the gap.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   not applicable
Worktree state:
```
