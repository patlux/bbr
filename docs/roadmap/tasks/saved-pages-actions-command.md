# Task packet — B2 Saved Page actions, Cmd-D, Palette provider, and menu command

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[Phase 5 command bar](../phases/05-command-bar.md) ·
[B1 contract/persistence](saved-pages-portable-persistence.md) ·
[command app composition](command-composition.md) ·
[Tab/Favorite menus](tab-favorite-context-menus.md) ·
[Page-background menu](page-background-context-menu.md) ·
[interaction contract](../../product/browser-interactions.md) ·
[DESIGN.md](../../../DESIGN.md) §11.13, §11.16 ·
Source decision: `/Users/patwoz/dev/Personal/macos/bbr-saved-pages-bookmarks-plan.md`.

Activation condition: B1 is integrated and command-provider ownership is free, as recorded in [Current delivery](../../delivery/current.md).
This packet is the canonical detail owner for Saved Page save/remove/open
semantics, Cmd-D, the in-memory Palette provider/result kind, and the one
app-owned menu command consumed by Z1/Z3. Z1 and Z3 remain canonical owners of
their menu placement, source binding, focus, Portal, and native-boundary
behavior; they must not duplicate this command logic.

## Outcome

`Cmd-D`, the Command Palette, and capability-gated Tab/Favorite and BBR
page-background menu rows use one app-owned Saved Page command authority.
Saving is canonical-URL-idempotent, removing never mutates a tab, closing the
source tab never removes the record, and opening a Palette result creates
exactly one fresh regular tab in the currently active Space/Profile through the
existing NewTab/navigation authority. No matching live tab is silently selected.

## Scope

- Compose B1's global Saved Page snapshot into the one AppStore/app bootstrap.
  Load once into an in-memory provider; rank/render never synchronously query
  SQLite. Mutations persist through the existing bounded nonblocking writer
  discipline without a second repository or lifecycle owner.
- Add one typed app-owned availability/dispatch authority for:
  `Bookmark Page`, `Remove Bookmark`, and `Open Saved Page`. Active-page and
  menu callers supply a current navigable URL/title or a `BookmarkId`; render
  callbacks never implement persistence or tab behavior directly.
- Bind conventional macOS `Cmd-D` to toggle the active/focused page's saved
  state. Repeated save creates no duplicate; remove is idempotent. Missing,
  malformed, dangerous, internal-unsupported, or non-navigable URLs reject or
  omit the action with a deterministic reason.
- Extend the provider-neutral command corpus with a distinct Saved Page result
  kind carrying `BookmarkId`, title/label, URL, and optional flat collection
  label. Deterministic ranking/search consumes only the loaded in-memory
  snapshot and does not make `bbr-command` depend on persistence.
- Render the DESIGN §11.16 result row: bookmark glyph, title or settled
  fallback, mid-ellipsized URL, optional collection label, stable result key,
  selected/focused states, and truthful empty/error behavior.
- Activating a Saved Page result routes through the existing NewTab/navigation
  authority and appends exactly one **regular** tab to the current active Space.
  That Space's current Profile supplies the native website-data store. The
  operation never silently activates an existing matching tab, switches Space
  or Profile, pins, favorites, splits, or reuses a session.
- Expose the same save/remove command and availability to Z1 Tab/Favorite menus
  and Z3 BBR page-background menu. Saving a tab/Favorite captures only its
  current URL/title; the resulting record has no `TabId` link. Menu packets own
  row placement/ids/source binding and prove their own dismissal/focus/Portal
  behavior; B2 owns semantic command tests and shared dispatch.
- Add typed saved-state/result snapshot evidence and deterministic testkit
  requests only where required. Add one focused real-app Saved Pages suite and
  optimized `saved_page_search_first_result` / `saved_page_open` scenarios.
- Update Phase 5/8 and architecture facts only after evidence passes. Do not
  implement the B3 Library.

## Non-goals

- No permanent Saved Pages sidebar region or full Library UI; B3 owns it after
  G1a.
- No address accessory requirement; the existing address control remains a
  display button and never becomes a second input.
- No open-tab dedup/switch behavior, background open policy, profile-scoped
  filtering, per-profile records, session/cookie capture, page snapshot,
  favicon blob, sync, import/export, tags, nested collections, or offline copy.
- No command/persistence authority inside Z1/Z3 menu components or `bbr-ui`.
- No Firefox/Chrome bookmark-sync behavior, browser extension rows, mobile
  send, route/QR, or bookmarklet.
- No migration/backcompat and no new dependency without evidence.

## Parallel-lane contract

```text
Base condition:       verified main with B1 integrated
Depends on:           B1 portable/repository contract; one AppStore; free
                      command-provider/perf/testkit territory; existing NewTab
                      and navigation authority
Provides/API:         app-owned SavedPageCommand availability/dispatch;
                      loaded in-memory provider; distinct command result kind;
                      Cmd-D toggle; fresh-tab activation seam consumed by Z1/Z3
Exclusive paths:      focused bbr-app saved-page/store/command modules;
                      additive bbr-command provider/result contracts where
                      required; focused testkit/E2E/perf modules; this packet
Shared integration points: app bootstrap/store snapshot, command corpus/rank
                      registries, keybindings/actions, testkit/perf registries,
                      Z1/Z3 menu models, Phase 5/8 and ROADMAP roll-ups
Integration order:    B1 → B2 command/provider core; Z1/Z3 consume the same
                      command when their menu surfaces integrate; B3 follows
                      B2 and G1aa
```

