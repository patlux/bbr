# Phase 3 — Multi-session macOS WebKit adapter

Parent: [Roadmap](../../../ROADMAP.md) · Detail: [Native integration track](../tracks/native-integration.md) · Prerequisite: [Phase 2](02-browser-core.md)

## Outcome

The macOS adapter hosts multiple profile-isolated WKWebViews and implements the controller contract faithfully.

## Checklist

- [x] Replace the single-view storage with `WebViewId → NativeWebSession` ownership.
- [x] Keep one retained delegate and balanced KVO observation per web view.
- [x] Create, show, hide, focus, action, and destroy by WebViewId.
- [x] Keep persistent website-data-store identity deterministic per ProfileId.
- [x] Reuse sessions across tab switches within one profile; same-profile space moves remain covered portably until Spaces UI lands.
- [ ] Recreate sessions when a tab crosses a profile boundary.
- [ ] Add progress and favicon-candidate events.
- [ ] Model target-blank/new-window requests.
- [ ] Model downloads, permissions, authentication challenges, and file panels.
- [ ] Recover a terminated WebKit process into the `Discarded`/reload flow.
- [ ] Add profile-isolation integration tests.
- [x] Add a real-app multi-session tab integration suite.
- [x] Keep portable crates free of macOS imports.

## Phase exit

- [ ] Two profiles can use the same site without sharing login state.
- [x] Switching warm tabs does not reload or recreate their WKWebViews.
- [ ] Closing or suspending a tab leaves no native view, delegate, or KVO observer.
