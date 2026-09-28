# Track — Testing and deterministic E2E

Parent: [Roadmap](../../../ROADMAP.md) · Architecture: [E2E_ARCHITECTURE.md](../../../E2E_ARCHITECTURE.md)

## Implemented foundation

- [x] Rust-first typed testkit protocol over a local authenticated Unix socket.
- [x] Test-only feature gate; no production network listener.
- [x] Sequence-numbered ring replay and client-side pending-event queue.
- [x] Fresh-event cursors prevent stale assertions.
- [x] Deterministic local HTTP fixtures.
- [x] Bounded command, event, process, and workflow deadlines.
- [x] Serialize every AppKit/WKWebView GUI recipe across sibling worktrees with
      the host-wide advisory gate from the [parallel-delivery foundation](../tasks/parallel-delivery-foundation.md).
- [x] stderr drainage, panic-safe cleanup, and failure artifacts.
- [x] Short Unix-socket test paths work under direnv and `nix develop`.
- [x] Real WKWebView smoke test.
- [x] Protocol v4 semantic browser events, command-bar modes, benchmark operation IDs, and typed profile/space/tab/session snapshots.
- [x] Sequential real-app tabs suite with stable native-session identity assertions.
- [x] Sequential real-app profiles suite with cookie-isolation and native lifecycle assertions.

## Suite growth

