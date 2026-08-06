# BBR Roadmap

Live product roadmap and status index. Product intent lives in [PRODUCT.md](PRODUCT.md), architectural rationale in [PLAN.md](PLAN.md), and implemented facts in [ARCHITECTURE.md](ARCHITECTURE.md).

Checklist ownership and update rules: [docs/roadmap/README.md](docs/roadmap/README.md). Repository workflow, mandatory E2E policy, and Angular commit rules: [AGENTS.md](AGENTS.md).

## Current focus

> **Phase 4 — establish BBR's interaction, performance, and visual foundation.** Native profile/session isolation is proven; next establish real-app startup and interaction benchmarks, resolve tab-focus behavior, define the visual system, then expose real controller-backed tabs and spaces in the GPUI sidebar.

- [ ] [Establish real-app startup and interaction performance E2E](docs/roadmap/tasks/performance-e2e-foundation.md) before broad sidebar implementation.
- [x] Resolve the tab-mutation focus matrix in the [browser interaction contract](docs/product/browser-interactions.md).
- [ ] Create `DESIGN.md` with a distinct native macOS visual direction and semantic tokens.
- [ ] Implement the first controller-backed vertical-tab and space-switching UI slice with functional and performance E2E.

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

Next release-shaped outcome:

- A benchmark-protected vertical-tab and space shell that opens with sidebar and
  selected web content presented together, keeps warm tab/space switching and
  sidebar interaction within explicit latency budgets, and has defined visual,
  focus, profile-identity, pointer, and keyboard behavior proven through the real app.