No concurrent writer may own command provider/ranking registries, Saved Page
app composition, or the shared menu command. Z1/Z3 coordinate their registry
edits serially rather than cloning behavior.

## Constraints

- Pin, Favorite, and Saved Page remain three different identities/lifetimes.
  Saved Page records never carry `TabId`, `SpaceId`, `ProfileId`, `WebViewId`,
  or session generation.
- Global records are searchable from every context. Open always uses the
  current active Space/Profile at activation time, not capture time.
- One activation creates exactly one regular tab and one corresponding native
  session through existing policy. Existing matching tabs are untouched.
- Rank/render use immutable in-memory data. No SQLite, filesystem, or async
  wait from GPUI render or `bbr-command` ranking.
- Persistence signals stay bounded/nonblocking and preserve the one AppStore/
  repository authority. No detached task may be accidentally dropped.
- Menu source identity remains Z1/Z3-owned and stable while open. B2 receives a
  prevalidated command input; it does not select the source merely to save it.
- Files stay below roughly 500 lines. No weakened budgets, samples, deadlines,
  validators, URL safety, profile isolation, or fast-but-wrong guards.

## Evidence

### Unit and contract

- [ ] Availability matrix covers saved/unsaved, navigable/invalid/missing URL,
      active tab, Favorite-backed tab, focused split pane, and stale/deleted
      `BookmarkId` without mutating source state.
- [ ] Cmd-D and menu save call the same command; repeated save returns the same
      identity, remove is idempotent, and neither changes tab/Favorite/pin/
      session/profile/space state.
- [ ] Command corpus/ranking has a distinct stable Saved Page result key,
      deterministic order/search, collection label projection, and no
      persistence dependency in `bbr-command`.
- [ ] Activation revalidates the record then dispatches exactly one fresh
      regular NewTab into the active Space/Profile; an existing matching tab is
      not selected or reused.

### Real-app E2E

- [ ] `Cmd-D` saves, repeated `Cmd-D` removes, and repeated save cycles never
      create duplicates; typed saved state and repository effects agree.
- [ ] Save a page, close its tab, restart the real app, search the Palette, and
      find the Saved Page while no original `TabId`/session exists.
- [ ] Palette activation creates exactly one new regular tab in the currently
      active Space/Profile, preserves profile isolation, and does not select a
      matching open tab in another context.
- [ ] Tab/Favorite and page-background menus, once their production packets are
      present, invoke the same B2 command and preserve their source/focus/Portal
      contracts. Their detailed menu assertions remain owned by Z1/Z3.

### Performance

- [ ] Seed/enforce `saved_page_search_first_result` over at least 1,000 loaded
      records and `saved_page_open`, each with at least 30 optimized accepted
      samples, production-path semantic commit, raw artifacts, and unchanged
      applicable command/session-recreation budgets.
- [ ] Fixture/repository setup is outside app timing; fast-but-wrong guards
      verify exact result identity and exactly-one-tab activation.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-command -p bbr-persistence
CARGO_BUILD_JOBS=4 cargo test -p bbr --features testkit --bin bbr
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-command --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-shell-saved-pages
nix develop -c just e2e-shell-command
nix develop -c just e2e-perf
nix develop -c just e2e
nix develop -c just roadmap
git diff --check
```

Add Z1/Z3 affected menu suites when those rows integrate. GUI gates run
sequentially under the repository lock.

## Delivery sequence

1. Confirm B1 is integrated and command/store/testkit/perf ownership is free.
2. Compose the loaded in-memory snapshot and shared save/remove/open command.
3. Add Cmd-D and prove idempotent save/remove without source mutation.
4. Add the distinct Palette provider/result and exactly-one-fresh-tab open.
5. Coordinate the shared command with Z1/Z3; those packets add only their
   presentation/source-bound rows and evidence.
6. Add real-app/performance evidence, then independent read-only review, one
   bounded remediation for concrete findings, and focused recheck.

## Iteration policy

Ship one vertical path first: Cmd-D save → restart → Palette find → fresh-tab
open. Then add remove and menu consumers. Diagnose failures as URL capability,
provider snapshot, ranking identity, AppStore write, NewTab activation, source
binding, profile isolation, or semantic presentation. Never query SQLite from
render or silently switch to an existing tab to make activation easier.

## Blocked stop condition

Stop with preserved state if B1 is not integrated, command/provider territory
is occupied, Z1/Z3 would require a second action authority, or the existing
NewTab path cannot guarantee exactly one regular tab in the active context.
Report the owner/conflict, evidence, attempted seam, and smallest settlement;
do not implement a fallback provider or menu-local command.

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
