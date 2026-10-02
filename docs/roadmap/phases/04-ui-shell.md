# Phase 4 — Vertical tabs, spaces, and first intentional UI

Parent: [Roadmap](../../../ROADMAP.md) · Canonical UI checklist: [UI design-system track](../tracks/ui-design-system.md) · Accessibility: [Accessibility track](../tracks/accessibility.md) · Performance: [Performance contract](../../product/performance-contract.md)

## Outcome

BBR becomes a coherent browser shell with real tabs and spaces—not a styled integration spike.

## Performance and design foundation first

- [x] Integrate the serial [GPUI-CE + gpui-component Root anchor](../tasks/gpui-ce-components-anchor.md), preserving native WKWebViews and every existing real-app gate before palette/sidebar/package lanes branch. *(Integrated at post-anchor SHA `f89a479`; `just verify`, testkit Clippy, vendored component tests, affected E2E, and canonical `just e2e` are green.)*
- [x] Integrate the [native Portal host above WebKit](../tasks/native-portal-host.md): one host per browser window, deferred-GPUI versus NativeAboveWeb ownership, child attachment/frame following/native hit-test precedence/dismissal, and unchanged browser-session identity are proven by the packet's completed contract and real-app evidence. Broader lifecycle hardening remains separately owned by [native Portal hardening](../tasks/native-portal-hardening.md).
- [x] Implement the bounded [browser-MVP sidebar UI packet](../tasks/browser-mvp-sidebar-ui.md): Favorites → Pinned → Folders → Tabs, footer controls and Plus menu shipped in the U2 integration; the later product revision replacing the Spaces toggle is completed by F2a below.
- [x] Turn the sidebar address into a display control and move all text entry into the palette through the [address control and palette packet](../tasks/url-address-control-palette.md): integrated at `963e94f`, including the domain-only pill, overlay geometry, selected full URL, no control focus/text entry, identity preservation, real-app E2E and unchanged performance budgets. The later G3 Command integration preserves that behavior while moving only the floating palette/overlay presentation to `bbr_ui::surfaces::command`; post-merge Command is 25/25, Portal is 7/7, and Gallery is 42/42 on the exact integrated tree.
- [x] Deliver the post-U2 product revision in the [footer space strip, appearance, and swipe packet](../tasks/space-footer-strip-appearance-ui.md): remove the `[Spaces]` reveal toggle, present spaces permanently as curated identity chips in a horizontally scrollable footer strip, tint sidebar chrome from the active space, and route pointer, keyboard, and swipe through the one unchanged warm space-switch authority. *(Integrated at `7180128`: permanent strip, Appearance/Tint, zero-delta-Ended Swipe Commit/Cancel, production drag-to-chip, `swipe_commit`, icon union, latest-main navigation-cover union, affected real-app suites, and unchanged-budget performance evidence independently reviewed and green.)*
- [x] Complete the [reference-faithful production shell](../tasks/visual-shell-reference.md) over the integrated Daily-driver Sidebar foundation: the authoritative light macOS rail, public full-size titlebar integration, active-Space hierarchy, reference-calibrated Favorites/tabs/folders/footer, real 2× production-path captures and independent visual review. *(Exact reviewed tree `5f2b8a04…` integrated as `4d1a5343`; the packet records the region-by-region capture ledger, affected real-app suites, and post-merge checks. Performance execution, startup, packaging, Accessibility, and Split remain explicitly outside this completed slice.)* The later bounded [Folder context menu](../tasks/folder-context-menu.md) remains deferred. These surfaces must not create a second sidebar/menu/navigation/Space/Favorite authority; the complete BBR-owned header disappears when the Sidebar is collapsed.
- [ ] Complete the [real-app performance E2E foundation](../tasks/performance-e2e-foundation.md) before broad shell implementation.
- [ ] Prove startup presents the complete sidebar and selected tab's visible native web view in the same first usable frame within the product budget.
- [x] Establish and document the visual direction in [DESIGN.md](../../../DESIGN.md) before broad shell implementation.
- [x] Define semantic color, typography, spacing, radius, icon, and motion tokens.
- [x] Define light, dark, increased-contrast, and reduced-motion behavior.
- [x] Define density rules for small, normal, and very large tab collections.
- [x] Define reusable states: hover, active, selected, loading, attention, error, disabled, drag target, and keyboard focus.

