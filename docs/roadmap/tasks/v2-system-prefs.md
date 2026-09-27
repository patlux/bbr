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

- [x] Theme composition: `theme_modes_for` maps both options, the override
      wins per option (`appearance.rs` tests); `Theme::with_motion_mode`
      leaves palette/density/contrast untouched (`theme.rs` test).
- [x] Testkit protocol: `SetAccessibilityOverride` and `ShellModes.reduced_motion`
      (serde default for older payloads) compile through the gateway and
      the snapshot.

### Real-app E2E

- [x] Override Increase Contrast → snapshot `contrast == increased` and the
      rendered chrome tint is the IC presentation of the active space;
      clearing restores (`e2e_shell_sidebar::accessibility`).
- [x] Override Reduce Motion → snapshot `reduced_motion == true`,
      independent of contrast; identity, sessions and geometry unchanged.

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
Outcome:        Theme.motion_mode + live Increase Contrast / Reduce Motion
                through the public NSWorkspace API and its change
                notification; snapshot `reduced_motion`; testkit override.
Commits:        feat(app): follow macOS Increase Contrast and Reduce Motion
Functional evidence: just verify (workspace fmt/check/test/clippy/roadmap/
                ui-deps/script tests); bbr testkit clippy -D warnings.
Real-app E2E:   e2e-shell-sidebar 17/17 (new accessibility scenario),
                e2e-smoke 1/1, e2e-gallery 45/45.
Performance evidence:   not applicable
Review:         documented self-review (no subagents authorized): public
                AppKit only; observer removed on drop; no polling; release
                builds carry no override (cfg testkit); snapshot reports
                effective values; the native install path runs in every E2E
                launch. The live-notification path itself is not E2E-driven
                (the host setting cannot be toggled headless); the override
                exercises the same theme composition.
Worktree state: main, serial
```
