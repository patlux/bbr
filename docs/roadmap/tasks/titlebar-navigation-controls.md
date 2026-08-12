# Task packet — Unified titlebar navigation controls

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[interaction contract](../../product/browser-interactions.md) ·
[DESIGN.md](../../../DESIGN.md) §11.14 ·
[UI library anchor](ui-library-anchor.md) ·
[Gallery browser components](component-gallery-browser-components.md) ·
[Gallery compositions](component-gallery-compositions.md).

Activation condition: complete F2a and G1a are integrated and titlebar/navigation territory is writer-free, as recorded in [Current delivery](../../delivery/current.md).
Implementation must not interrupt F2c → F2b → fresh F2a or start before the
settled G1a `bbr-ui` anchor.

Stable composition reference, not a visual copy:
`/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-titlebar-sidebar-navigation.png`.

## Outcome

BBR's expanded full-height Sidebar presents a traffic-light-safe
collapse/expand control and compact Back/Forward/Reload cluster at the top of
its rail. Controls route through the same authorities as Cmd-B and browser
navigation shortcuts, target the selected tab or focused split pane, preserve
native titlebar dragging, and never duplicate address entry or invent a second
navigation state. Collapsing the Sidebar removes the complete BBR-owned header
and navigation cluster; no replacement content toolbar remains over the page.

## Scope

- Extend the Sidebar rail to the top window edge and compose
  `[traffic lights] [sidebar toggle] … [Back] [Forward] [Reload]` inside the
  actual safe titlebar layout guide.
- The sidebar toggle sits immediately after the traffic lights and uses the
  identical sidebar geometry authority as Cmd-B. Its glyph reflects collapsed
  versus expanded state.
- Back/Forward/Reload target the selected tab in single-pane mode and the
  focused pane in split mode using existing controller/navigation state. The
  [web-content gesture packet](web-content-navigation-gestures.md) consumes the
  same history availability/action authority; titlebar and gesture paths never
  maintain separate navigation truth.
- Use production `bbr-ui` geometric 18 pt glyphs, normal-density hit areas at
  least 32 pt, transparent rest state, DESIGN hover/pressed/disabled/focus
  tokens, and no bordered toolbar capsule.
- Preserve draggable empty titlebar space, traffic-light clearance,
  fullscreen, multi-display, and narrow-window behavior.
- On Sidebar collapse, hide `bbr-titlebar`, `bbr-titlebar-sidebar-toggle`, and
  all three navigation controls with the rail. Native traffic lights remain;
  Cmd-B restores the Sidebar. Do not render a content-side replacement bar.
- Stable ids: `bbr-titlebar`, `bbr-titlebar-sidebar-toggle`, `bbr-nav-back`,
  `bbr-nav-forward`, `bbr-nav-reload`.
- Add typed geometry/availability/semantic state, click and keyboard parity
  E2E, relevant optimized presentation evidence, and Gallery G2/G3 stories
  from the same production components/composition.

## Non-goals

- No hardcoded traffic-light coordinates.
- No copied Zen translucent slab, bordered toolbar pill, second address field,
  or duplicated controls inside the address display.
- No new browser navigation authority, tab selection state, split focus state,
  opener-return policy, or sidebar width state in `bbr-ui`.
- No stop button or additional toolbar actions without a separate contract.
- No window-management redesign or multi-window work.
- No collapsed-state content toolbar, floating navigation cluster, or duplicate
  Sidebar toggle over the native webpage.

## Parallel-lane contract

```text
Base condition:       verified main after F2c → F2b → fresh F2a and settled G1a
Depends on:           G1a bbr-ui theme/icons/components; existing sidebar and
                      navigation authorities; focused-pane routing
Provides/API:         production TitlebarNavigationModel/composition and
                      macOS safe-titlebar adapter
Exclusive paths:      post-G1a bbr-ui titlebar/navigation modules; focused
                      bbr-app window/titlebar composition; focused E2E/perf
Shared integration points: app window options/composition, focus/input routing,
                      native safe-area geometry, testkit snapshot, perf and
                      Gallery G2/G3 registries, roadmap roll-ups
Integration order:    F2c → F2b → fresh F2a → G1a → this packet
```

