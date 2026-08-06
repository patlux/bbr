# BBR Roadmap

Live product roadmap and status index. Product intent lives in [PRODUCT.md](PRODUCT.md), architectural rationale in [PLAN.md](PLAN.md), and implemented facts in [ARCHITECTURE.md](ARCHITECTURE.md).

Checklist ownership and update rules: [docs/roadmap/README.md](docs/roadmap/README.md). Repository workflow, mandatory E2E policy, and Angular commit rules: [AGENTS.md](AGENTS.md).

## Current focus

> **Phase 4 — establish BBR's interaction, performance, and visual foundation.** Native profile/session isolation is proven; the real-app startup/interaction benchmark foundation, the focus contract, the visual system, the controller-backed vertical-tab shell, visible space switching, the adjustable, collapsible sidebar, the tab-organization slice (pin/unpin, region-bounded reorder, and move-between-spaces — keyboard and pointer drag), the dense-500 list slice (bounded lifecycle, windowed rendering, recency, and gate-wired scroll/select benchmarks), the pointer-state + typed semantic-tree slice (§10.2 hover / §10.3 press through the production listeners; the internal §15 semantic model), and the cross-context profile-hint slice (DESIGN §11.3: after a cross-profile space activation the destination profile chip + "in Profile · <title>" appears at the top of the command field for one interaction, with a deterministic interaction-driven lifetime, typed snapshot/semantic exposure, and a gate-enforced `space_switch_cross_profile` scenario) are in place. Next: the measured frame-time evidence for the dense acceptance on the canonical runner and the remaining cross-context polish before broad Phase-5 command-bar work.

- [ ] [Establish real-app startup and interaction performance E2E](docs/roadmap/tasks/performance-e2e-foundation.md) before broad sidebar implementation. *(Benchmark path, budgets, trend/artifact machinery, the warm space-switch scenario, and the tab pin/reorder/move scenarios shipped; the dense scroll/select scenarios are wired into the interaction matrix with contract-budget seeds; persisted warm-startup samples await the persistence repository.)*
- [x] Resolve the tab-mutation focus matrix in the [browser interaction contract](docs/product/browser-interactions.md).
- [x] [Create `DESIGN.md`](DESIGN.md) with a distinct native macOS visual direction and semantic tokens.
- [x] [Implement the first controller-backed vertical-tab shell slice](docs/roadmap/phases/04-ui-shell.md) — visible tab rows (pointer + keyboard open/select/close/reopen), pinned/regular regions, states, empty space, crash recovery — with functional and performance E2E.
- [x] [Render and switch spaces with unmistakable profile identity](docs/roadmap/phases/04-ui-shell.md) — visible profile/space navigation with round-robin identity hues, pointer + keyboard activation, per-space active-tab restoration without recreating warm sessions, and real-app E2E + warm-switch performance coverage. *(`Cmd-Shift-M` staged moves and drag-into-space arrive with the tab-organization slice; the cross-context command-field profile hint for cross-profile switching is the [cross-context profile-hint slice](docs/roadmap/tasks/cross-context-profile-hint.md).)*
- [x] [Add an adjustable, collapsible sidebar with stable native-content geometry](docs/roadmap/phases/04-ui-shell.md) — DESIGN §5/§11.4 widths (200–360 pt, default 260), a visible 4 pt drag hairline with pointer drag through the production GPUI listener path, keyboard equivalents on the focused divider (`←`/`→` step, `0` reset), Cmd-B collapse/expand that restores the last expanded width, narrow-window clamping that keeps the native content at its 64 pt floor (never zero/negative), typed snapshot state (visible, width, min/default/max, divider id/state, content rect), and real-app E2E + resize/restore performance coverage.
- [x] [Tab organization: pin/unpin, region-bounded reorder, and move between spaces](docs/roadmap/phases/04-ui-shell.md) — complete keyboard commands (`Cmd-Shift-P`, `Cmd-Shift-↑/↓`, staged `Cmd-Shift-M` + Enter) and visible controls (row pin dot, §10.9 insertion tick with legality, drag-target space rows), real pointer drag through the production GPUI press/move/release listeners, illegal boundary drops rejected whole, same-profile moves that preserve the WebViewId and warm native session, cross-profile moves that recreate exactly once in the destination store with cookie isolation, and pin/reorder/move presentation benchmarks. Proven by `just e2e-shell-organize` and the `tab_pin`/`tab_reorder`/`tab_move_same_profile`/`tab_move_cross_profile` scenarios in the gate-enforced perf matrix.
- [x] [Dense-500 list slice: bounded lifecycle, windowed rendering, recency, and gate-wired scroll/select benchmarks](docs/roadmap/tasks/dense-list-slice.md) — 500 tab identities with exactly 50 live sessions and 450 suspended through the production controller paths (bounded concurrent-session peak while preparing the state), a bounded-window regular region that follows the production scroll offset and never lays out the collection, a pinned + recently-used keep set with exactly-one-session suspended recreation, and `dense_scroll`/`dense_select` scenarios wired into the perf matrix with contract budgets and full-gate sample-count asserts. Proven by `just verify`, `just e2e-shell-dense`/`e2e_shell_dense_edge`, and the canonical `just e2e`; the measured frame-time acceptance on the canonical runner remains open in the task packet. *(The dense state is in-memory — 500 identities / 50 live / 450 suspended — and never claims persisted restoration, which Phase 7 owns.)*
- [x] [Pointer-state and typed semantic tree: §10.2 hover / §10.3 press through the production listeners and the internal §15 semantic model](docs/roadmap/tracks/accessibility.md) — hover wash + close reveal and the pressed wash derive from app state the real pointer paths drive; the typed tree (roles/names/values/enabled/selected/focused/hierarchy) mirrors the render from the same row/space visuals, with real-app E2E (`just e2e-shell-hover`) and `tab_hover`/`tab_press` scenarios in the gate-enforced perf matrix. *(Internal model only — GPUI 0.2.2 has no platform Accessibility layer; the macOS Accessibility/VoiceOver/Appium bridge and packaged-app acceptance are Phase 9.)*

