# Phase 8 — Daily-driver browser essentials

Parent: [Roadmap](../../../ROADMAP.md) · UI standards: [UI design-system track](../tracks/ui-design-system.md)

## Outcome

BBR covers the workflows required for a credible macOS alpha rather than only tab organization.

## Checklist

- [ ] Close the ordinary-browser behavior gaps tracked in the
  [daily-driver capability inventory](../../product/daily-driver-capability-inventory.md)
  through bounded packets rather than a phase-wide implementation. First
  canonical slice: [web-content Back/Forward gestures and child-tab opener
  return](../tasks/web-content-navigation-gestures.md). New-window disposition,
  background opening, and insertion policy remain separate explicit decisions.
- [x] Deliver downloads destination, progress, history, reveal, retry, and failure UI through the bounded [downloads sidebar panel](../tasks/downloads-sidebar-panel.md) — including the settled post-MVP follow-up (terminal-history persistence, durable Clear/Delete, cancelled Restart, active-only badge).
- [x] Deliver find in page with keyboard navigation and result count through the completed [find-in-page Portal bar](../tasks/find-in-page-portal-bar.md), including real WKWebView E2E and `find_open`/query/step performance evidence.
- [x] Define the portable request/decision authority through the completed [permissions contract](../tasks/permissions-contract.md).
- [ ] Deliver camera, microphone, location, notification, and clipboard adapter/UI surfaces where supported, consuming the one portable permissions authority above.
- [x] Define HTTP origin/proxy authentication and client-certificate request/response authority through the completed [authentication challenges contract](../tasks/auth-challenges-contract.md); the HTTP credential delegate/sheet is integrated through `53a9590`.
- [ ] Implement supported native client-certificate enumeration/selection without weakening the portable identity and secret-redaction contract.
- [x] Define portable upload/save request-completion semantics through the completed [file upload/save-panel contract](../tasks/file-panels-contract.md).
- [ ] Implement native AppKit/WebKit upload/save panel orchestration, sandbox access, cancellation, and real-app E2E.
- [x] Define the typed taxonomy and legal recovery actions through the completed [web error and recovery contract](../tasks/error-surface-contract.md), then deliver the integrated TLS/navigation/offline/process-crash surfaces (`a70e9a3`).
- [ ] History UI and clear-data controls per profile.
- [ ] Deliver durable global **Saved Pages** through the bounded sequence: [B1 portable contract + persistence](../tasks/saved-pages-portable-persistence.md) → [B2 Cmd-D, shared save/remove/open command, Palette provider, and menu consumers](../tasks/saved-pages-actions-command.md) → [B3 Saved Pages Library](../tasks/saved-pages-library.md). Pin and Favorite remain concrete live-tab presentations; Saved Pages are profile-free URL records that survive tab close and restart. *(B1 is integrated and B2 is in serial integration: `Cmd-D` toggles the active page, the Palette ranks the global Saved Page rows between Spaces and History, opening a result creates exactly one fresh regular tab in the active Space/Profile, and the current functional suites are green. B3 owns the Library UI. B2 performance code, budgets, baselines, and historical report-only samples remain preserved, but current integration performance acceptance is Patrick-deferred and not passed.)*
- [x] Default-browser registration and external URL opening through the completed [external-URL open contract](../tasks/external-url-open-contract.md) and native integration `e5dab25`.
- [ ] Native menus, Services, broader window lifecycle, and app updates. The BBR-owned/WebKit-owned context-menu boundary is integrated and its bounded [Page-background menu follow-up](../tasks/page-background-context-menu.md) remains dependency-gated; WebKit-native element menus remain untouched.
- [ ] Define a public-API-only [portable media-session capability contract](../tasks/media-session-capability-contract.md), then deliver the capability-driven [sidebar media control](../tasks/sidebar-media-control.md); no SPI, inferred transport controls, or fake duration.
- [ ] Define profile-isolated site keys, settings overrides, and zoom lifecycle through the bounded [per-site settings and zoom contract](../tasks/site-settings-zoom-contract.md), then integrate persistence, native zoom, and UI.
- [ ] Implement popup/new-window policy and target-blank flows, including the dedicated [Link Preview / Peek](../tasks/link-preview-peek.md) lifecycle for eligible user-created links. Requests resolved to a regular tab establish the session-local opener provenance consumed by [web-content gesture Back-at-root return](../tasks/web-content-navigation-gestures.md); Peek/external/block/background policy remains owned here, not by the gesture layer.
- [ ] Manual representative-site matrix: login, media, editors, conferencing, upload/download, payments, passkeys.
- [ ] Harden all empty, loading, error, cancellation, and destructive states.
- [ ] Harden the one-per-window native Portal lifecycle, focus, multi-window ownership, display geometry, and teardown through the bounded [Portal host hardening packet](../tasks/native-portal-hardening.md).

## Deferred architecture options

- [Profile-scoped singleton favorite runtime](../tasks/profile-scoped-favorite-runtime.md) — optional post-MVP ownership model; the current concrete `TabId` / owning-space contract remains authoritative until an explicit migration decision.

## Explicitly post-alpha unless reprioritized

- Extensions.
- Cloud sync.
- Mobile clients.
- AI features.
- Reader mode.
- Built-in ad blocking.
- Theme marketplace.
