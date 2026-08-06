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
- [ ] Emit deterministic-page-ready separately from shell/native-view readiness.
- [ ] Emit interaction presentation commits for tab/space selection, close,
      sidebar visibility/resize, focus target, and later split mutations. *(Initial tab selection, active close, Cmd-T, and sidebar toggle are covered.)*
- [x] Ensure request/socket round-trip time is excluded from measured app work.
- [x] Extend snapshots with the semantic state needed to reject fast-but-wrong
      samples for the implemented scenarios.

## Harness and statistics

- [ ] Spawn the optimized app with isolated fresh and persisted profiles. *(Fresh profile implemented; persisted mode remains.)*
- [x] Add warm-up and measured iteration control.
- [ ] Collect raw samples plus p50, p95, maximum, build revision, build profile,
      OS/hardware identity, display refresh rate, and fixture mode. *(Raw samples and p50/p95/max exist; machine/build metadata remains.)*
- [x] Enforce absolute budgets from the product performance contract for the implemented scenarios.
- [ ] Compare p95 with a checked-in accepted baseline on the canonical machine;
      fail regressions greater than 15%.
- [ ] Preserve JSON/CSV summary, raw samples, event ring, and app stderr as
      artifacts on failure.
- [ ] Document explicit report-only behavior for non-canonical machines without
      weakening functional E2E.

## Initial benchmark scenarios

- [x] Cold startup: fresh profile → first usable presentation contains both the full
      sidebar and selected tab's visible native web view.
- [ ] Warm startup: persisted state → correct selected tab and sidebar presented
      without eagerly creating every stored tab session.
- [ ] Deterministic fixture readiness measured separately from usable shell.
- [x] Warm tab switch in both directions with stable WebViewIds and no reload.
- [x] Active-tab close → correct neighbor visible and focused.
- [x] Sidebar open/close toggle with resulting geometry and state assertions.
- [ ] Space switching with already-warm selected sessions when Spaces land.
- [ ] Dense sidebar scroll/frame-time benchmark when the real Phase-4 list
      lands.

## Commands and continuous monitoring

- [x] Add bounded `just e2e-perf` for the short local/PR regression gate.
- [ ] Add bounded `just e2e-perf-full` for canonical repeated measurements and
      trend artifacts. *(Currently aliases the short gate.)*
- [x] Add the short performance gate to canonical sequential `just e2e`.
- [ ] Run the full suite on a dedicated stable macOS runner from the first
      feature that exposes real tab/sidebar interaction.
- [ ] Publish or retain trend artifacts so regressions are visible over time.

## Acceptance

- [x] Startup fails if sidebar and selected web content are not present in the
      same first usable presentation.
- [x] Warm-tab selection and sidebar toggle meet their absolute p95/max budgets.
- [ ] A deliberately injected delay causes the relevant benchmark gate to fail.
- [ ] Removing the delay restores a green distribution without changing the
      budget or baseline.
- [ ] Functional smoke/tabs/profile suites remain green.
- [ ] `just verify`, affected individual E2E suites, and canonical sequential
      `just e2e` pass.
- [ ] `ARCHITECTURE.md`, `E2E_ARCHITECTURE.md`, and roadmap status describe only
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
