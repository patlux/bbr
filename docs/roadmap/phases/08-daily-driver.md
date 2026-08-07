# Phase 8 — Daily-driver browser essentials

Parent: [Roadmap](../../../ROADMAP.md) · UI standards: [UI design-system track](../tracks/ui-design-system.md)

## Outcome

BBR covers the workflows required for a credible macOS alpha rather than only tab organization.

## Checklist

- [ ] Deliver downloads destination, progress, history, reveal, retry, and failure UI through the bounded [downloads sidebar panel](../tasks/downloads-sidebar-panel.md).
- [ ] Deliver find in page with keyboard navigation and result count through the bounded [find-in-page Portal bar](../tasks/find-in-page-portal-bar.md).
- [ ] Camera, microphone, location, notification, and clipboard permission surfaces where supported.
- [ ] Authentication challenges and supported client-certificate flows.
- [ ] File upload and save panels.
- [ ] TLS, navigation, offline, and process-crash error surfaces with recovery actions.
- [ ] History UI and clear-data controls per profile.
- [ ] Default-browser registration and external URL opening.
- [ ] Native menus, Services, window lifecycle, and app updates, including [BBR-owned context menus over web content](../tasks/bbr-context-menus-over-web.md).
- [ ] Zoom and per-site settings.
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
