# Phase 4 — Vertical tabs, spaces, and first intentional UI

Parent: [Roadmap](../../../ROADMAP.md) · Canonical UI checklist: [UI design-system track](../tracks/ui-design-system.md) · Accessibility: [Accessibility track](../tracks/accessibility.md) · Performance: [Performance contract](../../product/performance-contract.md)

## Outcome

BBR becomes a coherent browser shell with real tabs and spaces—not a styled integration spike.

## Performance and design foundation first

- [ ] Complete the [real-app performance E2E foundation](../tasks/performance-e2e-foundation.md) before broad shell implementation.
- [ ] Prove startup presents the complete sidebar and selected tab's visible native web view in the same first usable frame within the product budget.
- [x] Establish and document the visual direction in [DESIGN.md](../../../DESIGN.md) before broad shell implementation.
- [x] Define semantic color, typography, spacing, radius, icon, and motion tokens.
- [x] Define light, dark, increased-contrast, and reduced-motion behavior.
- [x] Define density rules for small, normal, and very large tab collections.
- [x] Define reusable states: hover, active, selected, loading, attention, error, disabled, drag target, and keyboard focus.

## Shell and interaction

- [x] Render pinned and regular tabs for the active space. *(Visible vertical tab
      surface: pinned region above the regular region with section labels and a
      hairline separator, both driven from the controller snapshot. Pin commands
      (keyboard/drag) are a later slice; a testkit-only pin seam seeds the region
      in E2E.)*
- [x] Show favicon/title/loading/crash state from the controller. *(Deterministic
      monogram favicon tiles (§7.2), title/URL fallbacks (§11.9), loading dim +
      spinner (§10.5), row error state + crash panel (§10.7).)*
- [x] Open, select, close, and reopen tabs from pointer and keyboard. *(Pointer
      rows and close controls through the production listener path; `Cmd-T`/Enter,
      `↑`/`↓`, `⌫`, `Cmd-W`, `Cmd-Shift-T`. Focus domain preserved per the
      interaction contract.)*
- [x] Make `Cmd-T` stage a new-tab destination until Enter and `Cmd-W` close the active tab with focus preservation. *(Visible tab controls remain open.)*
- [ ] Render and switch spaces with unmistakable profile identity. *(Next shell
      slice on the proven vertical-tab surface; `bbr-space-row`/profile-chip ids
      are specified in DESIGN.md §11.3.)*
- [ ] Preserve the active tab per space without recreating warm sessions.
- [ ] Add adjustable, collapsible sidebar behavior with presentation-commit benchmarks for open, close/toggle, and resize. *(Cmd-B toggle exists; resize/drag and the remaining benchmarks land with the sidebar slice.)*
- [ ] Add drag reorder and move between spaces after keyboard equivalents exist.
- [ ] Handle empty space, first launch, many tabs, long titles, missing favicons, and crashed pages. *(Empty space, long titles, missing favicons, and crashed
      pages are covered with E2E evidence; 500-tab density remains with the dense-list slice — do not mark complete without measured frame-time evidence.)*

## Quality acceptance

- [ ] Shell follows the [UI design-system track](../tracks/ui-design-system.md).
- [ ] Keyboard and VoiceOver acceptance in [Accessibility track](../tracks/accessibility.md) is green.
- [ ] Tab/space E2E covers open, switch, close, reopen, pin, reorder, and profile labels.
- [ ] Performance E2E covers warm tab/space switching, active-tab close, and sidebar open/close/toggle through the production presentation path.
- [ ] Warm tab/space switching and sidebar visibility changes meet the budgets in the [performance contract](../../product/performance-contract.md).
- [ ] Sidebar remains responsive with 500 persisted tabs and 50 live/warm sessions, with measured frame-time evidence.

## When this UI pass happens

This is the **first major beauty pass**. It happens immediately after portable orchestration and multi-session WebKit are reliable, not after every browser feature is complete.