No concurrent writer may own titlebar/window composition, sidebar toggle
presentation, navigation-control components, or their Gallery stories.

## Constraints

- Safe-area geometry comes from the native titlebar/layout guide, never magic
  x/y offsets.
- Empty titlebar background remains draggable and controls do not intercept
  drag outside their hit targets.
- Sidebar toggle and Cmd-B mutate the same requested/remembered width; no
  duplicate state. The BBR-owned header exists only while the Sidebar is
  expanded.
- Back/Forward enablement derives from real session navigation state; disabled
  controls cannot dispatch. Reload targets only the focused command pane.
- Portable UI receives plain presentation data/callbacks; AppKit details stay
  in the macOS composition edge.
- Stable ids, focus order, semantics, contrast, density, and reduced motion
  follow DESIGN §11.14.
- No new dependencies; files below roughly 500 lines; no weakened geometry,
  E2E, performance, or sample-count guards.

## Evidence

### Unit and contract

- [x] Availability/model matrix covers no history, back-only, forward-only,
      both, loading/reload, collapsed/expanded sidebar, and split focused pane.
      (bbr-ui `TitlebarModel` matrix + Gallery sidebar matrix; the plain model
      carries the genuine enablement and the callbacks decide nothing.)
- [x] Cmd-B and pointer titlebar toggle produce the same sidebar geometry and
      restore the same remembered width.
      (Real-app E2E `titlebar_toggle_parity_and_collapse_removes_the_header`:
      drag to 320 pt, pointer toggle collapse, Cmd-B expand restores 320 pt.)
- [x] Back/Forward/Reload dispatch once to the selected/focused view and never
      to the unfocused split sibling.
      (Real-app E2E `nav_clicks_match_keyboard_shortcuts` and
      `split_navigation_targets_only_the_focused_pane` over real WKWebViews;
      the controller routes through the shared `focused_tab` authority.)
- [x] Safe-area layout keeps controls out of the traffic-light region across
      supported window widths and scale factors.
      (Real-app E2E `titlebar_presents_safe_area_and_availability`; model tests
      `safe_area_clears_any_traffic_light_region`; `bbr-web-macos` frame-flip
      unit tests. The toggle starts at the live close-button frame's right
      edge — never a hardcoded coordinate.)

### Real-app E2E

- [x] Click controls and keyboard shortcuts have identical state/session/focus
      results over real WKWebViews.
      (`nav_clicks_match_keyboard_shortcuts`: the click and the shortcut reach
      the identical titlebar availability, URL, and session.)
- [x] Disabled controls are visible, semantically explained, and no-op.
      (`disabled_nav_controls_are_visible_and_noop`: disabled Back/Forward
      clicks change nothing on the live session; the semantic tree reports
      `enabled = false`.)
- [ ] Empty titlebar dragging moves the window; control presses do not.
      (The production wiring routes the `bbr-titlebar-drag` press to
      `start_window_move`/`titlebar_double_click` and disabled controls can
      never dispatch by construction; the recorder story observes the drag
      press. The native window move itself is an AppKit operation not
      observable in the deterministic snapshot protocol — not automated.)
- [x] Collapse removes the complete BBR header/navigation cluster and expand
      restores both the remembered width and header; narrow windows avoid
      overlap with traffic lights/sidebar/address/content.
      (`titlebar_toggle_parity_and_collapse_removes_the_header` asserts no
      `bbr-titlebar`/toggle/nav ids and no content-toolbar replacement after
      collapse, full-bleed slot, and full header restoration on Cmd-B;
      narrow-window coverage via the dense composition suite.)
