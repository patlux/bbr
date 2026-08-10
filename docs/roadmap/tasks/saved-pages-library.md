# Task packet — B3 Saved Pages Library

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[B1 contract/persistence](saved-pages-portable-persistence.md) ·
[B2 actions/command provider](saved-pages-actions-command.md) ·
[UI library anchor](ui-library-anchor.md) ·
[Gallery browser components](component-gallery-browser-components.md) ·
[Gallery compositions](component-gallery-compositions.md) ·
[DESIGN.md](../../../DESIGN.md) §11.16 ·
Source decision: `/Users/patwoz/dev/Personal/macos/bbr-saved-pages-bookmarks-plan.md`.

Activation condition: B1 and B2 are integrated and G1a is settled, as recorded in [Current delivery](../../delivery/current.md). This packet is the
canonical detail owner for the dedicated production Saved Pages Library. It
does not own Saved Page identity/persistence (B1), save/open/Palette/menu
commands (B2), or Gallery-only copies.

## Outcome

BBR provides a dedicated keyboard- and pointer-complete Library for hundreds or
thousands of cold Saved Page URL records. Users can search, sort, rename,
remove, multi-select, and organize records into flat collections; every
mutation persists and every open routes through B2 to exactly one fresh regular
tab in the current active Space/Profile. The full collection is never a
permanent Sidebar region.

## Scope

- Build one ordinary BBR-owned GPUI Library surface from shared production
  `bbr-ui` primitives after G1a. It is not a web page, native Portal, WebKit
  view, or permanent Sidebar section. A small entry point may integrate with
  the settled chrome/command/settings navigation, but it must not duplicate the
  Palette provider or add a second Saved Page authority.
- Consume B1's loaded global snapshot and B2's open/save/remove commands.
  Search/filter/sort operate on in-memory data; mutation persists through the
  one AppStore/repository writer.
- First-slice operations: text search, deterministic sort, rename user label,
  remove, multi-select, create/rename/remove one-level flat collections, and
  move selected records into or out of a collection.
- Opening any row creates exactly one fresh regular tab in the active
  Space/Profile through B2. Library navigation never silently switches to an
  existing matching tab.
- Use bounded/windowed rendering for at least 1,000 records. No full-list
  layout, synchronous repository scan, or favicon network work in render.
- Resolve favicons through the normal production favicon presentation path;
  use the deterministic monogram fallback. Store no favicon blob in Saved Page
  records.
- Deliver decided states: empty Library, loading repository snapshot,
  populated, no search results, selection/multi-selection, destructive remove
  confirmation, long title/URL, missing title fallback, missing favicon, and
  large collection. Keyboard focus/order, semantic names, contrast, reduced
  motion, and stable ids follow DESIGN §11.16.
- Add typed Library snapshot/semantics and real-app E2E for persistent
  mutations, collection operations, exactly-one-tab open, and 1k bounded
  rendering. Add optimized Library search/scroll presentation scenarios.
- After production integration, append the exact Library components and
  compositions to Gallery G2/G3 with stable new StoryIds; Gallery stories use
  production models/callbacks and never replace persistence or browser E2E.

## Non-goals

- No permanent Sidebar collection, nested folders, tags, smart collections,
  page archive, offline reading, thumbnail/page snapshot, favicon blob,
  bookmarklet, sync, collaboration, or profile-scoped Library.
- No Safari/Chrome/Firefox import and no Netscape bookmark HTML export in this
  slice. Those form a bounded explicit-preview follow-up after the native
  Library is stable; import never silently merges duplicates.
- No existing-open-tab switch/dedup behavior, background-open policy, bulk
  editing beyond the listed operations, or arbitrary user-defined sort rules.
- No copied Gallery components and no BrowserController/repository/network
  authority inside `bbr-ui` or `bbr-gallery`.
- No migration/backcompat or new dependency without evidence.

## Parallel-lane contract

