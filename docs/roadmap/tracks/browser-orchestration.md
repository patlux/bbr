# Track — Browser orchestration

Parent phases: [Phase 2](../phases/02-browser-core.md), [Phase 3](../phases/03-multi-session-webkit.md), [Phase 6](../phases/06-split-view.md)

## Purpose

Own the portable boundary between persistent browser objects (`bbr-domain`) and disposable native web sessions (`bbr-web`).

## Current task packets

- [ ] [BrowserController](../tasks/browser-controller.md)
- [ ] [Deterministic fake WebEngine](../tasks/fake-web-engine.md)

## Durable checklist

- [x] Every live `TabId` maps to exactly one `WebViewId`.
- [x] Every `WebViewId` maps back to one existing tab.
- [x] Visible single-pane tabs and both split panes own active sessions.
- [x] Warm tabs retain hidden sessions.
- [x] Suspended/discarded tabs retain no native session.
- [x] Same-profile moves preserve sessions.
- [x] Cross-profile moves destroy and recreate sessions in the destination store.
- [x] Commands target the active tab or focused pane explicitly.
- [x] WebEvents update the owning tab and navigation state deterministically.
- [x] Unknown/stale WebEvents are reported without corrupting state.
- [x] Controller operations expose typed outcomes suitable for UI and E2E events.
- [x] Controller invariants run after every mutation in tests.

## Verification

- [x] Framework-free fake-engine contract suite.
- [ ] Multi-session native integration suite.
- [ ] Real-app tabs/spaces/splits E2E as each UI slice lands.

Evidence: 46 bbr-browser tests (17 fake-engine/registry contract tests + 28
controller behavior tests + one snapshot test) cover the durable checklist against the deterministic
fake engine; `just verify` + `just e2e-smoke` green. The two unchecked
verification items land with the Phase 3 multi-session macOS adapter and its
real-app tab E2E slice.
