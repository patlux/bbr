# Task packet — sidebar slide on toggle

Parents: [Phase 4 — Vertical tabs, spaces, and first intentional UI](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN §6 1a, §11.4, §13, §18.5](../../../DESIGN.md) ·
[performance contract](../../product/performance-contract.md)

Requested by Patrick on 2026-10-17 with the Athas Browser toggle as the
reference (<https://x.com/athasdev/status/2111148141533962556>: the rail
slides past the leading edge while the content area moves and grows with it,
about 240 ms, ease-out). This supersedes the earlier "collapse instantly, no
slides" rule for this one transition only.

## Outcome

Showing or hiding the sidebar (⌘S, the titlebar toggle, View › Toggle
Sidebar, the command) slides the rail in or out while the content card and
its native web view follow the rail edge frame by frame. The toggle still
commits first: state, persistence, focus, semantic ids and the titlebar glyph
change on the commit frame, within the existing 50/100 ms budget. Reduce
Motion and docking the floating rail present the committed geometry at once.

## Scope

- `bbr-ui`: the `Transition::SidebarSlide` timing descriptor (`dur:emphasis`,
  `ease:standard`, `None` under Reduce Motion) and the pure
  `sidebar::transition` model (presence, mid-slide reversal, whole-point
  rail edge and blended `CardInsets`) with unit tests.
- `bbr-app`: `app::sidebar_motion` starts the slide after the commit, resolves
  each frame's presentation, requests frames while it travels, guarantees a
  settled frame by timer, lays the full-width rail into a sliding strip, veils
  a leaving rail against the pointer, blocks the edge reveal during the exit
  and lets `float_rail` end a running exit.
- Testkit evidence: `Snapshot.sidebar_motion` (variant, presence, presented
  edge, toggles, settle ms, bounded per-frame trace with the native slot x)
  and `MotionSnapshot.transition_frames` for the idle proof.
- Real-app E2E: the new `e2e_shell_sidebar::slide` scenario; every existing
  suite that toggles the sidebar and then asserts geometry waits for the
  settled slide (`e2e_common::wait_sidebar_settled`).
- Performance: the toggle sample keeps its 50/100 ms class; a new
  `sidebar_settle` distribution (p95 ≤ 300 ms, max ≤ 400 ms) with a seeded
  contract budget and baseline entry.
- DESIGN §6/§11.4/§13/§18.5, the performance contract and ARCHITECTURE §9f.

## Non-goals

- Animating sidebar resize, window restore, full-screen changes, split
  changes or tab/space selection (they keep snapping).
- A horizontal tab strip or the Athas top-strip cross-fade.
- Changing the floating rail's own slide (DESIGN §11.4, blueprint D02).
- Gallery motion stories or visual baselines for intermediate frames: at
  rest the chrome paints exactly as before.

## Parallel-lane contract

```text
Base condition:          the integrated `main` with desktop glass (`feat(ui): add native desktop material behind browser chrome`)
Depends on:              nothing beyond the base
Provides/API:            bbr_ui::theme::motion::Transition, bbr_ui::sidebar::{SidebarTransition, SidebarPresentation, presence_for}; Snapshot.sidebar_motion; MotionSnapshot.transition_frames
Exclusive paths:         crates/bbr-ui/src/sidebar/transition.rs, crates/bbr-app/src/app/sidebar_motion.rs, crates/bbr-app/tests/e2e_shell_sidebar/slide.rs
Shared integration points: crates/bbr-app/src/app/render.rs, app.rs (toggle_sidebar), rail_float.rs, content_card.rs, testkit snapshot (protocol), perf baseline/budget registration, e2e_common, DESIGN.md, performance-contract.md, ARCHITECTURE.md, Phase 4 roll-up
Integration order:       after the active desktop-glass correction settles on `main`; the protocol fields are additive (`serde(default)`)
```

## Constraints

- Commit first: nothing in the toggle path waits for the presentation; every
  command, the semantic tree and persistence read `sidebar_visible`.
- GPUI owns chrome, the native view owns the page: the WKWebView is re-framed
  per frame, never captured, scaled or composited into GPUI.
- An idle shell requests no animation frames; a slide requests frames only
  while it travels.
- Reduce Motion presents the committed geometry on the first frame.
- No weakened tests or budgets: geometry assertions now wait for the settled
  slide; the toggle budget is unchanged.

## Evidence

### Unit and contract

- [x] `bbr-ui` motion and transition tests: plan/variant, full slide timing,
      reversal from the presented share, blended card and rail edge on whole
      points, full-screen card.
- [x] `perf_common::baseline` seeds and enforces `sidebar_settle`.

### Real-app E2E