```text
Base condition:       verified main after B1+B2 and settled G1a
Depends on:           production bbr-ui primitives/Library composition seam;
                      one AppStore and loaded Saved Page snapshot; B2 commands;
                      free broad UI/Gallery/testkit/perf territory
Provides/API:         production SavedPagesLibraryModel + callbacks; Library
                      surface, typed semantic/snapshot state, E2E/perf; later
                      append-only Gallery stories of the same components
Exclusive paths:      focused bbr-ui Saved Page row/library modules;
                      focused bbr-app Library composition/assembler;
                      focused bbr-gallery stories only after product cutover;
                      focused E2E/perf modules; this packet
Shared integration points: app chrome/internal-surface navigation, bbr-ui and
                      Gallery registries, testkit/perf registries, Phase 8/UI
                      track and ROADMAP roll-ups
Integration order:    B1 → B2; G1a → B3 production cutover → append-only G2/G3
                      registration; import/export is a later packet
```

No concurrent writer may own the Library production components/composition or
their Gallery stories.

## Constraints

- `SavedPagesLibraryModel` is presentation data plus callbacks. `bbr-ui` sees
  no repository connection, BrowserController, WebEngine, native session, or
  network client. `bbr-app` owns snapshot building and command binding.
- Global Saved Page records remain profile-free. B2 resolves the active
  Space/Profile only when Open commits.
- Selection and collection mutation are deterministic; deletion requires a
  clear destructive confirmation for multi-select and never closes open tabs.
- Search/sort/filter never block render and never issue synchronous DB I/O.
- 1,000-record rendering is bounded and semantically complete for the visible
  window; tests use typed rows/anchors, not pixels or sleeps.
- Existing Chassis & Glass tokens/components, stable ids, focus rules,
  contrast, and reduced motion apply. No second visual system.
- Files stay below roughly 500 lines; no weakened tests, budgets, sample counts,
  validators, URL safety, or profile isolation.

## Evidence

### Unit and component contract

- [x] Model/row tests cover empty/populated/no-result, selected/multi-selected,
      long/missing title, mid-ellipsized URL, collection label, missing favicon,
      and destructive states with stable ids and semantic names. *(The
      `bbr-ui` Library row/model suites plus the app-owned Library model tests
      prove `decided_states_are_derived_from_the_model`, the
      `display_title_ladder`, `row_spec_uses_mid_ellipsis_and_the_monogram`,
      `state_transitions_are_deterministic`, `hidden_view_bookkeeping_restores_exactly`,
      and `collection_action_machine_names_are_stable` with the DESIGN §11.16
      stable ids and semantic names.)*
- [x] Search and all deterministic sort orders cover 0, 1, 1,000, equal-key,
      unicode, and collection-filter cases without repository access. *(The
      model tests cover `search_matches_title_label_and_url_case_insensitively`,
      `build_filters_sorts_and_windows_deterministically`, and
      `sort_orders_apply_the_deterministic_key` over in-memory fixtures; no
      repository or render path is touched.)*
- [x] Callback recorder tests prove rename/remove/open/collection operations
      dispatch exactly once against the intended `BookmarkId` set. *(The
      `bbr-ui` recorder-driven tests assert one dispatch per operation against
      the exact `BookmarkId` set; Gallery row/control E2E independently proves
      `library_row_controls_dispatch_exactly_once`,
      `library_collection_row_dispatches_exactly_once`, and
      `library_remove_confirm_dispatches_exactly_once` through the same
      recorder/observer seam.)*
- [x] Bounded-render tests prove the Library does not lay out all 1,000 rows and
      keeps selection/scroll anchors stable across mutation. *(`visible_window`
      is bounded/clamped, `one_thousand_records_stay_bounded_with_a_stable_anchor`
      proves the windowed row set and stable anchor, and
      `library_visible_ids` keeps the typed visible-id window stable across
      mutation.)*

### Real-app E2E

