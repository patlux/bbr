# Task packet — find-in-page Portal bar

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native portal host](native-portal-host.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Testing track](../tracks/testing.md).

Implementation state: integrated through lane `dd33e3d` and serial main merge
`c12ecbc`; Phase 8, native-integration, and testing roll-ups point here for the
completed adapter, Portal, real-app, and performance evidence.

## Outcome

`Cmd-F` opens a compact BBR find bar visibly and pointer-reachably above the
active native WKWebView. The bar drives the portable `bbr-web` find session,
shows the current ordinal and match count, supports next/previous navigation,
and returns focus to the page when dismissed without changing tab/session
identity.

## Scope

- Implement the macOS WebKit adapter for the L7 find request/result lifecycle.
- Render one `NativeAboveWeb` Portal presentation anchored to the content area's
  top edge with query field, count, previous, next, and close controls.
- Wire `Cmd-F`, Enter, Shift-Enter, Escape, pointer controls, focus entry, and
  deterministic focus restoration.
- Expose typed testkit requests, find snapshot state, stable `bbr-find-*` ids,
  and native Portal diagnostics.
- Add deterministic real-page fixtures with repeated, zero-match, boundary,
  replacement-query, stale-navigation, and page-mutation cases.
- Document implemented adapter and E2E behavior after evidence passes.

## Non-goals

- No regex, whole-word, case-sensitive, or selection-only UI unless separately
  prioritized.
- No search across tabs, history, PDFs, or browser chrome.
- No replacement of the L7 portable state machine.
- No GPUI-only overlay behind WebKit and no screenshot/pixel matching.

## Parallel-lane contract

Not parallelized. This consumes the shared Portal host, WebKit adapter, app
composition, testkit wire, and performance matrix.

## Constraints

- Use `PortalHost::NativeAboveWeb`; the bar must win real AppKit hit testing
  above the active WKWebView.
- The adapter echoes portable request ids and rejects stale completions through
  the existing contract.
- Opening, stepping, and closing never recreate or move the active web session.
- Escape and close clear highlights and restore the prior web/chrome focus
  domain deterministically.
- One active find presentation per browser window; tab/navigation changes
  invalidate or close it according to the canonical interaction contract.
- No new dependency unless pinned WebKit APIs cannot express the contract.
- Keep files below roughly 500 lines and preserve GUI serialization/deadlines.

## Evidence

### Unit and contract

- [x] Adapter mapping covers query start, count/ordinal report, next/previous,
      wrap/no-wrap boundaries, zero matches, finish, and stale invalidation.
      *(20 contract tests in `crates/bbr-web-macos/src/find_mapping/tests.rs`,
      green under `just verify`; the adapter echoes portable request ids and
      drops stale completions.)*
- [x] Portal placement/focus/dismissal tests preserve the L7 state invariants.
      *(Portal geometry/state/probe unit tests plus the `e2e-shell-portal`
      suite: one `NativeAboveWeb` host per window, non-key/non-main policy,
      frame sync, outside-dismiss — green.)*

### Real-app E2E

- [x] `just e2e-shell-find` opens over a real WKWebView and proves native
      visibility/frontmost ownership, query entry, exact count/ordinal, Enter
      and Shift-Enter navigation, zero matches, Escape dismissal, and focus
      return. *(6 scenarios, green; the shown-window scenario proves AppKit
      hit-test ownership at the portal frame center over the live WKWebView.)*
- [x] Resize while open proves Portal follow; tab/session/WebView identity and
      profile/space remain unchanged. *(`resize_isolation.rs`: resize follows
      in the same portal window, tab-switch discards, both sessions alive;
      every scenario asserts `assert_ownership_preserved`.)*
- [x] Typed snapshots/events and deterministic fixtures only; no sleeps or
      pixel comparisons. *(All scenarios poll typed snapshots with explicit
      deadlines; fixtures are deterministic local pages.)*

### Performance

- [x] Add `find_open`, `find_query_result`, and `find_step` optimized scenarios
      with at least 30 accepted samples and semantic/native presentation
      milestones. *(30 accepted samples each in `just e2e-perf`; each sample
      commits only from a portal render that drew the armed WebKit-reported
      open/report identity — fast-but-wrong guard, render-inclusive.)*
- [x] Opening and warm result navigation meet the command-surface warm budget;
      fixture/network time is excluded from app timings. *(Measured on the
      local benchmark host, release build, warm fixture page: `find_open`
      p95 0.26 ms / max 1.40 ms; `find_query_result` p95 0.68 ms / max
      2.23 ms; `find_step` p95 0.42 ms / max 0.69 ms against the 50/100 ms
      command-surface warm budget. The deterministic fixture is loaded before
      any sample, so fixture/network time never enters the measured path.)*

## Completion gates

```bash
just verify
cargo clippy -p bbr-web-macos --all-targets -- -D warnings
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-portal
just e2e-shell-find
just e2e-perf
just e2e
just roadmap
```

## Delivery sequence

1. Implement and contract-test the WebKit adapter.
2. Compose the smallest production Portal bar and focus choreography.
3. Add typed testkit state and real-app fixtures/E2E.
4. Add optimized performance evidence.
5. Run independent native-boundary review, focused remediation, and recheck.

## Iteration policy

First prove one query and one reported count in a real WKWebView. Then add
navigation, dismissal/focus restoration, page invalidation, and resize follow.
Diagnose failures as portable lifecycle, WebKit result mapping, Portal z-order,
focus, or test fixture timing before changing code.

## Blocked stop condition

If the pinned WebKit API cannot produce deterministic count/ordinal semantics,
stop with the L7 contract and Portal host intact. Report APIs tried, observed
callbacks, stale-result behavior, fixture evidence, and the smallest adapter or
product-contract decision required. Never fake counts in GPUI or accept a bar
hidden behind WebKit.

## Delivery ledger

```text
Outcome: post-MVP packet implemented and evidenced
Commits: lane `dd33e3d`; serial main integration `c12ecbc`; delivery record
  `efadc43`
Functional evidence: adapter mapping contract tests (20) green; probe find-guard
  unit tests (5) green; portal placement/state/probe unit tests green
Real-app E2E: just e2e-shell-find 6/6 green (open over real WKWebView with
  native frontmost-ownership evidence, count/ordinal, Enter/Shift-Enter wrap,
  zero matches, Escape/close focus restoration, stale navigation, page
  mutation, resize follow, tab-switch discard, page-world isolation)
Performance evidence: find_open / find_query_result / find_step scenarios,
  30 accepted samples each, p95 0.26 / 0.68 / 0.42 ms and max 1.40 / 2.23 /
  0.69 ms against the 50/100 ms command-surface warm budget (local host,
  report-only; absolute ceilings enforced). The strategy executes in
  WKContentWorld::defaultClientWorld (page-world isolation proven by the
  find-isolation E2E) and stays non-destructive (reads DOM + moves selection
  only; highlights/selection transient, cleared on finish/invalidate/zero)
Roadmap/documentation updates: task packet evidence + ledger; ARCHITECTURE.md
  adapter/portal-bar section; E2E_ARCHITECTURE.md find suite section
Open blockers and non-goals: none within scope; no regex/whole-word/
  case-sensitive/selection-only UI, no cross-tab/history/PDF search, no
  GPUI overlay behind WebKit, no screenshot matching
Worktree state: integrated on `main`; historical lane/worktree state is not a
  current execution authority
```
