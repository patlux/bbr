# Track — UI design system and visual quality

Parent phases: [Phase 4](../phases/04-ui-shell.md), [Phase 5](../phases/05-command-bar.md), [Phase 6](../phases/06-split-view.md), [Phase 8](../phases/08-daily-driver.md), [Phase 9](../phases/09-quality-release.md)

## Strategy

Do not spend a major polish pass on the current single-view spike. Establish BBR's visual world immediately before the real multi-tab shell in Phase 4, then hold every UI feature to that system. Run a final product-wide polish pass in Phase 9.

## Visual direction and tokens — Phase 4 start

- [ ] Create `DESIGN.md` through an explicit visual-direction process.
- [ ] Define what makes BBR visually distinct from Arc, Zen, Safari, and the Swift reference.
- [ ] Define semantic surface, text, border, accent, status, and profile-identity colors.
- [ ] Define typography roles and truncation behavior for dense browser data.
- [ ] Define spacing, sizing, radius, separator, material, and elevation rules.
- [ ] Define icon family, optical sizing, stroke/weight, and fallback rules.
- [ ] Define motion principles, durations, easing, and Reduce Motion alternatives.
- [ ] Implement reusable GPUI tokens/components instead of local magic values.

## Shell craft — Phase 4

- [ ] Clear hierarchy between navigation controls, command surface, pinned tabs, regular tabs, and spaces.
- [ ] Active tab and focused pane are unmistakable without visual noise.
- [ ] Profile identity is clear whenever tabs/spaces cross contexts.
- [ ] Hover, press, drag, selection, loading, attention, crash, and disabled states are coherent.
- [ ] Long titles, missing favicons, narrow windows, empty spaces, and 500-tab density remain usable.
- [ ] Sidebar collapse/resize feels stable and preserves native content geometry.

## Feature-level polish — Phases 5–8

- [ ] Command bar feels fast, layered, and visually integrated rather than a generic modal.
- [ ] Split divider and pane focus are clear but quiet.
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
