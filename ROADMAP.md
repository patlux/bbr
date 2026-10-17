# BBR Roadmap

Live product roadmap and the repository's sole stable delivery entry point.
Product intent lives in [PRODUCT.md](PRODUCT.md), target architecture in
[PLAN.md](PLAN.md), implemented facts in [ARCHITECTURE.md](ARCHITECTURE.md),
and the visual contract in [DESIGN.md](DESIGN.md).

Checklist ownership and update rules: [docs/roadmap/README.md](docs/roadmap/README.md).
Repository workflow and mandatory verification: [AGENTS.md](AGENTS.md).

## Current execution

- **Chrome material:** [Desktop glass for browser chrome](docs/roadmap/tasks/chrome-desktop-glass.md) shows native behind-window material through the sidebar and content surround; the near-opaque first tint is corrected to 55%/66% coverage. WebKit content remains opaque. Patrick's visual acceptance and native-composite proof are open. Settings › Appearance now carries a live [sidebar transparency](docs/roadmap/tasks/chrome-transparency-setting.md) slider.

- **Isolated build tooling:** [Linux-to-macOS cross-build](docs/roadmap/tasks/linux-macos-cross-build.md) builds and ad-hoc signs the ARM64 macOS app on Linux, with checksum-bound Tart functional evidence. Native defaults and Visual ownership remain unchanged; performance and independent acceptance remain open.

- **Portal lifecycle fix:** [Portal window lifecycle](docs/roadmap/tasks/portal-window-lifecycle.md) binds the Command Palette to its owning window; locally integrated with fresh combined-tree Tart verification, including the Titlebar reload-synchronization repair.

- **Visual tooling:** [Reusable visual snapshots](docs/roadmap/tasks/visual-snapshot-tooling.md) owns the active standalone GPUI Visual/Rust-consumer migration, retaining the integrated browser capture and [Native Gallery visual coverage](docs/roadmap/tasks/gallery-visual-coverage.md) foundation. Captures move to Tart, beginning with the [Tokens / Checkbox / Sidebar pilot](docs/roadmap/tasks/gallery-visual-pilot.md); story selection, independent coverage and shared review do not imply baseline approval. Combined-tree baseline review and Gallery supported-state/framing remain open; see Current delivery for ownership and evidence.

