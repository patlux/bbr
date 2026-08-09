# Task packet — Productive new-window request seam

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[Browser orchestration track](../tracks/browser-orchestration.md) ·
[Native integration track](../tracks/native-integration.md) ·
[Testing track](../tracks/testing.md) ·
[browser interaction contract](../../product/browser-interactions.md) ·
[performance contract](../../product/performance-contract.md).

Program position: **NW-SEAM**, the serial production-wiring predecessor for
[web-content navigation gestures and opener return](web-content-navigation-gestures.md)
and [Link Preview / Peek](link-preview-peek.md). The portable request,
decision, outcome, and macOS coordinator foundations already exist; this packet
makes exactly that seam production-live without adding opener provenance,
preview lifecycle, background-opening policy, or insertion policy.

## Outcome

A real `WKUIDelegate::createWebViewWithConfiguration` request reaches one
portable decision authority and one app/controller execution path. A direct
link activation opens exactly one fresh regular tab in the source tab's
Space/Profile. A script-created `window.open` is blocked and reported instead
of silently disappearing. The delegate always returns `None`; only the browser
controller creates and owns the new session.

## Scope

- Add one portable MVP default policy: user-initiated request → regular tab;
  script-initiated request → reported block.
- Wire the existing macOS `NewWindowCoordinator` into every production
  `WebViewDelegate`, extract the live request context, and forward outcomes to
  the app without creating a second WebKit/session authority.
- Add one controller operation that resolves the exact source `WebViewId`,
  opens one regular tab in that source tab's owning Space/Profile, preserves
  profile isolation, and rejects stale/unknown sources without mutation.
- Add app pump/counter state and a typed testkit seam that drives the same app
  execution method used by the native coordinator.
- Add deterministic local popup fixtures and a focused real-app E2E suite:
  real WebKit script block plus user-initiated injection through the production
  app seam.
- Register the focused functional recipe and preserve the required optimized
  production-path performance scenario, budgets, baselines, and sample counts.
- Update implemented architecture facts only after the production path and
  required functional evidence pass.

### Settled source-space policy

The source tab's owning Space/Profile is authoritative even when that Space is
not active. The new regular tab is created there and remains warm; the active
Space, its selection, and its focused session stay unchanged. Routing into the
currently active Space would violate request identity and can cross profile
isolation, so it is forbidden.

### Settled E2E evidence split

The hidden real-app harness cannot synthesize a trustworthy native
`WKNavigationType::LinkActivated` click. Therefore:

- the script-block case is fully native: a real loaded page executes
  `window.open`, WebKit invokes the real UI delegate, the coordinator applies
  the portable policy, and the app records the reported block;
- the user-initiated case uses a typed testkit request that enters the exact
  same `BbrApp::on_new_window_request` production method called by the native
  coordinator outcome pump. It may not call the controller operation directly
  or bypass app counters/outcome handling.

This split proves the native delegate bridge and the regular-tab execution path
without presenting synthetic controller input as a native click.

## Non-goals

- No Peek eligibility, transient preview, promotion, dismissal, or native
  preview z-order. Those belong to [Link Preview / Peek](link-preview-peek.md).
- No opener `TabId` provenance or Back-at-root return. Those belong to
  [web-content navigation gestures](web-content-navigation-gestures.md).
- No modifier/middle-click handling, foreground/background selection policy,
  custom insertion position, named-window reuse, popup allow-list, window-
  feature filtering, or second browser window.
- No external-URL router duplication, URL normalization, persistence/schema,
  Portal, context-menu, Gallery, or unrelated UI work.
- No private SPI, stronger JavaScript opener authority, or WebKit-created
  untracked view returned from the delegate.

## Parallel-lane contract

```text
Base condition:          integrated B2, Z2-1, and G2 Status-Line checkpoint
Depends on:              existing portable new-window contract/coordinator;
                         free delegate, controller, app-pump, testkit, fixture,
                         and focused E2E territory
Provides/API:            production delegate → coordinator → app/controller
                         execution; typed block/open counters and outcome
Exclusive paths:         focused new-window policy/wiring/controller/app/test
                         modules and new fixture/E2E files
Shared integration points: bbr-browser and bbr-web-macos public exports,
                         bbr-app composition/pump, testkit protocol envelopes,
                         fixture registry, justfile functional E2E list,
                         architecture/roadmap facts
Integration order:       B2 → Z2-1 → G2 Status Line → NW-SEAM → N1/PEEK
```

No concurrent writer may own new-window disposition, the WKUI delegate,
controller tab creation, the app event pump, global testkit request envelopes,
or the same E2E/Justfile entries.

## Constraints

- One policy authority in portable `bbr-web`; the macOS adapter extracts and
  executes but never chooses a disposition.
- One session authority in `BrowserController`; the delegate always returns
  `None` and never constructs a parallel WKWebView/session registry entry.
- The request URL is preserved verbatim. No `normalize_url`, scheme default,
  default-port rewrite, or loading into the source view.
