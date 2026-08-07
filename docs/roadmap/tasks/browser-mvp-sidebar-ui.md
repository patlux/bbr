# Task packet — browser-MVP sidebar UI (U2)

Parents: [Phase 4](../phases/04-ui-shell.md) · [Phase 5](../phases/05-command-bar.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Performance track](../tracks/performance-release.md) ·
[Performance contract](../../product/performance-contract.md) ·
[Interaction contract](../../product/browser-interactions.md) ·
[DESIGN.md](../../../DESIGN.md) §5/§7/§9/§11.2–§11.4/§11.10/§11.11/§12/§15.

## Superseded footer scope (product revision, not a defect)

This packet shipped correctly against the specification in force at the time.
After reviewing the delivered sidebar, Patrick revised the product target: the
footer `[Spaces]` toggle and its revealable space-row region are removed in
favor of a permanent, horizontally scrollable space-identity strip that takes
all remaining footer width
(`[Downloads icon] [Spaces strip] [Plus icon]`, DESIGN §11.3/§11.4). That
revision, including migration of the toggle-era assertions, is owned by the
[footer space strip, appearance, and swipe packet](space-footer-strip-appearance-ui.md).
Everything else in this packet remains current.

## Outcome

The sidebar renders the compact MVP hierarchy — brand block, command field,
toggleable spaces region *(superseded, see above)*,
**Favorites grid → Pinned → Folders → Tabs**,
status row, and the exact footer `[Downloads] [Spaces] [+]` — with a real
plus context menu (`New Space` / `New Folder`) that drives controller
actions. Every region exposes deterministic `bbr-*` ids and §15 semantic
nodes, existing row behavior and ids are preserved byte-for-byte where
asserted, the footer `[Spaces]` control reveals the §11.3 space rows, and
sidebar presentation budgets hold for the new surfaces.

## Scope

- **Rail composition** on the Root contract (`WindowHandle<Root>` +
  `Entity<BbrApp>`): region order per DESIGN §11.4 (brand, command field,
  spaces region, Favorites grid, Pinned, Folders, Tabs, status row with the
  collapse control, footer row). The status row relocates directly above
  the footer; `bbr-status` copy and `bbr-sidebar-toggle` ids are unchanged.
- **Footer row:** exactly `[Downloads] [Spaces] [+]`
  (`bbr-downloads`/`bbr-spaces`/`bbr-plus`) with §7 glyphs + labels,
  separated from the list by a hairline. `[Downloads]` is a placeholder
  control in this slice (the downloads *surface* is Phase 8; the portable
  capability contract is the L6 lane). `[Spaces]` toggles the spaces
  region.
- **Plus menu:** `[+]` opens a real context menu (`bbr-plus-menu`,
  `elev:2`, `material:overlay`, `radius:md`) with exactly two items in v1 —
  `bbr-plus-new-space` and `bbr-plus-new-folder` — wired to the
  controller actions provided by the favorites/folders domain slice (L5):
  New Space creates a real space; New Folder creates a session-local
  folder. Keyboard-operable (↑/↓, Enter commits, Escape dismisses without
  action), outside pointer press dismisses (one pointer action, §13),
  invoking focus domain preserved.
- **Spaces region toggle:** the `[Spaces]` control reveals/hides the §11.3
  space rows (rendered directly under the command field); toggle snaps
  (`dur:instant`); row ids, §12 chips, and activation behavior unchanged.
  Migrate the existing spaces suites to reveal the region first.
- **Favorites grid UI** (§11.10) and **Folders UI** (§11.11) consuming the
  L5 API: window-global favorite tiles resolving concrete
  `TabId + SpaceId + ProfileId`, visible unchanged across every space/profile
  in the window and activating the exact owning tab via its space/container;
  grid tiles with §7.2 favicon/monogram fallback and fixed per-density column
  counts; folder rows remain space-local with disclosure/count and nested
  §11.1 rows, expand/collapse snaps, `drag-target` on folder rows.
- **Icon glyphs:** extend the §7 `components::icons` catalog with the
  `Downloads`, `Spaces`, `Folder`, and `Disclosure` glyphs (icon-system
  slice carry-over); every icon context keeps its text alternative (§7).
- **Snapshot and semantic exposure:** new ids in `visible_ids`, region/
  menu/folder state on the typed snapshot, §15 nodes mirroring the render.
- **Real-app E2E:** extend `just e2e-shell-sidebar`; migrate
  `e2e-shell-spaces`/`e2e-shell-organize` traversal where the rail changed.
- **Performance:** `sidebar_plus_menu`, `sidebar_favorites`, and
  `sidebar_folder_toggle` scenarios in the gate-enforced matrix.
