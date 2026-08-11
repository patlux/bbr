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

- [ ] Insertion matrix covers no prior child, several children, closed/reordered/moved child, pinned/regular boundaries, inactive opener Space, stale opener, and invalid source Profile.
- [ ] Cmd-primary and middle dispositions create exactly one unselected regular child and record exact opener provenance.
- [ ] Plain click and script-created popup behavior remain unchanged.
- [ ] Fake-engine/controller tests prove no selection, focus-domain, split, or unrelated-session mutation.

### Real-app E2E

- [ ] Real link fixture: Cmd-click opens X after opener B while B remains visible/focused and X loads in the background.
- [ ] Repeated X/Y/Z opens remain ordered `[B, X, Y, Z]`; unrelated C remains after the child group.
- [ ] Middle-click is behaviorally identical.
- [ ] Child activation later reuses its already-created session; Back-at-root returns to the exact opener.
- [ ] Same-profile/profile-isolation, stale source, Peek, new-window, gestures, Tabs, and Portal regressions remain green.

### Performance

- [ ] Production-path `background_link_open` scenario is implemented with semantic guards for opener selection/focus, child identity/index/session, and no fast-but-wrong foreground activation.
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
