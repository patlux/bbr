# Track — UI design system and visual quality

Parent phases: [Phase 4](../phases/04-ui-shell.md), [Phase 5](../phases/05-command-bar.md), [Phase 6](../phases/06-split-view.md), [Phase 8](../phases/08-daily-driver.md), [Phase 9](../phases/09-quality-release.md)

## Strategy

Do not polish the current placeholder sidebar incrementally. Establish BBR's visual world before exposing the already-controller-backed tabs and spaces in Phase 4, then hold every UI feature to that system. Run a final product-wide polish pass in Phase 9.

## Visual direction and tokens — Phase 4 start

- [x] Create [DESIGN.md](../../../DESIGN.md) through an explicit visual-direction process.
- [x] Define what makes BBR visually distinct from Arc, Zen, Safari, and the Swift reference.
- [x] Define semantic surface, text, border, accent, status, and profile-identity colors.
- [x] Define typography roles and truncation behavior for dense browser data.
- [x] Define spacing, sizing, radius, separator, material, and elevation rules.
- [x] Define icon family, optical sizing, stroke/weight, and fallback rules.
- [x] Define motion principles, durations, easing, and Reduce Motion alternatives.
- [x] Implement reusable GPUI tokens/components instead of local magic values. *(Tokens shipped with the shell rendering commit; the reusable component module
      (tab rows, favicon tiles, section labels, close control, empty-space pane,
      crash panel) landed with the visible-tab slice — evidence `just verify`, `just
      e2e-shell-tabs`.)*

## Shell craft — Phase 4

- [x] Establish the serial [GPUI-CE + gpui-component Root anchor](../tasks/gpui-ce-components-anchor.md), then compose standard input/menu/button behavior through gpui-component while retaining BBR tokens, geometric icons, native WKWebViews, and browser-specific orchestration. *(Integrated and canonical-E2E green at `f89a479`.)*
- [ ] Refine the sidebar through the bounded [browser-MVP sidebar UI packet](../tasks/browser-mvp-sidebar-ui.md) into the compact Zed-inspired BBR hierarchy, including the exact `[Downloads icon] [Spaces] [Plus icon]` footer and a real plus-menu for New Space/New Folder.
- [ ] Replace placeholder chrome glyphs with the BBR-drawn §7 icon set. *(The
      [icon-system slice](../tasks/icon-system.md) swaps the close `×`, pin
      `●`/`○`, loading `●`, new-space `+`, crash `!`, empty-pane brand `B`,
      and status-line `⚠` placeholders for drawn 1.5 pt glyphs from the
      reusable `components::icons` catalog — ids, hit targets, semantics,
      focus, and listeners unchanged. Unit/geometry evidence green on the
      lane; the existing real-app shell suites re-run at integration under
      the host-wide GUI gate. The command-field prefix and sidebar brand text
      stay with command composition.)*
- [ ] Clear hierarchy between navigation controls, command surface, pinned tabs, regular tabs, and spaces.
- [ ] Active tab and focused pane are unmistakable without visual noise.
- [x] Profile identity is clear whenever tabs/spaces cross contexts. *(Every space row keeps its §12 chip with a round-robin hue and monogram, profile groups stack under hairlines, the empty pane names the owning profile, and a cross-profile space activation renders the destination profile chip + a `type:label` "in Profile · <title>" hint at the top of the command field for one interaction (the [cross-context profile-hint slice](../tasks/cross-context-profile-hint.md)) — identity never relies on color alone.)*
- [x] Hover, press, drag, selection, loading, attention, crash, and disabled states are coherent. *(The full §10 vocabulary renders through the production paths: `dragging` (92% opacity, elev:2 shadow, strong outline), the accent insertion tick, the muted invalid tick at a clamped boundary, `drag-target` space rows with the destination profile chip, the §10.2 hover wash + close reveal and the §10.3 pressed wash driven by the production `on_hover`/mouse-down/up listeners (never a snapshot mutation), the §10.4 selected rail + tint, the §10.5 loading dim/spinner, the §10.7 error surface, and the §10.9 disabled controls on a dragged row — proven by `just e2e-shell-hover`, `just e2e-shell-states`, `just e2e-shell-organize`, and the canonical gate. The `attention` state (§10.6) has no source yet and rides the Phase-9 pass with its reason-line companion.)*
- [ ] Long titles, missing favicons, narrow windows, empty spaces, and 500-tab density remain usable.
- [x] Sidebar collapse/resize feels stable and preserves native content geometry. *(The DESIGN §5/§11.4 slice pins the rail at its exact effective width (the flex_shrink fix) and proves through the real app that the native WKWebView slot always keeps strictly positive geometry — exact rail/content boundary, 64 pt content floor in compact windows, no drift across drags/resizes — see `just e2e-shell-sidebar`.)*

## Feature-level polish — Phases 5–8

- [ ] Command bar feels fast, layered, and visually integrated rather than a generic modal through the bounded [browser-MVP palette UI packet](../tasks/browser-mvp-palette-ui.md).
- [ ] Split divider and pane focus are clear but quiet.
- [ ] [Link Preview / Peek](../tasks/link-preview-peek.md) communicates its temporary state, fully covers the current page without looking like a generic modal, and makes dismissal versus promotion unmistakable.
- [ ] Persistence/recovery states communicate trust and reversibility.
- [ ] Downloads, permissions, errors, history, settings, and onboarding use the same components and language. Bounded follow-ups: [downloads sidebar panel](../tasks/downloads-sidebar-panel.md), [find-in-page Portal bar](../tasks/find-in-page-portal-bar.md), and [BBR-owned context menus over web content](../tasks/bbr-context-menus-over-web.md).
- [ ] Every new surface ships light/dark, empty/loading/error, keyboard-focus, and reduced-motion variants.

## Product-wide polish — Phase 9

- [ ] Complete the bounded [global UI scale and density pass](../tasks/global-ui-scale-density.md) as the first slice after browser-MVP closeout. Patrick's live U1 review found the whole chrome clearly too small; revise typography, leading, spacing, row/control/hit-target, sidebar, and palette geometry through one canonical DESIGN/Theme token decision, preserving semantic IDs and performance budgets.
- [ ] Run a bounded visual critique of all major workflows.
- [ ] Correct typography rhythm, alignment, optical icon balance, density, and inconsistent materials.
- [ ] Remove placeholder glyphs/copy and accidental one-off styling.
- [ ] Refine purposeful micro-interactions and transition choreography.
- [ ] Verify screenshots across light/dark, compact/wide, empty/dense, and error states.
- [ ] Confirm visual quality does not regress performance, energy, input, or Accessibility.
