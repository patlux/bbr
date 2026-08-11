# Task packet — Daily-driver Sidebar main

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [browser interaction contract](../../product/browser-interactions.md) · [G3 compositions](component-gallery-compositions.md) · [daily-driver delivery](../../delivery/2026-08-12-daily-driver-delivery.md).

Visual interaction references supplied by Patrick:

- tab rows: `/Users/patwoz/Library/Application Support/CleanShot/media/media_9pNl4Eyem3/Screenshot_2026-08-12_00-22-37@2x.png`
- pinned/regular divider: `/Users/patwoz/Library/Application Support/CleanShot/media/media_tVbMF5ZRzR/Screenshot_2026-08-12_00-22-57@2x.png`
- Favorites: `/Users/patwoz/Library/Application Support/CleanShot/media/media_CbmLvnxjt2/Screenshot_2026-08-12_00-23-19@2x.png`

References define scale, hierarchy, and interaction anatomy, not copied brand styling. BBR retains Chassis & Glass tokens and its own icons.

## Outcome

The expanded Sidebar main area is a coherent daily-use browser surface: readable tab rows with native-feeling hover/selection/close behavior, a clear pinned-to-regular boundary, compact Favorite icon tiles, and production drag from a concrete tab into Favorites without creating a copy or second session.

## Scope

- Refine the one production `bbr-ui::sidebar` Tab Row composition: favicon, title, state, hover-reveal close, selected surface, truncation, hit targets, and dense behavior.
- Present Pinned above regular tabs with a clear structural divider and no duplicate section noise.
- Present Favorites as compact icon tiles matching the approved scale/rhythm while retaining exact TabId ownership and accessibility names.
- Add production tab-row drag target for Favorites; drop toggles the exact tab through the existing Favorite authority.
- Preserve folders, scrolling, status, footer, address/command, native content geometry, focus, and 500-tab bounded rendering.
- Update DESIGN tokens/contracts only where the supplied references resolve a currently different canonical behavior.
- Add production Gallery matrices and real-app E2E from the same composition.

## Non-goals

- No new Favorite URL-shortcut model, copied tab, independent session, arbitrary tile reorder, Saved Page conflation, context-menu actions, folder redesign, or Split work.
- No broad brand replacement, glassmorphism, site tint sampling, or screenshot/pixel-comparison acceptance.
- No duplicate Sidebar renderer in `bbr-app` or Gallery.

## Parallel-lane contract

```text
Base condition:          committed Daily-driver delivery authority; integrate after accepted Titlebar and Space cuts
Depends on:              one production bbr-ui Sidebar composition; existing concrete-TabId Favorite commands; icon-only footer contract
Provides/API:            refined Tab/Pinned/Favorites production anatomy and tab-to-Favorite drag path
Exclusive paths:         focused bbr-ui sidebar tab/pinned/favorite modules; focused app drag/assembler modules; focused Sidebar/Organize/Gallery evidence
Shared integration points: sidebar composition root, drag state, snapshot/semantics, Gallery registry, performance registry, roadmap/DESIGN
Integration order:       after DD-SPACE, before context-menu cuts
```

## Constraints

- A Favorite is one additional presentation of an existing concrete `TabId`; drop never opens, copies, navigates, moves, or recreates the tab.
- Hover/press/selected states follow one token authority and stable semantic ids. Close appears only in the intended hover/focus/selected states and targets its row.
- Pinned/regular separation does not change pinning, selection, ordering, or dense-list math.
- Footer and full-height header remain composition siblings, not locally reimplemented.
- Real-app assertions use typed state/ids and geometry, not pixels. Screenshots supplement visual review only.
- No weakened tests, content floor, 500-tab bounds, session/focus/profile invariants, budgets, or sample guards; files below roughly 500 lines.

## Evidence

### Unit and contract

- [ ] Tab row visual/state matrix covers default/hover/pressed/selected/loading/error/unresponsive, long title, missing favicon, close visibility, and compact/dense geometry.
- [ ] Pinned/regular separator and ordering preserve domain state and deterministic row identity.
- [ ] Favorite tile model resolves exact TabId/SpaceId/ProfileId and drag toggle is exactly-once/no-copy/no-session-mutation.
- [ ] Rejected drag targets and Escape leave state unchanged.

### Real-app E2E

- [ ] Tab row pointer/keyboard selection and hover close target exact rows with focus-domain preservation.
- [ ] Pinned/regular boundary remains clear and pin/unpin/reorder behavior unchanged.
- [ ] Drag regular or pinned tab into Favorites creates one tile for the exact tab; repeated drop removes/toggles according to the canonical Favorite command without closing it.
- [ ] Activating the tile returns to its owning Space/Profile and exact warm tab.
- [ ] Dense 500-tab, compact window, missing icon, long title, Sidebar collapse/restore, Space footer, and native content geometry regressions pass.

### Performance

- [ ] Existing tab hover/press/select and Sidebar scenarios retain their code/guards; tab-to-Favorite drag gains a production semantic commit scenario if no existing Favorite toggle kind covers it.
- [ ] Execution remains unexecuted/unpassed under Patrick's override.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-ui -p bbr-browser
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets -- -D warnings
just ui-deps
just e2e-shell-sidebar
just e2e-shell-hover
just e2e-shell-organize
just e2e-shell-dense
just e2e-gallery
just roadmap
git diff --check
```

## Delivery sequence

1. Lock the production Tab Row visual/state model to the approved anatomy.
2. Refine Pinned/regular separation without changing organization behavior.
3. Refine Favorite tiles, then add exact-Tab drag toggle.
4. Add typed semantics, real-app matrices, performance code, and Gallery stories.
5. Independent read-only visual/product/architecture review; one bounded remediation; focused recheck; Orchestrator integration.

## Iteration policy

Start with one normal tab row and one Favorite tile through production rendering. Add state and density matrices, then drag. Classify failures as visual model, listener propagation, exact identity, drag geometry, focus, dense windowing, or composition duplication.

## Blocked stop condition

If the existing Favorite authority cannot toggle an arbitrary dragged concrete TabId without selecting or recreating it, stop with controller/API evidence and the smallest serial command seam required. Do not simulate a drop by selecting the tab or saving only its URL.

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

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```
