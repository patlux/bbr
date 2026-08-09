# Track — UI design system and visual quality

Parent phases: [Phase 4](../phases/04-ui-shell.md), [Phase 5](../phases/05-command-bar.md), [Phase 6](../phases/06-split-view.md), [Phase 8](../phases/08-daily-driver.md), [Phase 9](../phases/09-quality-release.md)

## Strategy

Do not polish the current placeholder sidebar incrementally. Establish BBR's visual world before exposing the already-controller-backed tabs and spaces in Phase 4, then hold every UI feature to that system. Run a final product-wide polish pass in Phase 9.

## Reusable UI library and development gallery — Post-MVP

- [ ] Execute the serial [UI-library anchor](../tasks/ui-library-anchor.md) after the verified F2a footer/space integration. This is the extraction and dependency contract for the shared production `bbr-ui` crate; it does not authorize a parallel UI writer before the required ownership preflight.
- [ ] Build the dev-only [BBR Gallery MVP](../tasks/component-gallery-mvp.md) from the settled anchor slice, using the same production tokens and primitives as `bbr-app`, with no WebKit or network listener.
- [ ] Add the bounded [browser-component stories](../tasks/component-gallery-browser-components.md) from real production presentation models and callbacks, never copied gallery implementations.
- [ ] Add the serial [composition stories](../tasks/component-gallery-compositions.md) for Sidebar, Command palette/overlay, Downloads, Find, Error/Crash, Auth, Context Menu, and Split Preview.

Gallery verification supplements but never replaces the affected real-browser E2E and performance gates.

## Zen-reference interaction surfaces — post-F2/G1

Zen supplies information architecture and native-macOS interaction reference,
not BBR feature inventory or visual identity. Production implementation remains
queued behind **F2c → F2b → fresh F2a → G1** except the Portal-specific page
menu follow-up, which may start after F2 closeout when that territory is free.

- [ ] Deliver one shared production [Tab/Favorite context-menu](../tasks/tab-favorite-context-menus.md) authority bound to the invoking persistent TabId, with capability-gated rows and no Firefox/container/mobile/extension invention.
- [ ] Deliver the bounded [Folder context menu](../tasks/folder-context-menu.md) over explicit flat, session-local folder semantics; nesting, conversion, persistence, and ambiguous deletion stay omitted until separately owned.
- [ ] Enrich the existing Portal-backed [Page-background context menu](../tasks/page-background-context-menu.md) after F2 closeout while preserving WebKit-native link/image/selection/editable/media/JavaScript menus.
- [ ] Move sidebar collapse plus Back/Forward/Reload into the traffic-light-safe [unified titlebar navigation](../tasks/titlebar-navigation-controls.md) without hardcoded geometry or duplicate authorities.
- [ ] Establish the public-API-only [media-session capability contract](../tasks/media-session-capability-contract.md), then render the conditional [sidebar media control](../tasks/sidebar-media-control.md) from advertised capabilities only.
- [ ] Append the corresponding G2/G3 stories to the existing Gallery packets from the exact production components; no copies and no Gallery substitution for browser/native E2E. Saved Page result/menu stories wait for integrated [B2](../tasks/saved-pages-actions-command.md); Library stories wait for integrated [B3](../tasks/saved-pages-library.md). Detailed story acceptance remains owned by the G2/G3 packets.

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
- [ ] Collapse address entry into one surface through the [address control and palette packet](../tasks/url-address-control-palette.md). The sidebar address becomes a borderless translucent domain pill with button semantics and no focus ring; the palette opens over it with the full URL preselected. Two editing surfaces produced two focus appearances and two answers to "what is the current address" - this removes the second one rather than restyling it.
- [ ] Refine the sidebar through the bounded [browser-MVP sidebar UI packet](../tasks/browser-mvp-sidebar-ui.md) into the compact Zed-inspired BBR hierarchy, including the footer row and a real plus-menu for New Space/New Folder. *(Patrick's post-delivery product revision replaces the footer `[Spaces]` toggle with the permanent scrollable space-identity strip owned by the [footer space strip, appearance, and swipe packet](../tasks/space-footer-strip-appearance-ui.md).)*
- [x] Replace placeholder chrome glyphs with the BBR-drawn §7 icon set. *(The
      [icon-system slice](../tasks/icon-system.md) swaps the close `×`, pin
      `●`/`○`, loading `●`, new-space `+`, crash `!`, empty-pane brand `B`,
      and status-line `⚠` placeholders for drawn 1.5 pt glyphs from the
      reusable `components::icons` catalog — ids, hit targets, semantics,
      focus, and listeners unchanged. The find bar's `‹`/`›`/`✕` controls
      use `ChevronLeft`/`ChevronRight`/`Close`; the semantic icon union with
      F2a retained the removed Spaces glyph absence. Integrated on `main` at
      `7180128`; `just verify`, Find 6/6, affected GUI suites, and optimized
      performance are green. Portal 5/7 and the missing `e2e_shutdown` target
      remain separate wider F2-closeout blockers. The command-field prefix
      and sidebar brand text stay with command composition.)*
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
- [ ] Downloads, permissions, errors, history, settings, Saved Pages, and onboarding use the same components and language. Bounded follow-ups: [downloads sidebar panel](../tasks/downloads-sidebar-panel.md), [find-in-page Portal bar](../tasks/find-in-page-portal-bar.md), [BBR-owned context menus over web content](../tasks/bbr-context-menus-over-web.md), and the post-G1 [B3 Saved Pages Library](../tasks/saved-pages-library.md) after B1+B2.
- [ ] Every new surface ships light/dark, empty/loading/error, keyboard-focus, and reduced-motion variants.

## Product-wide polish — Phase 9

- [ ] Complete the bounded [global UI scale and density pass](../tasks/global-ui-scale-density.md) as the first slice after browser-MVP closeout. Patrick's live U1 review found the whole chrome clearly too small; revise typography, leading, spacing, row/control/hit-target, sidebar, and palette geometry through one canonical DESIGN/Theme token decision, calibrated against the preserved user reference screenshots in `/Users/patwoz/dev/Personal/macos/bbr-design-references/` (approximately 36–40 pt rows, 14–15 pt text, 18–20 pt glyphs, inset rounded row surfaces, large Favorite tiles) rather than a blanket percentage, while preserving semantic IDs, dense-500 windowing, and performance budgets.
- [ ] Run a bounded visual critique of all major workflows.
- [ ] Correct typography rhythm, alignment, optical icon balance, density, and inconsistent materials.
- [ ] Remove placeholder glyphs/copy and accidental one-off styling.
- [ ] Refine purposeful micro-interactions and transition choreography.
- [ ] Verify screenshots across light/dark, compact/wide, empty/dense, and error states.
- [ ] Confirm visual quality does not regress performance, energy, input, or Accessibility.
