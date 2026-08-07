# Phase 4 — Vertical tabs, spaces, and first intentional UI

Parent: [Roadmap](../../../ROADMAP.md) · Canonical UI checklist: [UI design-system track](../tracks/ui-design-system.md) · Accessibility: [Accessibility track](../tracks/accessibility.md) · Performance: [Performance contract](../../product/performance-contract.md)

## Outcome

BBR becomes a coherent browser shell with real tabs and spaces—not a styled integration spike.

## Performance and design foundation first

- [x] Integrate the serial [GPUI-CE + gpui-component Root anchor](../tasks/gpui-ce-components-anchor.md), preserving native WKWebViews and every existing real-app gate before palette/sidebar/package lanes branch. *(Integrated at post-anchor SHA `f89a479`; `just verify`, testkit Clippy, vendored component tests, affected E2E, and canonical `just e2e` are green.)*
- [ ] Implement the bounded [browser-MVP sidebar UI packet](../tasks/browser-mvp-sidebar-ui.md): Favorites → Pinned → Folders → Tabs, exact `[Downloads] [Spaces] [+]` footer, and real New Space/New Folder menu actions.
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
      hairline separator, both driven from the controller snapshot. The pin
      commands and their keyboard/pointer equivalents shipped with the tab-
      organization slice.)*
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
      switches that never recreate warm sessions. The [cross-context
      profile-hint slice](../tasks/cross-context-profile-hint.md) completes
      DESIGN §11.3: a cross-profile activation renders the destination
      profile chip + "in Profile · <title>" at the top of the command field
      for one interaction with a deterministic interaction-driven lifetime,
      and same-profile switching never announces a transition. Proven by
      `just e2e-shell-spaces` and the gate-enforced `space_switch` /
      `space_switch_cross_profile` performance scenarios.)*
- [x] Preserve the active tab per space without recreating warm sessions.
      *(Per-space active tab + live session are restored on activation; warm
      switching only show/hides. Same-profile moves keep the WebViewId;
      cross-profile moves follow the recreation policy.)*
- [x] Add adjustable, collapsible sidebar behavior with presentation-commit benchmarks for open, close/toggle, and resize. *(Cmd-B toggle existed; the slice adds the DESIGN §5/§11.4 width contract (200–360 pt adjustable, 260 default, 64 pt native-content floor), a visible 4 pt drag hairline with pointer drag through the production GPUI listener path, keyboard equivalents on the focused divider (`←`/`→` step, `0` reset), collapse that preserves the last expanded width and expand that restores it, and narrow-window clamping that never hands the native slot zero or negative geometry. Proven by `just e2e-shell-sidebar` (pointer drag, keyboard parity, collapse/reopen restoration, compact-window clamp, engine-received native content rect, selected WebViewId/session preservation, chrome focus contract) and the `sidebar_resize`/`sidebar_restore` performance scenarios with the toggle already gate-enforced.)*
- [x] Add drag reorder and move between spaces after keyboard equivalents exist. *(Complete keyboard commands — `Cmd-Shift-P` pin/unpin toggle, `Cmd-Shift-↑/↓` region-bounded reorder, `Cmd-Shift-M` staged move + Enter commit / Escape cancel — plus the pointer drag: row press through the production listeners, an §10.9 insertion tick at the typed slot with an invalid variant for boundary crossings, drag-target space rows with the §12.3 destination chip, same-profile moves that keep the live WebViewId, cross-profile moves that recreate exactly once in the destination store (cookie isolation proven), Escape cancellation, and the chrome focus domain throughout. Proven by `just e2e-shell-organize`.)*
- [x] Render a dense-500 list with bounded lifecycle, windowed rendering, recency, and gate-wired scroll/select benchmarks. *(The [dense-list slice](../tasks/dense-list-slice.md): 500 identities with exactly 50 live sessions and 450 suspended through the production controller paths, a bounded concurrent-session peak during preparation, a bounded-window regular region that follows the production scroll offset, a pinned + recently-used keep set with warm-selection reuse and exactly-one-session suspended recreation, keyboard traversal with instant scroll-into-view, edge states, and the dense drag tick inside the window. Proven by `just e2e-shell-dense`/`e2e_shell_dense_edge` and the dense unit/contract suites; `dense_scroll`/`dense_select` are wired into the gate-enforced perf matrix with contract-budget seeds.)*
- [x] Render the §10.2 hover / §10.3 press states and the internal §15 semantic tree through the production paths. *(The hover wash + close reveal and the pressed wash derive from app state the production `on_hover` / mouse-down/up listeners drive; the typed semantic model — roles/names/values/enabled/selected/focused + hierarchy — mirrors the render from the same row/space visuals. Proven by `just e2e-shell-hover` and the `tab_hover`/`tab_press` scenarios in the gate-enforced perf matrix. Internal model only: GPUI 0.2.2 has no platform Accessibility layer, so the macOS Accessibility bridge and the packaged-app VoiceOver/Appium/manual acceptance are Phase 9.)*
- [ ] Handle empty space, first launch, many tabs, long titles, missing favicons, and crashed pages. *(Empty space, long titles, missing favicons, and crashed
      pages are covered with E2E evidence — including their typed semantic names —
      and the 500-identity in-memory density is implemented by the
      [dense-list slice](../tasks/dense-list-slice.md) with functional E2E; do not
      mark complete without measured frame-time evidence on the canonical runner.
      Persisted-restore acceptance is Phase 7, not Phase 4 — the dense state stays
      in-memory (500 identities / 50 live / 450 suspended).)*

