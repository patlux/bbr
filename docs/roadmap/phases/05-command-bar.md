# Phase 5 — Unified command bar

Parent: [Roadmap](../../../ROADMAP.md) · UI standards: [UI design-system track](../tracks/ui-design-system.md)

## Outcome

One fast, keyboard-first surface handles navigation, search, tab switching, spaces, history, and browser commands.

## Checklist

- [ ] Add portable `bbr-command` registry, parsing, ranking, and result types.
- [ ] Implement location, new-tab, switcher, and action modes.
- [ ] Select the current URL on `Cmd-L`.
- [ ] Make `Cmd-T` enter a distinct new-tab flow.
- [ ] Rank exact/open tabs, current-space/pinned tabs, other spaces, history, actions, and navigate/search fallback.
- [ ] Label cross-space results with profile identity.
- [ ] Support arrows, Enter, modified Enter, Escape, and origin-focus restoration.
- [ ] Add cancellable debounce without blocking the render path.
- [ ] Design loading, empty, no-result, error, and destructive-command states.
- [ ] Add table-driven parser/ranking tests and real-app command-bar E2E.
- [ ] Keep first local result below 50 ms for 10,000 history entries and 500 tabs.

## Phase exit

- [ ] Every command-bar path is keyboard complete, accessible, and visually consistent with Phase 4.
- [ ] Focus always returns to the originating web pane or browser control.