- The `NewWindowSession` source profile and the controller-resolved source
  profile must agree. Any stale/missing source or mismatch fails closed with no
  tab/session mutation.
- User-initiated open creates exactly one fresh regular tab/session. The source
  Space's existing open semantics decide visibility only when that Space is
  already active; inactive source Space stays inactive.
- Script-created requests create no tab/session and increment/report one typed
  blocked outcome. No silent suppression.
- Portable crates import no AppKit/WebKit/GPUI. Main-thread native ownership,
  profile isolation, existing external-URL authority, and release-build testkit
  absence remain intact.
- Files stay below roughly 500 lines. No compatibility layer, migration, new
  dependency, weakened assertion, budget, baseline, sample count, or deadline.

## Evidence

### Unit and contract

- [ ] Portable default-policy matrix proves direct activation → `OpenTab` and
      script request → `Block`; all existing request/decision/outcome tests stay
      green.
- [ ] Coordinator tests prove installed/missing handler, verbatim URL,
      source-profile preservation, and exactly-once blocked reporting.
- [ ] Controller fake-engine tests prove one source-Space/Profile tab/session,
      inactive-source-space neutrality, exact profile store, typed outcome,
      no duplicates, and stale/mismatched source refusal without mutation.
- [ ] App/testkit contract tests prove native-pump and injected requests share
      one `BbrApp::on_new_window_request` execution method and typed counters.

### Real-app E2E

- [ ] Real `popup.html` script invokes the production WKUI delegate and is
      blocked/reported with unchanged tab/session/source identity.
- [ ] Typed user-initiated injection through the production app seam opens
      exactly one fresh regular tab at the verbatim URL in the source
      Space/Profile.
- [ ] A non-default-profile source uses that profile's isolated native store;
      an inactive source Space remains inactive and the active selection is
      unchanged.
- [ ] Unknown/stale source and script-block paths create no tab/session.
- [ ] Stable typed events/snapshots, deterministic local fixtures, explicit
      deadlines, no sleeps, and no pixel assertions.

### Performance

- [ ] Preserve `new_window_regular_tab_present` as a fresh-session optimized
      production-path scenario with at least 30 accepted samples, semantic
      presentation commit, p95 ≤ 100 ms, and max ≤ 150 ms. External fixture or
      network latency stays outside application timing.
- [ ] Retain raw samples and fast-but-wrong guards. Execution may be deferred
      only by the active Current-delivery scheduling authority; deferred
      execution is explicit unpassed debt, never acceptance evidence.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-web
CARGO_BUILD_JOBS=4 cargo test -p bbr-web-macos
CARGO_BUILD_JOBS=4 cargo test -p bbr-browser
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 nix develop -c just verify
CARGO_BUILD_JOBS=4 nix develop -c just e2e-shell-new-window
CARGO_BUILD_JOBS=4 nix develop -c just e2e-tabs
CARGO_BUILD_JOBS=4 nix develop -c just e2e-profiles
CARGO_BUILD_JOBS=4 nix develop -c just e2e-perf
CARGO_BUILD_JOBS=4 nix develop -c just e2e
CARGO_BUILD_JOBS=4 nix develop -c just roadmap
```

GUI suites run sequentially through the repository GUI lock. When the active
Current-delivery authority defers performance execution, do not run
`e2e-perf`, do not run canonical `e2e` because it contains performance, and do
not alter either recipe. Run the required functional recipes individually and
record performance as unpassed debt.

## Delivery sequence

1. Bind one clean isolated writer from the integrated B2/Z2/G2 checkpoint and
   re-audit all shared ownership points.
2. Implement the portable default policy and controller operation with failing-
   first lower tests.
3. Wire coordinator ownership into the native delegate and app event pump;
   keep the delegate return structurally `None`.
4. Add typed testkit state, deterministic fixtures, and the smallest real-app
   E2E proving both native script block and production-seam regular open.
5. Run all non-performance and functional gates allowed by Current delivery.
6. One independent read-only review; one bounded remediation writer only for
   concrete findings; focused recheck; Orchestrator-only commit/integration.

## Iteration policy

Start with one source tab in one active Space/Profile, then add the inactive-
Space and non-default-profile cases. Classify failures as extraction,
coordinator ownership, portable decision, source identity resolution,
controller/session lifecycle, app-pump delivery, typed evidence, or fixture
behavior before editing. Change the smallest central cause and re-run focused
checks. Do not broaden into N1, PEEK, background opening, or UI.

## Blocked stop condition

If the public WKUI delegate cannot forward a plain request without requiring a
WebKit-owned returned view, or if source identity/profile cannot be resolved
without duplicating controller/session authority, stop with:

- exact API/source and runtime evidence;
- approaches attempted and observed ownership conflict;
- preserved worktree/branch fingerprints and current owner;
- affected unit/E2E/performance claims still open; and
- the smallest platform or product decision required next.

Do not fall back to private SPI, a detached WebKit view, arbitrary active-Space
routing, direct-test controller bypass presented as production evidence, or
weaker assertions.

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

Use measured timing only when runtime/command timestamps support it; otherwise
label values as estimates.
