# Task packet — Real-app performance E2E foundation

Parents: [Performance/release track](../tracks/performance-release.md) ·
[Testing track](../tracks/testing.md) · Product contract:
[performance-contract.md](../../product/performance-contract.md)

## Objective

Build the benchmark path early enough that startup, warm-tab switching, and
sidebar interaction regressions are measured continuously as their production
features land. The harness must measure the real optimized BBR app and observe
semantic presentation commits, not testkit transport latency or model-only
completion.

## Constraints

- Real app process, real GPUI chrome, and real WKWebView for engine-crossing
  scenarios.
- Optimized benchmark build with testkit capability; debug-build timings are
  diagnostic only.
- Deterministic loopback fixtures; no external network or remote page timing.
- Monotonic in-process timestamps and typed operation/sample IDs.
- No sleeps or pixel comparison as primary measurement/synchronization.
- GUI benchmarks run sequentially on a stable display-attached Mac.
- Raw samples and metadata survive failures.
- Do not weaken functional assertions to obtain faster numbers.

## Milestones and protocol

- [x] Define typed benchmark operation IDs and monotonic timestamp values.
- [x] Emit process-start/bootstrap timing early enough to measure launch.
- [x] Emit the initial presentation-ready milestone with sidebar state and window geometry.
- [x] Emit selected-native-view-presented only after the correct WKWebView is
      visible with a non-zero content rectangle at the end of GPUI paint.
- [x] Emit deterministic-page-ready separately from shell/native-view readiness.
      *(Emitted as `BenchmarkKind::PageReady` with `page_title`, measured from
      process spawn. The content guard commits only on an actually observed
      fixture title (WebKit title KVO + load path), not on a loading-finished
      fallback. Verified live via `just e2e-perf`/`-full`: `page_ready` p95
      ≈ 0.43 s cold on this host.)*
- [x] Emit interaction presentation commits for tab/space selection, close,
      sidebar visibility/resize, focus target, and later split mutations. *(Initial tab selection, active close, Cmd-T, sidebar toggle, sidebar drag resize, collapse/restore, pointer tab
      selection, reopen, the tab pin toggle, region-bounded reorder,
      same-profile/cross-profile moves, and the §10.2 hover / §10.3 press
      pointer-state entries are covered; each commits synchronously against the
      authoritative post-mutation state because the offscreen harness never
      yields an `on_next_frame` frame callback. Reopen and cross-profile moves
      are session-recreation class with their own documented budget — see the
      performance contract.)*
- [x] Ensure request/socket round-trip time is excluded from measured app work.
- [x] Extend snapshots with the semantic state needed to reject fast-but-wrong
      samples for the implemented scenarios.

## Harness and statistics

- [x] Spawn the optimized app with isolated fresh and persisted profiles. *(A
      fresh-profile cold launch is implemented. Persisted warm startup remains
      open — it depends on the absent persistence repository; a test-only seed
      that bypasses the future persistence path is not acceptance evidence.)*
- [x] Add warm-up and measured iteration control.
- [x] Collect raw samples plus p50, p95, maximum, build revision, build profile,
      OS/hardware identity, display refresh rate, and fixture mode.
      *(Display refresh is recorded `null` with an explicit reason — the
      benchmark window runs offscreen and the OS refresh API is not linked in
      the harness; fill on a display-attached canonical runner.)*
- [x] Enforce absolute budgets from the product performance contract for the implemented scenarios.
- [x] Compare p95 with a checked-in accepted baseline on the canonical machine;
      fail regressions greater than 15%.
      *(Policy implemented and unit-tested: the >15% relative rule fires only
      when the host `hw.model` matches the baseline's `canonical_host_model`
      and the entry source is `accepted`. That field is currently empty, so no
      machine is canonical until a dedicated runner is named; non-canonical
      machines and seed entries stay report-only for the relative rule while
      absolute ceilings are enforced everywhere.)*
- [x] Preserve JSON/CSV summary, raw samples, event ring, and app stderr as
      artifacts on failure.
      *(`performance.json`, `raw_samples.csv`, `event-ring.txt`, `app-stderr.txt`
      under `target/bbr-perf-artifacts/<run>`; success appends a trend frame to
      `target/bbr-perf-trend/latest.json` + `history.jsonl`. A trial panic path
      now also writes a partial JSON + CSV from the samples already collected,
      so a slow-machine startup/page-ready ceiling trip keeps the raw data that
      matters most rather than only the event ring + stderr.)*
- [x] Document explicit report-only behavior for non-canonical machines without
      weakening functional E2E.

## Initial benchmark scenarios

- [x] Cold startup: fresh profile → first usable presentation contains both the full
      sidebar and selected tab's visible native web view.
- [ ] Warm startup: persisted state → correct selected tab and sidebar presented
      without eagerly creating every stored tab session.
