# Task packet — native portal host above WebKit

Parents: [Phase 4](../phases/04-ui-shell.md) · [Phase 5](../phases/05-command-bar.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Testing track](../tracks/testing.md) · [DESIGN.md](../../../DESIGN.md) §11.4/§13/§15.

## Outcome

BBR has one reusable portal contract per browser window. Ordinary chrome can
escape local GPUI layout through a deferred GPUI host, while surfaces that must
cross the native-web boundary use one transparent, non-activating native portal
host attached above the main window's WKWebViews. A production-path probe proves
that the native host is attached to the same browser window, follows its content
geometry, wins AppKit hit testing above a real WKWebView, dismisses through one
outside pointer action, and does not create, copy, move, or replace any browser
session.

This infrastructure unblocks the preserved U1 command palette/overlay without
changing its product design or weakening the native-WKWebView architecture.

## Scope

- Define app-local portal host, placement, priority, dismissal, geometry, and
  typed snapshot contracts. `GpuiDeferred` mirrors the vendored
  `gpui-component` `deferred(anchored(...)).with_priority(...)` behavior;
  `NativeAboveWeb` selects the native host.
- Create exactly one transparent GPUI portal window per BBR browser window,
  initially hidden and non-activating. Attach its native `NSWindow` to the
  browser's parent `NSWindow` with AppKit child-window ordering above the parent.
- Keep the portal host synchronized to the parent content rect across resize and
  movement. Hide/order it out when no native portal is open; teardown removes
  the child relation.
- Render one minimal production portal probe through the shared portal API,
  including an outside-dismiss layer and a card that stops propagation. The
  probe exists only to prove the infrastructure; U1 owns the actual palette
  content migration afterward.
- Extend the feature-gated testkit with typed open/close/dismiss requests and
  snapshot diagnostics: logical visibility, host, portal rect, child-window
  attachment, native visibility, frame synchronization, and the frontmost
  AppKit window at a portal point.
- Add a bounded real-app `just e2e-shell-portal` suite over a real WKWebView and
  update implemented architecture/E2E facts only after the evidence passes.

## Non-goals

- No U1 palette/overlay migration in this packet; its preserved worktree remains
  untouched until this infrastructure integrates.
- No U2 sidebar redesign or menu behavior changes. U2 may later choose
  `GpuiDeferred` or `NativeAboveWeb` according to whether a surface crosses the
  native content boundary.
- No screenshot/texture composition of web content, no GPUI fork, no Wry or
  `gpui-component` webview feature.
- No cross-platform native host implementation. Portable portal state and
  placement remain platform-neutral inside the app; non-macOS uses the GPUI
  host until its adapter phase.
- No arbitrary multi-window browser ownership: the native portal window has no
  tabs, profiles, spaces, data store, history, or independent product lifetime.

## Parallel-lane contract

Not parallelized. This is a serial architecture kernel on `main` because it
touches the browser window composition, native host bridge, testkit wire, E2E
runner, architecture documents, and the same app integration surfaces U1/U2
will later consume. U1 stays frozen; U2 remains isolated and receives this
foundation only after integration.

## Constraints

- User-visible ownership remains one BBR browser window. The native portal
  `NSWindow` is an implementation-owned child: borderless/non-activating, no
  titlebar, Dock entry, Mission Control identity, independent movement,
  minimization, tab/profile/space ownership, or browser session.
- One native portal host per browser window, not one window per popup.
- GPUI owns portal content and state; AppKit owns only native z-order, parent
  attachment, frame synchronization, visibility, and hit-test routing.
- Native integration and unsafe Objective-C remain contained in
  `bbr-web-macos`; portable crates do not import AppKit.
- The WKWebView remains a real native child view. Never screenshot, texture-
  compose, reparent, or replace it to make chrome overlap.
- The portal starts hidden and must never flash as a standalone window.
- The portal must not become key/main merely by showing; keyboard focus remains
  in the existing BBR focus choreography unless a future portal explicitly
  requests focus.
- Geometry uses points and explicit host/AppKit coordinate conversion. Resize,
  move, hide, close, and parent teardown are deterministic and main-thread-only.
- No new dependencies. Keep files below roughly 500 lines.
- No weakened tests, semantic guards, deadlines, GUI serialization, or
  performance budgets.

