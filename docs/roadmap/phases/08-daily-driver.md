# Phase 8 — Daily-driver browser essentials

Parent: [Roadmap](../../../ROADMAP.md) · UI standards: [UI design-system track](../tracks/ui-design-system.md)

## Outcome

BBR covers the workflows required for a credible macOS alpha rather than only tab organization.

## Checklist

- [x] Deliver downloads destination, progress, history, reveal, retry, and failure UI through the bounded [downloads sidebar panel](../tasks/downloads-sidebar-panel.md) — including the settled post-MVP follow-up (terminal-history persistence, durable Clear/Delete, cancelled Restart, active-only badge).
- [ ] Deliver find in page with keyboard navigation and result count through the bounded [find-in-page Portal bar](../tasks/find-in-page-portal-bar.md).
- [ ] Define the portable request/decision authority through the bounded [permissions contract](../tasks/permissions-contract.md), then deliver camera, microphone, location, notification, and clipboard surfaces where supported.
- [ ] Define HTTP authentication and supported client-certificate request/response authority through the bounded [authentication challenges contract](../tasks/auth-challenges-contract.md), then implement native delegates and credential surfaces.
- [ ] Define portable upload/save request-completion semantics through the bounded [file upload/save-panel contract](../tasks/file-panels-contract.md), then implement native panel orchestration as a separate adapter slice.
- [ ] Define the typed taxonomy and legal recovery actions through the bounded [web error and recovery contract](../tasks/error-surface-contract.md), then deliver TLS, navigation, offline, and process-crash surfaces.
- [ ] History UI and clear-data controls per profile.
- [ ] Default-browser registration and external URL opening, starting with the portable [external-URL open contract](../tasks/external-url-open-contract.md).
- [ ] Native menus, Services, window lifecycle, and app updates, including the integrated [BBR-owned context-menu boundary over web content](../tasks/bbr-context-menus-over-web.md) and its bounded [Page-background menu follow-up](../tasks/page-background-context-menu.md). WebKit-native element menus remain untouched.
- [ ] Define a public-API-only [portable media-session capability contract](../tasks/media-session-capability-contract.md), then deliver the capability-driven [sidebar media control](../tasks/sidebar-media-control.md); no SPI, inferred transport controls, or fake duration.
- [ ] Define profile-isolated site keys, settings overrides, and zoom lifecycle through the bounded [per-site settings and zoom contract](../tasks/site-settings-zoom-contract.md), then integrate persistence, native zoom, and UI.
- [ ] Implement popup/new-window policy and target-blank flows, including the dedicated [Link Preview / Peek](../tasks/link-preview-peek.md) lifecycle for eligible user-created links.
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
