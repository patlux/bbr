# Task packet — Web-content navigation gestures and opener return

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[Browser orchestration track](../tracks/browser-orchestration.md) ·
[Native integration track](../tracks/native-integration.md) ·
[Testing track](../tracks/testing.md) ·
[browser interaction contract](../../product/browser-interactions.md) ·
[daily-driver capability inventory](../../product/daily-driver-capability-inventory.md) ·
[performance contract](../../product/performance-contract.md).

Status: **planned for the first feature-complete MVP.** This packet is the
canonical owner for trackpad navigation over native web content and the
opener-return fallback of regular child tabs. It does not replace the existing
sidebar space swipe or decide Peek eligibility/background-tab policy.

## Outcome

A standard macOS horizontal trackpad gesture over a visible webpage navigates
that exact page backward or forward through its real WebKit history. When a
regular child tab was opened from another tab and has no older page to return
to, committing a Back gesture closes that child through the normal tab-close
lifecycle and presents the exact opener tab again. Ordinary tabs without a
valid opener never close merely because history is empty.

## Product contract

### Gesture ownership and routing

- A horizontal gesture beginning over native web content belongs to the exact
  `WebViewId`/pane under the pointer. In single-pane mode this is the selected
  tab; in split mode it is that pane and becomes the focused command target
  according to the existing split-focus contract.
- A horizontal gesture beginning over the sidebar remains the existing space
  swipe. Titlebar, sidebar, Portal, palette, find, menu, and other chrome
  surfaces keep their own hit-testing and never leak a gesture into WebKit.
- The macOS adapter normalizes platform direction and natural-scroll settings
  into logical `Back`/`Forward` intent. Portable crates never infer direction
  from raw device deltas.
- Gesture progress is direct manipulation but no domain/tab mutation occurs
  before the native commit threshold. Cancellation, vertical dominance,
  boundary rubber-banding, or an interrupted gesture commits nothing.

### Real page-history navigation

- Logical Back with `can_go_back == true` dispatches exactly one Back action to
  the bound view. It never selects, closes, moves, recreates, or duplicates a
  tab.
- Logical Forward with `can_go_forward == true` dispatches exactly one Forward
  action to the bound view. Forward never closes a tab or follows an opener.
- A disabled direction is a deterministic no-op unless the Back-only opener
  fallback below applies.
- The correct URL/title/history state must come back through normal WebEvents;
  the gesture layer owns no second navigation-state copy.

### Regular child-tab provenance

- When the canonical new-window/link-opening policy resolves a user-originated
  web request to a **regular tab**, the controller records a session-local
  relation from the child `TabId` to the exact opener `TabId` and originating
  browser window.
- The relation is identity-based, never index-, URL-, title-, `WebViewId`-, or
  render-position-based. It follows either tab when it is reordered or moved
  between spaces/profiles in the same browser window.
- Reload, same-document navigation, ordinary navigation, suspension, crash,
  and native-session replacement do not rewrite the relation. Closing either
  tab or ending the browser process invalidates it. V1 does not persist opener
  relations across restart.
- `Cmd-T`, Saved Page activation, reopen-closed-tab, external URL handling,
  duplicate/bulk operations, and tabs created without a concrete source tab do
  not invent opener provenance.
- This packet consumes the canonical new-window request/disposition seam. The
  [Link Preview / Peek](link-preview-peek.md) packet still owns whether an
  eligible request becomes Peek, a regular tab, external handling, or a
  rejection. Only a request already resolved to a regular tab enters this
  packet's child-tab path.

### Back-at-root opener return

A committed logical Back gesture closes and returns only when all conditions
hold:

1. the bound child is the active/focused gesture target;
2. its real session reports `can_go_back == false`;
3. its initial document has committed, or it is showing a settled
   navigation-failure/crash state (not an ambiguous provisional load);
4. it has a valid session-local opener relation; and
5. the opener `TabId` still exists in the same browser window.

The operation is one typed controller transaction:

- close the child through the existing close path, destroy its native session,
  and keep the normal recently-closed record;
- activate the exact opener by its current `TabId` owner, switching to its
  current Space/Profile if it moved;
- reuse a warm opener session, or recreate it only under the existing
  suspended/discarded lifecycle policy;
- present and focus the opener's web content; and
- emit one causal typed outcome that names child, opener, source/destination
  Space/Profile, closed session, presented session, and whether recreation was
  required.

This opener-aware selection is specific to the atomic gesture fallback; it does
not silently change the normal Cmd-W/close-button neighbor-selection contract.
If the opener is missing/stale, history is unavailable, or the tab has no
provenance, Back is a no-op and **must not close an arbitrary tab**. A child
that first navigates within its own history uses ordinary Back until it reaches
its root; only a later committed Back may return to the opener.