- [x] Open the Library through its integrated production entry point; keyboard
      and pointer search, sort, selection, rename, remove, and flat collection
      operations expose typed semantics and persist across restart. *(`just
      e2e-shell-saved-pages-library` 12/12 on the exact semantic-union runtime
      tree `b4ab98113c6582caad2150764815d3885589dfa7`: Cmd-Shift-L entry + Escape
      dismissal, Cmd-T/Cmd-L replacing an open Library, deterministic
      search/sort, inline pointer rename draft confirm/cancel, destructive
      remove confirmation, flat collection create/rename/move/remove, the
      pointer selection-toolbar clear/remove/move, the pointer collection
      create/rename/remove surface, the production scroll-wheel path, and
      restart persistence — log
      `/tmp/bbr-b3-review-remediation-e2e-shell-saved-pages-library-20260810T144111Z.log`,
      SHA-256
      `06838f3556a34bb46d7dfe20f8b5f6f3bdd3b6d7ce91a19a59b633dfffa11673`.)*
- [x] Removing one or many Saved Pages leaves every open tab/session untouched;
      closing tabs leaves Library records untouched. *(`just
      e2e-shell-saved-pages` 4/4 re-run on the same tree confirms the tab/record
      independence contract on the same runtime tree — log
      `/tmp/bbr-b3-review-remediation-e2e-shell-saved-pages-20260810T144301Z.log`,
      SHA-256
      `440317455fbf499fada4515d404c1f460d0a5854a817490314e5ac65763273cc`.)*
- [x] Opening a Library row creates exactly one fresh regular tab in the
      current active Space/Profile through B2 and never selects a matching tab.
      *(Scenario `library_activation_opens_exactly_one_fresh_tab` in the 12/12
      run above routes through the B2 command authority and asserts one fresh
      regular tab + one live session in the active Space/Profile.)*
- [x] The 1k fixture proves bounded rows, deterministic scroll/selection, long
      data/fallback states, and no second Saved Page authority. *(Scenario
      `library_one_thousand_records_stay_bounded` drives the seeded
      ≥1,000-record provider and asserts the bounded window, deterministic
      scroll/selection, and long/missing fallbacks from the typed snapshot.)*

### Performance

- [ ] Seed/enforce `saved_pages_library_search` and
      `saved_pages_library_scroll` over 1,000 loaded records, at least 30
      optimized accepted samples each, production semantic presentation
      commits, raw artifacts, and predeclared budgets. *(The scenario drivers,
      fast-but-wrong guards, budget classes, and raw-artifact plumbing exist in
      `crates/bbr-app/tests/perf_common/library_drive.rs` and the
      `saved_pages_library_search`/`saved_pages_library_scroll` benchmark kinds,
      but they have **zero executions**: no sample has ever been accepted, no
      budget class has run against them, and this box stays unchecked and
      unpassed. Patrick deferred current-MVP performance execution; the
      canonical `just e2e`/`just e2e-perf` gates were not run for this slice.)*
- [ ] Repository/fixture setup and favicon network latency are outside app
      timing. Fast-but-wrong guards verify exact result window/count and scroll
      anchor. *(Guards assert the exact `LIBRARY_SEARCH_NARROW_TOTAL`=11 result
      window anchored on `BookmarkId` 1000 and the exact scroll window/anchor
      from the typed snapshot before a sample may be accepted; still never
      executed — unchecked.)*

### Gallery roll-up

