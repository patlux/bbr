# Task packet — v2 S6: motion core

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §6, §10.5, §10.6, §11.5, §18.5.

## Outcome

`bbr-ui` owns one easing/timing helper built from the §6 tokens; the loading
arc rotates; the attention dot plays the two-beat pulse with ring (opacity-only
under Reduce Motion); the address field shows the D5 progress line while the
active tab loads; and a real-app test proves the shell schedules no animation
frames while nothing transient is active.

## Scope

- `theme` motion helpers: `ease:standard/accelerate/decelerate` evaluation,
  transient-animation descriptors that honour the motion mode.
- Favicon loading arc rotation; attention pulse on entering `attention`.
- Address field progress line (indeterminate sweep; fill-and-fade on finish).
- Testkit evidence: typed transient-animation state (which moments are
  running, reduced or full) and a frame/animation-request counter for the
  idle assertion.
- DESIGN §6, §10.5, §10.6, §11.5; §18.6 row.

## Non-goals

- After-commit moments (S7), empty-state illustrations (S8).

## Parallel-lane contract

```text
Base condition:          S1 and S2 integrated
Depends on:              v2-system-prefs, v2-tokens
Provides/API:            motion helper, transient-animation evidence, idle counter
Exclusive paths:         bbr-ui theme motion, favicon, tab_row attention, address
Shared integration points: testkit snapshot, e2e_shell_states, e2e_shell_sidebar
Integration order:       before S7
```

## Constraints

- Animation never delays a state commit; the committed state is visible on the
  commit frame.
- Loops run only while their state holds; idle requests no frames.
- Reduce Motion removes rotation, scale and travel.

## Evidence

### Unit and contract

- [ ] Easing curves match the tokens at sample points; D11 correction.
- [ ] Transient descriptors under full and reduced motion.

### Real-app E2E

- [ ] Loading: arc animation reported running while loading, stopped after.
- [ ] Attention: pulse reported once, reduced variant under the override.
- [ ] Address progress present while the active tab loads, gone after.
- [ ] Idle: no animation frames requested over a bounded idle window.

### Performance

- [ ] Idle scenario code added (frame requests while idle); execution
      deferred.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-shell-hover
just e2e-shell-errors
just e2e-shell-tabs
just e2e-gallery
```

## Iteration policy

Helper and unit tests first, then one moment at a time with its E2E, then the
idle assertion.

## Blocked stop condition

If GPUI offers no way to observe frame requests, stop with the evidence and
use the app-owned transient registry as the idle proof.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   not run — execution deferred
Worktree state:
```
