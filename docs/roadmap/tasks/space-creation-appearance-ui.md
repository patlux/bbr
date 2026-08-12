# Task packet — Space creation and appearance UI

Parents: [Phase 4](../phases/04-ui-shell.md) · [Phase 8](../phases/08-daily-driver.md) · [UI design-system track](../tracks/ui-design-system.md) · [space appearance contract](space-appearance-contract.md) · [footer space strip](space-footer-strip-appearance-ui.md) · [daily-driver delivery](../../delivery/2026-08-12-daily-driver-delivery.md).

## Outcome

Creating a Space immediately creates and activates a real usable Space with deterministic defaults. The user can then change its name and curated identity through one production configuration surface, while the permanent footer remains an icon-only Space switcher with clear active state and accessible Space/Profile identity.

## Scope

- Route the existing Plus-menu and keyboard New Space entry through the one controller creation authority.
- Assign deterministic valid default name, theme, and `SpaceIdentity`, activate the new Space, and preserve profile ownership/isolation.
- Present one BBR-owned rename/identity configuration surface after creation and from the active Space's management affordance.
- Consume the existing curated emoji/BBR-glyph appearance contract; no free-form icon value.
- Render footer chips as icon-only identities in canonical order while retaining semantic names with Space and Profile titles.
- Add typed snapshot/semantics, deterministic E2E, performance code, and production Gallery stories from the same components.

## Non-goals

- No free color picker, arbitrary emoji/asset upload, nested Space hierarchy, profile creation redesign, Space deletion policy, schema migration, or Split behavior.
- No second Space switch/create/appearance state in `bbr-ui` or Gallery.
- No site-adaptive web-content tinting.

## Parallel-lane contract

```text
Base condition:          committed Daily-driver delivery authority and clean common activation base
Depends on:              integrated portable appearance contract, permanent footer strip, Plus menu, and controller Space creation
Provides/API:            app-owned Space configuration state; presentation-only picker model; icon-only footer composition
Exclusive paths:         focused Space create/config modules; focused footer Space presentation modules; focused Space E2E/Gallery modules
Shared integration points: bbr-app sidebar composition, controller facade, persistence/store seam, testkit snapshot, Gallery registry, performance registry, roadmap
Integration order:       after DD-TITLEBAR, before DD-SIDEBAR
```

## Constraints

- Creation commits before configuration. Escape/cancel keeps the valid created Space and defaults.
- New Space becomes active immediately and follows the existing warm-session/profile-hint contract.
- Identity is one curated emoji or BBR glyph; accessible names never rely on icon or color alone.
- Footer shows no Space title text at rest. Overflow scrolls horizontally and never starves Downloads or Plus.
- Renaming/identity changes target the exact SpaceId and cannot mutate another Space after switching or stale UI.
- No weakened isolation, session, focus, geometry, performance-code, or sample guards; files below roughly 500 lines.

## Evidence

### Unit and contract

- [x] Deterministic defaults, exact SpaceId targeting, validation, cancel, stale-source rejection, duplicate names, and identity catalog matrix.
- [x] Create activates once through the controller; configuration never creates a second Space.
- [x] Icon-only footer model preserves canonical order, active selection, accessible Space/Profile names, and overflow geometry.

### Real-app E2E

- [x] Plus → New Space creates and activates a Space immediately; canceling configuration leaves it usable.
- [x] Name and identity change only the created Space and update footer/semantics without session recreation.
- [x] Keyboard creation/configuration parity, Escape, outside dismissal, cross-profile hint, overflow, and restart/persistence truth are asserted according to available authority.
- [x] Existing Spaces, Sidebar, Organize drag-to-chip, Command, and profile-isolation suites remain green.

### Performance

