# Phase 2 — Portable browser orchestration

Parent: [Roadmap](../../../ROADMAP.md) · Current task packets: [Browser controller](../tasks/browser-controller.md), [Fake WebEngine](../tasks/fake-web-engine.md)

## Outcome

Profiles, spaces, tabs, lifecycle, navigation, and splits drive disposable web sessions through a portable controller, without GPUI or macOS frameworks.

## Domain model — complete

- [x] Profiles and spaces.
- [x] Ordered tabs and pinned-first invariant.
- [x] Active tab per space.
- [x] Open, close, reopen, select, reorder, move, pin, and unpin.
- [x] Two-pane split model and teardown invariants.
- [x] `Active`, `Warm`, `Suspended`, and `Discarded` lifecycle policy.
- [x] 34 framework-free domain tests.

## Orchestration — current work

- [ ] [Implement the portable BrowserController](../tasks/browser-controller.md).
- [ ] [Implement a deterministic fake WebEngine](../tasks/fake-web-engine.md).
- [ ] Map every live `TabId` to exactly one `WebViewId`.
- [ ] Route back/forward/reload/stop to the focused tab or split pane.
- [ ] Preserve sessions for moves within one profile.
- [ ] Recreate sessions for cross-profile moves.
- [ ] Destroy sessions for close/suspend and recreate on activation.
- [ ] Apply WebEvents to the correct tab and reject unknown WebViewIds.
- [ ] Prove no orphan or duplicate sessions through invariant tests.

## Phase exit

- [ ] Controller and fake-engine tests run without macOS frameworks or a display server.
- [ ] `just verify` remains green.
- [ ] Architecture docs describe controller ownership and event flow.
