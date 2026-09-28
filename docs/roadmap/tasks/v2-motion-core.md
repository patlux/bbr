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

- [x] Easing curves match the tokens at sample points; D11 correction
      (`theme/motion.rs`).
- [x] Transient descriptors under full and reduced motion, sample shapes,
      spinner geometry shared with the static glyph.

### Real-app E2E

- [x] Loading: arc animation reported running while loading, stopped after
      (`e2e_shell_sidebar/motion.rs`, slow fixture).
- [ ] Attention: pulse reported once, reduced variant under the override —
      **not expressible**: the app has no attention producer (the §10.6
      state is contract-only; `row_state` never yields it). The pulse is
      covered by the component (`components/attention.rs`), the moment
      descriptor tests and the gallery's Tab Row states; the typed
      `attention-pulse` moment evidence is wired and attaches when a
      producer lands.
- [x] Address progress present while the active tab loads, fill-and-fade on
      finish, gone after; static line and no finish under Reduce Motion.
- [x] Idle: no animation frames requested over a bounded idle window —
      render-side proof (a forced frame renders but builds zero animation
      elements); the display link does not tick for hidden/locked test
      windows, so a frame-tick count is not observable there.

### Performance

- [x] Idle scenario code added (`perf_common/idle.rs`: zero animation
      elements across forced frames after the interaction matrix);
      execution deferred.

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
Outcome:               `theme::motion` (easing + moment plans + samples +
                        built-element counter); rotating loading arc; one-shot
                        attention pulse; D5 address progress (sweep, finish
                        timer); typed `motion` snapshot (moments, root renders,
                        animation elements).
Commits:               see git log (feat(ui): add the v2 motion core …)
Functional evidence:   just verify ok; testkit clippy ok; bbr-ui 188 tests
Real-app E2E:          shell-sidebar 19/19 (new motion scenario),
                        shell-hover 4/4, shell-errors 10/10, shell-tabs 3/3,
                        shell-favicons 3/3, gallery 46/46
Performance evidence:   not run — execution deferred (idle budget code added)
Deviations:            attention E2E not expressible (no producer); idle
                        proof is render-side because the screen was locked
                        (IOConsoleLocked since 01:33) and hidden windows get no
                        display-link frames — the counter showed 3 renders in
                        30 s, all forced flushes; the command overlay's
                        existing open fade is not routed through the helper.
Review:                serial self-review: moments never delay commits (state
                        is applied before the moment is chosen), loops live
                        only while their state holds, one-shots rest.
Worktree state:        clean after commit
```