- [x] Production-path create/present and identity-update scenarios are implemented with exact SpaceId/active-chip/presentation guards.
- [x] Execution remains unexecuted/unpassed under Patrick's override; budgets and baseline policy remain unchanged.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-domain -p bbr-browser -p bbr-ui
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets -- -D warnings
just e2e-shell-spaces
just e2e-shell-sidebar
just e2e-shell-organize
just e2e-gallery
just roadmap
git diff --check
```

## Delivery sequence

1. Prove and centralize create+activate defaults through the existing controller authority.
2. Add exact-Space configuration model and production surface.
3. Cut the footer to icon-only presentation without changing switch/drag authority.
4. Add E2E, semantics, performance code, and production Gallery stories.
5. Independent read-only review; bounded remediation only for concrete findings; focused recheck; Orchestrator integration.

## Iteration policy

Start with create+activate and cancel-safe defaults. Add rename, then identity selection, then icon-only footer. Diagnose failures as controller creation, stale Space identity, profile ownership, picker state, footer geometry, focus/dismissal, or evidence.

## Blocked stop condition

If the current persistence schema cannot truthfully retain appearance/name across restart, ship no false durable claim: stop with exact repository/schema evidence and the smallest serial migration decision. Session-valid creation/configuration may not be mislabeled persistent.

## Delivery ledger

```text
Outcome: create→immediate-activate→optional-config lifecycle on both creation
paths; exact-SpaceId/stale-safe rename/theme/identity commits; icon-only
footer with accessible Space+Profile names; truthful restart boundary
(rename persists; curated appearance session-valid under the current schema);
Gallery story at StoryId 46; compile-only performance scenarios.
Commits: none (worktree preserved uncommitted at reviewed input tree
d1f0ab82…/66 paths; sole remediation writer dd-space-review-remediation).
Functional evidence: domain+controller rename/stale/duplicate-title tests;
space-config model/ids/callbacks tests; gallery matrix tests; app semantics
accessible-name tests; gated real-app suites listed below.
Real-app E2E: e2e_shell_spaces_config (5 scenarios: plus-menu create→
activate→config→cancel; exact-space rename/theme/identity commit; Escape +
outside-press cancel; stale-surface never mutates another space; restart
truth rename-persists/identity-session-valid). Registered in just
e2e-shell-spaces.
Performance evidence: BenchmarkKind::SpaceCreatePresent +
SpaceConfigIdentityUpdate with render-gated surface observation and exact
SpaceId/active-chip/presentation guards; drivers + dispatch + perf_common
scenario block; compile-only (execution stays forbidden under the override).
Roadmap/documentation updates: packet checkboxes; store/browser.rs truthful
persistence boundary; DESIGN.md §11.3 config-surface + persistence note.
Open blockers and non-goals: no schema migration, no free-form icon, no
Split; canonical perf and just verify/just e2e remain forbidden (override).
Worktree state: branch parallel/dd-space-appearance at base 4d5be311…;
all changes unstaged; reviewed input tree d1f0ab82…/66 paths extended by
the remediation below.

Planning/read-only: verify preserved reviewed tree d1f0ab82…/66 paths; read
delivery control, packet, DESIGN.md §11.3/§11.4, persistence DTO, perf probe.
Implementation: Escape-in-cancel_command_bar fix; e2e_shell_spaces_config
suite; Justfile registration; perf kinds/drivers/dispatch/perf_common;
performance/space_config render gate; store/browser.rs boundary; packet.
Test execution: cargo check bbr+testkit+all-targets and gallery (green);
bounded gates below.
Review: independent review found Gallery StoryId 46 appended but eight
pinned-prefix unit tests still expecting 46 stories, a stale ledger tree,
and a soft 551-line module-budget finding.
Remediation: extend the eight stale bbr-gallery registry/prefix tests from
46 to 47 and pin StoryId 46='Space configuration'; correct the ledger final
tree truth to the final candidate binding; split app/space_config.rs into
the surface-behavior module plus sibling benchmarks/tests concern modules
so every lane-owned production file stays under the ~500-line budget
without behavior/API change.
Waiting/retries: GUI gates serialized through /tmp/bbr-gui-gate-6ff732a0fdc0bb09.lock.
Total elapsed: see orchestration ledger.
Human intervention: none.

Remediation-recheck pass (`dd-space-review-remediation`): Gallery
non-ignored unit tests 145/145 green (the eight pinned-prefix tests now
expect 47 stories and StoryId 46 is pinned to 'Space configuration'); fmt;
both warning-denied Clippy gates (bbr testkit all-targets and workspace
all-targets) in fresh separate target dirs; targeted unit/contracts
(domain 221, browser 119, ui 127, bbr bin 262); affected Space/Sidebar/
Organize/Gallery GUI suites; roadmap and diff. Final candidate binding
tree 5b0d8dd4…/68 paths (worktree temp-index tree computed at
remediation closeout; the Orchestrator's canonical temp-index computation
of the exact handover state is authoritative and supersedes this writer
hash, per the delivery-control convention); no red gate.
```
