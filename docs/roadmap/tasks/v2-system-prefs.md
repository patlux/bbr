# Task packet — v2 S1: system contrast and motion preferences

Parents: [v2 program](v2-visual-refresh-program.md) ·
[Accessibility track](../tracks/accessibility.md) ·
[DESIGN.md](../../../DESIGN.md) §3.4, §6, §15, §18.5.

## Outcome

The shell follows macOS **Increase Contrast** and **Reduce Motion** live:
the theme resolves `contrast = increased` and a new `motion = reduced` mode
from the system, re-renders when either setting changes, and exposes both in
the testkit snapshot (`contrast`, `reduced_motion`). A testkit-only override
drives both modes deterministically in E2E.

## Scope

- Platform read + change observation of the two accessibility display options
  in the macOS adapter (public AppKit `NSWorkspace` API and its
  display-options notification); a portable plain value crosses into the app.
- `Theme` gains a motion mode; the app composes theme contrast and motion from
  the observed preferences (the appearance authority already owns light/dark).
- Snapshot field `reduced_motion` (§15 mode field) and the existing `contrast`
  field report the resolved values; testkit request to override both.
- Real-app E2E proving override → snapshot → rendered theme for both options.

## Non-goals

- No motion consumer yet (S6/S7 consume the mode); no palette value change
  (S2).
- No user-facing preference UI.

## Parallel-lane contract

```text
Base condition:          S0 integrated
Depends on:              v2-contract
Provides/API:            Theme motion mode, observed accessibility preferences
Exclusive paths:         bbr-web-macos accessibility observer, bbr-app appearance
Shared integration points: testkit protocol snapshot/request, bbr-ui theme
Integration order:       before S6
```

## Constraints

- Public AppKit only; portable crates stay free of macOS imports.
- The override is testkit-only and absent from release builds.
- Existing snapshot fields keep their meaning.

## Evidence

### Unit and contract

- [ ] Theme composition for every (appearance, contrast, motion) combination.
- [ ] Testkit protocol round-trips the new request and field.

### Real-app E2E

- [ ] Override Increase Contrast → snapshot `contrast == increased` and the
      rendered rail/palette values switch to the IC set; clearing restores.
- [ ] Override Reduce Motion → snapshot `reduced_motion == true`; clearing
      restores.

### Performance

- [x] Not applicable: no user-visible timing change.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-smoke
just e2e-gallery
```

## Iteration policy

Portable theme field first, then the observer, then snapshot/override and the
E2E; run the affected unit tests after each step.

## Blocked stop condition

If AppKit exposes no public read or change notification for either option,
stop with the API inventory and keep the override-only path.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   not applicable
Worktree state:
```
