# Task packet — Link Preview / Peek

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[Native integration track](../tracks/native-integration.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Accessibility track](../tracks/accessibility.md) ·
[Testing track](../tracks/testing.md)

## Objective

Open eligible link-created windows in a temporary, focused preview above the
current page instead of immediately adding a regular tab. The originating tab
and its native session stay intact underneath. The user can dismiss the preview
without changing tab state or promote it to a regular tab without reloading or
losing the preview's navigation state.

## Product behavior

- [ ] A user-initiated eligible `target="_blank"` or `window.open` request opens
      Peek by default instead of creating a regular tab immediately.
- [ ] Provide an explicit keyboard/pointer action to open an ordinary new tab
      directly and an explicit action to request Peek for a normal link.
- [ ] Cover the complete current web-content area with the preview while keeping
      browser chrome and preview controls operable.
- [ ] Keep the originating tab selected and its WKWebView warm underneath; it
      must not receive pointer, keyboard, scroll, drag, or Accessibility focus
      while Peek is open.
- [ ] Give the preview its own back/forward/reload/navigation state and route
      browser commands to it while it owns focus.
- [ ] Escape and the close control dismiss Peek, destroy its temporary native
      session, and restore focus to the exact originating control or web view.
- [ ] “Open as Tab” adopts the existing preview session into the originating
      space and profile without reload, preserving URL, history, form state,
      scroll position, and same-document state.
- [ ] If the origin tab/space/window disappears, apply a deterministic policy:
      close the preview unless the user has already promoted it.
- [ ] Support at most one Peek per pane initially; a new eligible request from
      inside Peek replaces or navigates the existing preview according to the
      documented policy rather than stacking unbounded previews.

## Eligibility and safety policy

- [ ] Add a typed portable new-window request containing source tab/session,
      target URL, disposition, user-gesture state, and relevant WebKit metadata.
- [ ] Keep policy decisions outside the macOS delegate: the adapter reports the
      request, while portable orchestration chooses Peek, regular tab, external
      handling, or rejection.
- [ ] Create Peek in the source profile's `WKWebsiteDataStore`; never copy
      cookies, credentials, or session state across profile boundaries.
- [ ] Do not route downloads, non-web schemes, authentication/challenge windows,
      permission flows, or explicitly sized/tool-style popups into Peek unless a
      later documented policy marks them safe.
- [ ] Preserve WebKit opener/security semantics deliberately; do not expose a
      stronger opener relationship than an ordinary browser-created tab would.
- [ ] Surface blocked or unsupported popup requests without silently losing the
      user's action.

## Architecture and lifecycle

- [ ] Model Peek as transient browser state, not as a persisted `Tab`, recently
      closed entry, sidebar row, or history item merely because it was opened.
- [ ] Give each preview a typed identity and disposable `WebViewId` owned by the
      controller/session registry until dismissal or promotion.
- [ ] Extend `WebEngine` with the minimal create/show/hide/focus/destroy/adopt
      operations or outcomes needed for transient sessions without leaking
      AppKit/WebKit types into portable crates.
- [ ] Stack the preview WKWebView as a native child view above the originating
      WKWebView; never screenshot or texture-compose page content into GPUI.
- [ ] Define explicit z-order and clipping between GPUI chrome, Peek controls,
      the preview WKWebView, and the covered originating WKWebView.
- [ ] Destroy delegates, KVO observations, native views, and registry mappings on
      every dismissal, failed creation, window close, and process-termination
      path.
- [ ] Promotion transfers ownership atomically so the existing native session is
      neither recreated nor temporarily orphaned.

## UI, input, and accessibility

- [ ] Design Peek as an intentional BBR surface with clear temporary status,
      close, “Open as Tab”, URL/origin context, loading, failure, and crash states.
- [ ] Support light/dark, increased contrast, reduced motion, compact windows,
      and safe-area/titlebar geometry.
- [ ] Use a bounded entrance/exit transition; reduced motion uses an immediate or
      non-spatial state change.
- [ ] Trap browser-chrome focus within Peek and its page while open, while still
      allowing predictable transfer between preview controls and preview content.
- [ ] Expose semantic role/name/state for the preview and announce opening,
      navigation failure, promotion, and dismissal to VoiceOver.
- [ ] Make every pointer action keyboard-operable; document shortcuts and avoid
      conflicts with page-reserved key handling.

## Automated evidence

- [ ] Portable controller tests cover request policy, transient identity,
      command routing, replacement, dismissal, promotion, origin removal, and
      profile isolation.
- [ ] macOS adapter tests prove new-window metadata reaches the portable policy
      and that preview sessions use the correct website data store.
- [ ] Real-app E2E opens a fixture `target="_blank"` link and proves Peek appears
      above the current native view without creating a regular tab.
- [ ] E2E proves the covered page receives no input and retains the same
      `WebViewId`, URL, and page state after Peek closes.
- [ ] E2E navigates inside Peek, promotes it, and proves the same preview
      `WebViewId` becomes a regular tab without another page load.
- [ ] E2E covers Escape/focus restoration, replacement policy, popup exclusions,
      failure/crash cleanup, and cross-profile isolation.
- [ ] Add a bounded `just e2e-peek` suite and include it in canonical sequential
      `just e2e` once stable; GUI suites remain sequential.
- [ ] Extend performance E2E with request-to-present, dismiss-to-origin-present,
      and promotion-to-tab-present milestones.
- [ ] Run `just verify`, `just e2e-peek`, all affected suites, and canonical
      sequential `just e2e` before checking this task complete.

## Non-goals

- Persisting open previews across app restart.
- Nested or multiple stacked previews in one pane.
- Replacing permission prompts, authentication panels, downloads, or arbitrary
  JavaScript utility windows with Peek.
- Cross-profile preview-session transfer.
- Pixel-composed webpage previews.
