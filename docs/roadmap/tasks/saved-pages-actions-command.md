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
  snapshot and does not make `bbr-command` depend on persistence. Saved Page
  results form one deterministic group after standalone Spaces and before
  History: `ExactTab < CurrentPinned < CurrentTab < OtherSpace < Space <
  SavedPage < History < Action < Fallback`. No existing group's relative order
  changes.
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
                      B2 and G1a
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

- [x] Availability matrix covers saved/unsaved, navigable/invalid/missing URL,
      active tab, Favorite-backed tab, focused split pane, and stale/deleted
      `BookmarkId` without mutating source state. *(The `SavedPageCommand`
      authority unit tests + the `canonical_seam` suite cover the missing/
      malformed/dangerous/unsupported URL set; the real-app availability
      probe covers saved/unsaved flips; opening a stale/deleted `BookmarkId`
      rejects truthfully. No availability call mutates source state.)*
- [x] Cmd-D and menu save call the same command; repeated save returns the same
      identity, remove is idempotent, and neither changes tab/Favorite/pin/
      session/profile/space state. *(Cmd-D, the Palette result, and the Z1/Z3
      menu consumers all route through the one `SavedPageCommand` authority;
      `cmd_d_toggles_saved_state_without_duplicates` proves idempotent
      save/remove cycles over the real app.)*
- [x] Command corpus/ranking has a distinct stable Saved Page result key,
      deterministic order/search, collection label projection, and no
      persistence dependency in `bbr-command`; the `SavedPage` group is exactly
      after `Space` and before `History`. *(`saved_page:<id>` key;
      `ResultGroup::SavedPage` ranks between `Space` and `History`; the
      provider-neutral corpus carries only the loaded in-memory snapshot;
      `bbr-command` never imports persistence.)*
- [x] Activation revalidates the record then dispatches exactly one fresh
      regular NewTab into the active Space/Profile; an existing matching tab is
      not selected or reused. *(`palette_activation_opens_exactly_one_fresh_tab`
      proves exactly one new regular tab + one live session in the active
      Space/Profile with the fresh view never the source.)*

### Real-app E2E

- [x] `Cmd-D` saves, repeated `Cmd-D` removes, and repeated save cycles never
      create duplicates; typed saved state and repository effects agree.
      *(`just e2e-shell-saved-pages` 4/4, green after the bounded writer
      flush.)*
- [x] Save a page, close its tab, restart the real app, search the Palette, and
      find the Saved Page while no original `TabId`/session exists. *(The
      restart restores the persisted browser state; the record survives with
      the restored tab closed, so the search proves the durable global record
      with zero live tabs.)*
- [x] Palette activation creates exactly one new regular tab in the currently
      active Space/Profile, preserves profile isolation, and does not select a
      matching open tab in another context. *(Real-app scenario over native
      WKWebViews, source tab navigated away first so the CurrentTab row never
      beats the Saved Page row.)*
- [x] Production quit durability: a Cmd-D save is re-persisted by the real
      owning-window-close quit flush — never the testkit flush — even when
      the store was emptied out-of-band (`Repository::replace_saved_pages`
      `&[]` with the persisted zero observed), the production `CloseWindow`
      exits the complete process successfully, and a same-profile restart
      restores exactly one record with remove offered. *(Real-app scenario:
      the persisted background-writer state is observed out-of-band under an
      explicit deadline; no testkit flush is used; the profile root is
      preserved across the exit.)*
- [ ] Tab/Favorite and page-background menus, once their production packets are
      present, invoke the same B2 command and preserve their source/focus/Portal
      contracts. Their detailed menu assertions remain owned by Z1/Z3.
      *(Z1/Z3 are not integrated yet; B2 provides the shared dispatch and the
      typed save/remove seams they consume.)*

### Performance

- [x] Seed/enforce `saved_page_search_first_result` over at least 1,000 loaded
      records and `saved_page_open`, each with at least 30 optimized accepted
      samples, production-path semantic commit, and raw artifacts.
      `saved_page_search_first_result` uses the existing warm command-type
      class (p95 `≤ 50 ms`, max `≤ 100 ms`, same as `command_type`);
      `saved_page_open` uses the existing session-recreation class (p95
      `≤ 100 ms`, max `≤ 150 ms`, same as `command_activate_new_tab` and
      `tab_reopen`). No new budget class is introduced. *(Green `just e2e-perf`
      with `regressions: []`: search 30 samples p95 41.0 ms / max 41.43 ms
      against 50/100; open 30 samples p95 27.13 ms / max 40.93 ms against
      100/150, both over the seeded ≥1,000-record provider, raw artifacts in
      `target/bbr-perf-artifacts/short`. These are preserved historical
      report-only measurements, not current integration acceptance; Patrick
      deferred all current-MVP performance execution, so performance remains
      unpassed debt.)*