- [ ] Fullscreen and multi-display windows avoid overlap with traffic lights/
      sidebar/address/content.
      (Fullscreen and multi-display overlap are not headless-provable in the
      deterministic snapshot protocol; manual verification remains open.)
- [x] Split navigation targets only the focused pane.
      (`split_navigation_targets_only_the_focused_pane`: the focused pane's
      history flips; the unfocused sibling's native geometry and sessions are
      untouched.)

### Performance

- [ ] Existing sidebar-toggle and Reload evidence remains green.
      (Blocked: Patrick's execution override forbids performance runs.)
- [ ] Add seeded/enforced warm `navigation_control_commit` only if existing
      kinds do not cover the visible presentation; at least 30 accepted samples.
      (The seeded/enforced `NavigationControl` benchmark is implemented
      (`benchmark_navigation_control`: genuine availability precondition + the
      exact production dispatch + `presentation_pending` enforcement) and
      compiles under the warning-denied all-target gate; sample collection is
      blocked by the execution override — unexecuted/unpassed.)

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-browser
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-shell-sidebar
nix develop -c just e2e-shell-titlebar-navigation
nix develop -c just e2e-shell-split-geometry
nix develop -c just e2e-perf
nix develop -c just e2e
nix develop -c just roadmap
```

GUI suites run sequentially under the repository lock.

## Delivery sequence

1. Confirm F2/G1a integration and free window/titlebar/UI ownership.
2. Establish the safe-titlebar geometry seam and plain production model.
3. Wire sidebar toggle, then Back/Forward/Reload through existing authorities.
4. Add semantics, real-app geometry/focus evidence, performance coverage, and
   real-production Gallery stories.
5. Independent native/UI read-only review; one remediation writer only for
   concrete findings; focused recheck.

## Iteration policy

Start with the sidebar toggle in the safe area while preserving window drag.
Add the navigation cluster only after focused-pane routing is proven. Classify
failures as safe-area geometry, titlebar drag hit testing, duplicate authority,
navigation availability, split routing, semantics, or presentation evidence.

## Blocked stop condition

If the pinned GPUI/native window APIs cannot expose a stable safe titlebar
layout without hardcoded traffic-light geometry, stop with API/source evidence,
the preserved worktree, and the smallest native composition decision required.
Do not ship magic coordinates or disable titlebar dragging.

## Delivery ledger

```text
Outcome:
  BBR's expanded full-height Sidebar presents the traffic-light-safe
  collapse/expand toggle and compact Back/Forward/Reload cluster at the top
  of its rail, routed through the same authorities as Cmd-B / Cmd-[ / Cmd-] /
  Cmd-R, inside the live native safe-titlebar layout guide (never hardcoded
  traffic-light coordinates). Collapsing the Sidebar removes the complete
  BBR-owned titlebar/navigation cluster with the rail; no content toolbar
  remains; Cmd-B restores the remembered width and the full header.
  Split-mode Back/Forward/Reload target only the focused pane.

Commits:
  None by the finalizer lane (forward-only writer; no commit/merge/push).
  The exact forward tree is preserved for Orchestrator integration.

Functional evidence:
  - `bbr-web` `SafeTitlebarGuide` + `bbr-web-macos` `safe_titlebar_guide`
    (live window frame − contentLayoutRect, close-button frame flipped to
    host top-left; pure flip math unit-tested without macOS).
  - `bbr-ui` `titlebar` module: plain `TitlebarModel`/`SafeArea` DTO, one
    `TitlebarCallbacks` set, the `[traffic lights] [toggle] … [Back]
    [Forward] [Reload]` row with 18 pt glyphs, ≥32 pt hits, transparent
    rest, hover/pressed/disabled tokens, no toolbar capsule, and the
    draggable empty spacer; availability matrix, hit-density, and
    traffic-light-clearance unit tests.
  - App composition: `BbrApp::titlebar_model` (native safe area + genuine
    controller nav state + live-session reload), hover/pressed presentation
    methods, empty-titlebar `start_window_move`/double-click entry; the
    row is the rail's first child and disappears with it (render and
    semantic tree in lockstep).
  - Testkit: typed `TitlebarSnapshot`, stable ids, click/hover/press seams
    through the exact production methods; `e2e-shell-titlebar-navigation`
    suite wired into the canonical e2e chain and the `just` recipe set.
  - Gallery: G2/G3 sidebar stories gain the titlebar row and recorder
    observations (`ToggleSidebar`, `TitlebarBack/Forward/Reload/Drag`,
    hover/press/release); matrix covers full-history, no-history, and
    collapsed states.