- **Docs:** this packet; DESIGN §11.4/§11.10/§11.11 (already prepared by
  the L4 docs lane); interaction-contract note for §9 traversal/order
  wording (owned by this packet's writer at implementation);
  ROADMAP/phase/track promotion is parent-owned at integration.

## Non-goals

- No downloads content surface (L6 contract feeds the `[Downloads]`
  control only; the surface is Phase 8).
- Dragging into Favorites is the required add/remove path; no additional tab
  context-menu management UI in this slice. L5 owns the corrected
  `bbr-domain`/`bbr-browser` concrete-tab favorite commands
  (NewFolder/NewSpace/Fav-Toggle/FavoriteActivate/Move-in-Folder) and their
  invariants.
- No persisted folders/favorites: session-local in v1; persistence and
  restoration are Phase 7.
- No dense-list virtualization changes, no split work, no palette surfaces
  ([U1](browser-mvp-palette-ui.md)), no cross-context profile-hint changes,
  no packaging.
- No platform Accessibility exposure (GPUI 0.2.2 has no layer; Phase 9).

## Parallel-lane contract

```text
Base commit:          f89a4799999d1d43f785da7c767a1df68759ee0b — U-lanes branch only
                      from the reviewed green gpui-ce-components-anchor
                      integration commit (parallel-delivery rule; the
                      argument is recorded when L0 lands). This packet was
                      prepared docs-only at
                      574750bd129df628a0f74ff95e48579f4c12e0ef.
Depends on:           L0 gpui-ce-components-anchor integrated; L5
                      favorites/folders domain + controller integrated
                      (NewFolder/NewSpace/Fav-Toggle/Move-in-Folder
                      commands and snapshot data); U1 palette merged before
                      U2 per the serial order; DESIGN §11.4/§11.10/§11.11
                      revision (this wave).
Provides/API:         sidebar rail composition, footer + plus menu, spaces
                      region toggle, favorites grid UI, folders UI, §7 icon
                      catalog additions.
Exclusive paths:      crates/bbr-app sidebar composition and shell
                      component files (sidebar_view.rs, components::
                      favorites/folders/footer/menu modules), §7 icon
                      catalog additions, and the e2e-shell-sidebar/
                      spaces/organize suite extensions.
Shared integration points: bbr-app composition root and global focus
                      routing; testkit snapshot wire fields; perf-matrix
                      scenario registration; ROADMAP/phase/track roll-ups.
Integration order:    L0 → L5 → U1 → U2 → L2 → L6 → L7 → L3 → L1 (serial;
                      orchestrator merges).
Integration base arg: f89a4799999d1d43f785da7c767a1df68759ee0b, recorded after L0 integration and its reviewed testkit shutdown remediation; merge command runs against this recorded anchor.
```

## Constraints

- Runs on the one pinned GPUI-CE type world and the Root window contract;
  no crates.io fallback, no dual paths, no `gpui-component::webview`.
- Preserve existing ids byte-for-byte where the tree asserts them:
  `bbr-sidebar`, `bbr-sidebar-column`, `bbr-brand`, `bbr-command-bar`,
  `bbr-scroll`, `bbr-status`, `bbr-sidebar-toggle`, `bbr-sidebar-divider`,
  `bbr-section-pinned` / `bbr-section-tabs`, `bbr-tab-row-<n>`,
  `bbr-space-list` / `bbr-space-row-<n>` / `bbr-space-new`,
  `bbr-drag-tick-*`; no semantic/protocol regression in the existing shell
  suites.
- Region order is fixed per DESIGN §11.4; tile/row indexes are
  deterministic (fixed per-density grid columns, 0-based folder nesting);
  no pixel comparisons, no sleeps, deterministic fixtures and deadlines.
- The footer row is exactly `[Downloads] [Spaces] [+]` — no fourth
  control; the plus menu has exactly the two v1 items.
- Folders are session-local: the app layer may hold folder presentation
  state only; no domain DTO/persistence/migration change in this packet
  (L5 owns the model).
- Tokens, states, and motion per DESIGN: §7 glyphs with text alternatives,
  §10 state vocabulary, `dur:instant` snaps for toggle/expand, no raw
  colors, no emoji, no animation on persistent state changes.
- Performance: new scenarios measured through semantic presentation
  commit with the warm budgets and ≥30 full-gate samples; sidebar
  open/close/toggle/resize rows stay gate-enforced; GUI suites sequential
  under the host-wide lock; no weakened budgets, sample counts, or guards.
- Keep files under ~500 lines; no new dependencies.

## Evidence

### Unit and contract

- [ ] §15 semantic-tree tests: footer controls (`bbr-downloads`/
      `bbr-spaces`/`bbr-plus`), plus-menu items (`bbr-plus-new-space`/
      `bbr-plus-new-folder`), favorites tiles (`bbr-favorites`/
      `bbr-fav-<n>`/`bbr-fav-icon-<n>`), folder rows (`bbr-folder-row-<n>`/
      `bbr-folder-name-<n>`/`bbr-folder-count-<n>`/`bbr-folder-toggle-<n>`
      with the expanded/collapsed value), and nested rows
      (`bbr-folder-item-<n>-<m>` + `-title`/`-favicon`/`-close`)
      mirror the render with the documented roles/names.
- [ ] `visible_ids` derivation tests: footer/region ids exposed only when
      the respective surface/region is shown.
- [ ] Plus-menu state tests: open/close, keyboard selection, Escape
      dismiss without action, outside-press dismiss, action dispatch to
      the L5 commands, focus-domain preservation.
- [ ] Favorites ownership/activation tests: identical global tile order across
      same-profile and cross-profile space switches; each tile carries the
      exact `TabId + SpaceId + ProfileId`; activation switches inside the same
      window to the owning space/container and selects that exact tab without
      copying/moving it or recreating its existing session; closing removes the
      tile; moving the tab makes the favorite follow the stable `TabId`.
- [ ] Folder expand/collapse and spaces-region toggle state tests (snap
      semantics, id stability across toggles).

### Real-app E2E

- [ ] Extended `just e2e-shell-sidebar`: region order and fixed footer ids
      (`bbr-downloads`/`bbr-spaces`/`bbr-plus`); `[Spaces]` reveal/hide with
      unchanged `bbr-space-row-<n>` behavior; plus menu via pointer and
      keyboard — New Space creates a real space visible in the revealed
      region, New Folder creates a session-local folder with count 0;
      favorites grid renders tiles (`bbr-favorites`, `bbr-fav-<n>`,
      `bbr-fav-icon-<n>`) with §7.2 fallback and activates; folder toggle
      expands nested rows with deterministic `bbr-folder-item-<n>-<m>` ids
      and stable `bbr-folder-row-<n>`/`-name`/`-count`/`-toggle` ids;
      status row + collapse/divider behavior unchanged; empty-region label
      hiding.
- [ ] Migrated `just e2e-shell-spaces` and `just e2e-shell-organize`
      suites still green against the reordered rail (spaces suites drive
      the `[Spaces]` toggle first).

### Performance

- [ ] `sidebar_plus_menu`, `sidebar_favorites`, `sidebar_folder_toggle`
      scenarios in the gate-enforced matrix: 30 samples each, measured
      through semantic presentation commit; warm 50/100 ms budget class;
      driver asserts the resulting ids/state; guards reject
      fast-but-wrong (missing/invented surfaces).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar   # extended sidebar/footer/menu suites
just e2e-shell-spaces    # migrated to drive the [Spaces] toggle
just e2e-shell-organize  # unaffected-rail regression
just e2e-perf
just e2e
just roadmap
```

## Delivery sequence

1. One exclusive writer implements the slice on the recorded post-anchor
   base and verifies targeted checks without committing.
2. Parent inspects the diff and runs the gates.
3. One independent read-only reviewer maps requirements to evidence.
4. One remediation writer only for concrete findings, then a focused
   recheck.
5. Parent commits and merges serially on the anchor with the recorded
   integration base arg.

## Iteration policy

Start with the smallest end-to-end path: the footer row + `[Spaces]`
toggle with the existing space suites migrated, verify targeted, then the
plus menu with New Space, then the favorites grid and folders UI on the L5
API, then the icons and perf scenarios. After each failing check, identify
the mechanism before changing the implementation; run targeted feedback
while iterating and the canonical sequential gate once the slice is stable.
Do not broaden scope to solve unrelated findings.

## Blocked stop condition

If no defensible path remains, stop with evidence gathered, approaches
attempted, preserved worktree/branch state and owner, the exact blocker, and
the smallest environment, contract decision, or user input required next
(e.g. an L5 command gap for New Folder, a menu-widget limitation in the
pinned GPUI-CE revision, or a DESIGN §11.4 region ruling that contradicts
the interaction contract).

## Delivery ledger

```text
Outcome:       reviewed prep complete; implementation remains pending in U2
Commits:       parent-owned docs integration pending from the reviewed L4 lane;
               implementation commits remain none
Functional evidence: pending
Real-app E2E:  pending
Performance evidence: pending
Roadmap/documentation updates: DESIGN §11.4/§11.10/§11.11 revised by L4;
               ROADMAP/phase/track linking is parent-owned at promotion
Open blockers and non-goals: L0 base is recorded; U2 still requires L5 and U1
               integration. Downloads surface (Phase 8), favorites add/remove
               UX (open decision), and folder persistence (Phase 7) remain excluded
Worktree state: reviewed L4 docs diff at base 574750b; parent promotion records
               U2 base f89a4799999d1d43f785da7c767a1df68759ee0b
```

Planning/read-only, implementation, test execution, review, remediation,
waiting/retries, total elapsed, and human intervention fields follow the
template at implementation time; none measured here (docs-only prep).
