# Track — Accessibility and input

Parent phases: [Phase 4](../phases/04-ui-shell.md), [Phase 5](../phases/05-command-bar.md), [Phase 6](../phases/06-split-view.md), [Phase 8](../phases/08-daily-driver.md), [Phase 9](../phases/09-quality-release.md)

## Implemented foundation

- [x] Native WKWebView remains available to platform Accessibility APIs.
- [x] GPUI and AppKit focus can move explicitly between browser controls and page content.
- [x] Core navigation shortcuts work while WKWebView owns native focus.

## Semantic browser controls

- [ ] Stable semantic IDs shared by internal testkit and platform Accessibility.
- [ ] Correct roles, names, values, enabled state, selection, and hierarchy.
- [ ] Predictable focus order across sidebar, command bar, panes, dialogs, and native web content.
- [ ] Visible keyboard focus that is not color-only.
- [ ] Every pointer action has a keyboard equivalent.
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
