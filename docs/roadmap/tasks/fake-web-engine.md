# Task packet — Deterministic fake WebEngine

Parents: [Phase 2](../phases/02-browser-core.md) · [Browser orchestration track](../tracks/browser-orchestration.md) · Consumer: [BrowserController](browser-controller.md)

## Objective

Provide a framework-free `WebEngine` implementation that records exact calls, owns fake sessions, and emits controlled WebEvents for controller contract tests.

## Constraints

- No GPUI, macOS frameworks, sockets, HTTP, sleeps, or wall-clock dependence.
- No HTML rendering or simulated DOM.
- Deterministic ordering and inspectable state.
- Test capability should live in a portable crate/API, not in production adapter conditionals.

## Recorded operations

- [x] `AttachSlot { rect, content_height }`.
- [x] `Create { web_view, profile, initial_url }`.
- [x] `Show { web_view, visible }`.
- [x] `Action { web_view, action }`.
- [x] `Destroy { web_view }`.
- [x] Native/chrome focus requests when relevant to controller contracts.

## Fake state

- [x] Track every existing fake session and its ProfileId.
- [x] Track visibility independently per WebViewId.
- [x] Reject duplicate creation and actions against missing sessions in tests.
- [x] Allow operation log reset without resetting sessions.
- [x] Provide deterministic event injection into the registered sink.
- [x] Provide helpers for URL/title/loading/history/crash/failure event sequences.

## Contract tests

- [x] WebEngine trait remains object-safe or controller-compatible.
- [x] Creation records the correct profile and URL.
- [x] Show/hide never creates or destroys implicitly.
- [x] Destroy removes the session exactly once.
- [x] Actions target only existing IDs.
- [x] Event injection preserves supplied ordering and IDs.
- [x] Two or more fake sessions remain independent.

## Delivery choice

Decide during implementation, based on dependency direction:

- [x] Put the fake in `bbr-browser::testing` because it currently exists only to test controller policy.
- A future second portable consumer may justify extracting `bbr-web-fake`; do not create that crate speculatively.

## Evidence

- [x] Controller test suite uses the fake exclusively.
- [x] `just verify` passes without a display server.
- [x] Roadmap and architecture document the chosen ownership location.

Evidence: `just verify` green, `just e2e-smoke` green, 46 bbr-browser tests
(17 fake-engine/registry contract tests + 28 controller behavior tests + one snapshot test), ownership
documented in `ARCHITECTURE.md` §9a.
