# Task packet — Address field, zoom accessory, and space identity

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §11.3, §11.4, §11.5, §17.14, §17.16.

Specification input: the design-branch DESIGN.md §11.4/§11.5 text at
`design/browser-shell-paper@414c97c7` and the Paper boards “Shell · Light/Dark”
and “Decision · Design branch vs main”. Patrick adopted drift ledger rows 3, 4,
5 and 10 on 2026-09-27.

## Outcome

The sidebar address reads as a field: a quiet bordered surface one step off
the rail that still follows the space tint. Its trailing accessory shows the
page zoom only when zoom is not 100 %, and pressing it resets to 100 %. The
active Space heading carries the same curated identity glyph as its footer
chip. The Paper shells use catalog glyphs only.

## Scope

- DESIGN.md §11.5: field fill derived from the rail material (not a fixed hex)
  plus a hairline-strength border; keyboard focus draws the §9 ring; still a
  button with no caret, selection or text entry.
- Resolve §17.14: the accessory hosts the zoom chip under the rule above; Copy
  Link and permission indicators stay future accessories.
- Heading identity (`bbr-active-space-identity`) on the favicon column, drawn
  from the §11.3 catalog.
- Redraw the Paper shells' footer and heading glyphs with catalog glyphs.
- Real-app E2E for the field states, zoom chip visibility/reset and heading
  identity in light and dark.

## Non-goals

- No second URL input; the palette stays the only editor.
- No permissions surface, no Copy Link accessory.
- No new identity glyphs beyond the curated catalog.

## Parallel-lane contract

```text
Base condition:          seated content card integrated
Depends on:              seated-content-card; integrated site zoom contract
Provides/API:            address field presentation, zoom accessory, heading identity
Exclusive paths:         bbr-ui address component, sidebar heading, zoom accessory wiring
Shared integration points: testkit snapshot, e2e_shell_command, e2e_shell_zoom, DESIGN.md §11.5/§17
Integration order:       after seated-content-card, before favorites-drag-offers
```

## Constraints

- The address control keeps button semantics and the §11.4 overlay geometry.
- Zoom state comes only from the existing site-zoom authority.
- Identity never relies on color alone; no Unicode stand-ins.

## Evidence

### Unit and contract

- [x] Field fill/border tokens hold the §3.1 text bar in all four palettes and
      follow the space tint.
- [x] Zoom accessory visibility is a pure function of the site zoom level.

### Real-app E2E

- [x] `just e2e-shell-zoom` proves the chip appears off 100 %, resets on press
      and disappears at 100 %.
- [x] `just e2e-shell-command` proves the overlay still covers the field with
      the URL preselected.
- [x] `just e2e-shell-spaces` proves the heading identity matches the chip.

### Performance

- [ ] Palette/overlay open budgets unchanged; execution follows the active
      performance override.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-command
just e2e-shell-zoom
just e2e-shell-spaces
just e2e-perf
just e2e
```

## Iteration policy

Tokens and DESIGN text first, then the field, then the accessory, then the
heading identity; one real-app suite after each step.

## Blocked stop condition

If a tint-following fill cannot hold the text bar in every palette, stop with
the computed candidates and ask for a contract decision.

## Delivery ledger

```text
Outcome: the address reads as a quiet bordered field one step off the rail
  (a white veil over the tinted rail plus a hairline-strength border), the
  accessory slot hosts the zoom chip only off 100 % (press resets through
  the site-zoom authority and never opens the overlay), and the active-space
  heading renders the active footer chip's own identity on the favicon
  column.
Commits: this implementation commit on main.
Functional evidence: palette test (veil over the rail, ink >= 4.5:1 on the
  composited field in all four palettes, hover direction, border strength);
  zoom_accessory unit test; shared identity_mark renderer for chip and
  heading.
Real-app E2E: new zoom accessory scenario (absent at 100 %, press on an
  absent chip rejected, 125 % shown, press resets to 100 %, overlay stays
  closed, identity unchanged); spaces-config asserts the heading identity
  equals the chip before and after a space switch; reference polish asserts
  the bordered-field rest fill in both modes; full `just e2e` aggregate
  30/30 plus e2e-peek 7/7 and e2e-gallery 45/45 in 320 s. Visual capture
  shows the bordered field and the heading identity.
Performance evidence: open (performance-execution override).
Roadmap/documentation updates: DESIGN §11.4 heading anatomy and ids, §11.5
  field/zoom accessory/hover/focus, §17.14 landed.
Open blockers and non-goals: keyboard focus ring on the address control has
  no traversal path to it yet (Cmd-L opens the palette directly); redrawing
  the Paper shells' glyphs waits for the Paper file to be opened; the
  performance gate is blocked by the override.
Worktree state: main, serial writer.
```
