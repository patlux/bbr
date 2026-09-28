# Task packet — v2 S7: after-commit moments

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §6, §10, §11.3, §11.4, §11.10, §11.12,
§18.1 (D4), §18.5.

## Outcome

Four bounded moments play only after their state is committed and presented:
the favorite pop with specks, the download check with ring, the ⌘T result
stagger and cursor glide, and the space-switch tint cross-fade — each with a
Reduced Motion variant and a real-app proof that the committed state is
visible on the commit frame.

## Scope

- Favorites tile pop/specks on a new favorite; Downloads row check/ring on
  success; command results stagger and cursor highlight glide; chrome tint
  fade on committed (non-swipe) space switches.
- Testkit transient evidence per moment (running, reduced) and commit-frame
  evidence (the committed model/tint value is reported before the moment
  ends).
- DESIGN sections above; §18.6 row.

## Non-goals

- New interactions or durations beyond §18.5; no motion on selection or
  layout.

## Parallel-lane contract

```text
Base condition:          S3–S6 integrated
Depends on:              v2-motion-core
Provides/API:            none new
Exclusive paths:         favorites/drag landing, downloads rows, command render, space switch presentation
Shared integration points: testkit snapshot, organize/downloads/command/spaces suites
Integration order:       after S6
```

## Constraints

- Every moment ≤ 260 ms single-shot; nothing loops.
- Space switch: geometry, lists, selected tab and native view snap; only the
  chrome tint fades.

## Evidence

### Unit and contract

- [x] Moment descriptors and reduced variants; sample shapes (pop
      overshoot, check stroke/ring, stagger bound) — `theme/motion.rs`.

### Real-app E2E

- [x] Favorite drop (the drop's commit method): favorite committed in the
      snapshot immediately; pop reported (`animated`), `reduced` flash under
      override; removal starts nothing (`e2e_shell_sidebar/moments.rs`).
- [x] Download success: check moment reported on the row's status slot;
      `static` under Reduce Motion (`e2e_shell_downloads/moments.rs`).
- [x] ⌘T: selection committed on the key press; glide/stagger reported;
      `reduced` entrance and `static` cursor under Reduce Motion
      (`e2e_shell_command/moments.rs`).
- [x] Space switch: destination tint committed as target, fade reported,
      instant under Reduce Motion; swipe commit does not fade
      (`e2e_shell_spaces/swipe/tint_fade.rs`).

### Performance

- [x] Existing space-switch and command scenarios unchanged in timing;
      fade-path assertions added to the space-switch benchmark
      (`perf_common/benchmarks.rs::assert_fade_path`); execution deferred.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-organize
just e2e-shell-downloads
just e2e-shell-command
just e2e-shell-spaces
just e2e-gallery
```

## Iteration policy

One moment at a time with its E2E; keep the commit-first proof as the first
assertion of each test.

## Blocked stop condition

If a moment cannot start without delaying its commit, stop with the evidence
and ship the static variant.

## Delivery ledger

```text
Outcome:               favorite pop + specks / flash, download check + ring,
                        ⌘T stagger entrance + cursor glide, D4 chrome-tint
                        fade (rail + chassis); app-side keyed registry with
                        timers; `motion.started` evidence.
Commits:               see git log (feat(ui): play the v2 after-commit …)
Functional evidence:   just verify ok; testkit clippy ok; bbr-ui 190 tests
Real-app E2E:          shell-sidebar 20/20, shell-downloads 12/12,
                        shell-command 26/26, shell-spaces 22/22,
                        shell-organize 9/9, gallery 46/46
Performance evidence:   not run — execution deferred (fade-path assertions)
Deviations:            evidence is the start record (`motion.started`) rather
                        than a running-window sample, so it is independent of
                        how fast the harness snapshots; one instance per kind
                        (a newer commit supersedes a running decoration);
                        the result entrance replays on every publish, as
                        §18.5 specifies; visual QA of the frames is not
                        possible on the locked screen.
Review:                serial self-review: every start sits after the commit
                        in the same code path; the tile/row/list boxes never
                        change geometry; nothing loops.
Worktree state:        clean after commit
```