## Reference-design alignment (2026-08-13)

Patrick's dark reference capture
(`/Users/patwoz/Library/Application Support/CleanShot/media/media_lbaS6F07Ws/Screenshot_2026-08-12_21-45-38@2x.png`)
extends the integrated light shell: same hierarchy and geometry, system-driven
appearance, real favicons, reference row rhythm and drawn identity glyphs.
Settled decisions: the production theme follows the macOS system appearance
(both modes supported). Patrick's later 2026-08-14 design decision defines
folders as grouped pinned tabs inside one organized area; loose pinned rows
remain above folder rows, so arbitrary mixed interleaving is still rejected.
The portable folder-member/pinned invariant is implemented on `main` through
the [folder membership packet](../tasks/folder-pinned-invariant.md); only its
performance gate waits for the performance-execution override to lift.

- [x] Follow the macOS system appearance with the complete dark token set via the [system-appearance production theme](../tasks/system-appearance-theme.md).
- [x] Present real site favicons in every favicon slot via the [real favicon pipeline](../tasks/real-favicon-pipeline.md).
- [ ] Keep acquired favicons across a relaunch via the [favicon disk cache](../tasks/favicon-disk-cache.md) (Patrick, 2026-10-01).
- [x] Match the reference row pitch, folder anatomy, rest states and identity-glyph catalog via the [sidebar reference polish](../tasks/sidebar-reference-polish.md). *(Integrated through merge `35c91189`; Sidebar 15/15, Dense 3/3, Organize 2/2 plus Drag-Chip 4/4, Spaces 21/21, Gallery 45/45, and the packet's non-GUI gates passed on the final tree. The later folder/pinned domain invariant is a separate open slice.)*

## Kept places: closed pins and Favorites (2026-09-30)

Patrick's decision: pinned tabs and Favorites are kept places, not open
pages. A closed pinned tab stays greyed in place and reopens its pinned URL;
a Favorite leaves the tab list (the tile is the tab) and greys out when
closed; the unplanned row pin control is removed.

- [ ] Deliver [closed pinned tabs and Favorites that leave the list](../tasks/closed-pins-and-favorites.md).
- [ ] Drag pinned rows and Favorite tiles [back out of the pinned area](../tasks/drag-back-out-of-pins.md) (Patrick, 2026-10-01).

## v2 visual refresh and motion (2026-09-28)

After the Arc study in Paper, Patrick adopted DESIGN §18 (decisions D1–D11):
one tinted body, a lifted selection, translucent veils, calmer floating
surfaces and a bounded set of after-commit motion moments. Slices and
acceptance live in the program packet.

- [x] Deliver the [v2 visual refresh and motion program](../tasks/v2-visual-refresh-program.md) (S0–S11).
- [x] Align the shipped v2 shell with the Paper references via [S12 Paper alignment](../tasks/v2-paper-alignment.md).
- [x] Design the complete browser in Paper — every surface, state and process, with a coverage matrix and a delta ledger against the code — via the [complete browser blueprint](../tasks/paper-browser-blueprint.md) (2026-09-28).

The sidebar media control is prioritized immediately after these slices; its
canonical owner stays Phase 8 through the
[media-session capability contract](../tasks/media-session-capability-contract.md)
→ [sidebar media control](../tasks/sidebar-media-control.md) sequence.

## Shell and interaction

The bounded [Portal window-lifecycle regression](../tasks/portal-window-lifecycle.md)
keeps the Command Palette attached to its owning browser window across activation.

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
- [x] Add adjustable, collapsible sidebar behavior with presentation-commit benchmarks for open, close/toggle, and resize. *(Cmd-B toggle existed; the slice adds the DESIGN §5/§11.4 width contract (200–360 pt adjustable, 260 default, 64 pt native-content floor), a visible 4 pt drag hairline with pointer drag through the production GPUI listener path, keyboard equivalents on the focused divider (`←`/`→` step, `0` reset), collapse that preserves the last expanded width and expand that restores it, and narrow-window clamping that never hands the native slot zero or negative geometry. Proven by `just e2e-shell-sidebar` (pointer drag, keyboard parity, collapse/reopen restoration, compact-window clamp, engine-received native content rect, selected WebViewId/session preservation, chrome focus contract) and the `sidebar_resize`/`sidebar_restore` performance scenarios with the toggle already gate-enforced. The G3 composition cutover later moved the rail composition to `bbr_ui::sidebar` with `bbr-app` as assembler; the real-app composition E2E (full rail region/semantic ordering, command/downloads child composition, split collapse/expand geometry, dense window) keeps this acceptance green.)*
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

Visual regression tooling is owned by [Reusable visual snapshots](../tasks/visual-snapshot-tooling.md) and its [Native Gallery coverage extension](../tasks/gallery-visual-coverage.md) in the [Testing track](../tracks/testing.md); it supplements semantic real-app E2E. The first migration runtime gate is the bounded [Tokens / Checkbox / Sidebar pilot](../tasks/gallery-visual-pilot.md), not full-coverage or baseline acceptance.

- [ ] Shell follows the [UI design-system track](../tracks/ui-design-system.md).
- [ ] Keyboard and VoiceOver acceptance in [Accessibility track](../tracks/accessibility.md) is green.
- [x] Tab/space E2E covers open, switch, close, reopen, pin, reorder, and profile labels. *(Open/select/close/reopen via `just e2e-shell-tabs`, switching + identity via `just e2e-shell-spaces{,_hint,_keyboard}` (including the §11.3 cross-context command-field hint: copy, chip/hint ids, typed semantic node, deterministic clear boundaries, same-profile absence, the separate staged-move hint, and the empty cross-profile branch), and pin/unpin, legal/illegal reorder, same-profile/cross-profile moves, cancellation, focus, session identity, and isolation via `just e2e-shell-organize`.)*
- [x] Performance E2E covers warm tab/space switching, active-tab close, and sidebar open/close/toggle through the production presentation path. *(Warm tab switching, warm space switching, cross-profile space switching with the §11.3 hint presented, active-tab close, sidebar toggle, pointer selection, reopen, keyboard activation, sidebar drag resize, collapse/restore, tab pin toggle, region-bounded reorder, same-profile/cross-profile moves, pointer hover/press state transitions, and dense-list scroll/select are all measured through the production presentation path.)*
- [x] Warm tab/space switching and sidebar visibility changes meet the budgets in the [performance contract](../../product/performance-contract.md). *(Warm tab/space switching, sidebar toggle, sidebar drag resize, and collapse/restore are gate-enforced against the 50/100 ms warm budgets; the dense scroll/select scenarios are wired into the gate with contract-budget seeds — measured frame-time evidence on the canonical runner remains.)*
- [ ] Sidebar remains responsive with 500 tab identities and 50 live/warm sessions (in-memory dense state; persisted restoration is Phase 7), with measured frame-time evidence. *(The dense lifecycle + bounded-window render are implemented and the `dense_scroll`/`dense_select` scenarios are wired into the perf gate; this acceptance requires the measured frame-time evidence from the canonical benchmark runner.)*

## When this UI pass happens

This is the **first major beauty pass**. It happens immediately after portable orchestration and multi-session WebKit are reliable, not after every browser feature is complete.
