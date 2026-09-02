# Task packet — Sidebar reference polish (row pitch, folder anatomy, identity glyphs)

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Reference-faithful production shell](visual-shell-reference.md) · [DESIGN.md](../../../DESIGN.md).

Historical implementation boundary (Patrick, 2026-08-13): this completed
slice kept loose pinned rows above folder rows and made no ordering-model or
folder-membership change. Patrick's superseding 2026-08-14 design decision
classifies folders as grouped pinned tabs inside one organized area while still
rejecting arbitrary mixed row interleaving. The separate
[folder membership invariant](folder-pinned-invariant.md) owns the portable
`folder member => pinned` transition work; this packet remains a truthful
record of the already integrated visual polish.

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

- No mixed pinned/folder ordering; no structural-model, membership-invariant,
  or drag-semantics change. The later invariant packet owns grouping semantics.
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

- [x] Density table exposes the new pitch tokens; row/pitch math is proven
      for S/N/XL. The row surface remains 32/36/44 pt and the exact pitch is
      36/40/48 pt through the shared 4 pt gap authority.
- [x] Folder row renders glyph-led anatomy with distinct closed/open variants
      and a typed hover/focus-driven `count_revealed` state; nested rows and
      dense extents step on the same pitch.
- [x] Every curated `BbrGlyph` resolves to a drawn path; footer chips and the
      space-config picker share the classifier, while only non-curated values
      retain the two-letter badge fallback.

### Real-app E2E

- [x] Sidebar suite asserts the hairline before New Tab, quiet address/Reload
      rest states and glyph-rendered Space chips: 15/15 on the final tree.
- [x] Dense suite passes with the updated pitch constants for 500 identities,
      including scroll-into-view exactness: 3/3 across the main and edge
      suites on the final tree.
- [x] Organize/Spaces suites stay green: regions, drag, reorder and space
      switching are behaviorally unchanged. Organize passed 2/2 plus
      Drag-Chip 4/4, Spaces passed 21/21, and Gallery passed 45/45.

### Performance

- [ ] Not executed: performance execution remains blocked under the active
      override. All three performance test targets compile with `--no-run`;
      budgets, scenarios and sample guards are unchanged.

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
Outcome: Reference-calibrated Sidebar implementation and final-tree evidence
         complete; integrated on main through merge 35c91189a4a895e3c7f627534acf00a56899fc7b.
Commits: e1081aeb1dd63ba36883740dd295232aa619e249 fixes the normal-build
         theme seam; 2ffc27fa8ba68d4322a4e20f7ee38eb676124eb9 implements the
         reference anatomy and regression coverage; this documentation commit
         records the settled contract and evidence.
Functional evidence: cargo fmt --all -- --check; bbr-ui 147/147; bbr-browser
         261/261; bbr app bins 282/282; bbr-testkit 30/30; bbr-gallery unit
         catalog 148 passed / 45 ignored; fresh warning-denied bbr-ui and bbr
         all-target Clippy; roadmap, UI-dependency, 46 script tests and
         diff-check. Logs: /tmp/bbr-polish-commit-gates/.
Real-app E2E: final tree passed Sidebar 15/15, Dense 3/3, Organize 2/2,
         Drag-Chip 4/4, Spaces 21/21 and Gallery 45/45 through the host GUI
         lock. The first Sidebar attempt was invalidated by a concurrently
         restarted manual app; the uncontended full rerun passed. Focused
         remediation preserved the no-target contract above the list and gave
         the zero-height Gallery tick an explicit width authority.
Performance evidence: execution intentionally forbidden by the active override;
         e2e_performance, e2e_performance_full and e2e_performance_inject all
         compile with --no-run.
Roadmap/documentation updates: DESIGN.md §8/§11.3/§11.4/§11.11 records the
         implemented pitch, glyph catalog, organized hairline and folder anatomy;
         this packet records only passed evidence.
Open blockers and non-goals: no blocker remains for this completed visual
         slice. Media controls and the later folder-member/pinned invariant
         remain separately owned and open.
Worktree state: the three lane commits are ancestors of main through merge
         35c91189a4a895e3c7f627534acf00a56899fc7b; the historical lane no longer
         owns live delivery state.
```