After B3 production integration, append the exact production row/control and
Library composition stories to [G2](component-gallery-browser-components.md)
and [G3](component-gallery-compositions.md). Those Gallery packets remain the
canonical story/checklist owners; they must not claim persistence, browser
activation, restart, or product-performance evidence. *(Done: the exact
production `bbr-ui` Library rows/controls/surface and the Library composition
are registered through the recorder/observer seam as Gallery stories at ids
23–30; the integrated G2-TAB Tab Row occupies id 22, with its temporary B3-lane
reservation machinery removed by the orchestrator's semantic union. Gallery
final `just e2e-gallery` 22/22 on the exact semantic-union runtime tree — log
`/tmp/bbr-b3-review-remediation-e2e-gallery-20260810T144301Z.log`, SHA-256
`bc41dd1df11d887dea03a10fa30949a33be05edb916462c59fd9cbd25a2ff7e9`. The
Gallery stories prove renderability/recorder dispatch only; they claim no
persistence, browser activation, restart, or product-performance evidence.)*

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo check --workspace --all-targets
CARGO_BUILD_JOBS=4 cargo test -p bbr-ui -p bbr-gallery
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-ui --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-gallery --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-shell-saved-pages-library
nix develop -c just e2e-shell-saved-pages
nix develop -c just e2e-perf
nix develop -c just e2e-gallery
nix develop -c just e2e
nix develop -c just roadmap
git diff --check
```

GUI suites run sequentially under the repository lock.

## Delivery sequence

1. Confirm B1+B2 and G1aa are integrated and broad UI/Gallery ownership is free.
2. Establish the presentation model and smallest empty/populated Library.
3. Add search/sort, then single-record rename/remove/open, then multi-select and
   flat collection mutations; keep every stage working end to end.
4. Add 1k bounded rendering, typed semantics, restart E2E, and performance.
5. Append the exact production stories to G2/G3 only after cutover.
6. One independent read-only review; one remediation writer only for concrete
   findings; focused recheck.

## Iteration policy

Start with empty/populated plus search and exactly-one-tab open. Add mutation
and collections in separate working layers. Diagnose failures as model,
selection, bounded render, AppStore mutation, B2 activation, focus/semantics,
or Gallery duplication. Never move repository/controller authority into
`bbr-ui` or reduce the 1k corpus to make the UI pass.

## Blocked stop condition

Stop with preserved state if G1a/B1/B2 is not integrated, no defensible ordinary
GPUI Library host exists without a second browser authority, UI/Gallery
territory is occupied, or 1k bounded rendering cannot preserve typed
selection/scroll evidence. Report the exact owner/API/architecture blocker and
smallest required decision; do not fall back to a web page, Portal, permanent
Sidebar region, or copied Gallery UI.

## Delivery ledger

```text
Outcome:
  The dedicated production Saved Pages Library is implemented and proven over
  the real app: search, deterministic sort, rename, remove, multi-select,
  one-level flat collections and moves, 1k bounded rendering, and the DESIGN
  §11.16 decided states with stable ids/semantics — all through one app-owned
  Library composition and the one AppStore/B2 authority. No commit,
  integration, or MVP claim is made by this packet; this is lane evidence only.
Commits:
  None — sole bounded writer branch parallel/saved-pages-b3-library; changes
  stay uncommitted for the orchestrator's serial integration.
Functional evidence:
  Unit/contract: bbr-ui Library row/model suites, app-owned Library model
  tests (visible-window bounding, display-title ladder, mid-ellipsis +
  monogram, deterministic sort keys, decided states, staged rename,
  hidden-view bookkeeping, collection action-machine names) — all green.
Real-app E2E:
  just e2e-shell-saved-pages-library 12/12 on the exact semantic-union runtime
  tree b4ab98113c6582caad2150764815d3885589dfa7 (entry/Escape, symmetric
  Cmd-T/Cmd-L replacement, search/sort,
  rename draft, select+destructive remove, flat collections, pointer
  selection-toolbar clear/remove/move, pointer collection
  create/rename/remove surface, production scroll-wheel path, restart
  persistence, exactly-one-fresh-tab activation, 1k bounded window) — log
  /tmp/bbr-b3-review-remediation-e2e-shell-saved-pages-library-20260810T144111Z.log,
  SHA-256 06838f3556a34bb46d7dfe20f8b5f6f3bdd3b6d7ce91a19a59b633dfffa11673;
  just e2e-shell-saved-pages 4/4 re-run on the same tree — log
  /tmp/bbr-b3-review-remediation-e2e-shell-saved-pages-20260810T144301Z.log,
  SHA-256 440317455fbf499fada4515d404c1f460d0a5854a817490314e5ac65763273cc;
  just e2e-shell-command 25/25 — log
  /tmp/bbr-b3-review-remediation-e2e-shell-command-20260810T144111Z.log,
  SHA-256 953138116c7ad011402cba6a8269c75c9fed690cb4545df52d940a132bcc7b6f.