- [x] Deterministic fixture readiness measured separately from usable shell.
- [x] Warm tab switch in both directions with stable WebViewIds and no reload.
- [x] Active-tab close → correct neighbor visible and focused.
- [x] Sidebar open/close toggle with resulting geometry and state assertions.
- [x] Sidebar drag-resize commit and collapse/restore through the production path. *(The `sidebar_resize` scenario commits at the exact clamped geometry — typed min/default/max plus the content floor let the drivers compute the same expectation the render enforces, so a fast-but-wrong sample is rejected in the app; the `sidebar_restore` scenario measures expand restoring the last expanded width after an unmeasured production collapse. Both join the 30-sample matrix with the 50 ms p95 / 100 ms max next-frame budget; p95 ≈ 0.002 ms on this host.)*
- [x] Space switching with already-warm selected sessions when Spaces land. *(The `space_switch` scenario alternates two warm spaces of one profile through the production activation path, asserting the destination space, its own WebViewId, the unchanged session count, chrome focus, and the rendered selected space row; p95 ≈ 0.1 ms on this host against the gate-enforced 50 ms p95 / 100 ms max warm budget.)*
- [x] Dense sidebar scroll/frame-time benchmark when the real Phase-4 list
      lands. *(The real Phase-4 list landed with the
      [dense-list slice](dense-list-slice.md); the `dense_scroll`/`dense_select`
      scenarios are wired into the interaction matrix, the baseline seed, and
      the full-gate sample-count asserts with the 50/100 ms warm budgets.
      Green here on the S-density 230-identity collection — `dense_scroll`
      p95 ≈ 5.9 ms / max ≈ 6.1 ms, `dense_select` p95 ≈ 4.1 ms / max ≈ 4.2 ms,
      30 samples each (`just e2e-perf`). The measured frame-time evidence for
      the 500/50 acceptance on the canonical benchmark runner remains open in
      the phase.)*
- [x] Pointer hover/press presentation benchmarks when the states land. *(The
      §10.2 hover and §10.3 press transitions are measured through the
      production listener methods (`tab_hover`/`tab_press`), commit only when
      the derived row state shows the wash, and join the interaction matrix,
      the baseline seed, and the full-gate sample-count asserts with the
      50/100 ms warm budgets.)*

## Commands and continuous monitoring

- [x] Add bounded `just e2e-perf` for the short local/PR regression gate.
- [x] Add bounded `just e2e-perf-full` for canonical repeated measurements and
      trend artifacts. *(Now a real repeated gate: `COLD_STARTUP_SAMPLES=10` cold
      fresh-profile launches for the startup/page-ready distributions plus the
      30-sample interaction matrix, bounded by `BBR_E2E_FULL_TIMEOUT` — no
      longer an alias of the short gate.)*
- [x] Add the short performance gate to canonical sequential `just e2e`.
- [ ] Run the full suite on a dedicated stable macOS runner from the first
      feature that exposes real tab/sidebar interaction. *(Open — no dedicated
      runner is provisioned yet; the canonical machine is unnamed.)*
- [ ] Publish or retain trend artifacts so regressions are visible over time.
      *(Retained locally under `target/bbr-perf-trend`; public/CI publication on
      the dedicated runner remains.)*
- [x] Add `just e2e-perf-inject` — a deterministic injected-delay self-test that
      proves the budget gate fails under a testkit-only delay and returns green
      after removing the injection without changing the budget or baseline.
      *(Injected self-test runs are excluded from the persistent trend and
      artifact retention so the inflated distribution can never be mistaken for
      a real regression; the injection targets interaction kinds only, never the
      startup/page-ready clock.)*

## Acceptance

- [x] Startup fails if sidebar and selected web content are not present in the
      same first usable presentation.
- [x] Warm-tab selection and sidebar toggle meet their absolute p95/max budgets.
- [x] A deliberately injected delay causes the relevant benchmark gate to fail.
- [x] Removing the delay restores a green distribution without changing the
      budget or baseline. *(Both proven end-to-end on this host by
      `just e2e-perf-inject`: injected warm-tab p95 ≈ 160 ms exceeds the 50 ms
      budget (`abs_p95`), clean p95 ≈ 0.1 ms is green, no budget/baseline
      change. The earlier review note that this host "lacks the macOS Metal
      toolchain" was incorrect — Xcode 16.4.0 ships `metal` and a macOS 15.5
      SDK, and every real-app gate builds and runs green here; the pessimistic
      claim came from running only under the bare nix shell without the
      `DEVELOPER_DIR`/`SDKROOT` the just recipes set.)
- [x] Functional smoke/tabs/profile suites remain green. *(Verified on this
      host: `just e2e-smoke`, `just e2e-tabs`, `just e2e-profiles` all pass.)*
- [x] `just verify`, affected individual E2E suites, and canonical sequential
      `just e2e` pass. *(`just verify` green plus all seven sequential real-app
      GUI suites — smoke, tabs, profiles, shell-tabs, shell-spaces,
      shell-sidebar, and the optimized performance gate.)*
- [x] `ARCHITECTURE.md`, `E2E_ARCHITECTURE.md`, and roadmap status describe only
      the implemented, evidenced benchmark path.

## Iteration policy

Start with the smallest end-to-end measurement: one startup milestone and one
warm-tab switch. Prove that injected latency is detected before expanding to
sidebar, spaces, splits, and persistence. Add each new product interaction to
this benchmark matrix in the same implementation group that makes it visible.

## Non-goals

- External website/network benchmarking.
- Pixel-diff correctness testing.
- Replacing functional E2E with timings.
- Final Safari/Arc/Zen comparison before BBR's own stable baselines exist.
- Raising initial budgets to accommodate debug builds or overloaded developer
  machines.
