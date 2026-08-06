# Phase 3 — Multi-session macOS WebKit adapter

Parent: [Roadmap](../../../ROADMAP.md) · Detail: [Native integration track](../tracks/native-integration.md) · Prerequisite: [Phase 2](02-browser-core.md)

## Outcome

The macOS adapter hosts multiple profile-isolated WKWebViews and implements the controller contract faithfully.

## Checklist

- [x] Replace the single-view storage with `WebViewId → NativeWebSession` ownership.
- [x] Keep one retained delegate and balanced KVO observation per web view.
- [x] Create, show, hide, focus, action, and destroy by WebViewId.
- [x] Keep persistent website-data-store identity deterministic per ProfileId.
- [x] Reuse sessions across tab switches and same-profile space moves.
- [x] Recreate sessions when a tab crosses a profile boundary.
- [x] Add profile-isolation integration tests using real cookie behavior in two native data stores.
- [x] Add a real-app multi-session tab integration suite.
- [x] Destroy and recreate native sessions through the suspend/reactivate lifecycle.
- [x] Keep portable crates free of macOS imports.
- [ ] Add progress and favicon-candidate events. *(deferred to [Phase 4](04-ui-shell.md), where the shell consumes them)*
- [ ] Model target-blank/new-window requests. *(deferred to [Phase 8](08-daily-driver.md), including [Link Preview / Peek](../tasks/link-preview-peek.md))*
- [ ] Model downloads, permissions, authentication challenges, and file panels. *(deferred to [Phase 8](08-daily-driver.md))*
- [ ] Recover a terminated WebKit process into the `Discarded`/reload flow. *(deferred to [Phase 8](08-daily-driver.md))*

## Phase exit

- [x] Two profiles can use the same site without sharing login state.
- [x] Switching warm tabs does not reload or recreate their WKWebViews.
- [x] Closing or suspending a tab leaves no native view, delegate, or KVO observer.

Evidence: `direnv exec . just verify`, `direnv exec . just e2e`, commit `f51cde6`.
