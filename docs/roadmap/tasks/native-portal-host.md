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

- [x] Portal placement/clamping tests cover centered and anchored geometry,
      including compact windows and off-edge anchors.
- [x] Portal state tests prove one active presentation, host selection,
      priority ordering, id stability, and deterministic dismissal.
- [x] Native bridge diagnostics prove child attachment, frame synchronization,
      visibility transitions, non-activating window policy, and teardown.
- [x] No dependency/lock drift; portable crates remain AppKit-free.

### Real-app E2E

- [x] `just e2e-shell-portal` launches the real app with a real WKWebView,
      opens the native portal through the production portal seam, and asserts:
      same browser process/window ownership; active WebViewId and session count
      unchanged; `NativeAboveWeb` snapshot state; child-window attachment;
      exact parent-content frame synchronization; and AppKit hit-test/z-order
      ownership at a point covered by the portal. The deterministic attached-
      child/frame invariant is corroborated by at least one real Window Server
      query: absolute BBR ownership or parent directly below the portal. macOS
      may report a foreign always-on-top/system overlay through the other raw
      query, so requiring both would assert global desktop state rather than
      portal-over-WKWebView ownership.
- [x] Resize the parent while the portal is open and prove the native portal
      follows without recreating the web session; dismiss by the production
      outside-action seam and prove the native window is ordered out and the
      original focus/session state remains valid.
- [x] Stable `bbr-portal-*` ids and typed semantic/snapshot fields expose only
      the active presentation. No sleeps or pixel comparisons; typed requests,
      snapshots, events, and deadlines only.

### Performance

- [x] `portal_open` does not join the measured matrix: the implemented design
      constructs and attaches one host at browser-window creation, outside the
      open interaction. Opening is a state update + `orderFront` + GPUI
      presentation; two complete canonical `just e2e` runs kept the existing
      optimized startup and interaction budgets green.

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
Outcome:       complete; one reusable NativeAboveWeb/GpuiDeferred portal contract and one pinned native child host per browser window
Commits:       planning 292a0b8; implementation commit/merge recorded by orchestrator after this lane handoff
Functional evidence: 23 app portal tests + 9 bbr-web-macos tests; Box-pinned observer target; typed non-key/non-main/attachment/frame/teardown diagnostics
Real-app E2E:  portal 3/3 green in three focused runs; command 11/11; sidebar 3/3; two complete canonical just e2e sequences green
Performance evidence: both canonical short optimized gates green; no portal_open scenario because host creation/attachment is outside the open interaction
Roadmap/documentation updates: packet evidence and implemented architecture/E2E facts updated in lane
Open blockers and non-goals: none for infrastructure; U1 migration remains the next separate packet
Worktree state: parallel/native-portal-host; implementation uncommitted until orchestrator integration

Planning/read-only: recorded in workflow run bbr-native-portal-host-msj9f75q-1k0tf0
Implementation: contract, native host, production probe, testkit diagnostics and E2E complete
Test execution: just verify; both packet clippies; portal ×3; command; sidebar; canonical just e2e ×2
Review: found deterministic observer use-after-move and evidence/structure gaps
Remediation: heap-pin before observer installation; teardown/hit-test diagnostics; module/E2E splits; all findings closed
Waiting/retries: GUI gate serialized; no sleep-based product assertions
Total elapsed: workflow 8,621 s plus focused orchestrator remediation/reverification
Human intervention: user approved the general Portal direction
```
