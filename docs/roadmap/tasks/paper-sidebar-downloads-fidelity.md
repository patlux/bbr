# Task packet — Paper fidelity: Downloads panel and rail footer

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §7, §11.4, §11.12 ·
[complete browser blueprint](paper-browser-blueprint.md).

Specification input: Patrick's 2026-10-11 request to fix every finding from
the comparison of the GPUI Visual captures (`downloads-history`,
`downloads-missing-file`, `ordinary-rail`, `representative-rail`) with the
Paper file "BBR — Browser Shell" boards M-4 (Downloads panel), S-7 (Footer
strip), W-1/W-2 (Shell) and S-2 (Favorites), and to align them with Paper.

## Outcome

The Downloads panel and the rail footer render like their Paper boards: the
History label heads its rows, every row holds its content, terminal rows
carry no progress bar, actions use the Paper pill/button presentation, the
footer is the 48 pt Paper band with 28 pt controls and chips, and the
Downloads and back/forward glyphs follow the Paper paths.

## Scope

- Panel paint order and geometry from one layout authority
  (`PanelLayout`/`DownloadsMetrics`): Active 56 / History 48 pt rows at N,
  19 pt labels, the 9 pt hairline band, 6 pt panel padding; mixed-height
  scroll offsets and anchors.
- Row anatomy: name, Active-only progress bar, secondary line in
  `ink.faint` (failure reason in `status.error`); 24 pt action pills with
  the bordered Retry button; trash Delete only on cancelled rows; trailing
  Clear; the 120 pt empty band.
- Footer band (48 pt, 6/10/10 insets, 4/2 pt gaps), 28 pt Downloads/Plus
  controls and chips with centred 16 pt glyphs.
- Downloads, trash and back/forward glyph geometry from the Paper paths.
- DESIGN §7/§11.4/§11.12/§15 text; regression tests; recaptured visuals.

## Non-goals

- DESIGN-backed differences from Paper stay as decided: the 16 pt footer
  glyph size (§18.3; Paper draws 18), the lifering failed-row glyph (§10.7;
  Paper draws a triangle), the icon-free empty hint (§11.12), the empty
  Favorites drop band (§11.10), the close glyph only on hover/keyboard focus
  (§11.1) and the native titlebar height (§2).
- The footer badge form (Paper draws a dot, DESIGN §11.4/§17.11 a count).
- Visual baseline acceptance: changed references need Patrick's review.

## Parallel-lane contract

Not parallelized (single serial writer on `main`, see Current delivery).

## Constraints

- One layout authority: render, scroll math, snapshot ids, benchmarks and
  the Gallery fixtures read the same `PanelLayout`.
- No weakened assertions: changed E2E expectations state the new Paper
  contract at the same strictness.

## Evidence

### Unit and contract

- [x] Red→green: paint order, item positions vs layout math, row heights
      vs content at every density, footer band metrics, Downloads glyph
      direction/baseline, back/forward proportions (`bbr-ui` lib: 6 new
      tests red on the old code, 233/233 green).
- [x] `bbr`, `bbr-ui`, `bbr-gallery` unit tests and strict Clippy; the
      `just verify` steps through `mbx` (workspace check/test/Clippy, fmt,
      roadmap, ui-deps, script tests).

### Real-app E2E

- [x] `e2e-shell-downloads` 13/13: terminal rows expose no progress id; the
      painted panel holds label, row and Clear.
- [x] `e2e-shell-sidebar` 21/21: footer controls/chip 28 pt, 12 pt above the
      rail bottom, Paper insets and gaps.
- [x] `e2e-shell-spaces` (+ keyboard, hint, config, menus): strip overflow
      with seven chips; 2 pt chip gap.
- [x] `e2e-gallery` 48/48: footer control squares and centred glyphs.
- [x] Browser visual capture A/B in Tart (84 byte-identical PNGs) and a
      side-by-side check against the Paper boards.

### Performance

- Presentation-only geometry change; the existing Downloads scenarios keep
  their probes. Performance execution stays deferred (Current delivery).

## Completion gates

```bash
mbx test -p bbr-ui -p bbr-gallery
mbx test -p bbr --features testkit --bins
mbx clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-vm e2e-shell-downloads e2e-shell-sidebar e2e-shell-spaces e2e-gallery
browser visual capture A/B (Tart)
just roadmap
```

## Iteration policy

Change one surface at a time from the Paper computed values; after each
failing check, find the mechanism before changing code. Keep DESIGN-backed
differences unless Patrick decides otherwise.

## Blocked stop condition

If no defensible path remains, stop with the evidence gathered, the
approaches attempted, the preserved worktree state, the exact blocker and
the smallest decision or input required next.

## Delivery ledger

```text
Outcome:                Downloads panel and rail footer follow Paper M-4,
                        S-7 and W-1/W-2; DESIGN §7/§11.4/§11.12/§15 updated.
Commits:                none yet (uncommitted, see Current delivery).
Functional evidence:    bbr-ui 233, bbr bins 379, bbr-gallery 153; workspace
                        test/check/Clippy through mbx; fmt; roadmap.
Real-app E2E:           Tart: downloads 13, sidebar 21, spaces 27, gallery 48,
                        first-run 2; full functional aggregate (59 binaries)
                        before the last footer/assertion edits: 57 green, the
                        downloads height assertion (fixed) and one first-run
                        persistence failure that passed on the targeted rerun.
Performance evidence:   not run (deferred).
Roadmap/documentation updates: DESIGN, this packet, Phase 4, Current delivery,
                        visual tooling packet.
Open blockers and non-goals: 62 reviewed references now differ in the footer,
                        titlebar arrows and Downloads glyph only; they need
                        Patrick's review before acceptance.
```
