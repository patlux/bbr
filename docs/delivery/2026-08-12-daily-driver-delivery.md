# Daily-driver browser delivery — 2026-08-12

Parent: [Roadmap](../../ROADMAP.md) · [Phase 4](../roadmap/phases/04-ui-shell.md) · [Phase 8](../roadmap/phases/08-daily-driver.md) · [browser interactions](../product/browser-interactions.md) · [daily-driver inventory](../product/daily-driver-capability-inventory.md).

## Outcome

BBR closes the ordinary browser-muscle-memory and chrome gaps Patrick identified before Split work resumes. The delivered browser opens modifier-activated links predictably in background tabs, exposes reliable web-content history gestures, presents a full-height macOS sidebar header, creates and configures Spaces, renders a daily-use Sidebar main area, and provides production context menus.

This delivery is the finite product contract for the current wave. Split remains scheduling-deferred and is not an implied dependency or parallel lane.

## Product decisions

### Background link opening

- `Cmd` + primary click and middle click on a web link open one regular tab in the background.
- The opener remains selected, visible, and web-focused.
- The first background child inserts immediately after its opener.
- Further background children from the same opener form one contiguous group after it in click order.
- The child records the existing session-local opener provenance. Navigation, reorder, same-window move, suspension, crash, and native-session recreation preserve that relation; closing either tab or ending the process invalidates it.
- Plain primary click retains normal page navigation. `Shift` combinations remain outside this bounded decision until separately specified.

### Full-height Sidebar header

- When expanded, the Sidebar owns the complete left rail from the top window edge to the footer, including the traffic-light-safe titlebar row.
- Expanded layout: `[traffic lights] [sidebar toggle] … [Back] [Forward] [Reload]`, followed by the address display.
- Empty safe-titlebar space remains a native window-drag region.
- Collapsing the Sidebar removes the complete BBR-owned Sidebar header and navigation cluster. No replacement content toolbar appears over the webpage.
- Native traffic lights remain available. `Cmd-B` restores the Sidebar.

### Spaces

- Creating a Space immediately creates and activates the real Space through the one controller authority.
- A deterministic default name and identity allow immediate use.
- The user can then configure name and one curated identity icon/emoji; cancelling keeps the valid defaults and does not delete the Space.
- The footer shows icon-only Space identities in canonical order, with an unmistakable active state and an accessible name containing Space and Profile names.

### Sidebar main and Favorites

- Regular tab rows follow the supplied Arc/Zen interaction references: clear favicon/title anatomy, inset hover/selected surface, trailing hover close control, one-line truncation, and stable dense-list behavior.
- Pinned and regular regions have a visible structural separator without duplicating identity or navigation authority.
- Favorites are compact icon tiles bound to concrete persistent `TabId` values.
- Dragging a tab into Favorites toggles that exact tab as a Favorite; it creates no copied tab or second native session.

### Context menus

- Tabs, Favorites, folders, and page background receive production menus bound to the exact invoking identity.
- Opening, traversing, or dismissing a menu is a no-op for selection, focus, sessions, order, expansion, Space, and Profile.
- WebKit-owned link/image/selection/editable/media/JavaScript menus remain native.
- Only actions with an existing typed product authority ship. Ambiguous or unavailable actions are omitted rather than faked.

## Bounded lanes

| Lane | Canonical packet | Independently valuable outcome |
|---|---|---|
| DD-LINK | [Background link tabs](../roadmap/tasks/background-link-tabs.md) | Cmd-click and middle-click open grouped background children directly after the opener. |
| DD-GESTURE | [Gesture real-app revalidation](../roadmap/tasks/web-gesture-revalidation.md) | The already integrated two-finger Back/Forward path is re-proven against the current integrated app and remediated only if the production path regressed. |
| DD-TITLEBAR | [Unified titlebar navigation](../roadmap/tasks/titlebar-navigation-controls.md) | Full-height expanded Sidebar header with safe traffic-light geometry; complete BBR header disappears when collapsed. |
| DD-SPACE | [Space creation and appearance UI](../roadmap/tasks/space-creation-appearance-ui.md) | Create, activate, name, and choose a curated identity; icon-only footer remains the permanent switcher. |
| DD-SIDEBAR | [Daily-driver Sidebar main](../roadmap/tasks/daily-driver-sidebar-main.md) | Production tab/pinned/Favorites anatomy and tab-to-Favorite drag match the approved interaction references. |
| DD-TAB-MENU | [Tab/Favorite context menus](../roadmap/tasks/tab-favorite-context-menus.md) | One source-bound production menu for Tab and Favorite presentations. |
| DD-PAGE-MENU | [Page-background context menu](../roadmap/tasks/page-background-context-menu.md) | Existing conservative Portal-backed page menu gains its currently owned actions without replacing native element menus. |
| DD-FOLDER-MENU | [Folder context menu](../roadmap/tasks/folder-context-menu.md) | Source-bound folder menu follows after the shared menu and folder-command seams settle; ambiguous destructive/conversion actions remain omitted. |

## Parallel execution and integration

All runnable lanes start from one recorded clean authority base in isolated sibling worktrees. Exactly one writer owns each worktree. Implementations and non-GUI checks may proceed concurrently; every real-app GUI suite remains host-exclusive through the repository lock.

Shared contracts remain serial. The intended merge order is:

1. DD-LINK;
2. DD-GESTURE if it produces a remediation candidate;
3. DD-TITLEBAR;
4. DD-SPACE;
5. DD-SIDEBAR;
6. DD-TAB-MENU;
7. DD-PAGE-MENU;
8. DD-FOLDER-MENU.

Expected conflicts in `bbr-app` composition, testkit envelopes, Gallery registry, `justfile`, snapshots, semantics, and roadmap roll-ups are integration costs. The Orchestrator resolves them semantically and preserves every accepted product authority.

## Verification policy

Each user-visible lane requires lower-level contract tests and deterministic real-app E2E through the production GPUI/WKWebView path. Performance code, typed milestones, budgets, baselines, sample guards, and registrations remain mandatory.

Patrick's active execution override remains in force for this wave:

- do not run performance recipes or generate samples;
- do not run canonical `just e2e` while it includes performance;
- do not run `just verify` while it reaches performance tests;
- run bounded non-performance checks, fresh-target warning-denied Clippy, roadmap, dependency, script, diff, and affected functional GUI suites;
- record performance as `unexecuted/unpassed` without weakening any contract.

## Boundaries

- No Split/SPLIT-COMPOSITION preflight, inspection, review, port, reconciliation, or writer work.
- No stale Split worktree inspection or reuse.
- No second tab, navigation, Space, Favorite, menu, Portal, profile, or native-session authority.
- No private WebKit SPI, global event tap, page screenshot/texture composition, or copied Gallery anatomy.
- No First-MVP completion claim until the lanes above and the separately recorded release debts are truthfully classified.
- Startup remains `535.866375 ms > 500 ms`; Phase-9 performance, packaging, accessibility, and release acceptance remain open.

## Completion and stop condition

The wave reaches a natural handoff when every started lane is either integrated with independent review and required non-performance/functional evidence, or explicitly blocked with preserved state and one named unblocker. If no defensible path remains for a lane, stop that lane without weakening tests or inventing product behavior; continue independent lanes where ownership and dependencies permit.
