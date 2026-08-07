# Track — Native web-engine integration

Parent phases: [Phase 1](../phases/01-native-integration.md), [Phase 3](../phases/03-multi-session-webkit.md), [Phase 6](../phases/06-split-view.md), [Phase 8](../phases/08-daily-driver.md)

## Implemented baseline

- [x] Native WKWebView remains an AppKit child view; no screenshot/texture composition.
- [x] GPUI layout publishes native slot geometry.
- [x] Focus moves between GPUI controls and WKWebView.
- [x] Navigation/title/loading/history/crash/failure events cross the adapter.
- [x] KVO title observation is balanced during teardown.
- [x] Real-app smoke covers load, navigation, reload, resize, focus, and quit.
- [x] Real-app tabs suite covers two native sessions, warm-tab reuse, focused navigation, and destruction on close.
- [x] Real-app profiles suite covers cookie isolation, same-profile session reuse, cross-profile recreation, and suspend cleanup.

## Manual integration matrix

- [ ] Mouse, trackpad scroll, text selection, copy/paste, and context menu.
- [ ] Keyboard input, IME/composition, and browser shortcuts while web content is focused.
- [ ] Drag/drop and file inputs.
- [ ] Fullscreen, occlusion, window movement, and display/scale changes.
- [ ] VoiceOver traversal and focus hand-off.
- [ ] Video/audio, editors, conferencing, WebGL/WebGPU where supported.
- [ ] Downloads, uploads, payments, authentication, and passkeys where supported.
- [ ] Idle CPU/wakeups and native-view geometry stability.

## Adapter growth

- [x] Multi-WebView session registry.
- [x] Deterministic persistent ProfileId → WKWebsiteDataStore mapping.
- [ ] Popup/new-window policy, including transient native sessions for [Link Preview / Peek](../tasks/link-preview-peek.md). *(Phase 8)*
- [ ] Complete the [find-in-page WebKit adapter and Portal bar](../tasks/find-in-page-portal-bar.md). *(Phase 8)*
- [ ] Route [BBR-owned context menus over web content](../tasks/bbr-context-menus-over-web.md) without replacing WebKit-owned page menus. *(Phase 8)*
- [ ] Complete the [downloads adapter and sidebar panel](../tasks/downloads-sidebar-panel.md). *(Phase 8)*
- [ ] Map HTTP/client-certificate delegates after the portable [authentication challenges contract](../tasks/auth-challenges-contract.md). *(Phase 8)*
- [ ] Implement AppKit/WebKit upload/save orchestration after the portable [file upload/save-panel contract](../tasks/file-panels-contract.md). *(Phase 8)*
- [ ] Apply per-site values and page zoom after the portable [per-site settings and zoom contract](../tasks/site-settings-zoom-contract.md). *(Phase 8)*
- [ ] Implement native permission delegate mapping after the portable [permissions contract](../tasks/permissions-contract.md). *(Phase 8)*
- [ ] Map native failures and web-process recovery after the portable [web error and recovery contract](../tasks/error-surface-contract.md). *(Phase 8)*
- [ ] Complete [native Portal host hardening](../tasks/native-portal-hardening.md) for focus, multi-window/display lifecycle, and teardown. *(Phase 8/9)*
- [ ] Per-view split geometry.