## Quality acceptance

- [ ] Shell follows the [UI design-system track](../tracks/ui-design-system.md).
- [ ] Keyboard and VoiceOver acceptance in [Accessibility track](../tracks/accessibility.md) is green.
- [x] Tab/space E2E covers open, switch, close, reopen, pin, reorder, and profile labels. *(Open/select/close/reopen via `just e2e-shell-tabs`, switching + identity via `just e2e-shell-spaces{,_hint,_keyboard}` (including the §11.3 cross-context command-field hint: copy, chip/hint ids, typed semantic node, deterministic clear boundaries, same-profile absence, the separate staged-move hint, and the empty cross-profile branch), and pin/unpin, legal/illegal reorder, same-profile/cross-profile moves, cancellation, focus, session identity, and isolation via `just e2e-shell-organize`.)*
- [x] Performance E2E covers warm tab/space switching, active-tab close, and sidebar open/close/toggle through the production presentation path. *(Warm tab switching, warm space switching, cross-profile space switching with the §11.3 hint presented, active-tab close, sidebar toggle, pointer selection, reopen, keyboard activation, sidebar drag resize, collapse/restore, tab pin toggle, region-bounded reorder, same-profile/cross-profile moves, pointer hover/press state transitions, and dense-list scroll/select are all measured through the production presentation path.)*
- [x] Warm tab/space switching and sidebar visibility changes meet the budgets in the [performance contract](../../product/performance-contract.md). *(Warm tab/space switching, sidebar toggle, sidebar drag resize, and collapse/restore are gate-enforced against the 50/100 ms warm budgets; the dense scroll/select scenarios are wired into the gate with contract-budget seeds — measured frame-time evidence on the canonical runner remains.)*
- [ ] Sidebar remains responsive with 500 tab identities and 50 live/warm sessions (in-memory dense state; persisted restoration is Phase 7), with measured frame-time evidence. *(The dense lifecycle + bounded-window render are implemented and the `dense_scroll`/`dense_select` scenarios are wired into the perf gate; this acceptance requires the measured frame-time evidence from the canonical benchmark runner.)*

## When this UI pass happens

This is the **first major beauty pass**. It happens immediately after portable orchestration and multi-session WebKit are reliable, not after every browser feature is complete.
