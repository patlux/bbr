# Task packet — Sidebar reference polish (row pitch, folder anatomy, identity glyphs)

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Reference-faithful production shell](visual-shell-reference.md) · [DESIGN.md](../../../DESIGN.md).

Settled product decision (Patrick, 2026-08-13): pinned tabs and folders stay
**separate regions** in the documented §11.4 order. The reference's mixed
pinned/folder interleaving is explicitly rejected; no ordering-model change
belongs to this or any later packet without a new decision.

## Outcome

The rail matches the reference's visual rhythm and iconography: rows sit on
the reference's ~40 pt pitch (36 pt surfaces separated by 4 pt gaps), folder
rows lead with a folder glyph, a hairline separates the organized regions
from the New Tab row, the address pill and Reload sit quiet at rest, and
footer Space chips render the complete curated BBR identity-glyph catalog
instead of two-letter badges.

## Scope

- **Row pitch:** close the deliberate deviation recorded in the
  [visual-shell ledger](visual-shell-reference.md) — normal-density rows keep
  their 36 pt surface and gain a 4 pt inter-row gap (S/XL proportional from
  the §8 table). Dense scroll math, bounded-window rendering and the Dense
  E2E geometry update to the new constant pitch without weakening any bound.
- **Folder row anatomy (§11.11):** leading glyph becomes a folder glyph with
  closed/open variants carrying the disclosure state; the count moves to a
  hover-revealed `type:meta` value; nested indent and ids unchanged.
- **Region separation:** one hairline between the organized regions
  (Pinned/Folders) and the New Tab row, per the reference composition.
- **Rest-state calibration:** the address pill renders the §11.5 translucent
  borderless fill and Reload loses any raised rest surface (§11.14
  transparent rest state), in both theme modes.
- **Identity-glyph catalog (§11.3/§7):** draw the curated `BbrGlyph` set in
  the §7 geometric family and render footer Space chips (and the space-config
  glyph picker) from real glyph paths; the canonical two-letter badge remains
  only as the §7 missing-glyph fallback for non-curated values.
- DESIGN.md updates in the same group: §8 pitch values, §11.11 anatomy,
  §11.4 hairline, §11.3 glyph-catalog status flip from "until the catalog
  ships".
- Gallery: affected row/folder/footer stories updated; new stories append-only.

## Non-goals

- No mixed pinned/folder ordering (settled above); no structural-model or
  drag-semantics change.
- No theme-selection, favicon or media work — owned by the sibling packets.
- No density auto-switch, tooltips or pinned-overflow decisions (§17).

## Parallel-lane contract

```text
Base condition:          integrated system-appearance-theme (calibrates both modes)
Depends on:              system-appearance-theme
Provides/API:            reference row pitch, folder glyphs, identity-glyph catalog
Exclusive paths:         bbr-ui row/folder/footer/icon components and density tokens
Shared integration points: Gallery registry, dense/organize/sidebar E2E, testkit snapshot, DESIGN.md
Integration order:       after system-appearance-theme; favicon pipeline independent
```

## Constraints

- Geometry changes are token-value changes in the §8 authority; ids, the §10
  state vocabulary, interaction routes and hit-target minima are unchanged.
- Dense-500 bounded-window rendering and scroll math stay exact under the new
  pitch; no Dense assertion is weakened — geometry constants update to the
  contract values.
- Glyphs follow §7 (geometric, 1.5 pt stroke, no emoji fallback for chrome);
  curated emoji identity rendering is unchanged.
- Both theme modes are calibrated; the contrast gate keeps recomputing from
  tokens. Performance execution remains blocked under the active override.

## Evidence

### Unit and contract

- [ ] Density table exposes the new pitch tokens; row/pitch math is proven
      for S/N/XL.
- [ ] Folder row renders glyph-led anatomy with disclosure variants and
      hover-revealed count.
- [ ] Every curated `BbrGlyph` resolves to a drawn path; non-curated values
      still fall back to the two-letter badge.

### Real-app E2E

- [ ] Sidebar suite asserts the hairline before New Tab, quiet address/Reload
      rest states and glyph-rendered Space chips.
- [ ] Dense suite passes with the updated pitch constants for 500 identities,
      including scroll-into-view exactness.
- [ ] Organize/Spaces suites stay green: regions, drag, reorder and space
      switching are behaviorally unchanged.

### Performance

- [ ] Not executed: performance execution remains blocked under the active
      override. Scenario code, budgets and sample guards compile unchanged.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-ui -p bbr-browser
cargo test -p bbr --features testkit --bins
cargo clippy -p bbr-ui --all-targets -- -D warnings
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-shell-dense
just e2e-shell-organize
just e2e-shell-spaces
just e2e-gallery
just roadmap
```

## Delivery sequence

1. One exclusive writer implements and verifies the bounded slice
   (pitch → folder anatomy → hairline/rest states → glyph catalog).
2. One independent read-only reviewer maps requirements to evidence.
3. One remediation writer only for concrete findings, then a focused recheck.

## Iteration policy

Compare each change against Patrick's reference captures region by region;
fix the smallest owning token/component for the highest-impact mismatch and
re-run the focused geometry/semantic gate before the next capture.

## Blocked stop condition

Stop fail-closed if the new pitch cannot preserve exact dense scroll math, if
a curated glyph cannot meet the §7 family within the tile geometry, or if a
required gate fails without a defensible bounded fix. Preserve exact
fingerprints, findings and the smallest input needed.

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