Real-app E2E:
  - just e2e-shell-titlebar-navigation: 5/5 (presentation + safe area +
    semantics; toggle/Cmd-B parity + complete-header disappearance + width
    restoration; click/keyboard parity; disabled visible/no-op; split
    focused-pane routing).
  - just e2e-shell-sidebar: 11/11 (composition, region order, dense,
    footer/strip, collapse/restore).
  - just e2e-shell-split-geometry: 1/1. just e2e-shell-hover: 3/3
    (semantics). just e2e-shell-tabs: 3/3.

Performance evidence:
  - `NavigationControl` benchmark seeded/enforced (availability
    precondition, exact production dispatch, `presentation_pending`
    enforcement); compiles under the warning-denied all-target gate.
  - Execution (samples/retries), `just verify`, and canonical `just e2e`
    remain blocked by Patrick's override — no performance numbers are
    claimed.

Roadmap/documentation updates:
  - Packet evidence boxes updated (unit/contract and real-app E2E proven;
    drag and performance items honestly left open with reasons).
  - `justfile`: `e2e-shell-titlebar-navigation` recipe added and chained
    into `e2e`.
  - Existing sidebar composition/semantics/tabs E2E updated to the §11.14
    rail order (titlebar row replaces the status-row toggle as the top
    strip; no `bbr-sidebar-toggle` remains).

Open blockers and non-goals:
  - Perf execution and canonical aggregate E2E blocked by the execution
    override (compile-only performance code).
  - Native window-drag start and fullscreen/multi-display overlap are not
    observable in the deterministic snapshot protocol (wiring proven at
    render/recorder level).
  - Non-goals honored: no hardcoded traffic-light coordinates, no second
    address field, no duplicate nav authority, no collapsed content
    toolbar, no stop button, no window-management redesign.

Worktree state:
  branch parallel/dd-titlebar @ 4d5be3116ec090aebb7eefa4861970710dba7a5b;
  real index zero staged paths; all changes unstaged in the worktree.

Planning/read-only: 0m (finalizer lane)
Implementation:
  warning-denied unused-assignment fix (parity test event sync), semantics
  ordering fix (titlebar first), three event-sync/expectation fixes in the
  new E2E suite, five existing-sidebar-E2E updates to the §11.14 rail order.
Test execution:
  fmt, bbr-browser 218, bbr bin 261, bbr-ui/bbr-web/bbr-web-macos/
  bbr-testkit unit suites, Gallery 142, warning-denied all-target Clippy
  (fresh target dir), e2e-shell-titlebar-navigation 5/5, e2e-shell-sidebar
  11/11, e2e-shell-split-geometry 1/1, e2e-shell-hover 3/3,
  e2e-shell-tabs 3/3, just roadmap.
Review:
  none in this lane (single remediation/finalization writer).
Remediation:
  fixed exact unused-assignment denial; fixed semantic-tree ordering vs the
  render; synchronized the new suite on real WebKit history/title events
  (initial-commit precondition and CanGoBackChanged waits); corrected the
  split expectation to the documented domain-active-tab focus.
Waiting/retries:
  GUI suites waited for the repository GUI lock (background-links and
  sidebar review lanes ran first); no test retries.
Total elapsed:
  (single session)
Human intervention:
  none; Patrick's override (no perf execution, no just verify, no
  canonical just e2e) applied throughout.
```