- [x] `browser-controller`: framework-free orchestration contracts.
- [ ] `tabs`: open/select/close and native session lifecycle are covered; the pin/reorder/move surface lives in the shell-organize suite. *(The shell-tabs + shell-states suites cover open/select/close/reopen, pinned/regular rendering, states, and crash recovery; `e2e_shell_organize` covers pin/unpin, region-bounded reorder, and same-profile/cross-profile moves through keyboard and pointer drag.)*
- [x] `spaces`: visible switching UI and last-active restoration are covered. *(The shell-spaces, shell-spaces-hint, and shell-spaces-keyboard suites assert profile identity (hues/chips/ids), pointer + keyboard activation through the production paths, per-space active-tab restoration with warm WebViewIds and no reloads, empty-space pane naming, and the cross-profile recreation policy; the §11.3 cross-context command-field hint is asserted for copy/ids/semantic node/lifetime on the pointer and keyboard paths plus the empty cross-profile branch (cross-profile sets, same-profile absent, deterministic clear boundaries, staged-move hint separate); the `space_switch` and `space_switch_cross_profile` performance scenarios measure warm switching against the gate-enforced 50/100 ms budget.)*
- [x] `sidebar`: adjustable/collapsible rail geometry is covered. *(The shell-sidebar suite asserts pointer drag through the production listener path (the testkit seam drives the exact app methods the element handlers call), keyboard resize/reset parity on the focused divider, Cmd-B collapse/expand with last-expanded-width restoration, compact-window clamping that keeps the native content at its 64 pt floor, engine-received native content rect correctness, selected WebViewId/session preservation, and the chrome focus contract; the G3 composition scenarios add the full §11.4 rail region/semantic ordering through the one `bbr_ui::sidebar` composition, the app-owned command/downloads child composition with focus/Escape/collapse and child visibility/exclusion, split collapse/expand native geometry with no identity churn, and the bounded dense window through the composition; the `sidebar_resize`/`sidebar_restore` performance scenarios measure the width-commit and expand paths against the next-frame budget.)*
- [x] `pointer-states` + `semantics`: the §10.2 hover / §10.3 press states and the internal §15 semantic tree are covered. *(The shell-hover suite drives the production hover/press listener methods, asserts the composed §10 row states (more specific wins; hover/press never select), the typed tree (roles/names/values/enabled/selected/focused + hierarchy; focused list + selected row; disabled controls on a dragged row; profile chips; long-title/missing-favicon names; crash/empty/collapsed branches), and the `tab_hover`/`tab_press` scenarios join the gate-enforced perf matrix.)*
- [ ] `splits`: create/focus/resize/close/move teardown.
- [x] `commandbar`: the current 25-scenario real-app suite covers staged New Tab and Location modes, cancellation and origin-focus restoration, typed loading/empty/results/error states, deterministic ranking and result navigation, activation/failure, cross-space/profile identity, History/Saved Pages providers, current-owner moved-tab activation, address-control coexistence, and native Portal identity/placement/reuse. G3 presentation is integrated with all 25 scenarios green after moving only the floating renderer/ids to `bbr_ui::surfaces::command`; post-merge Gallery 42/42 and Portal 7/7 supplement that product authority.
- [x] `webkit-delivered`: profile isolation, downloads, find-in-page, HTTP credential challenges, page zoom, typed web errors/recovery, external URL delivery, and the BBR-owned/WebKit-owned context-menu boundary have real-app coverage through their completed packets and bounded suites.
- [ ] `webkit-remaining`: web-content [Back/Forward gesture and child-tab opener return](../tasks/web-content-navigation-gestures.md), popups/[Link Preview / Peek](../tasks/link-preview-peek.md), [renderer isolation and unresponsive-tab recovery](../tasks/renderer-isolation-unresponsive-tabs.md), permission adapter/UI surfaces, client-certificate selection, upload/save panels, broader per-site settings UI, [Portal hardening](../tasks/native-portal-hardening.md), and process lifecycle closeout remain with Phase 8/9.
- Saved Pages verification is owned by the bounded [B1](../tasks/saved-pages-portable-persistence.md) → [B2](../tasks/saved-pages-actions-command.md) → [B3](../tasks/saved-pages-library.md) packets: portable/repository evidence first, then real-app Cmd-D/restart/Palette/menu/open behavior, then Library mutation and 1k bounded UI. This track adds no duplicate acceptance checklist.
- [ ] `peek`: eligible new-window request, native overlay isolation, dismissal/focus restoration, no-reload promotion, exclusions, cleanup, and profile isolation.
- [ ] `renderer-responsiveness`: real WKWebView hang detection, chrome-close/switch responsiveness, stale-event rejection, beforeunload/dialog boundaries, and the productive typed crash/recovery transport are covered through `e2e-shell-responsiveness`; the portable multi-view crash-burst reconciliation is covered by `bbr-browser` FakeEngine tests. The obsolete helper process case was removed with the lifecycle acceptance decision (no helper enumeration, ownership inference, or termination remains). The bounded [renderer-isolation packet](../tasks/renderer-isolation-unresponsive-tabs.md) awaits whole-candidate review/integration.
- [x] `persistence-core`: repository restore, schema mismatch/discard policy, corrupt optional state, graceful restart, SIGKILL recovery, reorder recovery, and visible-set-only hydration are covered.
- [ ] `persistence-remaining`: first-run/migration-error/recovery UI and later schema-owned product extensions remain.
- [ ] `accessibility`: semantic ids, labels, order, real input.
- [ ] `performance`: optimized real-app startup, first usable sidebar + selected native content presentation, tab/space switching, active-tab close, sidebar open/close/toggle/resize/restore, command latency, and memory lifecycle; see the [foundation task](../tasks/performance-e2e-foundation.md).

## Harness improvements

The bounded [visual snapshot tooling packet](../tasks/visual-snapshot-tooling.md)
owns production-chrome Light/Dark captures, exact comparisons and reviewed
baselines; native pixels and functional E2E remain separate evidence.

- [ ] Emit semantic window-resized events instead of snapshot polling.
- [x] Emit `Quitting` before graceful shutdown, after the gateway writer has flushed the successful Quit response so process exit cannot race the client response.
- [ ] Add model snapshots for complete profile/space/split collections; active profile/space and tab/session snapshots exist. *(Complete profile/space collections with per-space active tab, live session, and tab count are exposed; split collections remain.)*
- [ ] Add protocol commands as product capabilities land; profile/space create/activate/move/suspend and open/select/close tabs exist.
- [x] Add bounded individual `just e2e-smoke` / `just e2e-tabs` / `just e2e-profiles` / `just e2e-shell-tabs` / `just e2e-shell-spaces` / `just e2e-shell-sidebar` / `just e2e-shell-organize` recipes and a canonical sequential `just e2e` gate. *(The sidebar and organize suites joined the canonical gate with the slices that proved them stable.)*
- [x] Prove those recipes retain their functional/performance evidence while
      acquiring one lock derived from Git's common directory, so different
      worktrees cannot overlap GUI execution.
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
