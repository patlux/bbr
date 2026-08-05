# BBR Roadmap

Live product roadmap and status index. Product intent lives in [PRODUCT.md](PRODUCT.md), architectural rationale in [PLAN.md](PLAN.md), and implemented facts in [ARCHITECTURE.md](ARCHITECTURE.md).

Checklist ownership and update rules: [docs/roadmap/README.md](docs/roadmap/README.md). Repository workflow, mandatory E2E policy, and Angular commit rules: [AGENTS.md](AGENTS.md).

## Current focus

> **Phase 3 — finish native profile/session isolation.** Real tabs now run through the portable controller and multiple WKWebViews; next prove profile-separated website data, cross-profile recreation, and suspended-session cleanup.

- [ ] Real-app two-profile cookie-isolation scenario.
- [ ] Cross-profile tab move recreates the native session in the destination store.
- [ ] Suspending a warm tab removes its native view, delegate, and KVO observation.

## Product phases

- [x] [0 — Reproducible foundation](docs/roadmap/phases/00-foundation.md)
- [x] [1 — Native GPUI + WKWebView integration baseline](docs/roadmap/phases/01-native-integration.md)
- [x] [2 — Portable browser orchestration](docs/roadmap/phases/02-browser-core.md)
- [ ] [3 — Multi-session macOS WebKit adapter](docs/roadmap/phases/03-multi-session-webkit.md) **← current**
- [ ] [4 — Vertical tabs, spaces, and first intentional UI](docs/roadmap/phases/04-ui-shell.md)
- [ ] [5 — Unified command bar](docs/roadmap/phases/05-command-bar.md)
- [ ] [6 — Two-pane split view](docs/roadmap/phases/06-split-view.md)
- [ ] [7 — Persistence and crash recovery](docs/roadmap/phases/07-persistence.md)
- [ ] [8 — Daily-driver browser essentials](docs/roadmap/phases/08-daily-driver.md)
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
- [Performance, energy, and release safety](docs/roadmap/tracks/performance-release.md)

## Status snapshot

Completed today:

- Reproducible Nix/direnv Rust environment.
- Multi-crate workspace and portable WebEngine contracts.
- Native macOS WKWebView child-view bridge inside GPUI.
- Real focus hand-off, title observation, resize synchronization, and navigation events.
- Deterministic Unix-socket testkit, fixture server, event replay/cursors, deadlines, and cleanup.
- Real-app WKWebView smoke covering load, navigation, back/forward, reload, resize, focus, and shutdown.
- Portable profile/space/tab/split/lifecycle domain model with 40 tests.
- Portable `bbr-browser` controller + deterministic fake engine: 46 tests
  covering the first tab/session/navigation/split/lifecycle slices, session
  policy invariants, and typed controller events (`just verify` green).

Next release-shaped outcome:

- Use the same site in two profiles without sharing cookies, then move a tab across the profile boundary and prove its native session is recreated in the destination data store.
