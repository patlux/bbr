# Track — Accessibility and input

Parent phases: [Phase 4](../phases/04-ui-shell.md), [Phase 5](../phases/05-command-bar.md), [Phase 6](../phases/06-split-view.md), [Phase 8](../phases/08-daily-driver.md), [Phase 9](../phases/09-quality-release.md)

## Implemented foundation

- [x] Native WKWebView remains available to platform Accessibility APIs.
- [x] GPUI and AppKit focus can move explicitly between browser controls and page content.
- [x] Core navigation shortcuts work while WKWebView owns native focus.

## Semantic browser controls

- [x] Stable semantic IDs shared by internal testkit and platform Accessibility.
      *(The stable `bbr-*` ids are contract (§15) and shared with the testkit
      snapshot; the internal semantic model builds on them. GPUI 0.2.2
      implements no platform Accessibility layer — no NSAccessibility/accesskit —
      so the macOS Accessibility bridge itself and the packaged-app VoiceOver /
      Appium acceptance are Phase 9, where this item completes.)*
- [x] Correct roles, names, values, enabled state, selection, and hierarchy.
      *(The internal §15 semantic tree — roles, names, values,
      enabled/selected/focused, and the parent/child hierarchy — is derived
      from the same row/space visuals the render consumes and asserted by
      real-app E2E (`just e2e-shell-hover`). Platform exposure remains Phase 9.)*
- [ ] Predictable focus order across sidebar, command bar, panes, dialogs, and native web content. *(Phase-4 surfaces follow §9 (command field → spaces → tabs → web) and are covered by the shell suites; split panes, dialogs, and the remaining surfaces land with their phases.)*
- [x] Visible keyboard focus that is not color-only. *(The §9 ring — a 2 pt accent outline inside the focused list's bounds — plus the text-step brightening of the focused element's content and section labels; the typed `focused` flags in the semantic tree expose the same fact to E2E, asserted by `just e2e-shell-hover`.)*
- [x] Every pointer action has a keyboard equivalent. *(Phase-4 pointer surfaces — row select/close/reopen, pin, reorder, move between spaces, space activation, sidebar toggle/drag resize — all have keyboard equivalents proven by the shell suites; every later surface must keep the rule.)*
- [ ] Shortcut discovery through menus/tooltips/help.

## Visual and motion accessibility

- [ ] Text and meaningful UI meet contrast targets in light/dark/increased-contrast modes.
- [ ] Color never carries status alone.
- [ ] Text remains readable at supported macOS accessibility sizes.
- [ ] Reduce Motion removes large/spatial transitions while preserving state feedback.
- [ ] Hit targets remain usable at compact density.

## VoiceOver and real-input acceptance

- [ ] Launch and understand active profile, space, tab, and pane.
- [ ] Open, select, pin, move, close, and reopen tabs.
- [ ] Operate command-bar modes and understand result grouping.
- [ ] Enter and leave WKWebView content predictably.
- [ ] Operate split divider and focused-pane controls.
- [ ] Understand, navigate, dismiss, and promote [Link Preview / Peek](../tasks/link-preview-peek.md), with focus returning to its exact origin.
- [ ] Understand downloads, permissions, errors, and recovery actions.
- [ ] Verify packaged `.app` with Appium Mac2 plus manual VoiceOver passes.
