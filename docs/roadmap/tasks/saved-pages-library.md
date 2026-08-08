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

Status: **queued after integrated B1+B2 and settled G1.** This packet is the
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
  `bbr-ui` primitives after G1. It is not a web page, native Portal, WebKit
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
Base commit:          verified main after B1+B2 and settled G1
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
Integration order:    B1 → B2; G1 → B3 production cutover → append-only G2/G3
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

- [ ] Model/row tests cover empty/populated/no-result, selected/multi-selected,
      long/missing title, mid-ellipsized URL, collection label, missing favicon,
      and destructive states with stable ids and semantic names.
- [ ] Search and all deterministic sort orders cover 0, 1, 1,000, equal-key,
      unicode, and collection-filter cases without repository access.
- [ ] Callback recorder tests prove rename/remove/open/collection operations
      dispatch exactly once against the intended `BookmarkId` set.
- [ ] Bounded-render tests prove the Library does not lay out all 1,000 rows and
      keeps selection/scroll anchors stable across mutation.

### Real-app E2E

- [ ] Open the Library through its integrated production entry point; keyboard
      and pointer search, sort, selection, rename, remove, and flat collection
      operations expose typed semantics and persist across restart.
- [ ] Removing one or many Saved Pages leaves every open tab/session untouched;
      closing tabs leaves Library records untouched.
- [ ] Opening a Library row creates exactly one fresh regular tab in the
      current active Space/Profile through B2 and never selects a matching tab.
- [ ] The 1k fixture proves bounded rows, deterministic scroll/selection, long
      data/fallback states, and no second Saved Page authority.

### Performance

- [ ] Seed/enforce `saved_pages_library_search` and
      `saved_pages_library_scroll` over 1,000 loaded records, at least 30
      optimized accepted samples each, production semantic presentation
      commits, raw artifacts, and predeclared budgets.
- [ ] Repository/fixture setup and favicon network latency are outside app
      timing. Fast-but-wrong guards verify exact result window/count and scroll
      anchor.

### Gallery roll-up

After B3 production integration, append the exact production row/control and
Library composition stories to [G2](component-gallery-browser-components.md)
and [G3](component-gallery-compositions.md). Those Gallery packets remain the
canonical story/checklist owners; they must not claim persistence, browser
activation, restart, or product-performance evidence.

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

1. Confirm B1+B2 and G1 are integrated and broad UI/Gallery ownership is free.
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

Stop with preserved state if G1/B1/B2 is not integrated, no defensible ordinary
GPUI Library host exists without a second browser authority, UI/Gallery
territory is occupied, or 1k bounded rendering cannot preserve typed
selection/scroll evidence. Report the exact owner/API/architecture blocker and
smallest required decision; do not fall back to a web page, Portal, permanent
Sidebar region, or copied Gallery UI.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:
Gallery evidence:
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