- [x] `e2e_shell_sidebar::slide`: commit first; monotonic slide through
      intermediate rail edges and intermediate native WKWebView frames;
      settle after the full slide on the exact collapsed geometry; no reveal
      from a leaving rail; mid-slide reversal without a jump; idle proof;
      Reduce Motion snaps. *(Tart VM: the trace moved the rail edge
      300 → 268 → 232 → … → 0 pt with the native slot one frame behind and
      settled after 263 ms.)*
- [x] Existing suites that toggle the sidebar stay green with the slide
      (`e2e-shell-sidebar`, `e2e-shell-titlebar-navigation`,
      `e2e-shell-downloads`, `e2e-shell-tabs`, `e2e-shell-hover` incl.
      semantic branches, `e2e-shell-responsiveness`, `e2e-windows`). The
      Tart golden image runs with macOS Reduce Motion on, so the suites that
      toggle and then assert geometry force full motion and assert the
      animated variant.

### Visual (steady state)

- [x] `just visual-check` on the slice and on its base commit: all 162
      captured browser PNGs are byte-identical, so the slice changes no
      chrome pixel at rest (batches
      `reports/visual/batches/1792199813187650000-92577` and
      `reports/visual/batches/1792200198684690000-26111`). Both batches
      report the same 88 changed / 74 same cases against the accepted
      baselines; those differences come from the desktop-glass base and
      await its own acceptance. The slide itself has no official stills.

### Performance

- [ ] `just e2e-vm e2e-perf`: the sidebar block measures `sidebar_toggle`
      p50 0.0 / p95 0.1 / max 0.4 ms and `sidebar_settle` p50 260.5 /
      p95 261.1 / max 261.3 ms over 30 samples (VM, report-only). The
      recipe as a whole stays red in the deferred renderer-responsiveness
      scenario (`responsiveness.rs`: exact session/unresponsive source
      assumptions that the full matrix no longer meets; known since
      2026-09-27, independent of the sidebar), so it is not claimed green.
- [ ] Canonical host evidence for both distributions (needs Patrick's go).

## Completion gates

```bash
just verify            # run with mbx-backed cargo
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-vm e2e-shell-sidebar e2e-shell-titlebar-navigation e2e-shell-downloads e2e-shell-tabs e2e-shell-semantic-branches e2e-shell-responsiveness e2e-windows
just e2e-vm e2e-perf
just e2e-vm e2e
```

At rest the chrome paints exactly as before, so `just visual-check` proves
only the unchanged steady state; it is not evidence for the slide itself.

## Delivery sequence

1. One exclusive writer implements and verifies the slice in its lane.
2. One independent read-only reviewer maps requirements to evidence.
3. Remediation only for concrete findings, then a focused recheck.

## Iteration policy

Keep the transition one pure model plus one app module. After a failing
check, identify whether the commit, the presentation or the harness timing
broke before changing code. Do not slow, shorten or disable the slide to get
green tests; wait for the settled state instead.

## Blocked stop condition

If the native slot cannot be re-framed per frame without visible tearing,
or the VM route cannot run, stop with the evidence, the preserved lane
branch, the exact blocker and the decision needed (for example: snap the web
view and slide only chrome).

## Delivery ledger

```text
Outcome:            implemented on the lane branch
Commits:            the e2e runner's offline dependency import, then this slice
Functional evidence: just verify (via mbx), clippy -D warnings (testkit), bbr-ui transition/motion and perf baseline unit tests
Real-app E2E:       Tart VM: the affected suites with full motion green; the full `just e2e` sequence on the slice commit green except `e2e-perf` (57 of 58 executables; settle p95 261.8 ms)
Visual evidence:    162 of 162 browser captures byte-identical to the base commit
Performance evidence: VM report-only sidebar numbers above; the e2e-perf recipe red in the deferred responsiveness scenario
Roadmap/documentation updates: DESIGN §6 1a/§11.4/§13/§18.5, performance contract, ARCHITECTURE §9f, Phase 4
Open blockers and non-goals: canonical host performance evidence needs Patrick's go; integration into `main` waits for the active desktop-glass writer
Worktree state:     lane worktree `parallel/sidebar-slide`, committed

Planning/read-only: estimate 30 min (reference video analysis, code reading)
Implementation:     estimate 75 min
Test execution:     Tart VM runs 02:11–02:58 (measured run stamps)
Review:             not run (no independent reviewer requested)
Remediation:        reversal evidence, f32 settle residue, ⌘L field anchor, glass/motion override
Waiting/retries:    two guest prepare failures (offline guest), fixed by the runner commit
Total elapsed:      about 2 h (estimate)
Human intervention: none
```