### Split, profile, focus, and lifecycle rules

- In split mode, history navigation targets only the pane under the gesture.
  Returning from a child pane closes that child tab, dissolves the split through
  the existing close contract, and presents the opener in single-pane mode.
- Returning to an opener that moved across a profile boundary activates its
  existing isolated profile/session; no cookies, credentials, or page state
  move between profiles.
- Successful history navigation leaves focus in the same native web view.
  Successful opener return gives native web focus to the opener. Cancel/no-op
  preserves the prior focus domain.
- Portal/native overlays and modal WebKit flows win hit-testing while visible;
  no gesture may dismiss them and navigate/close a tab in one input.

## Scope

- Define the minimal portable logical gesture intent/outcome and opener
  provenance model.
- Extend the controller with one ID-bound atomic child-close/opener-activate
  operation and invariants.
- Extend the macOS adapter using public AppKit/WebKit gesture facilities without
  disabling native scrolling, text selection, magnification, or page-owned
  interactions.
- Wire user-originated regular child-tab creation from the accepted
  new-window/link disposition seam.
- Extend typed testkit requests/events/snapshots only as required to drive real
  gesture phases and assert the bound pane, history availability, provenance,
  target identity, session lifecycle, and focus.
- Add deterministic local fixtures for multi-entry history, root child tabs,
  removed/moved openers, forward history, failures, and target-blank regular-tab
  disposition.
- Add optimized production-path performance scenarios and update implemented
  architecture facts only after evidence passes.

## Non-goals

- No decision here about Peek eligibility, popup allow/block policy,
  middle-click, Cmd-click, background opening, insertion position, or whether a
  background child becomes selected. Those remain canonical new-window/tab
  product decisions.
- No replacement for titlebar Back/Forward, Cmd-[ / Cmd-], page-background menu
  actions, or sidebar space swipe.
- No opener-tree UI, breadcrumbs, tab groups, multi-level automatic unwind, or
  restoring a closed opener.
- No cross-window opener activation and no persisted opener graph in v1.
- No screenshot/texture animation of webpages and no second navigation state in
  GPUI.
- No threshold, budget, baseline, sample-count, or assertion relaxation.

## Parallel-lane contract

```text
Base commit:          fresh verified integrated checkpoint chosen by the
                      delivery orchestrator
Depends on:           existing navigation/history authority; exact pane focus;
                      accepted new-window request/disposition seam for regular
                      child tabs; free bbr-web/controller/app/testkit contracts
Provides/API:         logical content-gesture intent/outcome; session-local
                      child→opener provenance; atomic close-child-and-return
Exclusive paths:      focused new gesture/provenance modules and focused
                      unit/E2E/performance files
Shared integration points: bbr-web action/event seam, BrowserController public
                      facade/snapshot, macOS WKWebView delegate/input wiring,
                      bbr-app composition/focus, testkit protocol, justfile,
                      canonical E2E/performance registries, roadmap
Integration order:    serial new-window/provenance contract if absent → this
                      product slice → Gallery stories only after production
```

This packet may run in an isolated sibling worktree only after the orchestrator
records the clean base, exact exclusive paths, shared conflicts, owner, and
merge order. No concurrent writer may own native WKWebView gesture routing,
new-window disposition, controller navigation facade, global testkit envelope,
or the same E2E/performance registry entries.

## Constraints

- One navigation authority: `BrowserController` plus real WebKit history state.
  Presentation and adapter layers must not cache an independent Back/Forward
  truth.
- One close authority: opener return composes the normal close/session teardown
  semantics atomically; it does not hand-delete a tab or native view.
- Portable crates import no AppKit/WebKit/GPUI. Native raw gesture metadata stays
  in `bbr-web-macos`; portable code receives plain logical data.
- Use public platform APIs. If public APIs cannot distinguish committed history
  navigation from the fallback without stealing WebKit gestures, stop rather
  than shipping private SPI or a global event tap.
- Preserve profile isolation, warm-session reuse, split invariants, recently
  closed behavior, native responder-chain shortcuts, text selection, page
  scrolling, magnification, and accessibility.
- Stable identity and causal typed events; no sleeps or pixel assertions.
- Files remain below roughly 500 lines; no compatibility layer or migration.

## Evidence

### Unit and contract

- [ ] Gesture decision matrix covers Back/Forward availability, provisional
      loading, committed root, failure/crash, cancel, vertical dominance, and
      repeated terminal events with exactly-once outcomes.
