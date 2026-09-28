# Task packet — Folder context menu

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[interaction contract](../../product/browser-interactions.md) ·
[DESIGN.md](../../../DESIGN.md) §11.13 ·
[UI library anchor](ui-library-anchor.md) ·
[Gallery browser components](component-gallery-browser-components.md) ·
[Gallery compositions](component-gallery-compositions.md).

Activation condition: complete F2a and G1a are integrated and Folder/menu territory is writer-free, as recorded in [Current delivery](../../delivery/current.md).
Implementation must not interrupt F2c → F2b → fresh F2a or start before the
settled G1a `bbr-ui` anchor.

Stable information-architecture reference, not a feature or visual copy:
`/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-folder-context-menu.png`.

Blueprint D25 (DESIGN §19, 2026-09-29): Paper board M-2 draws this menu as
**Rename…**, **New Tab in Folder**, **Open All Tabs**, then **Delete Folder —
keep its tabs** (the settled Unpack) and a danger row **Close Folder and Its
Tabs…**. The danger row conflicts with the 2026-09-27 settlement below and
stays omitted until Patrick answers §19 question 13. The board's rows replace
the initial action list for the first slice; Change Icon…, Suspend All Tabs and
Move to Space › remain scoped follow-ups of this packet. Rename reuses the D20
inline rename.

**First slice implemented 2026-09-29 (D25):** Rename…, New Tab in Folder and
Delete Folder — keep its tabs ship through the Tab-menu machinery bound to a
`FolderId` (`TabMenuState.bound_folder`, `app/tab_menu/folder.rs`). Open All
Tabs is omitted as well: BBR has no settled meaning for it (wake every
suspended member, or select them?), recorded in §19 question 13. Unpack is
`FolderStore::unpack_folder` → `OrganizationController::unpack_folder`
(`OrganizationEvent::FolderUnpacked`).

## Outcome

Secondary-clicking a folder opens one production BBR menu bound to the exact
session-local `FolderId`. Opening, traversing, or dismissing the menu does not
expand/collapse the folder, select or move a tab, unload a session, or change
the active profile/space. Every committed action has explicit folder semantics
and routes through a portable organization authority.

## Scope

- Add an app/controller-owned typed `FolderMenuSource`/availability matrix
  keyed by `FolderId`, owning `SpaceId`/`ProfileId`, member identities/order,
  expanded state, active/split membership, lifecycle mix, and advertised
  capabilities.
- Project it into a presentation-only `bbr-ui::FolderMenuModel` with callbacks
  pre-bound by `bbr-app` to the real FolderId. `bbr-ui` imports no FolderId or
  organization authority. The menu is confined to sidebar chrome and does not
  use `NativeAboveWeb`.
- Initial actions only where semantics are settled:
  - `Rename Folder…`;
  - `Change Icon…` through the curated BBR glyph catalog;
  - `Suspend All Tabs` for eligible members, with a typed result rather than
    silent partial failure;
  - `Move to Space ›` with explicit destination profile identity;
  - `Unpack Folder`, retaining tabs in deterministic order.
- Deleting a folder is `Unpack Folder` (settled by Patrick, 2026-09-27): the
  container is removed and its members stay as loose pinned tabs in folder
  order, per the [folder membership invariant](folder-pinned-invariant.md).
  There is no separate `Delete Folder` row, and no folder row ever closes
  tabs.
- Keep `New Subfolder…` omitted while folders remain flat and session-local.
- Keep `Convert Folder to Space` as a separate product/ownership decision; it
  is not a cosmetic menu callback.
- Stable ids: `bbr-folder-menu`, `bbr-folder-menu-rename`, `-icon`,
  `-suspend-all`, `-move-space`, `-convert-space`, `-unpack`.
- Add typed semantics/snapshot state, real-app E2E, optimized open/commit
  evidence, and Gallery G2/G3 stories from the same production component.

## Non-goals

- No nested folders before a domain and persistence contract owns them.
- No durable folder claim while the current model remains session-local.
- No implicit tab closing from folder deletion.
- No `Convert Folder to Space` until ownership, ordering, persistence,
  selection, profile, and session consequences are resolved in a separate
  bounded packet.
- No render-callback mutations, copied Gallery menu, WebKit/Portal work, or
  Zen styling.

## Parallel-lane contract

```text
Base condition:       verified main after F2c → F2b → fresh F2a and settled G1a
Depends on:           G1a bbr-ui menu/folder components; existing session-local
                      folder organization authority
Provides/API:         app/controller FolderMenuSource + availability;
                      presentation-only bbr-ui FolderMenuModel; typed folder
                      commands/results and production menu composition
Exclusive paths:      post-G1a bbr-ui folder-menu modules; focused organization
                      domain/controller modules; sidebar menu composition;
                      focused E2E/perf modules
Shared integration points: controller facade, bbr-app focus/input routing,
                      testkit snapshot, perf registry, Gallery G2/G3 registry,
                      roadmap roll-ups
Integration order:    F2c → F2b → fresh F2a → G1aa → this packet
```

