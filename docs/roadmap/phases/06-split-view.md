# Phase 6 — Two-pane split view

Parent: [Roadmap](../../../ROADMAP.md) · Prerequisites: [Phase 2](02-browser-core.md), [Phase 3](03-multi-session-webkit.md)

## Outcome

Two existing tabs can be viewed and navigated side by side with clear focus and no orphan sessions.

## Parallel lane

Split work is not yet safe to fan out. The bounded serial
[split-view shared-contract packet](../tasks/split-shared-contract.md) extracts
the authoritative controller pane facade, per-`WebViewId` native geometry, and
typed testkit proof on `main`. After that exact anchor lands, split-core may own
new split-focused modules in a sibling worktree; native geometry and GPUI
composition remain serial integration surfaces. See the
[parallel-delivery contract](../../parallel-delivery.md).

## Checklist

- [ ] Extend the WebEngine geometry contract to address each WebViewId independently.
- [ ] Split the active tab left or right with an existing or new secondary tab.
- [ ] Show and clip two native WKWebViews simultaneously.
- [ ] Add a resizable divider without continuous idle invalidation.
- [ ] Track and render the focused pane.
- [ ] Route navigation and command-bar actions to the focused pane.
- [ ] Close one pane without closing its tab unless explicitly requested.
- [ ] Dissolve splits safely when a pane tab closes or moves.
- [ ] Persist split composition later through the Phase 7 repository contract.
- [ ] Add split controller tests and real-WKWebView E2E.
- [ ] Polish divider, focus affordance, hover states, keyboard movement, and compact-window behavior.

## Phase exit

- [ ] Both WKWebViews remain interactive during resize and divider drag.
- [ ] No split mutation leaves an orphan native view or ambiguous keyboard target.
