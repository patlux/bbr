# Task packet — find-in-page Portal bar

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native portal host](native-portal-host.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Testing track](../tracks/testing.md).

Status: **post-MVP; depends on the integrated L7 portable contract and native
Portal host.**

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

- [ ] Adapter mapping covers query start, count/ordinal report, next/previous,
      wrap/no-wrap boundaries, zero matches, finish, and stale invalidation.
- [ ] Portal placement/focus/dismissal tests preserve the L7 state invariants.

### Real-app E2E

- [ ] `just e2e-shell-find` opens over a real WKWebView and proves native
      visibility/frontmost ownership, query entry, exact count/ordinal, Enter
      and Shift-Enter navigation, zero matches, Escape dismissal, and focus
      return.
- [ ] Resize while open proves Portal follow; tab/session/WebView identity and
      profile/space remain unchanged.
- [ ] Typed snapshots/events and deterministic fixtures only; no sleeps or
      pixel comparisons.

### Performance

- [ ] Add `find_open`, `find_query_result`, and `find_step` optimized scenarios
      with at least 30 accepted samples and semantic/native presentation
      milestones.
- [ ] Opening and warm result navigation meet the command-surface warm budget;
      fixture/network time is excluded from app timings.

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
Outcome: post-MVP packet documented
Commits: planning only
Functional evidence: pending
Real-app E2E: pending
Performance evidence: pending
Roadmap/documentation updates: task packet + Phase 8/track links
Open blockers and non-goals: requires integrated Portal host and L7 adapter work
Worktree state: no implementation lane active
```
