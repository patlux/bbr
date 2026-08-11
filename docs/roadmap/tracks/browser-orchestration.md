# Track — Browser orchestration

Parent phases: [Phase 2](../phases/02-browser-core.md), [Phase 3](../phases/03-multi-session-webkit.md), [Phase 6](../phases/06-split-view.md), [Phase 8](../phases/08-daily-driver.md)

## Purpose

Own the portable boundary between persistent browser objects (`bbr-domain`) and disposable native web sessions (`bbr-web`).

## Completed task packets

- [x] [BrowserController](../tasks/browser-controller.md)
- [x] [Deterministic fake WebEngine](../tasks/fake-web-engine.md)

## Planned task packets

- [ ] [Background link tabs](../tasks/background-link-tabs.md) — Cmd-primary and middle click create grouped background children directly after the opener without selection/focus mutation.
- [ ] [Web-content gesture real-app revalidation](../tasks/web-gesture-revalidation.md) — re-prove the integrated gesture path on the current product base and remediate only a reproduced regression.

## Durable checklist

- [x] Every live `TabId` maps to exactly one `WebViewId`.
- [x] Every `WebViewId` maps back to one existing tab.
- [x] Visible single-pane tabs and both split panes own active sessions.
- [x] Warm tabs retain hidden sessions.
- [x] Suspended/discarded tabs retain no native session.
- [x] Same-profile moves preserve sessions.
- [x] Cross-profile moves destroy and recreate sessions in the destination store.
- [x] Commands target the active tab or focused pane explicitly.
- [x] A web-content Back/Forward gesture targets the exact native pane without
      colliding with sidebar space swipe or chrome/Portal hit testing. *(Functionally
      integrated at `542af1a`; current-base revalidation is separately scheduled.)*
- [x] A regular link-created child records exact session-local opener `TabId`
      provenance; Back at settled history root closes only that child and
      activates the surviving opener atomically. *(Integrated at `542af1a`.)*
- [ ] Cmd-primary and middle-click link activations create one grouped
      background child immediately after the opener while preserving opener
      selection, web focus, profile, and session identity.
- [x] WebEvents update the owning tab and navigation state deterministically.
- [x] Unknown/stale WebEvents are reported without corrupting state.
- [x] Controller operations expose typed outcomes suitable for UI and E2E events.
- [x] Controller invariants run after every mutation in tests.

## Deferred architecture options

- [Profile-scoped singleton favorite runtime](../tasks/profile-scoped-favorite-runtime.md) — optional post-MVP model for one profile-bound live session presented across same-profile spaces. It does not change the current window-global concrete-`TabId`, owning-space contract until an explicit product/migration decision.

## Verification

- [x] Framework-free fake-engine contract suite.
- [x] Multi-session native integration suite for the tab slice.
- [x] Real-app profile/space session-policy suite before visible Spaces UI.
- [ ] Real-app visible spaces/splits E2E as each UI slice lands.

Evidence: 47 `bbr-browser` tests cover portable policy; `just e2e-tabs` proves
two real WKWebViews, stable warm-session identities, focused navigation, and
native destruction; `just e2e-profiles` proves profile isolation,
same-profile reuse, cross-profile recreation, and suspend cleanup. Visible
spaces and split UI remain owned by their later phases.
