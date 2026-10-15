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

- [x] Card-rect arithmetic for expanded, collapsed, narrow and full-screen
      windows, including `CONTENT_MIN` on the inner box.

### Real-app E2E

- [x] `just e2e-shell-sidebar` asserts card geometry and radius through
      resize, collapse/expand and full screen, with the native slot rect
      matching the card's inner box and no session recreation. *(Full
      screen through the typed presentation seam: programmatic AppKit
      full screen never completes on this host's Window Server.)*

### Performance

- [ ] `sidebar_toggle`, `sidebar_resize` and `sidebar_restore` keep their
      budgets with the card; execution follows the active performance
      override in Current delivery.

### Follow-up — seated edge (2026-10-12)

Patrick compared the card against macOS Tahoe's inset content panels
(Terminal, Sessions): a 1 pt dark seam outside a 1 pt light rim, then a
short seat shadow — instead of one soft glow. Paper board W-10 · Content
card — seated edge (page 20 — Window & shell) holds the proposal. The rings
are spread-only layers of `elev:1.content.shadow`, so the native slot keeps
the whole `sp:content-inset` frame and radius 10 and no geometry contract
changes; the visible chassis gap beyond the seam becomes 8 pt.

- [x] `elev:1.content.shadow` is the rim + seam + seat stack in both modes
      (DESIGN §2 token and card rule 3); `bbr-ui` card tests pin the stack
      order, spreads and the dark-seam/light-rim relation.
- [x] Real-app `just e2e-vm e2e-shell-sidebar` and
      `e2e-shell-titlebar-navigation` stay green on the unchanged
      geometry contract (card frame, native radius 10 → 0 → 10, slot rect).
- [x] Official sealed capture from this lane worktree (`just visual-check`,
      batch `1792102072680971000-69492`): 36 changed = exactly the
      card-bearing cases (Ordinary/Representative full+rail, Library,
      History, light and dark), 36 same (Settings windows, Palette child),
      6 missing-baseline (the visual lane's pending Find candidates). The
      Prüfstand diff marks only the card perimeter and the old glow's bleed
      onto the rail.
- [ ] Patrick reviews that report and accepts the 36 changed cases
      explicitly (`just visual-accept`); until then the baselines show the
      previous edge.

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
Outcome: the native slot is the §2 card's inner box (flush with the rail,
  10 pt elsewhere, radius:lg), the WKWebView and its navigation cover clip
  through their backing layers, GPUI paints the chassis (rail paint) and the
  static elev:1.content shadow, the rail lost its right hairline and the
  divider is empty paint at rest, a collapsed sidebar keeps the native
  titlebar band as empty draggable chassis, and full screen is full-bleed.
Commits: this implementation commit on main.
Functional evidence: bbr-ui card geometry/shadow tests and the updated rail
  clamp (the inset is charged to the slot); Gallery compact-family floor.
Real-app E2E: new content_card scenario (frame, native layer radius 10 →
  0 → 10 read back from the WKWebView, collapse band = safe titlebar height,
  resize, full-screen presentation, identity); existing geometry suites
  moved to the card contract (sidebar 16/16, titlebar navigation 5/5, split
  geometry 1/1); full `just e2e` aggregate 30/30 plus e2e-peek 7/7 and
  e2e-gallery 45/45 in 321 s. Visual check: representative capture shows
  the seated card, rounded corners, shadow and the chassis notch.
Performance evidence: open (performance-execution override); the perf
  driver's resize expectation now charges the inset like the app.
Roadmap/documentation updates: DESIGN §2 (layers, chassis, elevation, the
  content card), §5 (sp:content-inset, widths, floor), §11.4, §13, §17.16.
Open blockers and non-goals: real AppKit full screen is environment-blocked
  on this host (same as the portal hardening suite); split-pane cards stay
  with the Split composition; performance gate blocked by the override.
Worktree state: main, serial writer.
```
