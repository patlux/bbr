# Task packet — UI contract conformance fixes

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §3, §7, §10.4, §11.16, §11.17.

## Outcome

Shipped chrome matches the existing visual contract where a 2026-09-27 review
of the running code against DESIGN.md found deviations: primary actions are
readable, a selected Saved Page row uses the selection surface, drawn §7 glyphs
replace Unicode stand-ins and misleading glyphs, identity glyphs share one
optical size, the dark selection surface is visible, and every portal surface
follows the shell appearance.

## Scope

- Primary actions on an accent fill use one `on_accent` text token
  (`material:glass`); the auth sheet's **Log In** and the space
  configuration's **Done** stop rendering `accent.ink` on `accent`.
- A selected Saved Pages Library row paints `tint.selected` (§10.4), never an
  accent fill; the selection-box mark uses `on_accent`.
- Monogram letters use one mode-aware authority (`ink` on the light slate
  ramp, white on the dark ramp) in tab rows, Favorites, folders, and the
  Library; white letters on the light ramp were ≈ 1.7:1.
- Light increased contrast deepens the row washes (7 % / 10 %) like dark.
- New drawn §7 glyphs `Icon::Bookmark` and `Icon::Pencil`. The Library row and
  the palette's Saved Page result use the bookmark glyph instead of the text
  `★` (in-rail list) and the `→` fallback (palette/overlay had no Saved Page
  branch); the Library rename control uses the pencil instead of the check
  mark.
- `Icon::Clock` draws two hands from the center (10:00 reading), as its
  contract comment states.
- Identity glyphs (§11.3 catalog) share one optical extent; compact glyphs are
  no longer drawn at half the size of `code` and `folder`.
- The dark (and dark increased-contrast) `tint.selected` is recalibrated to a
  visible surface below the rail's luminance while keeping the §3.1/§3.5 text
  bars.
- The auth sheet and the Peek bar resolve the shell theme instead of a fixed
  `Theme::dark()`, and report the presented mode in their typed snapshots.
- DESIGN.md drift: §3.5 values, the §10.2 hover-wash hex, the N row inset in
  §10.2/§11.12, the new §7 glyphs, and the §11.17/§16 Peek theme statements.

## Non-goals

- No new interaction, layout, geometry, id, or state; no design-branch
  adoption (owned by [seated content card](seated-content-card.md),
  [address field and space identity](address-field-space-identity.md), and
  [Favorites grid and drag offers](favorites-drag-offers.md)).
- No Peek entrance/exit animation or reduced-motion matrix beyond following the
  shell theme (§16 keeps them deferred).
- No change to light-mode token values other than the documentation fix.

## Parallel-lane contract

Not parallelized. One serial writer on `main`.

## Constraints

- Tokens stay the single color authority: components consume palette fields,
  never literals. Contrast bars are recomputed from token hex values by tests.
- §7 glyphs stay stroke-only, inside their box at 16/18/20/32/64 pt, with no
  Unicode substitutes.
- No weakened test, assertion, deadline, budget, or validator.

## Evidence

### Unit and contract

- [ ] `on_accent` text reaches ≥ 4.5:1 on `accent` in all four palettes; the
      auth submit and space-config Done colors come from it.
- [ ] A selected Library row fills with `tint.selected`; ink on it stays ≥ 4.5:1;
      the selection-box mark reaches ≥ 3:1 on its accent fill.
- [ ] Monogram letters reach ≥ 3:1 on every tile of both slate ramps.
- [ ] Increased contrast deepens the hover/active washes in both modes.
- [ ] `Icon::Bookmark` and `Icon::Pencil` are named, catalogued, and inside
      their box; the Library row, the palette Saved Page mark, and the rename
      control use them.
- [ ] Both `Icon::Clock` hands start at the glyph center.
- [ ] Every curated identity glyph spans at least 80 % of the widest glyph's
      extent and stays inside its box.
- [ ] The dark and dark increased-contrast selection tints separate from the
      dark rail by at least 4 CIE L* units while keeping ink ≥ 14:1 and the
      secondary bars; increased contrast still deepens the tint.
- [ ] Auth sheet and Peek bar models resolve the shell theme.

### Real-app E2E

- [ ] `just e2e-auth` asserts the presented sheet's theme under light and dark
      appearance overrides.
- [ ] `just e2e-peek` asserts the presented bar's theme under a light override.
- [ ] Affected suites stay green: `e2e-shell-saved-pages`,
      `e2e-shell-saved-pages-library`, `e2e-shell-spaces`, `e2e-shell-sidebar`,
      `e2e-shell-responsiveness`, `e2e-gallery`.

### Performance

- [ ] Not applicable: colors and glyph paths only; no interaction, timing, or
      presentation-commit change. Existing scenarios are unaffected and remain
      under the active performance-execution override.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-auth
just e2e-peek
just e2e-shell-saved-pages
just e2e-shell-saved-pages-library
just e2e-shell-spaces
just e2e-shell-sidebar
just e2e-shell-responsiveness
just e2e-gallery
just e2e
```

GUI suites run sequentially through the repository lock.

## Delivery sequence

1. One writer implements each deviation with its regression test.
2. One independent read-only review maps the findings to evidence.
3. Remediate only concrete findings, then recheck.

## Iteration policy

Fix one deviation at a time, smallest causal unit test first, then the affected
real-app suite. A failing check is diagnosed before any change; token values are
chosen from computed contrast, never by weakening a bar.

## Blocked stop condition

If a dark selection tint cannot satisfy both the §3.1/§3.5 text bars and the
visibility bar, stop with the computed candidates and ask for a contract
decision rather than relaxing either bar. If a portal surface cannot receive the
shell theme without a second theme authority, stop with the ownership evidence.

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
