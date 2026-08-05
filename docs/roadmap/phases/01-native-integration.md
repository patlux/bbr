# Phase 1 — Native GPUI + WKWebView integration baseline

Parent: [Roadmap](../../../ROADMAP.md) · Detail: [Native integration track](../tracks/native-integration.md) · Tests: [Testing track](../tracks/testing.md)

## Outcome

A real GPUI browser window hosts a real native WKWebView without screenshot or texture composition.

## Completed checklist

- [x] Portable `bbr-web` WebEngine contract exists.
- [x] macOS adapter embeds WKWebView as an AppKit child view of the GPUI host.
- [x] Native view follows GPUI slot geometry and window resize.
- [x] URL load, back, forward, reload, and stop actions are routed.
- [x] URL, title, loading, history availability, navigation failure, and crash events are modeled.
- [x] Asynchronous WKWebView title propagation is observed through KVO.
- [x] GPUI chrome can take focus and return it to WKWebView.
- [x] Keyboard actions work while WKWebView owns AppKit focus.
- [x] Test mode uses non-persistent website data stores.
- [x] Real-app smoke proves navigation, resize, focus, and clean shutdown.

## Still-manual acceptance

These do not block the architectural baseline, but must be checked before alpha:

- [ ] Text selection, copy/paste, context menus, file inputs, drag, and IME are manually exercised.
- [ ] Fullscreen and moving between displays are manually exercised.
- [ ] VoiceOver can traverse web content and return predictably to browser controls.
- [ ] Idle CPU/wakeups show no continuous geometry polling.

Canonical follow-up: [Native integration track](../tracks/native-integration.md).
