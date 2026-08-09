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

BBR's unified macOS titlebar presents a traffic-light-safe sidebar
collapse/expand control and compact Back/Forward/Reload cluster. Controls route
through the same authorities as Cmd-B and browser navigation shortcuts, target
the selected tab or focused split pane, preserve native titlebar dragging, and
never duplicate address entry or invent a second navigation state.

## Scope

- Compose `[traffic lights] [sidebar toggle] … [Back] [Forward] [Reload]`
  inside the actual safe titlebar layout guide.
- The sidebar toggle sits immediately after the traffic lights and uses the
  identical sidebar geometry authority as Cmd-B. Its glyph reflects collapsed
  versus expanded state.
- Back/Forward/Reload target the selected tab in single-pane mode and the
  focused pane in split mode using existing controller/navigation state.
- Use production `bbr-ui` geometric 18 pt glyphs, normal-density hit areas at
  least 32 pt, transparent rest state, DESIGN hover/pressed/disabled/focus
  tokens, and no bordered toolbar capsule.
- Preserve draggable empty titlebar space, traffic-light clearance,
  fullscreen, multi-display, narrow-window, and sidebar-collapsed behavior.
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
  or sidebar width state in `bbr-ui`.
- No stop button or additional toolbar actions without a separate contract.
- No window-management redesign or multi-window work.

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
Integration order:    F2c → F2b → fresh F2a → G1aa → this packet
```

No concurrent writer may own titlebar/window composition, sidebar toggle
presentation, navigation-control components, or their Gallery stories.

## Constraints

- Safe-area geometry comes from the native titlebar/layout guide, never magic
  x/y offsets.
- Empty titlebar background remains draggable and controls do not intercept
  drag outside their hit targets.
- Sidebar toggle and Cmd-B mutate the same requested/remembered width; no
  duplicate state.
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

- [ ] Availability/model matrix covers no history, back-only, forward-only,
      both, loading/reload, collapsed/expanded sidebar, and split focused pane.
- [ ] Cmd-B and pointer titlebar toggle produce the same sidebar geometry and
      restore the same remembered width.
- [ ] Back/Forward/Reload dispatch once to the selected/focused view and never
      to the unfocused split sibling.
- [ ] Safe-area layout keeps controls out of the traffic-light region across
      supported window widths and scale factors.

### Real-app E2E

- [ ] Click controls and keyboard shortcuts have identical state/session/focus
      results over real WKWebViews.
- [ ] Disabled controls are visible, semantically explained, and no-op.
- [ ] Empty titlebar dragging moves the window; control presses do not.
- [ ] Collapse/expand restores width; fullscreen, multiple displays, and
      narrow windows avoid overlap with traffic lights/sidebar/address/content.
- [ ] Split navigation targets only the focused pane.

### Performance

- [ ] Existing sidebar-toggle and Reload evidence remains green.
- [ ] Add seeded/enforced warm `navigation_control_commit` only if existing
      kinds do not cover the visible presentation; at least 30 accepted samples.

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
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```
