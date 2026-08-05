# Phase 4 — Vertical tabs, spaces, and first intentional UI

Parent: [Roadmap](../../../ROADMAP.md) · Canonical UI checklist: [UI design-system track](../tracks/ui-design-system.md) · Accessibility: [Accessibility track](../tracks/accessibility.md)

## Outcome

BBR becomes a coherent browser shell with real tabs and spaces—not a styled integration spike.

## Design foundation first

- [ ] Establish and document the visual direction in `DESIGN.md` before broad shell implementation.
- [ ] Define semantic color, typography, spacing, radius, icon, and motion tokens.
- [ ] Define light, dark, increased-contrast, and reduced-motion behavior.
- [ ] Define density rules for small, normal, and very large tab collections.
- [ ] Define reusable states: hover, active, selected, loading, attention, error, disabled, drag target, and keyboard focus.

## Shell and interaction

- [ ] Render pinned and regular tabs for the active space.
- [ ] Show favicon/title/loading/crash state from the controller.
- [ ] Open, select, close, and reopen tabs from pointer and keyboard.
- [ ] Make `Cmd-T` create a real tab and `Cmd-W` close the focused tab.
- [ ] Render and switch spaces with unmistakable profile identity.
- [ ] Preserve the active tab per space without recreating warm sessions.
- [ ] Add adjustable, collapsible sidebar behavior.
- [ ] Add drag reorder and move between spaces after keyboard equivalents exist.
- [ ] Handle empty space, first launch, many tabs, long titles, missing favicons, and crashed pages.

## Quality acceptance

- [ ] Shell follows the [UI design-system track](../tracks/ui-design-system.md).
- [ ] Keyboard and VoiceOver acceptance in [Accessibility track](../tracks/accessibility.md) is green.
- [ ] Tab/space E2E covers open, switch, close, reopen, pin, reorder, and profile labels.
- [ ] Sidebar remains responsive with 500 persisted tabs and 50 live/warm sessions.

## When this UI pass happens

This is the **first major beauty pass**. It happens immediately after portable orchestration and multi-session WebKit are reliable, not after every browser feature is complete.
