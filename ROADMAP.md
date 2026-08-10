# BBR Roadmap

Live product roadmap and the repository's sole stable delivery entry point.
Product intent lives in [PRODUCT.md](PRODUCT.md), target architecture in
[PLAN.md](PLAN.md), implemented facts in [ARCHITECTURE.md](ARCHITECTURE.md),
and the visual contract in [DESIGN.md](DESIGN.md).

Checklist ownership and update rules: [docs/roadmap/README.md](docs/roadmap/README.md).
Repository workflow and mandatory verification: [AGENTS.md](AGENTS.md).

## Current execution

- **Current delivery:** [Current delivery](docs/delivery/current.md)
- **Active product slices:** [Link Preview / Peek](docs/roadmap/tasks/link-preview-peek.md) serial integration, followed by the reviewed [renderer isolation and unresponsive tabs](docs/roadmap/tasks/renderer-isolation-unresponsive-tabs.md) first-MVP stability lane; both build on the integrated [productive new-window seam](docs/roadmap/tasks/new-window-production-seam.md) and [web-content navigation gestures](docs/roadmap/tasks/web-content-navigation-gestures.md)
- **Current product milestone:** [Phase 4 — Vertical tabs, spaces, and first intentional UI](docs/roadmap/phases/04-ui-shell.md)

The current-delivery document is the only owner of volatile execution state:
verified base SHA, active/frozen lanes, writer ownership, next executable gate,
blockers, and serial integration order. Task packets own scope and acceptance;
phases and tracks own durable product acceptance. Historical ledgers are
preserved as evidence and never resumed as current execution after a recorded
cutover.

## Product phases

- [x] [0 — Reproducible foundation](docs/roadmap/phases/00-foundation.md)
- [x] [1 — Native GPUI + WKWebView integration baseline](docs/roadmap/phases/01-native-integration.md)
- [x] [2 — Portable browser orchestration](docs/roadmap/phases/02-browser-core.md)
- [x] [3 — Multi-session macOS WebKit adapter](docs/roadmap/phases/03-multi-session-webkit.md)
- [ ] [4 — Vertical tabs, spaces, and first intentional UI](docs/roadmap/phases/04-ui-shell.md) **← current product milestone**
- [x] [5 — Unified command bar](docs/roadmap/phases/05-command-bar.md)
- [ ] [6 — Two-pane split view](docs/roadmap/phases/06-split-view.md)
- [ ] [7 — Persistence and crash recovery](docs/roadmap/phases/07-persistence.md)
- [ ] [8 — Daily-driver browser essentials](docs/roadmap/phases/08-daily-driver.md)
- [ ] [9 — Accessibility, polish, performance, and release hardening](docs/roadmap/phases/09-quality-release.md)
- [ ] [10 — Cross-platform adapters](docs/roadmap/phases/10-cross-platform.md)

A phase checkbox is an acceptance roll-up, not an executable queue item. It is
checked only when every required linked acceptance item is directly proven.
Partially implemented outcomes remain open or are split at their canonical
owner; they are never mass-checked from commit history alone.

## Cross-cutting tracks

- [Testing and deterministic E2E](docs/roadmap/tracks/testing.md)
- [Accessibility and input](docs/roadmap/tracks/accessibility.md)
- [UI design system and visual quality](docs/roadmap/tracks/ui-design-system.md)
- [Performance, energy, and release safety](docs/roadmap/tracks/performance-release.md)
- [Browser orchestration](docs/roadmap/tracks/browser-orchestration.md)
- [Native web-engine integration](docs/roadmap/tracks/native-integration.md)

## Delivery contracts

- [Development workflow](docs/development-workflow.md): bounded slice,
  writer/reviewer/remediation lifecycle, evidence, and closeout.
- [Parallel delivery](docs/parallel-delivery.md): ownership, isolated worktrees,
  serial integration kernel, and host-wide GUI lock.
- [Task-packet template](docs/roadmap/templates/task-packet.md): required
  outcome, scope, constraints, evidence, gates, iteration policy, and blocked
  stop condition.

## Long-lived product programs

These links make every prepared task discoverable without turning the roadmap
into a duplicate execution queue:

- **Performance foundation and remaining canonical-runner evidence:**
  [performance E2E foundation](docs/roadmap/tasks/performance-e2e-foundation.md)
- **Persistence:** [portable core](docs/roadmap/tasks/persistence-core.md),
  [chrome-first hydration](docs/roadmap/tasks/persistence-hydration.md), and
  [schema-v2 decision](docs/roadmap/tasks/schema-v2-migration-decision.md)
- **Settings and appearance:** [global search settings contract](docs/roadmap/tasks/general-search-settings-contract.md),
  [space appearance contract](docs/roadmap/tasks/space-appearance-contract.md),
  and [footer appearance UI](docs/roadmap/tasks/space-footer-strip-appearance-ui.md)
- **Saved Pages:** [B1 portable persistence](docs/roadmap/tasks/saved-pages-portable-persistence.md)
  → [B2 actions and command provider](docs/roadmap/tasks/saved-pages-actions-command.md)
  → [B3 library](docs/roadmap/tasks/saved-pages-library.md)
- **Shared UI and Gallery:** [UI-library anchor](docs/roadmap/tasks/ui-library-anchor.md)
  → [Gallery MVP](docs/roadmap/tasks/component-gallery-mvp.md)
  → [browser components](docs/roadmap/tasks/component-gallery-browser-components.md)
  → [compositions](docs/roadmap/tasks/component-gallery-compositions.md)
- **Daily-driver browser essentials:** [capability inventory](docs/product/daily-driver-capability-inventory.md)
  → [web-content navigation gestures](docs/roadmap/tasks/web-content-navigation-gestures.md)
  → [Link Preview / Peek](docs/roadmap/tasks/link-preview-peek.md)
  → [renderer isolation and unresponsive tabs](docs/roadmap/tasks/renderer-isolation-unresponsive-tabs.md)
- **Release:** [deterministic app packaging](docs/roadmap/tasks/macos-app-packaging.md)
  and [deterministic process shutdown](docs/roadmap/tasks/process-lifecycle-shutdown.md)

The detailed dependency and scheduling state for these programs lives only in
the Current-delivery pointer above. Delivery-control completion is audited
requirement-by-requirement in the
[delivery-control completion audit](docs/delivery/delivery-control-completion-audit.md).
