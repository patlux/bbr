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
- [x] `spaces`: visible switching UI and last-active restoration are covered. *(The shell-spaces and shell-spaces-keyboard suites assert profile identity (hues/chips/ids), pointer + keyboard activation through the production paths, per-space active-tab restoration with warm WebViewIds and no reloads, empty-space pane naming, and the cross-profile recreation policy; the `space_switch` performance scenario measures warm switching against the gate-enforced 50/100 ms budget.)*
- [x] `sidebar`: adjustable/collapsible rail geometry is covered. *(The shell-sidebar suite asserts pointer drag through the production listener path (the testkit seam drives the exact app methods the element handlers call), keyboard resize/reset parity on the focused divider, Cmd-B collapse/expand with last-expanded-width restoration, compact-window clamping that keeps the native content at its 64 pt floor, engine-received native content rect correctness, selected WebViewId/session preservation, and the chrome focus contract; the `sidebar_resize`/`sidebar_restore` performance scenarios measure the width-commit and expand paths against the next-frame budget.)*
- [ ] `splits`: create/focus/resize/close/move teardown.
- [ ] `commandbar`: staged New Tab mode, cancellation, and initial focus restore are covered; ranking, result navigation, and complete mode UX remain.
- [ ] `webkit`: profile isolation is covered; popups, [Link Preview / Peek](../tasks/link-preview-peek.md), permissions, downloads, and process recovery remain with Phase 8.
- [ ] `peek`: eligible new-window request, native overlay isolation, dismissal/focus restoration, no-reload promotion, exclusions, cleanup, and profile isolation.
- [ ] `persistence`: restore, force quit, migrations, corrupt optional state.
- [ ] `accessibility`: semantic ids, labels, order, real input.
- [ ] `performance`: optimized real-app startup, first usable sidebar + selected native content presentation, tab/space switching, active-tab close, sidebar open/close/toggle/resize/restore, command latency, and memory lifecycle; see the [foundation task](../tasks/performance-e2e-foundation.md).

## Harness improvements

- [ ] Emit semantic window-resized events instead of snapshot polling.
- [ ] Emit `Quitting` before graceful shutdown.
- [ ] Add model snapshots for complete profile/space/split collections; active profile/space and tab/session snapshots exist. *(Complete profile/space collections with per-space active tab, live session, and tab count are exposed; split collections remain.)*
- [ ] Add protocol commands as product capabilities land; profile/space create/activate/move/suspend and open/select/close tabs exist.
- [x] Add bounded individual `just e2e-smoke` / `just e2e-tabs` / `just e2e-profiles` / `just e2e-shell-tabs` / `just e2e-shell-spaces` / `just e2e-shell-sidebar` recipes and a canonical sequential `just e2e` gate. *(The sidebar suite joined the canonical gate with the slice that proved it stable.)*
- [ ] Add suite manifests and a generic `just e2e <suite>` runner when more suites justify it.
- [ ] Add testkit release-exclusion gate.
- [x] Add optimized benchmark mode, typed presentation milestones, raw-sample artifacts, and canonical-gated baseline comparison. *(Mode/milestones/raw samples and the >15% p95 relative policy are implemented; the relative rule activates once a canonical machine is named.)*
- [x] Add bounded `just e2e-perf` / `just e2e-perf-full` and include the short gate in canonical sequential `just e2e`. *(`e2e-perf-full` is the repeated cold gate, not an alias, and `just e2e-perf-inject` proves delay detection.)*
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