## Product phases

- [x] [0 — Reproducible foundation](docs/roadmap/phases/00-foundation.md)
- [x] [1 — Native GPUI + WKWebView integration baseline](docs/roadmap/phases/01-native-integration.md)
- [x] [2 — Portable browser orchestration](docs/roadmap/phases/02-browser-core.md)
- [x] [3 — Multi-session macOS WebKit adapter](docs/roadmap/phases/03-multi-session-webkit.md)
- [ ] [4 — Vertical tabs, spaces, and first intentional UI](docs/roadmap/phases/04-ui-shell.md) **← current**
- [ ] [5 — Unified command bar](docs/roadmap/phases/05-command-bar.md)
- [ ] [6 — Two-pane split view](docs/roadmap/phases/06-split-view.md)
- [ ] [7 — Persistence and crash recovery](docs/roadmap/phases/07-persistence.md)
- [ ] [8 — Daily-driver browser essentials](docs/roadmap/phases/08-daily-driver.md) — includes [Link Preview / Peek](docs/roadmap/tasks/link-preview-peek.md)
- [ ] [9 — Accessibility, polish, performance, and release hardening](docs/roadmap/phases/09-quality-release.md)
- [ ] [10 — Cross-platform adapters](docs/roadmap/phases/10-cross-platform.md)

## When the UI becomes beautiful

UI work is deliberately staged rather than deferred until the end:

1. **Phase 4 starts with visual direction and design tokens** before the real tab/sidebar shell is implemented.
2. **Every UI feature phase includes its own interaction, accessibility, empty/error-state, and visual-polish acceptance items.**
3. **Phase 9 contains a dedicated alpha/beta polish pass** across the complete product, including motion, typography, iconography, density, dark mode, VoiceOver, performance, and consistency.

This avoids polishing placeholder UI now, while also preventing a functional-but-ugly shell from accumulating until release.

## Cross-cutting tracks

- [Testing and deterministic E2E](docs/roadmap/tracks/testing.md)
- [Accessibility and input](docs/roadmap/tracks/accessibility.md)
- [UI design system and visual quality](docs/roadmap/tracks/ui-design-system.md)
- [Performance, energy, and release safety](docs/roadmap/tracks/performance-release.md) · [performance contract](docs/product/performance-contract.md)

## Status snapshot

Completed today:

- Reproducible Nix/direnv Rust environment.
- Multi-crate workspace and portable WebEngine contracts.
- Native macOS WKWebView child-view bridge inside GPUI.
- Real focus hand-off, title observation, resize synchronization, and navigation events.
- Deterministic Unix-socket testkit, fixture server, event replay/cursors, deadlines, and cleanup.
- Real-app WKWebView smoke covering load, navigation, back/forward, reload, resize, focus, and shutdown.
- Portable profile/space/tab/split/lifecycle domain model with 41 tests.
- Portable `bbr-browser` controller + deterministic fake engine: 54 tests
  covering tab/session/navigation/split/lifecycle policy, active-space visibility,
  profile moves, and typed controller events (`just verify` green).
- Real-app profiles suite proving cookie isolation across two WKWebsiteDataStores,
  same-profile WebView reuse, cross-profile recreation, and suspend cleanup.
- First controller-backed vertical-tab shell slice: reusable Chassis tokens/components,
  visible pinned/regular tab surface, favicon/title/loading/crash states, pointer +
  keyboard open/select/close/reopen with the focus-domain contract, empty-space
  handling, and crash recovery — proven by `just e2e-shell-tabs` and real-app
  performance scenarios (`tab_select_pointer`, `tab_reopen`).
- Visible profile/space navigation slice: complete profile/space collections with
  round-robin identity hues and per-row chips, pointer + keyboard activation,
  per-space active-tab restoration without recreating warm sessions, empty-space
  pane naming, and cross-profile switches that preserve cookie isolation — proven
  by `just e2e-shell-spaces` and the `space_switch` performance scenario
  (p95 ≈ 0.1 ms against the 50 ms warm budget).