- [ ] Opener relation tests cover creation, reorder/move, source navigation,
      suspension/recreation, opener close, child close, process-reset policy,
      and rejection of non-child tabs.
- [ ] Atomic return tests prove normal child teardown/recently-closed recording,
      exact opener selection, same-profile warm reuse, cross-profile activation,
      suspended opener recreation, split dissolution, and no partial mutation
      on errors.
- [ ] Fake-engine logs prove exactly one Back/Forward dispatch or exactly one
      close+activate transaction, never both.

### Native adapter

- [ ] Public macOS gesture mapping emits logical direction/phase for only the
      bound WKWebView and coexists with scrolling, text selection, zoom, and
      responder-chain browser shortcuts.
- [ ] Sidebar/chrome/Portal hit-testing prevents cross-surface gesture leakage.
- [ ] New-window metadata reaches the portable disposition/provenance path
      without exposing stronger JavaScript opener authority.

### Real-app E2E

- [ ] `just e2e-web-navigation-gestures` drives real trackpad gesture phases
      against real WKWebViews and deterministic history fixtures.
- [ ] Back and Forward navigate the exact pane, update URL/title/availability,
      preserve TabId/WebViewId/session count, and never mutate unrelated tabs.
- [ ] Back at a regular child root closes only that child and presents the exact
      opener with correct Space/Profile/session/focus and one causal outcome.
- [ ] A child with older history navigates first; another Back at root returns.
- [ ] No opener, stale opener, provisional load, cancellation, vertical gesture,
      and disabled Forward are no-ops with unchanged tabs/sessions/focus.
- [ ] Split, moved opener/child, profile isolation, crash/failure, Portal-open,
      sidebar-swipe, Cmd-[, titlebar Back, and Cmd-W regressions remain distinct
      and green.

### Performance

- [ ] Add `web_history_swipe_back` with at least 30 accepted optimized samples
      on a deterministic already-loaded history entry; accept only after the
      correct URL/history state is presented. Use the existing warm interaction
      p95 ≤ 50 ms / max ≤ 100 ms ceiling.
- [ ] Add `child_tab_swipe_return` with at least 30 accepted optimized samples;
      accept only after the child session is gone and the exact warm opener is
      visible/focused. Use the active-tab-close warm ceiling p95 ≤ 50 ms /
      max ≤ 100 ms.
- [ ] Keep external page/network latency outside BBR timings and retain raw
      samples plus typed fast-but-wrong guards.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-browser
CARGO_BUILD_JOBS=4 cargo test -p bbr-web
CARGO_BUILD_JOBS=4 cargo test -p bbr-web-macos
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-smoke
nix develop -c just e2e-tabs
nix develop -c just e2e-profiles
nix develop -c just e2e-shell-spaces
nix develop -c just e2e-shell-split-geometry
nix develop -c just e2e-web-navigation-gestures
nix develop -c just e2e-perf
nix develop -c just e2e
nix develop -c just roadmap
```

GUI/performance suites run sequentially under the repository GUI lock.

## Delivery sequence

1. Re-read the complete delivery authority and prove new-window, native gesture,
   controller, app, testkit, and GUI-lock ownership is free.
2. If absent, land the smallest serial typed new-window/provenance contract
   before parallel implementation; do not duplicate the Link Preview policy.
3. Implement portable provenance and the atomic Back-at-root transaction with
   fake-engine regressions.
4. Implement one real native content-gesture path and history navigation before
   adding the opener fallback.
5. Add real-app E2E, then optimized performance evidence and affected regression
   suites.
6. Independent read-only review; one bounded remediation writer only for
   concrete findings; focused recheck; orchestrator integration and canonical
   sequential gate.

## Iteration policy

Start with a single-pane child and warm opener. Classify every failure as native
phase/direction mapping, hit-test ownership, stale history state, provenance,
atomic controller mutation, session lifecycle, split/profile routing, focus,
or presentation evidence before editing. Add split/move/failure matrices only
on top of a working end-to-end production path. Re-audit the full delivery queue
at each settlement so independent Phase-8 lanes continue in parallel.

## Blocked stop condition

If public WebKit/AppKit APIs cannot supply a deterministic content-gesture
boundary without disabling native WebKit behavior, or if new-window disposition
has no accepted single authority, stop with:

- exact API/source and runtime evidence;
- attempted public approaches and observed conflicts;
- preserved worktree/branch/fingerprints and current owner;
- affected contract/E2E/performance claims still open; and
- the smallest serial platform or product decision required next.

Do not fall back to private SPI, a global event tap, synthetic direct controller
commands presented as gesture E2E, arbitrary tab closing, or weaker assertions.

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