No concurrent writer may own folder components, organization commands,
sidebar rendering, or their Gallery stories.

## Constraints

- App-owned source identity is the invoking `FolderId`; opening never toggles
  it. `bbr-ui` receives only presentation data and pre-bound callbacks.
- Current folders are flat and session-local. No UI copy may imply nested or
  persisted behavior.
- Folder operations never silently mutate tab canonical order, space ownership,
  profile isolation, or live session identity beyond the named command.
- Same-profile moves preserve sessions; cross-profile folder moves recreate
  each moved tab exactly once in the destination store and report every
  result deterministically.
- `Suspend All Tabs` preserves every tab identity and excludes active/otherwise
  ineligible members according to the controller's lifecycle policy.
- Unpack and partial-failure copy must be deterministic and testable; color is
  never the sole danger/disabled cue.
- No new dependencies; files below roughly 500 lines; no weakened assertions,
  budgets, or sample counts.

## Evidence

### Unit and contract

- [ ] Availability matrix covers empty/populated, collapsed/expanded, mixed
      live/suspended, active-tab member, split member, same-/cross-profile
      destination, and missing capabilities.
- [ ] Opening/dismissing leaves folder expansion, browser selection, order,
      sessions, and ownership unchanged. *(First slice: proven in the real-app
      E2E below; no portable availability matrix yet.)*
- [ ] Rename/icon validation, suspend-all result accounting, move isolation,
      and unpack ordering (the settled delete semantics) are proven. *(Unpack
      ordering, pinning and session preservation:
      `unpacking_a_folder_keeps_its_members_as_loose_pinned_tabs`; menu rows:
      `tab_menu::folder` tests. Icon, suspend-all and move remain open.)*
- [ ] Nested/convert rows cannot appear without their owning capabilities.

### Real-app E2E

- [ ] Open without toggling the folder; traverse by keyboard; Escape/outside
      dismissal is a no-op.
- [ ] Rename and icon update the bound folder only.
- [ ] Suspend All preserves identities and reports ineligible/failed members.
- [ ] Same-/cross-profile move repairs source selection and preserves isolation.
- [ ] Unpack preserves deterministic order, keeps every member as a loose pin,
      and never closes a tab.

### Performance

- [ ] Seed and enforce warm `folder_context_menu_open` and a representative
      folder-menu commit kind with at least 30 accepted full-gate samples.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-domain -p bbr-browser
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-browser --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-shell-sidebar
nix develop -c just e2e-shell-organize
nix develop -c just e2e-shell-folder-menu
nix develop -c just e2e-perf
nix develop -c just e2e
nix develop -c just roadmap
```

GUI suites run sequentially under the repository lock.

## Delivery sequence

1. Confirm F2/G1a and folder-membership-invariant integration; the delete
   rule is settled as Unpack above.
2. Add typed folder availability and commands to the portable organization
   seam.
3. Cut over the production `bbr-ui` menu and sidebar anchor.
4. Add real-app/semantic/performance evidence and real-production Gallery
   stories.
5. Independent read-only review; one remediation writer only for concrete
   findings; focused recheck.

## Iteration policy

Start with no-op open/dismiss plus Rename. Add one command group at a time only
after its portable semantics are explicit. Diagnose failures as source
identity, folder-domain semantics, lifecycle accounting, profile isolation,
focus/dismissal, or presentation evidence.

## Blocked stop condition

If delete, conversion, nesting, persistence, or cross-profile group movement
cannot be defined without inventing product behavior, stop with the preserved
worktree, evidence, and smallest product/domain decision required. Leave the
row omitted rather than implementing ambiguous or destructive behavior.

## Delivery ledger

```text
Outcome:                first slice (D25) — Rename…, New Tab in Folder, Delete
                        Folder — keep its tabs; secondary-click on a folder row
Commits:                see git log (feat commit carrying blueprint D25)
Functional evidence:    cargo test -p bbr-browser organization_space_tests;
                        cargo test -p bbr tab_menu::folder
Real-app E2E:           just e2e-shell-organize (e2e_shell_folder_menu:
                        open without toggle, Escape no-op, rename, new tab
                        joins pinned, unpack keeps tabs and sessions)
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.13 folder menu, §19 D25 + question 13
Open blockers and non-goals: Open All Tabs / closing row (question 13); Change
                        Icon…, Suspend All Tabs, Move to Space › follow-ups
Worktree state:         main, clean after commit

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```
