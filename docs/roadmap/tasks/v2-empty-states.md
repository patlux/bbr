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

- [x] Illustration frame function is deterministic and ends on the rest frame.

### Real-app E2E

- [x] Empty space, crash and load-error panes report their illustration and
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
Outcome:                D10 illustrations landed. bbr-ui
                        `components/illustration.rs` draws the empty-space
                        pills around a ⌘ key cap, the crash heartbeat and
                        the offline broken link from tokens (quads + one
                        canvas path); frame functions are pure in the
                        cycle phase and end on the rest frame.
                        `Moment::PaneIllustration` = 2 × 1400 ms linear,
                        no repeat, None under Reduce Motion. The app
                        derives the presented kind at render time
                        (`content_illustration`, mirroring the content
                        branch order); a new kind restarts the appearance
                        key; after two cycles the render passes `None`, so
                        the rest frame builds no animation element. The
                        offline class is bound through
                        `IncidentCategory::Offline`
                        (`ErrorSurfaceModel.illustration`); TLS/navigation
                        keep the lifering, so the blocked-stop fallback was
                        not needed.
Commits:                feat(ui): draw the v2 empty-state illustrations
Functional evidence:    bbr-ui 192 unit tests (frames deterministic, rest
                        frame, two cycles, reduced = none); workspace +
                        testkit clippy clean; just verify; testkit bins.
Real-app E2E:           e2e-shell-errors 11/11 incl. new
                        `illustrations::pane_illustrations_play_two_cycles_then_rest`
                        (offline playing → rest → 0 animated elements per
                        forced frame; reduced appearance starts at rest;
                        crashed playing → rest; empty space after closing
                        the last tab playing → rest → 0); e2e-shell-tabs
                        3/3; e2e_shell_states 1/1 (crash recovery; no just
                        recipe, run under the GUI lock); e2e-gallery 46/46.
                        Visual capture (release, visual-capture) of all
                        three rest frames light + dark read.
Performance evidence:   not applicable
Self-review:            the illustration kind is observed in render (the
                        only place that knows which content branch won);
                        the snapshot reads the stored kind, so it is the
                        presented state, not a recomputation. Offline
                        detection uses the incident taxonomy rather than
                        copy. Gallery stories pass `None` (rest frame) so
                        gallery tests stay deterministic. No copy/action
                        change in the panes.
Worktree state:         clean after commit
```
