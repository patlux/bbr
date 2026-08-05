# Track — Browser orchestration

Parent phases: [Phase 2](../phases/02-browser-core.md), [Phase 3](../phases/03-multi-session-webkit.md), [Phase 6](../phases/06-split-view.md)

## Purpose

Own the portable boundary between persistent browser objects (`bbr-domain`) and disposable native web sessions (`bbr-web`).

## Current task packets

- [ ] [BrowserController](../tasks/browser-controller.md)
- [ ] [Deterministic fake WebEngine](../tasks/fake-web-engine.md)

## Durable checklist

- [ ] Every live `TabId` maps to exactly one `WebViewId`.
- [ ] Every `WebViewId` maps back to one existing tab.
- [ ] Visible single-pane tabs and both split panes own active sessions.
- [ ] Warm tabs retain hidden sessions.
- [ ] Suspended/discarded tabs retain no native session.
- [ ] Same-profile moves preserve sessions.
- [ ] Cross-profile moves destroy and recreate sessions in the destination store.
- [ ] Commands target the active tab or focused pane explicitly.
- [ ] WebEvents update the owning tab and navigation state deterministically.
- [ ] Unknown/stale WebEvents are reported without corrupting state.
- [ ] Controller operations expose typed outcomes suitable for UI and E2E events.
- [ ] Controller invariants run after every mutation in tests.

## Verification

- [ ] Framework-free fake-engine contract suite.
- [ ] Multi-session native integration suite.
- [ ] Real-app tabs/spaces/splits E2E as each UI slice lands.
