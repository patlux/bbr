# Track — Native web-engine integration

Parent phases: [Phase 1](../phases/01-native-integration.md), [Phase 3](../phases/03-multi-session-webkit.md), [Phase 6](../phases/06-split-view.md), [Phase 8](../phases/08-daily-driver.md)

## Implemented baseline

- [x] Native WKWebView remains an AppKit child view; no screenshot/texture composition.
- [x] GPUI layout publishes native slot geometry.
- [x] Focus moves between GPUI controls and WKWebView.
- [x] Navigation/title/loading/history/crash/failure events cross the adapter.
- [x] KVO title observation is balanced during teardown.
- [x] Real-app smoke covers load, navigation, reload, resize, focus, and quit.

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

- [ ] Multi-WebView session registry.
- [ ] Deterministic persistent ProfileId → WKWebsiteDataStore mapping.
- [ ] Popup/new-window policy.
- [ ] Download, permission, authentication, and file-panel events.
- [ ] Web-process recovery.
- [ ] Per-view split geometry.
