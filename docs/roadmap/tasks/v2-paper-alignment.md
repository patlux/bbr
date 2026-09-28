# Task packet — v2 S12: Paper alignment and comparison findings

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §11.2, §11.3, §11.4, §11.8, §11.10, §11.11,
§11.12, §18 · [Phase 4](../phases/04-ui-shell.md).

Specification input: the renderer captures of 2026-09-28 compared with the
Paper references (`after-shell-{light,dark,tinted}.png`,
`after-palette.png`, the motion loops `empty_*`, `micro_favorite_v2`).
Patrick asked for every finding to be addressed.

## Outcome

The shipped shell matches the Paper references wherever the adopted v2
decisions do not deliberately differ, and the three defects the comparison
exposed are fixed with regression coverage.

## Scope

Defects:

- Activating a space hydrates its active tab when the tab is session-less
  (restored/suspended), instead of presenting the empty-space pane.
- Grouped (folder-member) pinned tabs are no longer also listed as loose
  pinned rows (DESIGN §11.11 open gap), with drag and keyboard geometry
  consistent.
- The command field caret sits after the query text (text height), not as a
  full-height bar at the end of the row.

Paper alignment:

- ⌘T palette anatomy: centred over the content card, leading search glyph,
  the mode chip as an uppercase pill, result rows with favicon/letter badge,
  the query match emphasised, the URL in the mono role and a trailing kind
  hint.
- Space tint model: each step keeps the untinted rail's luminance and adds
  hue (the Paper cobalt reads as blue, the Ink token as neutral); the three
  intensity steps become chroma steps; every D3 bar holds.
- Footer strip: no separate footer band on the rail; space chips follow the
  Downloads control from the leading edge.
- Folders: names at the label weight in `ink`, the Paper folder glyph,
  nested members indented one step, member close revealed on hover or
  selection, member titles in `ink`; the organized hairline is inset.
- Address zoom chip: `ink` text on the accessory fill.
- Drawn empty states: the Paper illustrations (floating tab cards around the
  ⌘ key, an accent heartbeat with a travelling beat, laptop ↔ globe with
  packets and a broken link).
- Favorite pop: the accent outline on the tile while it pops and the specks
  radiating outside the tile.

## Non-goals

- The D4 list snap (Paper cross-fades lists) and the D5 address progress
  (Paper shows a card-edge bar) stay as decided in DESIGN §18.1.
- The ⌘L overlay keeps the S5 minimum width (the Paper ⌘L reference predates
  v2).
- Media card, attention producer and real favicons in captures stay with
  their owning packets.

## Parallel-lane contract

Not parallelized.

## Constraints

- No weakened tests or bars; a test whose metric changes with a DESIGN
  change is rewritten to the new rule in the same commit.
- Warm-switch and commit-first budgets unchanged; hydration on activation
  creates exactly one session.

## Evidence

### Unit and contract

- [x] Controller: activating a space with a session-less active tab creates
      exactly that tab's session.
- [x] Sidebar model: loose pinned rows exclude folder members; drag slots and
      keyboard traversal follow the rendered rows.
- [x] Tint gates across tokens × steps × modes × contrast with the new model.
- [ ] Palette/illustration/pop frame and anatomy tests.

### Real-app E2E

- [x] Restored second space: activation presents its tab's native view, never
      the empty pane.
- [x] Folder member is not listed as a loose pinned row; selecting/closing
      still works.
- [ ] Affected suites (spaces, sidebar, organize, command, errors, persistence,
      gallery) and the canonical aggregate without `e2e-perf`.

### Performance

- [ ] Unpassed — execution deferred (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-spaces
just e2e-shell-sidebar
just e2e-shell-organize
just e2e-shell-command
just e2e-shell-errors
just e2e-shell-persistence
just e2e-gallery
just e2e              # minus e2e-perf while deferred, plus e2e-peek
```

## Iteration policy

Defects first (each with its failing regression test), then the palette,
the tint model, the rail details and the illustrations. Recapture and compare
with the Paper references after each visual group.

## Blocked stop condition

If a Paper value cannot hold a D3 bar, keep the bar, record the closest
holding value and the gap.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   unpassed — execution deferred
Worktree state:
```