- Adjustable/collapsible sidebar slice: DESIGN §5/§11.4 width contract
  (200–360 pt adjustable, 260 default, 64 pt native-content floor), the visible
  4 pt drag hairline with pointer drag through the production listener path,
  keyboard resize/reset parity on the focused divider, Cmd-B collapse/expand
  restoring the last expanded width, and narrow-window clamping that keeps the
  native WKWebView geometry strictly positive — proven by
  `just e2e-shell-sidebar` (pointer drag, keyboard parity, collapse/reopen,
  compact-window clamp, native content rect, WebViewId/session preservation,
  focus contract) and the `sidebar_resize`/`sidebar_restore` performance
  scenarios (p95 ≈ 0.002 ms against the 50 ms next-frame budget).
- Tab-organization slice: complete keyboard commands (`Cmd-Shift-P` pin/unpin,
  `Cmd-Shift-↑/↓` region-bounded reorder, `Cmd-Shift-M` staged move + Enter /
  Escape), visible pin control + §10.9 drag tick with invalid variant + drag-
  target space rows, real pointer drag through the production press/move/
  release listeners, illegal boundary drops rejected whole, same-profile moves
  that preserve the WebViewId and warm session, cross-profile moves that
  recreate exactly once in the destination store with cookie isolation, and
  Escape cancellation — proven by `just e2e-shell-organize` and the
  `tab_pin`/`tab_reorder`/`tab_move_same_profile`/`tab_move_cross_profile`
  performance scenarios (session-recreation class for the cross-profile move).
- Dense-500 list slice: 500 identities with exactly 50 live sessions and 450
  suspended through the production lifecycle paths (bounded concurrent-session
  peak during preparation), the bounded-window regular region with a
  deterministic scroll anchor, the pinned + recent recency keep set with
  warm-selection reuse and exactly-one-session suspended recreation, keyboard
  traversal with instant scroll-into-view, edge states, and the dense drag
  tick — proven by `just e2e-shell-dense` / `e2e_shell_dense_edge` plus the
  dense unit/contract suites, with `dense_scroll`/`dense_select` scenarios
  wired into the gate-enforced perf matrix (contract budgets seeded;
  measured frame-time evidence on the canonical runner remains open). The
  dense state is in-memory only — persisted restoration is Phase 7.
- Pointer-state + typed semantic-tree slice: the §10.2 hover wash + close
  reveal and the §10.3 pressed wash derive from app state the production
  `on_hover` / mouse-down/up listeners drive (never a snapshot mutation), the
  composed §10 row states expose `hover`/`active` with the more specific
  state winning, and the internal §15 semantic model (roles/names/values/
  enabled/selected/focused + hierarchy) mirrors the render from the same
  row/space visuals — proven by `just e2e-shell-hover` (hover/press
  transitions, close-reveal presence, typed focus clarity, disabled controls
  while dragging, profile chips, long-title/missing-favicon names, crash /
  empty / collapsed-sidebar branches) and the `tab_hover`/`tab_press`
  scenarios in the gate-enforced perf matrix. Internal model only: GPUI 0.2.2
  has no platform Accessibility layer; the macOS Accessibility bridge and the
  packaged-app VoiceOver/Appium/manual acceptance are Phase 9.
- Cross-context profile-hint slice: DESIGN §11.3 — after a successful
  cross-profile space activation the destination profile appears at the top
  of the command field for one interaction as the §12 chip plus a `type:label`
  "in Profile · <title>" hint, so a cross-profile destination is unmistakable
  without relying on color; same-profile switching never announces a profile
  transition; the lifetime is a deterministic user-interaction boundary (the
  next chrome context mutation — a space activation, tab mutation, or
  command-bar mode entry), never a timer; and the staged `MoveTab` hint is a
  separate surface that never coexists with it. The hint renders through the
  production command-field path with a stable `bbr-command-profile-hint` id,
  typed snapshot exposure, and a §15 `Chip` semantic node — proven by
  `just e2e-shell-spaces` (pointer + keyboard suites) and the new
  `space_switch_cross_profile` scenario in the gate-enforced perf matrix
  (p95 ≈ 0.195 ms against the 50 ms warm budget, sessions never recreated).
  Full slice: [cross-context profile-hint task packet](docs/roadmap/tasks/cross-context-profile-hint.md).

Next release-shaped outcome:

- A benchmark-protected vertical-tab and space shell that opens with sidebar and
  selected web content presented together, keeps warm tab/space switching,
  sidebar interaction (toggle, drag resize, keyboard resize/reset, collapse /
  restore), tab organization (pin/unpin, reorder, move between spaces), and
  dense-list interaction (scroll/select through the bounded-window render)
  within explicit latency budgets, and has defined visual, focus,
  profile-identity, pointer, and keyboard behavior proven through the real app.
  Next shell slice: the remaining shell craft items and the measured
  dense frame-time evidence on the canonical runner.
