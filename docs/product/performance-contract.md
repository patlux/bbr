# Browser performance contract

Status: **canonical product behavior and budgets** for perceived responsiveness.
Implementation facts remain in [ARCHITECTURE.md](../../ARCHITECTURE.md), test
mechanics in [E2E_ARCHITECTURE.md](../../E2E_ARCHITECTURE.md), and delivery
status in [ROADMAP.md](../../ROADMAP.md).

## Product promise

BBR must feel instant in ordinary browser operation. Performance is a product
invariant from the first browser shell onward, not a Phase-9 cleanup exercise.
Phase 9 may optimize and tighten budgets, but earlier phases must establish and
continuously preserve the baseline.

"Instant" means two things:

1. **Correct content is present immediately.** The first usable window frame
   contains the complete sidebar and the native web view for the currently
   selected tab. BBR must not visibly present an empty chrome-only window,
   placeholder page, wrong tab, or delayed native-view attachment first.
2. **Direct manipulation responds within the perception threshold.** Tab,
   space, pane, and sidebar mutations commit on the next practical frame and
   remain below the hard latency budgets below.

External network and remote page performance are not attributed to BBR. Startup
page-readiness benchmarks use deterministic local fixtures and are reported
separately from shell/native-view presentation.

## Initial budgets

These are initial hard product budgets on the canonical macOS benchmark
machine. The target describes the intended experience; the gate is the maximum
allowed distribution before the test fails.

| Operation | Target | Hard gate |
|---|---:|---:|
| Process start → first usable window frame with sidebar and selected tab's visible, non-zero native web view | ≤ 250 ms | p95 ≤ 500 ms; max ≤ 750 ms |
| Process start → selected deterministic fixture page ready | ≤ 500 ms warm | warm p95 ≤ 1,000 ms; cold p95 ≤ 1,500 ms |
| Select an already-warm tab → correct native view presented | next display frame | p95 ≤ 50 ms; max ≤ 100 ms |
| Switch space → its already-warm selected tab presented | next display frame | p95 ≤ 50 ms; max ≤ 100 ms |
| Close active tab → correct neighbor and native view presented | next display frame | p95 ≤ 50 ms; max ≤ 100 ms |
| Pin/unpin or reorder a tab → new row order presented (region boundary moved, session untouched) | next display frame | p95 ≤ 50 ms; max ≤ 100 ms |
| Move a tab between spaces of the same profile → tab presented in the destination with its warm session | next display frame | p95 ≤ 50 ms; max ≤ 100 ms |
| Move a tab across profiles → fresh native session presented in the destination profile's store | ≤ 100 ms | p95 ≤ 100 ms; max ≤ 150 ms |
| Reopen a closed tab → fresh native session presented with preserved identity | ≤ 100 ms | p95 ≤ 100 ms; max ≤ 150 ms |
| Open, close, or toggle the sidebar → chrome and native content geometry committed | next display frame | p95 ≤ 50 ms; max ≤ 100 ms |
| Focus a tab/split pane or route a browser command to the active target | next display frame when visual | p95 ≤ 50 ms; max ≤ 100 ms |

Budgets cover the production mutation path, render/layout work, native-view
visibility or geometry updates, and presentation-commit observation. Testkit
socket round-trip time must not be counted as application latency for direct
interaction metrics.

**Session-recreation class.** Reopening a closed tab and moving a tab across a
profile boundary present a *freshly created* native session rather than a warm
one. WKWebView creation on the benchmark
host measured 24–60 ms across four 10-sample runs (p95 30–60 ms), so the
reopen/cross-profile-move rows above are the session-recreation ceiling, not
the warm-interaction ceiling. The gate keeps ≥ 2.5× headroom over observed
worst-case p95 and still catches real regressions (a 2× slowdown trips it); a
warm-session selection, reorder, pin, or same-profile move with no recreation
must stay within the 50 ms warm ceiling.

