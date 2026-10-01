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

Current bounded Portal work: [window lifecycle regression](../tasks/portal-window-lifecycle.md).

- [x] Multi-WebView session registry.
- [x] Deterministic persistent ProfileId → WKWebsiteDataStore mapping.
- [ ] Popup/new-window policy, including transient native sessions for [Link Preview / Peek](../tasks/link-preview-peek.md). Requests resolved to regular tabs establish the opener provenance used by [web-content navigation gestures](../tasks/web-content-navigation-gestures.md). *(Phase 8)*
- [ ] Public-API web-content horizontal Back/Forward gesture routing, isolated from sidebar space swipe and chrome/Portal hit testing, through the bounded [navigation gesture/opener-return packet](../tasks/web-content-navigation-gestures.md). *(Phase 8)*
- [x] Complete the [find-in-page WebKit adapter and Portal bar](../tasks/find-in-page-portal-bar.md). *(Phase 8; real-app and performance evidence in the packet.)*
- [x] Route [BBR-owned context menus over web content](../tasks/bbr-context-menus-over-web.md) without replacing WebKit-owned page menus. *(Integrated at `1d35693`; richer Page-background actions remain a separate follow-up.)*
- [x] Complete the [downloads adapter and sidebar panel](../tasks/downloads-sidebar-panel.md). *(Phase 8; terminal-history follow-up integrated at `cd1bc65`.)*
- [x] Map HTTP origin/proxy authentication after the portable [authentication challenges contract](../tasks/auth-challenges-contract.md). *(The credential sheet was integrated at `53a9590` behind a testkit seam only; the WebKit delegate rejected every HTTP challenge until the [reality gaps packet](../tasks/reality-gaps.md) connected it 2026-09-30, proven by `e2e_shell_auth::native`.)*
- [ ] Implement supported client-certificate native enumeration and selection using the same portable authentication contract.
- [ ] Implement AppKit/WebKit upload/save orchestration after the portable [file upload/save-panel contract](../tasks/file-panels-contract.md). *(Phase 8)*
- [x] Apply page zoom through the portable [per-site settings and zoom contract](../tasks/site-settings-zoom-contract.md). *(Command ladder/native WebKit wiring integrated at `1d35693`; settings UI remains separately open.)*
- [ ] Implement native permission delegate mapping after the portable [permissions contract](../tasks/permissions-contract.md). *(Phase 8)*
- [x] Map native failures and web-process recovery through the portable [web error and recovery contract](../tasks/error-surface-contract.md). *(Visible recovery surfaces integrated at `a70e9a3`.)*
- [ ] Detect and recover from unresponsive WebContent sessions without blocking browser chrome through the public-API-only [renderer isolation and unresponsive tabs packet](../tasks/renderer-isolation-unresponsive-tabs.md). The lifecycle acceptance decision replaces the invalid deterministic helper-kill gate with a public-API split evidence chain: real busy/heavy WKWebView E2E, direct tests of BBR's own public termination delegate and `WKErrorWebContentProcessTerminated` mapping, portable multi-view crash-burst reconciliation, and real-app recovery through the productive typed event pump. Only the external WebKit → callback/error hop remains an explicitly unpassed Apple platform contract. No per-tab PID, heuristic ownership, foreign-helper operation, or private SPI; the finalization candidate awaits whole-candidate review/integration.
- [ ] Complete [native Portal host hardening](../tasks/native-portal-hardening.md) for focus, multi-window/display lifecycle, and teardown. *(Phase 8/9)*
- [ ] Per-view split geometry.
