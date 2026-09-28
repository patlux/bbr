# Task packet — v2 S11: gallery, documentation and closeout

Parents: [v2 program](v2-visual-refresh-program.md) ·
[Component Gallery compositions](component-gallery-compositions.md) ·
[DESIGN.md](../../../DESIGN.md) §18.6.

## Outcome

The Gallery presents the v2 tokens and motion moments from the production
components, ARCHITECTURE.md and E2E_ARCHITECTURE.md record the motion-mode
and transient-animation evidence facts, DESIGN §18.6 shows every slice
landed, and the canonical aggregate passes from the integrated commit.

## Scope

- Gallery foundation stories: v2 colour/elevation and motion tokens.
- Documentation facts; program ledger; current-delivery closeout; resume
  RA-MEDIA as next.
- Final canonical run.

## Non-goals

- Performance execution (deferred by Patrick; recorded as unpassed).

## Parallel-lane contract

Not parallelized.

## Constraints

- Gallery uses production components only; no second renderer.

## Evidence

### Unit and contract

- [x] Gallery story registry tests.

### Real-app E2E

- [ ] `just e2e` aggregate without `e2e-perf`, plus `e2e-peek` and
      `e2e-gallery`, green from the integrated commit. *(31 of 32 suites
      green from `92263be8`; `e2e-shell-portal` is 6/7 — the
      `hardening::host_hides_on_minimize_and_restores` AppKit hit-test
      cannot converge while the macOS session is locked. Rerun
      `just e2e-shell-portal` in an unlocked session.)*

### Performance

- [ ] Unpassed: execution deferred.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-gallery
just e2e-peek
just e2e              # minus e2e-perf while deferred
just roadmap
```

## Iteration policy

Stories, then documents, then the canonical run; fix regressions at their
owning slice's mechanism.

## Blocked stop condition

If the canonical run fails for a reason outside v2, stop with the failing
suite, its evidence and the owning packet.

## Delivery ledger

```text
Outcome:                Gallery Motion page reads durations, easing and one
                        row per Moment::ALL (standard plan | Reduce Motion
                        variant) from bbr-ui and plays the D10
                        illustrations live; the Color page adds the v2 veils
                        over the rail and every token's three intensity
                        steps plus the static grain from the production
                        resolver/grain layer. Existing pages were extended
                        (story ids stay append-only). ARCHITECTURE §9e/§9f
                        and E2E_ARCHITECTURE §7q record schema v4, the
                        motion authority/after-commit rule and the
                        render-side motion evidence. DESIGN §18.6 lists
                        every slice landed.
Commits:                92263be8 feat(gallery): present the v2 tokens and
                        motion moments; this closeout docs commit.
Functional evidence:    just verify; testkit clippy; bbr-gallery 150 unit
                        tests (moment table covers Moment::ALL from the one
                        authority).
Real-app E2E:           canonical aggregate from 92263be8 (screen locked
                        since 01:33; hidden windows + FlushFrame):
                        smoke 1, shutdown 3, tabs 1, profiles 1, shell-tabs
                        3, shell-spaces 22, shell-sidebar 20, favicons 3,
                        titlebar-navigation 5, split-geometry 1, organize 9,
                        hover 4, command 26, saved-pages 4,
                        saved-pages-library 12, new-window 4,
                        background-links 4, persistence 7, find 6, dense 4,
                        downloads 12, errors 11, responsiveness 7,
                        external-url 5, zoom 5, context-menu 11, tab-menu
                        12, web-navigation-gestures 10, auth 5, peek 7,
                        gallery 46 — all green.
                        shell-portal: 6/7. hardening::
                        host_hides_on_minimize_and_restores fails twice at
                        wait_for_frontmost_evidence: the frontmost window at
                        the probe point is 26632 = loginwindow (layer 2004,
                        full-screen lock shield) and the window below the
                        portal is Finder, so the restored parent can be
                        neither frontmost nor directly below. v2 changed no
                        portal/window code (only the S1
                        accessibility_display observer in bbr-web-macos);
                        the suite was green in the unlocked 2026-09-27 run.
Performance evidence:   unpassed — execution deferred
Blocked:                owning packet native-portal-hardening; unblocker:
                        an unlocked macOS session, then
                        `BBR_E2E_TIMEOUT=10m just e2e-shell-portal`.
Worktree state:         clean after commit
```
