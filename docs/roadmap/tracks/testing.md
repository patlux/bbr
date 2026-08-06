# Track — Testing and deterministic E2E

Parent: [Roadmap](../../../ROADMAP.md) · Architecture: [E2E_ARCHITECTURE.md](../../../E2E_ARCHITECTURE.md)

## Implemented foundation

- [x] Rust-first typed testkit protocol over a local authenticated Unix socket.
- [x] Test-only feature gate; no production network listener.
- [x] Sequence-numbered ring replay and client-side pending-event queue.
- [x] Fresh-event cursors prevent stale assertions.
- [x] Deterministic local HTTP fixtures.
- [x] Bounded command, event, process, and workflow deadlines.
- [x] stderr drainage, panic-safe cleanup, and failure artifacts.
- [x] Short Unix-socket test paths work under direnv and `nix develop`.
- [x] Real WKWebView smoke test.
- [x] Protocol v4 semantic browser events, command-bar modes, benchmark operation IDs, and typed profile/space/tab/session snapshots.
- [x] Sequential real-app tabs suite with stable native-session identity assertions.
- [x] Sequential real-app profiles suite with cookie-isolation and native lifecycle assertions.

## Suite growth

- [x] `browser-controller`: framework-free orchestration contracts.
- [ ] `tabs`: open/select/close and native session lifecycle are covered; reopen/pin/reorder/move remain with their visible UI slices.
- [ ] `spaces`: profile identity, activation, and moves are covered; visible switching UI and last-active restoration remain with Phase 4.
- [ ] `splits`: create/focus/resize/close/move teardown.
- [ ] `commandbar`: staged New Tab mode, cancellation, and initial focus restore are covered; ranking, result navigation, and complete mode UX remain.
- [ ] `webkit`: profile isolation is covered; popups, [Link Preview / Peek](../tasks/link-preview-peek.md), permissions, downloads, and process recovery remain with Phase 8.
- [ ] `peek`: eligible new-window request, native overlay isolation, dismissal/focus restoration, no-reload promotion, exclusions, cleanup, and profile isolation.
- [ ] `persistence`: restore, force quit, migrations, corrupt optional state.
- [ ] `accessibility`: semantic ids, labels, order, real input.
- [ ] `performance`: optimized real-app startup, first usable sidebar + selected native content presentation, tab/space switching, active-tab close, sidebar open/close/toggle/resize, command latency, and memory lifecycle; see the [foundation task](../tasks/performance-e2e-foundation.md).

## Harness improvements

- [ ] Emit semantic window-resized events instead of snapshot polling.
- [ ] Emit `Quitting` before graceful shutdown.
- [ ] Add model snapshots for complete profile/space/split collections; active profile/space and tab/session snapshots exist.
- [ ] Add protocol commands as product capabilities land; profile/space create/activate/move/suspend and open/select/close tabs exist.
- [x] Add bounded individual `just e2e-smoke` / `just e2e-tabs` / `just e2e-profiles` recipes and a canonical sequential `just e2e` gate.
- [ ] Add suite manifests and a generic `just e2e <suite>` runner when more suites justify it.
- [ ] Add testkit release-exclusion gate.
- [ ] Add optimized benchmark mode, typed presentation milestones, raw-sample artifacts, and baseline comparison. *(Mode/milestones/raw samples implemented; checked-in baseline comparison remains.)*
- [x] Add bounded `just e2e-perf` / `just e2e-perf-full` and include the short gate in canonical sequential `just e2e`. *(`e2e-perf-full` currently aliases the short gate.)*
- [ ] Add a small Appium Mac2 packaged-app blackbox suite in Phase 9.

## Definition of done for feature work

These are mandatory rules, not future roadmap items:

- Every new or changed browser feature has automated real-app E2E coverage in the same implementation group.
- Unit, fake-engine, integration, and contract tests are added where appropriate but never replace browser-feature E2E.
- Browser-behavior bug fixes include a real-app E2E regression test whenever automatable.
- If the harness cannot express the behavior, extending the harness is part of the feature.
- No sleep/pixel assertion is the primary synchronization mechanism.
- Relevant bounded checks and affected E2E suites pass before a roadmap item is checked or an implementation group is committed.
- Every user-visible interaction adds or extends a real-app performance scenario in the same implementation group; timings never replace functional state assertions.
- Benchmarks measure monotonic in-process production-path work through semantic presentation commit, excluding testkit transport and external network latency.