## Evidence

### Unit and contract

- [ ] Portal placement/clamping tests cover centered and anchored geometry,
      including compact windows and off-edge anchors.
- [ ] Portal state tests prove one active presentation, host selection,
      priority ordering, id stability, and deterministic dismissal.
- [ ] Native bridge diagnostics prove child attachment, frame synchronization,
      visibility transitions, non-activating window policy, and teardown.
- [ ] No dependency/lock drift; portable crates remain AppKit-free.

### Real-app E2E

- [ ] `just e2e-shell-portal` launches the real app with a real WKWebView,
      opens the native portal through the production portal seam, and asserts:
      same browser process/window ownership; active WebViewId and session count
      unchanged; `NativeAboveWeb` snapshot state; child-window attachment;
      exact parent-content frame synchronization; and AppKit frontmost hit-test
      ownership at a point covered by the portal.
- [ ] Resize the parent while the portal is open and prove the native portal
      follows without recreating the web session; dismiss by the production
      outside-action seam and prove the native window is ordered out and the
      original focus/session state remains valid.
- [ ] Stable `bbr-portal-*` ids and typed semantic/snapshot fields expose only
      the active presentation. No sleeps or pixel comparisons; typed requests,
      snapshots, events, and deadlines only.

### Performance

- [ ] Add a `portal_open` optimized production-path scenario only if the host
      open/attach path remains in the interaction hot path after the one-host-
      per-window design. Preferred design constructs and attaches the host once
      at window creation, so opening is only state update + order/front + GPUI
      presentation; if user-visible timing changes, measure it with ≥30 samples,
      semantic/native presentation commit, and the command-surface warm budget.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
cargo clippy -p bbr-web-macos --all-targets -- -D warnings
just e2e-shell-portal
just e2e-shell-command
just e2e-shell-sidebar
just e2e-perf          # when portal_open joins the measured matrix
just e2e               # once after the complete infrastructure slice is stable
just roadmap
```

GUI suites run sequentially through the host-wide lock.

## Delivery sequence

1. Commit this task contract and roadmap ownership before native implementation.
2. Implement portable portal state/placement and the one-host native bridge in
   the smallest production probe path.
3. Add typed testkit diagnostics and real-app E2E over a real WKWebView.
4. Run one independent read-only architecture/native-boundary review.
5. Run one remediation writer only for concrete findings, then a focused
   recheck and the canonical sequential gate.
6. Integrate the portal foundation; only then refresh the preserved U1 worktree
   and migrate its existing palette/overlay presentation to `NativeAboveWeb`.

## Iteration policy

First prove an initially hidden native child portal can be attached above one
real WKWebView without changing browser ownership or session identity. Then add
frame synchronization and dismissal. Then expose the generic portal contract
and bounded E2E evidence. After any failure, identify whether the cause is GPUI
window lifecycle, AppKit parent/ordering, coordinate conversion, focus, or
hidden-harness behavior before changing code. Do not broaden into U1 content or
U2 behavior while the infrastructure contract is unsettled.

## Blocked stop condition

If no defensible path remains, stop with the preserved `main` state and:

- GPUI and AppKit APIs inspected;
- the exact child-window/transparent-window/focus behavior observed;
- attempted attachment, ordering, geometry, and hit-test paths;
- whether the blocker is the pinned GPUI window API, AppKit child-window
  behavior, hidden test-window limitations, or an ownership contradiction;
- the smallest required GPUI-CE patch, AppKit composition change, test
  environment, or product/design decision.

Never fall back to screenshot/texture web composition or silently accept a
semantic-only portal that remains behind WebKit.

## Delivery ledger

```text
Outcome:       packet active; implementation pending
Commits:       parent-owned planning commit pending
Functional evidence: pending
Real-app E2E:  pending
Performance evidence: pending; one-host creation is expected outside open hot path
Roadmap/documentation updates: packet + Phase 4/5/current-focus links pending
Open blockers and non-goals: U1 remains preserved and blocked until this slice integrates
Worktree state: main, exclusive orchestrator writer

Planning/read-only: measured in session history; exact roll-up at delivery
Implementation: pending
Test execution: pending
Review: pending
Remediation: pending
Waiting/retries: pending
Total elapsed: pending
Human intervention: user approved the general Portal direction
```