- **Current delivery:** [Current delivery](docs/delivery/current.md)
- **Reality gaps:** the serial [reality gaps packet](docs/roadmap/tasks/reality-gaps.md) (Patrick 2026-09-30) makes the tests run what users run: default profile directory, native HTTP authentication, real input events, background links, pop-ups from script clicks, persistent website data.
- **Design blueprint:** the [complete browser blueprint](docs/roadmap/tasks/paper-browser-blueprint.md) is drawn in Paper (W0–W7 done, 2026-09-28); Patrick adopted all 24 deltas (DESIGN §19); they are implemented serially in the §19 order, performance evidence deferred with the rest of the queue.
- **Active product slice:** the [v2 visual refresh and motion program](docs/roadmap/tasks/v2-visual-refresh-program.md) (adopted 2026-09-28, DESIGN §18) has S0–S11 integrated on `main` ([closeout](docs/roadmap/tasks/v2-closeout.md)) and the follow-up [S12 Paper alignment](docs/roadmap/tasks/v2-paper-alignment.md) integrated (2026-09-28; functional aggregate green including Portal 7/7 on the unlocked console; performance execution deferred). RA-MEDIA is paused after its integrated portable contract and resumes after the program. Earlier: [UI contract conformance](docs/roadmap/tasks/ui-contract-conformance.md) and the [folder membership/pinned invariant](docs/roadmap/tasks/folder-pinned-invariant.md) are integrated (2026-09-27; the invariant's performance gate waits for the override). The adopted design-branch direction (DESIGN §17.16) is underway: the [seated content card](docs/roadmap/tasks/seated-content-card.md), the [address field and space identity](docs/roadmap/tasks/address-field-space-identity.md) and the [Favorites grid and drag offers](docs/roadmap/tasks/favorites-drag-offers.md) are integrated (performance gates deferred to the end of the queue); the [media-session capability contract](docs/roadmap/tasks/media-session-capability-contract.md) is next. The completed [Daily-driver browser delivery](docs/delivery/2026-08-12-daily-driver-delivery.md) remains the semantic foundation. Split remains fully scheduling-deferred and its stale worktree stays untouched.
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
  → [background link tabs](docs/roadmap/tasks/background-link-tabs.md)
  → [web-content navigation gestures](docs/roadmap/tasks/web-content-navigation-gestures.md)
  → [current-base gesture revalidation](docs/roadmap/tasks/web-gesture-revalidation.md)
  → [Link Preview / Peek](docs/roadmap/tasks/link-preview-peek.md)
  → [renderer isolation and unresponsive tabs](docs/roadmap/tasks/renderer-isolation-unresponsive-tabs.md)
- **v2 visual refresh and motion (Phase 4):** [program](docs/roadmap/tasks/v2-visual-refresh-program.md)
  → S0 [contract](docs/roadmap/tasks/v2-contract.md) → S1 [system preferences](docs/roadmap/tasks/v2-system-prefs.md)
  / S2 [tokens](docs/roadmap/tasks/v2-tokens.md) → S3 [rail](docs/roadmap/tasks/v2-rail.md)
  → S4 [space paper](docs/roadmap/tasks/v2-space-paper.md) → S5 [floating surfaces](docs/roadmap/tasks/v2-overlays.md)
  → S6 [motion core](docs/roadmap/tasks/v2-motion-core.md) / S9 [brand icon](docs/roadmap/tasks/v2-brand-icon.md)
  → S7 [after-commit moments](docs/roadmap/tasks/v2-motion-moments.md)
  → S8 [empty states](docs/roadmap/tasks/v2-empty-states.md) / S10 [theme picker](docs/roadmap/tasks/v2-theme-picker.md)
  → S11 [closeout](docs/roadmap/tasks/v2-closeout.md) → S12 [Paper alignment](docs/roadmap/tasks/v2-paper-alignment.md)
- **Reference-design alignment (Phase 4):** [system-appearance theme](docs/roadmap/tasks/system-appearance-theme.md)
  → [real favicon pipeline](docs/roadmap/tasks/real-favicon-pipeline.md)
  → [favicon disk cache](docs/roadmap/tasks/favicon-disk-cache.md)
  → [sidebar reference polish](docs/roadmap/tasks/sidebar-reference-polish.md)
  → [UI contract conformance](docs/roadmap/tasks/ui-contract-conformance.md)
  → [folder membership/pinned invariant](docs/roadmap/tasks/folder-pinned-invariant.md)
  → adopted design-branch direction: [seated content card](docs/roadmap/tasks/seated-content-card.md)
  → [address field and space identity](docs/roadmap/tasks/address-field-space-identity.md)
  → [Favorites grid and drag offers](docs/roadmap/tasks/favorites-drag-offers.md)
  → [closed pinned tabs and Favorites that leave the list](docs/roadmap/tasks/closed-pins-and-favorites.md)
  → [dragging pins and Favorites back out](docs/roadmap/tasks/drag-back-out-of-pins.md);
  separately, the prioritized [media-session capability contract](docs/roadmap/tasks/media-session-capability-contract.md)
  → [sidebar media control](docs/roadmap/tasks/sidebar-media-control.md)
- **Daily-driver Sidebar:** [full-height titlebar navigation](docs/roadmap/tasks/titlebar-navigation-controls.md)
  → [Space creation and appearance UI](docs/roadmap/tasks/space-creation-appearance-ui.md)
  → [daily-driver Sidebar main](docs/roadmap/tasks/daily-driver-sidebar-main.md)
  → [Tab/Favorite menus](docs/roadmap/tasks/tab-favorite-context-menus.md)
  → [Page-background menu](docs/roadmap/tasks/page-background-context-menu.md)
  → [reference-faithful production shell](docs/roadmap/tasks/visual-shell-reference.md)
- **Release:** [deterministic app packaging](docs/roadmap/tasks/macos-app-packaging.md)
  and [deterministic process shutdown](docs/roadmap/tasks/process-lifecycle-shutdown.md)

The detailed dependency and scheduling state for these programs lives only in
the Current-delivery pointer above. Delivery-control completion is audited
requirement-by-requirement in the
[delivery-control completion audit](docs/delivery/delivery-control-completion-audit.md).
