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
- [x] 40 framework-free domain tests.

## Orchestration — current work

- [ ] [Implement the portable BrowserController](../tasks/browser-controller.md).
- [ ] [Implement a deterministic fake WebEngine](../tasks/fake-web-engine.md).
- [ ] Close the [browser-orchestration durable checklist](../tracks/browser-orchestration.md#durable-checklist) (TabId↔WebViewId mapping, command routing, profile moves, session policy, WebEvent application, and invariant tests are owned there, not duplicated here).

## Phase exit

- [x] Controller and fake-engine tests run without macOS frameworks or a display server.
- [x] `just verify` remains green.
- [x] Architecture docs describe controller ownership and event flow.

Exit items above are individually proven (46 framework-free bbr-browser tests;
`just verify` + `just e2e-smoke` green; `ARCHITECTURE.md` §9a). The phase
roll-up in `ROADMAP.md` stays `[ ]` until the linked packets close — notably
the real-app tab E2E vertical slice of `browser-controller.md`, which needs
the Phase 3 multi-session macOS adapter.
