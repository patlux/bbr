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

- [ ] Clear hierarchy between navigation controls, command surface, pinned tabs, regular tabs, and spaces.
- [ ] Active tab and focused pane are unmistakable without visual noise.
- [ ] Profile identity is clear whenever tabs/spaces cross contexts. *(Every space row keeps its §12 chip with a round-robin hue and monogram, profile groups stack under hairlines, and the empty pane names the owning profile; the command-field destination hint during switching remains with the cross-context slice.)*
- [ ] Hover, press, drag, selection, loading, attention, crash, and disabled states are coherent. *(The drag surface renders the full §10.9 vocabulary — `dragging` (92% opacity, elev:2 shadow, strong outline), the accent insertion tick, the muted invalid tick at a clamped boundary, and `drag-target` space rows with the destination profile chip — through the production drag state machine; the remaining hover/press/attention polish rides the Phase-9 pass.)*
- [ ] Long titles, missing favicons, narrow windows, empty spaces, and 500-tab density remain usable.
- [x] Sidebar collapse/resize feels stable and preserves native content geometry. *(The DESIGN §5/§11.4 slice pins the rail at its exact effective width (the flex_shrink fix) and proves through the real app that the native WKWebView slot always keeps strictly positive geometry — exact rail/content boundary, 64 pt content floor in compact windows, no drift across drags/resizes — see `just e2e-shell-sidebar`.)*

## Feature-level polish — Phases 5–8

- [ ] Command bar feels fast, layered, and visually integrated rather than a generic modal.
- [ ] Split divider and pane focus are clear but quiet.
- [ ] [Link Preview / Peek](../tasks/link-preview-peek.md) communicates its temporary state, fully covers the current page without looking like a generic modal, and makes dismissal versus promotion unmistakable.
- [ ] Persistence/recovery states communicate trust and reversibility.
- [ ] Downloads, permissions, errors, history, settings, and onboarding use the same components and language.
- [ ] Every new surface ships light/dark, empty/loading/error, keyboard-focus, and reduced-motion variants.

## Product-wide polish — Phase 9

- [ ] Run a bounded visual critique of all major workflows.
- [ ] Correct typography rhythm, alignment, optical icon balance, density, and inconsistent materials.
- [ ] Remove placeholder glyphs/copy and accidental one-off styling.
- [ ] Refine purposeful micro-interactions and transition choreography.
- [ ] Verify screenshots across light/dark, compact/wide, empty/dense, and error states.
- [ ] Confirm visual quality does not regress performance, energy, input, or Accessibility.