Performance evidence:
  Scenario drivers/guards/budgets for saved_pages_library_search and
  saved_pages_library_scroll exist but have zero executions: no optimized
  sample accepted, boxes unchecked and unpassed. Patrick deferred
  current-MVP performance execution; the canonical just e2e / just e2e-perf
  gates were not run for this slice.
Gallery evidence:
  just e2e-gallery final 22/22 on the exact semantic-union runtime tree — log
  /tmp/bbr-b3-review-remediation-e2e-gallery-20260810T144301Z.log, SHA-256
  bc41dd1df11d887dea03a10fa30949a33be05edb916462c59fd9cbd25a2ff7e9;
  Library stories registered at ids 23-30 through the recorder/observer seam;
  the integrated G2-TAB Tab Row occupies id 22 and the temporary B3-lane
  reservation machinery is absent from the semantic union.
Roadmap/documentation updates:
  This packet's evidence checkboxes (unit/E2E) and Gallery roll-up with the
  actual final logs/tree (evidence remediation lane); the ARCHITECTURE.md
  §9d follow-up and E2E_ARCHITECTURE.md §7m suite facts corrected to the
  final tree/log evidence; DESIGN.md §11.16 production status from the
  earlier lane stage. ROADMAP.md, phase, and track roll-ups are the
  orchestrator's serial-integration product and were not touched by this
  remediation lane.
Open blockers and non-goals:
  Performance execution remains unpassed (deferred). No commit/integration
  claim. Non-goals unchanged: no Sidebar region, no nested folders/tags/
  sync/profile-scoped records, no import/export, no dedup/switch behavior.
Worktree state:
  /Users/patwoz/dev/Personal/macos/bbr-saved-pages-b3-library; exact source
  tree the final semantic-union evidence runs happened on:
  b4ab98113c6582caad2150764815d3885589dfa7 (temporary-index full-worktree
  snapshot, verified); the final worktree tree
  including these doc corrections is recorded in the orchestrator's external
  fingerprint — any tree value embedded in this self-referential delivery
  ledger is intentionally non-binding documentary prose. Source remediation
  (no behavior change): split app/library/model.rs (568→181 lines),
  shell/semantics/library.rs (521→384), and bbr-ui saved_pages_library/model.rs
  (513→308) below the ~500-line budget by extracting the visible-id mirror,
  the row-level semantic nodes, and the test blocks into sibling modules.
  No commit or reset.

Planning/read-only:
Implementation:
  Evidence remediation lane (authority 0c25cbb7): the three file splits
  above, cargo fmt, and the doc corrections in this packet plus
  ARCHITECTURE.md §9d and E2E_ARCHITECTURE.md §7m.
Test execution:
  Fresh isolated-target Xcode26.2 just verify (fmt/check/test/clippy/
  roadmap/ui-deps/script-tests) passed; cargo clippy -p bbr --features
  testkit --all-targets -- -D warnings passed; the semantic-union runtime tree
  passed Library 12/12, Saved Pages 4/4, Command 25/25, and Gallery 22/22 under
  the serial GUI lock. Final `just verify` log:
  `/tmp/bbr-b3-review-remediation-verify-20260810T143909Z.log`, SHA-256
  `8de8c1fe61ce1a5eb1088bdabd88decddb0557f5212679c6df9ff35b433339e2`.
Review:
  None in this remediation lane (bounded evidence writer).
Remediation:
  This lane is the B3 evidence remediation (source splits + doc evidence).
Waiting/retries:
  None.
Total elapsed:
Human intervention:
  None.
```
