# Phase 6 — Two-pane split view

Parent: [Roadmap](../../../ROADMAP.md) · Prerequisites: [Phase 2](02-browser-core.md), [Phase 3](03-multi-session-webkit.md)

## Outcome

Two existing tabs can be viewed and navigated side by side with clear focus and no orphan sessions.

## Parallel lane

Split work is not yet safe to fan out. The bounded serial
[split-view shared-contract packet](../tasks/split-shared-contract.md) extracts
the authoritative controller pane facade, per-`WebViewId` native geometry, and
typed testkit proof on `main`. The per-`WebViewId` geometry contract is
implemented and verified (real-app split-geometry suite, single-pane sidebar
regression, canonical sequential `just e2e`, and report-only optimized
performance gate are green on this host); the phase's broader split interaction
items remain open. After that exact
anchor landed at `29c34e68f4fad5f5338f7b69e50c44dd84145e74`.
The bounded [portable split-core packet](../tasks/split-core.md) is reviewed and
integrated at `9eeb6b2f7abed2451a613e3ee4e8d4624db4cfc8`; the bounded
[split app-composition packet](../tasks/split-app-composition.md) now owns the
serial production divider, projected layout, focused-pane presentation,
testkit, E2E, and performance slice. Production split creation over native
WebKit glass remains a later contract decision. The G3 composition sequence has
integrated Sidebar and Command. Split Preview / `bbr-ui::split` remains the
next serial G3 composition, but Patrick's active scheduling override defers
every Split/SPLIT-COMPOSITION preflight, reconciliation, review, port, and
writer until he explicitly re-enables it. After that unblock, the first gate is
a fresh read-only preflight against the exact post-Command checkpoint; no Split
writer is authorized before that preflight and a committed activation contract.
See the [parallel-delivery contract](../../parallel-delivery.md).

**Blueprint D23 (2026-09-29/30):** the Split block is lifted and the split
becomes a tiling of rows and columns with several groups per space; the
serial [split tiling packet](../tasks/split-tiling.md) owns the six slices
(tiling contract, domain/controller, persistence, composition, commands,
rail and drag) and supersedes the two-pane constraints below.

## Checklist

- [x] Extend the WebEngine geometry contract to address each WebViewId independently. *(Proven by the keyed `set_web_view_rect` contract on null/fake/macOS engines, the controller pane-layout tests, and the real-app split-geometry suite.)*
- [x] Split the active tab left or right with an existing or new secondary tab. *(Blueprint D23, 2026-09-30: [split tiling](../tasks/split-tiling.md), `just e2e-split`.)*
- [x] Show and clip two native WKWebViews simultaneously. *(Blueprint D23, 2026-09-30: [split tiling](../tasks/split-tiling.md), `just e2e-split`.)*
- [x] Add a resizable divider without continuous idle invalidation. *(Blueprint D23, 2026-09-30: [split tiling](../tasks/split-tiling.md), `just e2e-split`.)*
- [ ] Track and render the focused pane.
- [ ] Route navigation and command-bar actions to the focused pane.
- [x] Close one pane without closing its tab unless explicitly requested. *(Blueprint D23, 2026-09-30: [split tiling](../tasks/split-tiling.md), `just e2e-split`.)*
- [x] Dissolve splits safely when a pane tab closes or moves. *(Blueprint D23, 2026-09-30: [split tiling](../tasks/split-tiling.md), `just e2e-split`.)*
- [x] Persist split composition later through the Phase 7 repository contract. *(Blueprint D23: in `splits.txt` beside the store — the repository's two-pane table cannot hold a tree; see [split tiling](../tasks/split-tiling.md).)*
- [x] Add split controller tests and real-WKWebView E2E. *(Blueprint D23, 2026-09-30: [split tiling](../tasks/split-tiling.md), `just e2e-split`.)*
- [ ] Polish divider, focus affordance, hover states, keyboard movement, and compact-window behavior.

## Phase exit

- [ ] Both WKWebViews remain interactive during resize and divider drag.
- [ ] No split mutation leaves an orphan native view or ambiguous keyboard target.
