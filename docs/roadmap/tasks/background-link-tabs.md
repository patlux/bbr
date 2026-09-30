# Task packet — Background link tabs

Parents: [Phase 8](../phases/08-daily-driver.md) · [Browser orchestration track](../tracks/browser-orchestration.md) · [browser interaction contract](../../product/browser-interactions.md) · [daily-driver delivery](../../delivery/2026-08-12-daily-driver-delivery.md).

## Outcome

A primary click with the Command modifier and a middle click on a webpage link open one regular child tab in the background directly after the exact opener. Repeated background opens from the same opener remain contiguous in click order while the opener stays selected, visible, and web-focused.

## Scope

- Extend the one existing WebKit new-window/link request seam with explicit input disposition for Command-primary-click and middle-click.
- Add portable grouped-child insertion policy keyed by exact opener `TabId`, never by URL, title, render index, or `WebViewId`.
- Preserve the integrated session-local opener provenance consumed by Back-at-root return.
- Route the background child through the source Space/Profile and the normal native-session creation authority without selecting it or dissolving unrelated state.
- Add deterministic fixtures, typed testkit events/snapshots, real-WKWebView E2E, performance code, and truthful roadmap/architecture updates.

## Non-goals

- No Shift-click decision, external-window behavior, named-window reuse, popup allow-list, Peek-policy rewrite, cross-window opener graph, or persisted opener graph.
- No change to plain primary-click navigation, Cmd-T append semantics, ordinary Cmd-W neighbor selection, or explicit Back-at-root behavior.
- No tab-tree UI or visual opener hierarchy.

## Parallel-lane contract

```text
Base condition:          committed Daily-driver delivery authority and clean common activation base
Depends on:              integrated productive new-window seam, Peek disposition, opener provenance, and tab controller
Provides/API:            explicit background-link disposition and grouped opener-relative insertion
Exclusive paths:         focused bbr-web new-window input metadata; focused bbr-web-macos delegate/new-window bridge; focused bbr-browser insertion/provenance modules; focused E2E/fixtures
Shared integration points: bbr-web public request enum, BrowserController facade/snapshot, bbr-app event pump, testkit envelope, performance registry, justfile, roadmap
Integration order:       first product merge in the Daily-driver wave
```

## Constraints

- The opener remains selected and web-focused; no temporary selection or focus bounce is acceptable.
- First child inserts immediately after the opener. Later background children from that opener insert after the last surviving member of its contiguous child group, preserving click order.
- Reorder or move may change positions afterward; provenance remains identity-based. If group members are manually separated, the next insertion must follow a deterministic tested policy rather than infer hidden adjacency.
- Source Space/Profile validation fails closed. Cross-profile session creation is never inferred from the destination URL.
- Public WebKit/AppKit APIs only. Native input metadata stays outside portable crates.
- No weakened tests, deadlines, budgets, sample guards, or profile-isolation assertions; files below roughly 500 lines.

## Evidence

### Unit and contract

- [x] Insertion matrix covers no prior child, several children, closed/reordered/moved child, pinned/regular boundaries, inactive opener Space, stale opener, and invalid source Profile.
- [x] Cmd-primary and middle dispositions create exactly one unselected regular child and record exact opener provenance.
- [x] Plain click and script-created popup behavior remain unchanged.
- [x] Fake-engine/controller tests prove no selection, focus-domain, split, or unrelated-session mutation.

### Real-app E2E

- [x] Real link fixture: Cmd-click opens X after opener B while B remains visible/focused and X loads in the background.
- [x] Repeated X/Y/Z opens remain ordered `[B, X, Y, Z]`; unrelated C remains after the child group.
- [x] Middle-click is behaviorally identical.
- [x] Correction 2026-09-30 ([reality gaps](reality-gaps.md), item D): the
      items above injected the classified request; no real click reached
      the classifier. With real AppKit clicks only a Command-click on a link
      that asks for a new window worked. An ordinary link navigated in place
      on ⌘-click and middle-click, and `classify_input` read NSEvent button
      numbers (middle `== 2`) while `WKNavigationAction.buttonNumber` is
      WebKit's button mask (left 1, right 2, middle 4), so a real middle
      click never classified and a right-button navigation would have. The
      navigation-action policy now sends a background link activation to
      the same coordinator, and the classifier tests the mask;
      `e2e_real_input::web::command_and_middle_clicks_on_a_link_open_background_tabs`
      proves both with real clicks.
- [x] Child activation later reuses its already-created session; Back-at-root returns to the exact opener.
- [x] Same-profile/profile-isolation, stale source, Peek, new-window, gestures, and Tabs regressions remain green.
- [x] Portal is outside this lane's scope: the lane does not touch the Portal suite, and the locked host proves only 6/7 for `e2e-shell-portal` under the already committed environmental console blocker, so no Portal-green claim is made.

### Performance