- [x] Fixture/repository setup is outside app timing; fast-but-wrong guards
      verify exact result identity and exactly-one-tab activation. *(The
      provider is seeded pre-spawn; the in-app probe rejects a stale
      generation or a wrong first row; the driver asserts the exact
      `saved_page:1` identity, the ≥1,000 corpus, and exactly one new
      tab/session.)*

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
  B2 Saved Page actions, Cmd-D, the Palette provider/result kind, and the one
  app-owned command authority are implemented and proven over the real app.
Commits:
  None — sole bounded writer branch `parallel/saved-pages-b2-command`;
  changes stay uncommitted for the orchestrator's serial integration.
Functional evidence:
  just verify (fmt/check/test/clippy/roadmap/ui-deps/script-tests) green;
  bbr-command + bbr-persistence unit/contract suites green; bbr testkit bin
  246 passed; both -D-warnings Clippy gates green.
Real-app E2E:
  just e2e-shell-saved-pages 4/4 (Cmd-D toggle, restart survival with the
  source tab closed, exactly-one-fresh-tab activation, production quit
  durability: out-of-band store clear + owning-window close re-persists the
  provider and a same-profile restart restores exactly one record); just
  e2e-shell-command 25/25.
Performance evidence:
  Preserved historical report-only run: just e2e-perf had regressions: [] —
  saved_page_search_first_result 30 samples p95 41.0 / max 41.43 ms (50/100),
  saved_page_open 30 samples p95 27.13 / max 40.93 ms (100/150), both over the
  seeded ≥1,000-record provider; raw artifacts in
  target/bbr-perf-artifacts/short. Patrick deferred current-MVP performance
  execution; this historical run is not integration acceptance and current
  performance remains unpassed debt.
Roadmap/documentation updates:
  Task-packet evidence checkboxes updated; Phase 5/8 roll-ups updated;
  ARCHITECTURE.md §9b follow-up and E2E_ARCHITECTURE.md suite facts updated;
  ROADMAP.md current-slice pointer retained.
Open blockers and non-goals:
  Z1/Z3 menu rows land with their own packets (B2 provides the shared seams);
  no B3 Library, no sync, no profile-scoped records, no dedup/switch behavior.
Worktree state:
  /Users/patwoz/dev/Personal/macos/bbr-saved-pages-b2-command, branch
  parallel/saved-pages-b2-command; base HEAD c41b06c3… / tree 1df4b846…;
  reviewed input candidate tree 989185c1… (B2 read-only review snapshot);
  the exact reviewed evidence candidate was tree 7805cc9c…; any tree value
  embedded in this self-referential delivery ledger is intentionally
  non-binding documentary prose, while the orchestrator-bound external
  fingerprint is authoritative; 49 paths preserved forward.

B2 remediation (sole bounded writer, this pass):
  Implemented production Saved Page quit durability parity (BbrApp/AppStore
  flush_saved_pages no longer testkit-gated; quit_when_browser_window_closes
  flushes browser state then saved pages before cx.quit in one app.update;
  dead_code allow removed; window/main/store docs truthful; no command-history
  flush parity added); added the fourth real-app scenario
  saved_page_survives_production_quit_flush_after_out_of_band_clear (same-profile
  Cmd-D save, out-of-band load_saved_pages==1 wait, out-of-band
  replace_saved_pages(&[]) with persisted-zero wait, production CloseWindow
  without any testkit flush, successful full process exit, preserved profile
  root, same-profile restart restores one record + remove availability);
  aligned ARCHITECTURE.md §9c, E2E_ARCHITECTURE.md §7l, and this packet's
  evidence/4-scenario/ledger wording.
Non-GUI evidence (CARGO_BUILD_JOBS=4, repository Xcode): just fmt green;
  bbr-command + bbr-persistence suites green; bbr testkit bin 246 passed;
  testkit + non-testkit all-target clippy -D warnings green; just roadmap
  valid (89 files); just verify green (66 ok test blocks, scripts OK);
  e2e_shell_saved_pages compiled --no-run, never executed.
Current integration evidence: just e2e-shell-saved-pages 4/4,
  just e2e-shutdown 3/3, and just e2e-shell-command 25/25 passed on the exact
  candidate under the repository GUI lock; just verify, explicit testkit
  Clippy, roadmap, and diff-check passed. The canonical just e2e recipe was not
  run because it includes e2e-perf. Patrick deferred all current-MVP
  performance execution and host-idle waiting; performance tests/contracts stay
  unchanged and current performance remains unpassed debt.

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```
