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
- [x] Render and switch spaces with unmistakable profile identity. *(Visible
      space hierarchy on the proven shell: per-row §12 profile chips with
      round-robin hues, hairlines between profile groups, active/selected
      rails, pointer + `↑/↓`/Enter + `Cmd-Shift-N` activation through the
      production paths, empty-space pane naming, and cross-profile view-only
      switches that never recreate warm sessions. Proven by `just
      e2e-shell-spaces` and the gate-enforced `space_switch` performance
      scenario.)*
- [x] Preserve the active tab per space without recreating warm sessions.
      *(Per-space active tab + live session are restored on activation; warm
      switching only show/hides. Same-profile moves keep the WebViewId;
      cross-profile moves follow the recreation policy.)*
- [x] Add adjustable, collapsible sidebar behavior with presentation-commit benchmarks for open, close/toggle, and resize. *(Cmd-B toggle existed; the slice adds the DESIGN §5/§11.4 width contract (200–360 pt adjustable, 260 default, 64 pt native-content floor), a visible 4 pt drag hairline with pointer drag through the production GPUI listener path, keyboard equivalents on the focused divider (`←`/`→` step, `0` reset), collapse that preserves the last expanded width and expand that restores it, and narrow-window clamping that never hands the native slot zero or negative geometry. Proven by `just e2e-shell-sidebar` (pointer drag, keyboard parity, collapse/reopen restoration, compact-window clamp, engine-received native content rect, selected WebViewId/session preservation, chrome focus contract) and the `sidebar_resize`/`sidebar_restore` performance scenarios with the toggle already gate-enforced.)*
- [ ] Add drag reorder and move between spaces after keyboard equivalents exist.
- [ ] Handle empty space, first launch, many tabs, long titles, missing favicons, and crashed pages. *(Empty space, long titles, missing favicons, and crashed
      pages are covered with E2E evidence; 500-tab density remains with the dense-list slice — do not mark complete without measured frame-time evidence.)*

## Quality acceptance

- [ ] Shell follows the [UI design-system track](../tracks/ui-design-system.md).
- [ ] Keyboard and VoiceOver acceptance in [Accessibility track](../tracks/accessibility.md) is green.
- [ ] Tab/space E2E covers open, switch, close, reopen, pin, reorder, and profile labels.
- [x] Performance E2E covers warm tab/space switching, active-tab close, and sidebar open/close/toggle through the production presentation path. *(Warm tab switching, warm space switching, active-tab close, sidebar toggle, pointer selection, reopen, keyboard activation, sidebar drag resize, and collapse/restore are all measured through the production presentation path.)*
- [x] Warm tab/space switching and sidebar visibility changes meet the budgets in the [performance contract](../../product/performance-contract.md). *(Warm tab/space switching, sidebar toggle, sidebar drag resize, and collapse/restore are gate-enforced against the 50/100 ms warm budgets; the dense-list scenario remains.)*
- [ ] Sidebar remains responsive with 500 persisted tabs and 50 live/warm sessions, with measured frame-time evidence.

## When this UI pass happens

This is the **first major beauty pass**. It happens immediately after portable orchestration and multi-session WebKit are reliable, not after every browser feature is complete.
