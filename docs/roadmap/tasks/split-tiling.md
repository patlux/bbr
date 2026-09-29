# Task packet — Split tiling (blueprint D23)

Parents: [Phase 6](../phases/06-split-view.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[split app composition](split-app-composition.md) ·
[portable split core](split-core.md) ·
[DESIGN.md](../../../DESIGN.md) §11.7, §19 (D23).

Status: adopted 2026-09-29 (DESIGN §19, D23; Split block lifted by
Patrick). Supersedes the two-pane constraints of the split packets above.
Paper boards: G-9, G-10, PR-09 (file “BBR — Browser Shell”); explainers
`~/Downloads/bbr-d23-split-entscheidungen.html` and
`~/Downloads/bbr-d23-mehrere-seiten.html`.

## Outcome

A space can show several of its tabs at once as a tiling of rows and
columns (any number of panes, side by side and stacked), and a space may
hold several such split groups. Splits are created from the keyboard
(⌃⌘S right, ⌃⌥⌘S below, through the ⌘T palette in split mode) and by
dropping a rail tab on a pane edge; ⌘W closes the focused pane, ⇧⌃⌘S
dissolves the group and keeps its tabs, ⌃⌘+arrow moves focus to the
neighbour. Dividers resize only their two neighbours (double-click
equalizes the row or column). A group that no longer fits (panes below
320 × 200 pt) shows its focused pane with an "n of N" chip that lists the
panes. The rail shows a group's tabs as a block with a thin accent
bracket; dragging a tab out of the block leaves the split. Groups and
ratios survive a restart.

## Scope

The work lands in serial slices, each with its tests, affected E2E and
DESIGN text:

1. **Tiling contract (portable).** `bbr-domain`: a split group is a tree of
   rows and columns over `TabId` leaves with positive weights per branch;
   normalized (branches hold ≥ 2 children, a child never repeats its
   parent's axis, weights sum to 1). Operations: pair, insert beside a
   pane on an edge, remove (collapsing single-child branches, dissolving a
   group of one), replace, move to another pane's edge, resize the two
   neighbours of one divider, equalize a branch, reading-order members.
   A space holds `Vec<SplitGroup>`; each tab is in at most one group.
   `bbr-split`: project a group onto the card (2 pt dividers), report
   divider hit rects with their neighbour pair, the fit test against the
   320 × 200 pt minimum, and the geometric neighbour for ⌃⌘+arrow.
2. **Domain and controller.** The active tab's group (if any) is the
   visible layout; selecting a tab outside it keeps the group, selecting a
   member brings it back. Every member of the visible group is Active;
   `PaneLayout` carries the visible panes, their tree and the focused view.
   Closing a member removes its pane; moving a member to another space
   removes it from the group.
3. **Persistence.** Groups and weights persist in `splits.txt` next to the
   store (no SQLite schema change); a legacy two-pane `splits` row loads
   as a group.
4. **Composition.** One native slot per visible pane at its projected
   rect, dividers with neighbour-only drags and double-click equalize,
   focus by click, the collapsed "n of N" chip and its list.
5. **Commands.** ⌃⌘S / ⌃⌥⌘S open the ⌘T palette in split mode (recents
   first, or a typed address) and append right / below the focused pane;
   ⇧⌃⌘S dissolves; ⌃⌘+arrow moves focus; ⌘W closes the focused pane.
   Menu items and ⌘T actions share the actions.
6. **Rail and drag.** The group's tabs form a contiguous block with an
   accent bracket; dragging a tab out leaves the split. Dragging a rail tab
   over the card shows the four edge zones of the pane under the pointer
   (the Portal draws the preview over the page); a drop inserts beside that
   pane, the centre replaces it.

## Non-goals

- Splits across spaces or profiles (a group lives in one space).
- Pane title bars (rearranging uses the rail tabs).
- Performance evidence (deferred with every blueprint delta, Patrick
  2026-09-27).

## Parallel-lane contract

Not parallelized.

## Constraints

- `bbr-domain` owns the tree and its invariants; `bbr-split` stays
  portable geometry/planning; GPUI owns dividers and chips; native web views
  are only framed, never sampled; overlays over web content use the one
  Portal.
- No SQLite schema change; app-side state in plain files next to the store.
- Profile isolation unchanged: every member of a group is a tab of the
  group's space.
- No weakened tests, budgets or acceptance criteria.

## Evidence

### Unit and contract

- [x] Tree normalization, insert/remove/replace/move/resize/equalize and
      dissolve cases; projection, fit and neighbour geometry. *(Slice 1,
      2026-09-30: `bbr_domain::tiling` with 12 tests, `bbr_split::tiling`
      with 6 tests — projection in whole points, the 320 × 200 fit,
      ⌃⌘+arrow neighbours, the 24 % drop zones, divider drags clamped to the
      minimum.)*
- [x] Controller pane layout, selection keeping a group, lifecycle of
      members, close/move removing panes; persistence round trip. *(Slices
      2–3, 2026-09-30: `Space.splits`, `split_groups` domain ops with 7
      tests, controller `split_*` commands and `split_layout()` with 3
      tests, `splits.txt` codec with 2 tests; the legacy two-pane restore
      E2E and the split-geometry E2E stay green.)*

### Real-app E2E

- [ ] `just e2e-split` covers creation from the keyboard and by drop, focus
      movement, divider resize and equalize, collapse under the minimum,
      dissolve, ⌘W, the rail block and restore, over real WKWebViews.
      *(Slices 4–5, 2026-09-30: `e2e_split` — a three-pane row/column framed
      at the projection, ⌃⌘← and a pane press focus, divider drag and
      double-click, a tab outside keeping the group, the collapsed chip and
      its list, ⇧⌃⌘S, ⌃⌘S with the palette commit, restore of the group and
      its share after a restart. Open: creation by drop and the rail block
      (slice 6).)*

### Performance

- [ ] Deferred (blueprint deltas, Patrick 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-split
just e2e-shell-split-geometry
just e2e
```

## Iteration policy

One slice at a time in the order above; each slice leaves `main` green.
Diagnose failures as tree invariant, projection, lifecycle, composition or
input routing before changing tests.

## Blocked stop condition

If a slice needs a decision the §19 answers do not cover, stop with the
evidence and the question (HTML explainer in `~/Downloads`) instead of
guessing.

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