A slower development machine may run the suite in report-only mode, but the
canonical benchmark machine enforces both absolute budgets and the checked-in
baseline regression policy. Functional E2E remains mandatory everywhere.

## Startup contract

The startup benchmark must distinguish these milestones:

1. process spawned;
2. domain/window state restored or initialized;
3. sidebar/chrome laid out;
4. selected tab's WKWebView created or restored, attached, visible, and given a
   non-zero final content rectangle;
5. first usable presentation committed;
6. deterministic fixture content ready.

The first visible/usable presentation is valid only when milestones 3 and 4
refer to the same selected tab and window state. A fast sidebar followed by a
late web view does not pass the startup contract.

Measure at least:

- cold process launch with a fresh test profile;
- warm process launch with persisted browser state and warmed filesystem cache;
- restoration with many persisted tabs while instantiating only the selected
  tab's required native session;
- startup with the sidebar expanded and collapsed.

## Interaction contract

Every user-visible state transition that can affect perceived responsiveness
must add or extend a real-app performance scenario in the same implementation
group. This includes, as capabilities land:

- tab open, selection, close, reopen, reorder, pin, and move — all measured;
- space switching and profile-boundary moves — both measured;
- sidebar open, close, toggle, resize, and dense-list scrolling;
- command-bar activation, first result, filtering, and activation;
- split creation, pane focus, divider movement, and split close;
- session suspend, resume, discard recovery, and persistence restoration;
- window creation and restoration.

A fast but incorrect transition fails. Benchmarks must assert the selected tab,
visible native session, sidebar state, focus target, and final geometry before
accepting a latency sample.

## Benchmark methodology

Performance E2E must:

- launch the real optimized BBR app with the test-only local harness;
- exercise real GPUI rendering and real WKWebViews when the web-engine boundary
  is involved;
- use deterministic loopback fixtures and no external network dependency;
- use typed operation IDs and monotonic in-process timestamps around the real
  production command path;
- observe a semantic presentation-commit milestone, not merely command receipt
  or model mutation;
- run GUI scenarios sequentially on a stable, idle, display-attached Mac;
- discard explicit warm-up iterations;
- retain raw samples, p50, p95, maximum, build/profile metadata, and benchmark
  machine identity as failure artifacts;
- use enough repetitions to make percentile gates meaningful: at least 10
  startup samples per mode and 30 samples for repeated interactions in the full
  gate;
- avoid sleeps, pixel polling, wall-clock page dependencies, and debug-build
  timings as acceptance evidence.

A short PR gate may use fewer repetitions for rapid regression detection, but
it must still enforce absolute safety ceilings. The full canonical run enforces
percentiles and updates trend data.

## Regression policy

- Fail when an absolute hard gate is exceeded.
- On the canonical machine, also fail when p95 regresses by more than 15%
  against the checked-in accepted baseline, even if the absolute ceiling still
  passes.
- Do not hide regressions by averaging unrelated operations or dropping slow
  valid samples.
- Do not increase a budget or replace a baseline merely to make a change pass.
  A relaxation requires an explicit product decision, raw before/after
  evidence, the identified cause, and updated roadmap acceptance.
- Performance fixes require a benchmark regression case that demonstrates the
  old failure and the improved distribution whenever reproducible.
- Store benchmark artifacts and trends; do not rely on a single manually quoted
  timing.

## Required gates

The benchmark foundation must provide:

- `just e2e-perf` — bounded PR/local real-app performance regression gate;
- `just e2e-perf-full` — repeated canonical benchmark and trend artifact run;
- inclusion of the short performance gate in the canonical sequential GUI gate
  once runtime is stable enough for normal development;
- a dedicated stable macOS runner for authoritative absolute and baseline
  enforcement.

Until that harness exists, no feature may claim that these budgets are proven.
The foundational task is tracked in
[performance-e2e-foundation.md](../roadmap/tasks/performance-e2e-foundation.md).
