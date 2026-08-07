# Task packet — native Portal host hardening

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[Phase 9](../phases/09-quality-release.md) ·
[native portal host](native-portal-host.md) ·
[native-integration track](../tracks/native-integration.md) ·
[Testing track](../tracks/testing.md).

Status: **post-MVP follow-up after the base Portal host and U1 migration.**

## Outcome

The one-per-browser-window native Portal host has an explicit, tested contract
for focus/key routing, multiple BBR windows, display/screen changes, compact
geometry, presentation stacking, teardown, and packaged-app behavior. It stays
an implementation detail of its parent browser window and cannot become an
orphan, unexpected key/main window, or cross-window presentation owner.

## Scope

- Define focus/key routing for Escape, Cmd-W, command shortcuts, text input,
  pointer activation, and focus restoration for nonactivating and explicitly
  focus-requesting presentations.
- Define one-host-per-browser-window identity and isolation across multiple BBR
  windows; presentations never attach to or inspect another browser window.
- Harden movement/resizing across displays, scale changes, Spaces/fullscreen,
  minimization, occlusion, parent close, app hide/show, activation, and compact
  screen clamping.
- Define deterministic Portal stack priority, replacement, dismissal, and
  teardown for palette, find bar, and BBR-owned menus.
- Extend typed snapshots/ids/events so E2E can identify browser-window owner,
  parent/child numbers, frames, host visibility, key/main state, active
  presentation, and frontmost hit-test owner.
- Add packaged-app/manual and Appium-ready acceptance notes after automated
  evidence is stable.

## Non-goals

- No new Portal visual identity; consuming surfaces remain owned by their task
  packets and `DESIGN.md`.
- No independent browser lifetime for the native child window.
- No screenshot/texture composition, WebView reparenting, GPUI fork, or one
  window per popup.
- No cross-platform native host implementation in this slice.

## Parallel-lane contract

Not parallelized. Browser-window ownership, AppKit attachment, focus, testkit
wire, and all consuming Portal surfaces are shared serial contracts.

## Constraints

- Exactly one native Portal host per BBR browser window; zero cross-window
  attachment or presentation leakage.
- The host remains borderless, implementation-owned, and excluded from Dock,
  Mission Control, independent movement/minimization, and browser state.
- Nonactivating presentations never become key/main. A future presentation that
  needs text focus must use one explicit focus request/restore contract rather
  than accidental window activation.
- Parent close/teardown detaches and closes the host deterministically on the
  main thread.
- Geometry uses AppKit points with explicit screen/content conversion and
  clamps to the owning browser window/display.
- Keep native/unsafe code in `bbr-web-macos`, files below roughly 500 lines,
  and preserve host-wide GUI serialization and deadlines.

## Evidence

### Unit and contract

- [ ] Focus/key-routing matrix covers nonactivating, text-focus, Escape, Cmd-W,
      replacement, outside dismissal, and restoration.
- [ ] Window registry/state tests cover two parents, unique hosts, no
      cross-attachment, stack priority, display conversion, and teardown.

### Real-app E2E

- [ ] `just e2e-shell-portal` covers two BBR windows, distinct child hosts,
      independent presentations, parent close, hide/show, minimize/restore,
      movement/resize, and no orphan window/session.
- [ ] Display/compact-frame tests prove clamping and frame synchronization; key/
      main/frontmost diagnostics prove focus and hit-test ownership.
- [ ] Palette, find, and context-menu suites share the hardened host contract
      without weakening their own assertions.

### Performance

- [ ] Measure only user-visible open/switch paths that remain in the hot path;
      host construction/attachment stays once per browser-window lifetime.
- [ ] Multiple-window idle state adds no continuous frame invalidation and is
      included in Phase-9 energy/idle evidence.

## Completion gates

```bash
just verify
cargo clippy -p bbr-web-macos --all-targets -- -D warnings
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-portal
just e2e-shell-command
just e2e-shell-find        # when integrated
just e2e-shell-context-menu # when integrated
just e2e-perf
just e2e
just roadmap
```

## Delivery sequence

1. Freeze the focus/key and multi-window ownership contracts.
2. Implement registry/geometry/teardown hardening with lower-level diagnostics.
3. Extend real-app multi-window/display/visibility evidence.
4. Re-run consuming Portal suites and optimized measurements.
5. Run independent AppKit-boundary and packaged-app review, remediation, and
   final recheck.

## Iteration policy

First prove two browser windows cannot share or steal hosts. Then harden
focus/key routing, teardown, display movement, minimization/activation, and
stacking. Diagnose failures as owner registry, AppKit parent relation, frame
conversion, key/main policy, focus choreography, or GPUI lifecycle before
changing architecture.

## Blocked stop condition

If the pinned GPUI/AppKit APIs cannot preserve ownership or focus across these
lifecycle cases, stop with the base one-window Portal contract intact. Report
APIs tried, parent/child/key/main/frame observations, affected packaged-app
behavior, and the smallest GPUI-CE patch or product limitation required. Never
silently accept an orphan/independent Portal window.

## Delivery ledger

```text
Outcome: post-MVP hardening packet documented
Commits: planning only
Functional evidence: pending
Real-app E2E: pending
Performance evidence: pending
Roadmap/documentation updates: task packet + Phase 8/9/track links
Open blockers and non-goals: begins only after base Portal and U1 migration
Worktree state: no implementation lane active
```
