# BBR Roadmap

Live product roadmap and status index. Product intent lives in [PRODUCT.md](PRODUCT.md), architectural rationale in [PLAN.md](PLAN.md), and implemented facts in [ARCHITECTURE.md](ARCHITECTURE.md).

Checklist ownership and update rules: [docs/roadmap/README.md](docs/roadmap/README.md). Repository workflow, mandatory E2E policy, and Angular commit rules: [AGENTS.md](AGENTS.md).

## Current focus

> **Phase 4 — establish BBR's interaction, performance, and visual foundation.** Native profile/session isolation is proven; the real-app startup/interaction benchmark foundation, the focus contract, the visual system, the controller-backed vertical-tab shell, and visible space switching are in place. Next: the adjustable, collapsible sidebar slice (drag/keyboard resize, collapse memory, compact-window clamping) — then tab drag/reorder UI.

- [ ] [Establish real-app startup and interaction performance E2E](docs/roadmap/tasks/performance-e2e-foundation.md) before broad sidebar implementation. *(Benchmark path, budgets, trend/artifact machinery, and the warm space-switch scenario shipped; persisted warm-startup samples await the persistence repository.)*
- [x] Resolve the tab-mutation focus matrix in the [browser interaction contract](docs/product/browser-interactions.md).
- [x] [Create `DESIGN.md`](DESIGN.md) with a distinct native macOS visual direction and semantic tokens.
- [x] [Implement the first controller-backed vertical-tab shell slice](docs/roadmap/phases/04-ui-shell.md) — visible tab rows (pointer + keyboard open/select/close/reopen), pinned/regular regions, states, empty space, crash recovery — with functional and performance E2E. *(Space switching is the next shell slice.)*
- [x] [Render and switch spaces with unmistakable profile identity](docs/roadmap/phases/04-ui-shell.md) — visible profile/space navigation with round-robin identity hues, pointer + keyboard activation, per-space active-tab restoration without recreating warm sessions, and real-app E2E + warm-switch performance coverage. *(Tab drag/move UI remains.)*
- [x] [Add an adjustable, collapsible sidebar with stable native-content geometry](docs/roadmap/phases/04-ui-shell.md) — DESIGN §5/§11.4 widths (200–360 pt, default 260), a visible 4 pt drag hairline with pointer drag through the production GPUI listener path, keyboard equivalents on the focused divider (`←`/`→` step, `0` reset), Cmd-B collapse/expand that restores the last expanded width, narrow-window clamping that keeps the native content at its 64 pt floor (never zero/negative), typed snapshot state (visible, width, min/default/max, divider id/state, content rect), and real-app E2E + resize/restore performance coverage. *(Tab drag/move UI remains.)*

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
- Portable profile/space/tab/split/lifecycle domain model with 40 tests.
- Portable `bbr-browser` controller + deterministic fake engine: 47 tests
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

Next release-shaped outcome:

- A benchmark-protected vertical-tab and space shell that opens with sidebar and
  selected web content presented together, keeps warm tab/space switching and
  sidebar interaction (toggle, drag resize, keyboard resize/reset, collapse /
  restore) within explicit latency budgets, and has defined visual, focus,
  profile-identity, pointer, and keyboard behavior proven through the real app.
  Next shell slice: tab drag/reorder and move-between-spaces UI.
