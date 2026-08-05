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

- [ ] `AttachSlot { rect, content_height }`.
- [ ] `Create { web_view, profile, initial_url }`.
- [ ] `Show { web_view, visible }`.
- [ ] `Action { web_view, action }`.
- [ ] `Destroy { web_view }`.
- [ ] Native/chrome focus requests when relevant to controller contracts.

## Fake state

- [ ] Track every existing fake session and its ProfileId.
- [ ] Track visibility independently per WebViewId.
- [ ] Reject duplicate creation and actions against missing sessions in tests.
- [ ] Allow operation log reset without resetting sessions.
- [ ] Provide deterministic event injection into the registered sink.
- [ ] Provide helpers for URL/title/loading/history/crash/failure event sequences.

## Contract tests

- [ ] WebEngine trait remains object-safe or controller-compatible.
- [ ] Creation records the correct profile and URL.
- [ ] Show/hide never creates or destroys implicitly.
- [ ] Destroy removes the session exactly once.
- [ ] Actions target only existing IDs.
- [ ] Event injection preserves supplied ordering and IDs.
- [ ] Two or more fake sessions remain independent.

## Delivery choice

Decide during implementation, based on dependency direction:

- [ ] Put the fake in `bbr-browser::testing` when it exists only to test controller policy; or
- [ ] Create `bbr-web-fake` if multiple portable crates/integration harnesses need it.

Do not create a separate crate speculatively before a second real consumer exists.

## Evidence

- [ ] Controller test suite uses the fake exclusively.
- [ ] `just verify` passes without a display server.
- [ ] Roadmap and architecture document the chosen ownership location.
