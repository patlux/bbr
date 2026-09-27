# Task packet — Seated content card

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Native web-engine integration track](../tracks/native-integration.md) ·
[DESIGN.md](../../../DESIGN.md) §2, §5, §11.4, §13, §17.16.

Specification input: the design-branch DESIGN.md text at
`design/browser-shell-paper@414c97c7` (§2 “The content card”, §5
`sp:content-inset`/`win:titlebar`, §11.4 collapse, §13) and the Paper boards
“Shell · Light/Dark” and “Decision · Sidebar collapsed ⌘B”, option B. Patrick
adopted the direction on 2026-09-27 (drift ledger rows 1 and 2).

## Outcome

The native web view sits in a seated content card: inset by
`sp:content-inset` (10 pt) on top, trailing and bottom edges, flush against the
rail, clipped to `radius:lg` by the hosting AppKit layer, with one static
`elev:1.content` shadow on the chassis. The rail loses its right hairline. A
collapsed sidebar keeps an empty, draggable `win:titlebar` (52 pt) band so the
native traffic lights never sit on web content. Native full screen is
full-bleed.

## Scope

- Land the design-branch §2/§5/§11.4/§13 text into DESIGN.md as contract,
  with `material:chassis`, `elev:1.content` and `sp:content-inset` tokens.
- Production geometry: card rect, collapsed top inset, full-screen full-bleed,
  `CONTENT_MIN` measured on the card's inner box.
- Native slot clipping through the hosting layer's `cornerRadius`/`masksToBounds`
  (public AppKit only); GPUI paints chassis and shadow outside the clip.
- Typed snapshot: `bbr-content-card` with `content_inset`,
  `content_inset_top`, `card_radius`; real-app E2E for expanded, collapsed,
  resized and full-screen windows; unchanged session identity.

## Non-goals

- No compact toolbar, floating navigation, or control in the chassis gap.
- No split-pane cards (owned by [Split composition](split-app-composition.md)).
- No vibrancy, blur, or animated shadow.

## Parallel-lane contract

```text
Base condition:          UI contract conformance integrated; no writer owns shell geometry
Depends on:              ui-contract-conformance
Provides/API:            card geometry and native slot clipping
Exclusive paths:         shell geometry, web-macos slot hosting, sidebar rail paint
Shared integration points: testkit snapshot, e2e_shell_sidebar, DESIGN.md §2/§5/§13
Integration order:       before address-field-space-identity
```

## Constraints

- GPUI never draws over the native view; clipping is the native layer's.
- Profile isolation, session identity and the 64 pt native floor stay intact.
- Next-frame geometry commits; inset, radius and shadow land in one frame.
- No weakened geometry, performance, or E2E assertions.

## Evidence

### Unit and contract

- [ ] Card-rect arithmetic for expanded, collapsed, narrow and full-screen
      windows, including `CONTENT_MIN` on the inner box.

### Real-app E2E

- [ ] `just e2e-shell-sidebar` asserts card geometry and radius through
      resize, collapse/expand and full screen, with the native slot rect
      matching the card's inner box and no session recreation.

### Performance

- [ ] `sidebar_toggle`, `sidebar_resize` and `sidebar_restore` keep their
      budgets with the card; execution follows the active performance
      override in Current delivery.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-shell-portal
just e2e-perf
just e2e
```

## Iteration policy

Land geometry and snapshot first, then native clipping, then the chassis paint.
Diagnose any native-slot mismatch from the typed rect before touching paint.

## Blocked stop condition

If public AppKit cannot clip the WKWebView slot without re-rendering or
sampling web content, stop with the evidence and keep the full-bleed contract.

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
```