- [x] Production-path `background_link_open` scenario is implemented with semantic guards for opener selection/focus, child identity/index/session, and no fast-but-wrong foreground activation.
- [ ] Execution remains unexecuted/unpassed while Patrick's performance override is active; budgets and sample guards are not changed.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-browser -p bbr-web -p bbr-web-macos
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets -- -D warnings
just e2e-new-window
just e2e-web-navigation-gestures
just e2e-tabs
just e2e-profiles
just roadmap
git diff --check
```

Performance and canonical aggregate E2E are intentionally not executed under the active override.

## Delivery sequence

1. Add explicit native-to-portable modifier/button disposition without changing existing plain/script/Peek routing.
2. Add the identity-bound grouped insertion transaction and contract tests.
3. Prove one real Cmd-click path, then repeated grouping and middle-click.
4. Add performance code and affected regressions.
5. Independent read-only review; one bounded remediation writer only for concrete findings; focused recheck; Orchestrator integration.

## Iteration policy

Start with one opener and one Command-click child. Classify failures as native input metadata, disposition routing, grouped insertion, provenance, focus/selection, session lifecycle, or evidence. Add repeated/move/stale matrices only after the smallest real path works.

## Blocked stop condition

If public WebKit delegate metadata cannot distinguish Command-primary or middle activation without replacing native link handling globally, stop with exact API/runtime evidence and the smallest native seam decision required. Do not use private SPI, global event taps, DOM injection presented as general browser input, or foreground-open-then-hide workarounds.

## Delivery ledger

```text
Outcome:
  Cmd-primary and middle-click web links open one grouped regular child tab in
  the background immediately after the exact opener; the opener stays
  selected, visible, and web-focused. Native modifier/button metadata flows
  through the one new-window seam as an explicit portable input disposition;
  the identity-bound grouped insertion policy is keyed by exact opener TabId;
  provenance, profile isolation, plain/script/Peek routing, and Back-at-root
  are preserved.
Commits:
  none (lane worktree; no commit/integration per DD-LINK lane contract)
Functional evidence:
  - bbr-web portable policy: Background input → OpenBackgroundTab for direct
    activation, Block for script claims; explicit intents unchanged; decide()
    preserves source profile; 348 bbr-web tests green (322 unit + 25
    integration + 1 doc-test), including the background-disposition matrix.
  - bbr-web-macos: classify_input(Command bit / buttonNumber==2) → Background;
    context→request carries the disposition; 124 tests green (108 unit +
    16 integration), including the classification/context matrix.
  - bbr-domain: Browser::insert_tab (positional, no selection/split mutation,
    pinned-boundary rejection); 121 domain tests green (4 new insert_tab
    tests included).
  - bbr-browser: open_background_tab_from_new_window + grouped insertion
    matrix (no child, several children, closed/reordered/moved child,
    pinned boundary, inactive Space, stale opener, profile mismatch, split
    preservation, session reuse, script-block); 230 tests green.
Real-app E2E:
  e2e_shell_background_links (just e2e-background-links): real links.html
  opener + bg-x/y/z fixtures in real WKWebViews; opener stays selected/
  visible/web-focused while the child loads in the background; [B, X, Y, Z]
  grouping with unrelated C after; session reuse + Back-at-root return;
  stale source / profile mismatch reject without mutation.
Performance evidence:
  background_link_open implemented (BenchmarkKind + typed request + in-app
  probe with opener-selection/focus/child-index/session fast-but-wrong guards
  + perf_common driver + baseline 100/150 fresh-session class + seed test).
  Execution unexecuted/unpassed under Patrick's active performance override;
  budgets and sample guards unchanged.
Roadmap/documentation updates:
  packet checkboxes above; browser-orchestration track (completed packet +
  durable checklist item + evidence count); ARCHITECTURE.md §6/§9a + domain
  insert_tab; E2E_ARCHITECTURE.md background-links suite section; justfile
  recipe + canonical e2e chain entry.
Open blockers and non-goals:
  Shift-modified link behavior (DD-SHIFT-LINK, separately planned); no
  external-window/named-window/popup-allow-list/Peek-policy rewrite; no
  cross-window opener graph; no persisted opener graph; no tab-tree UI.
Worktree state:
  /Users/patwoz/dev/Personal/macos/bbr-dd-background-links @ 4d5be311…
  (parallel/dd-background-links), uncommitted lane candidate.

Planning/read-only:
  forward-recovery writer dd-link-finalizer: authority + packet read, full diff
  review, fingerprint verification, gate audit (no new planning).
Implementation:
  none (candidate inherited; only two documentation-truth fixes applied: ledger
  test counts → verified numbers, protocol version claim 6 → 7).
Test execution:
  cargo fmt --all -- --check (0); cargo test -p bbr-domain/-p bbr-web/-p
  bbr-browser/-p bbr-web-macos (121/348/230/124); cargo test -p bbr --features
  testkit --bin bbr (261); clippy -p bbr --features testkit --all-targets -D
  warnings (fresh dir, clean); clippy --workspace --all-targets -D warnings
  (fresh dir, clean); e2e-background-links 4/4; e2e-shell-new-window 4/4;
  e2e-peek 6/6; e2e-web-navigation-gestures 9/9; e2e-tabs 1/1; e2e-profiles
  1/1; just roadmap valid; git diff --check clean. Performance code compiled
  but unexecuted (Patrick's override).
Review:
  none (first report truncated before any review; no independent review ran).
Remediation:
  two doc-truth fixes only (ledger counts, protocol version); no code defect
  found across the complete native→portable→controller→app/testkit/fixtures/
  E2E/perf product.
Waiting/retries:
  none.
Total elapsed:
  forward-recovery session only (fingerprint verify, gate runs, doc fixes):
  ≈ 30 min wall (measurements above are per-gate). First-writer timing not
  recoverable from truncated transport.
Human intervention:
  none required.
```
