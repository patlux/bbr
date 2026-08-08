# Task packet — BBR context menus over web content

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native portal host](native-portal-host.md) ·
[native-integration track](../tracks/native-integration.md) ·
[UI design-system track](../tracks/ui-design-system.md).

Status: **implemented ownership/Portal foundation.** The bounded action and
ownership-matrix enrichment is owned by the
[Page-background context-menu follow-up](page-background-context-menu.md);
that packet preserves this conservative router and may start only after F2
closeout when Portal/context-menu territory is free.

## Outcome

BBR-owned context menus that geometrically overlap native web content remain
visible, keyboard-operable, and pointer-reachable above the WKWebView through
the shared Portal infrastructure. WebKit-owned page context menus continue to
use their native path and are never replaced by generic BBR chrome.

## Scope

- Define the ownership/routing decision between WebKit-owned page menus and
  BBR-owned chrome menus.
- Route qualifying BBR menus through `NativeAboveWeb`; keep purely GPUI chrome
  menus on `GpuiDeferred`.
- Provide anchored placement, screen/content clamping, keyboard traversal,
  selection, Escape/outside dismissal, focus restoration, and teardown.
- Expose typed menu ownership/host/anchor/selection diagnostics and stable ids.
- Add a minimal real production menu whose anchor crosses the native content
  boundary, plus real-app E2E over a WKWebView.

## Non-goals

- No replacement or restyling of WebKit's link/image/selection/editable/media
  context menus; the follow-up keeps these classes native.
- No interception of page JavaScript context-menu behavior beyond explicit
  product policy.
- No menu-specific native window; use the one host per browser window.
- No screenshot/texture composition or pixel-based evidence.

## Parallel-lane contract

Not parallelized. Menu ownership, Portal stack priority, focus, AppKit hit
testing, and testkit state are shared browser-window contracts.

## Constraints

- Exactly one native Portal host per browser window; menus are presentations in
  its stack, not independent windows.
- AppKit owns only attachment, z-order, frame, visibility, and hit testing;
  GPUI owns menu state/content.
- The ownership decision must be explicit and testable: page-owned remains
  native, BBR-owned uses the appropriate Portal host.
- Menu open/close never changes tab/session/profile/space ownership.
- Preserve native right-click behavior in ordinary page content.
- Keep files below roughly 500 lines; no new dependency unless existing
  gpui-component/GPUI behavior is insufficient and proven so.

## Evidence

### Unit and contract

- [ ] Ownership routing tests cover page-owned, GPUI-only, and
      native-above-web BBR menus.
- [ ] Placement, priority, keyboard selection, dismissal, and focus tests cover
      edge anchors and stacked Portal presentations.

### Real-app E2E

- [ ] `just e2e-shell-context-menu` proves a BBR menu over a real WKWebView is
      frontmost/hittable, navigable by pointer and keyboard, and dismisses
      without session mutation.
- [ ] A WebKit-owned page context menu still follows the native page path and
      is not converted into the BBR Portal.
- [ ] Resize/move and teardown leave no visible or attached orphan.

### Performance

- [ ] Add a warm `context_menu_open` scenario with at least 30 accepted samples
      when the production menu path becomes user-visible.

## Completion gates

```bash
just verify
cargo clippy -p bbr-web-macos --all-targets -- -D warnings
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-portal
just e2e-shell-context-menu
just e2e-perf
just e2e
just roadmap
```

## Delivery sequence

1. Specify ownership routing with one production example per host class.
2. Implement anchored Portal menu composition and focus/dismissal.
3. Add typed testkit diagnostics and real-app E2E.
4. Add optimized interaction evidence and native-boundary review.

## Iteration policy

First prove one BBR-owned menu over WebKit without disturbing the page menu.
Then add keyboard/focus, edge clamping, and stacked-priority cases. Diagnose
failures as ownership routing, Portal ordering, anchor conversion, focus, or
AppKit hit testing before changing the design.

## Blocked stop condition

If BBR cannot reliably distinguish ownership or preserve WebKit's native page
menu, stop with existing menu behavior intact. Report event sources, APIs tried,
observed ownership ambiguity, and the smallest product/native decision needed.
Do not globally suppress WebKit context menus as a workaround.

## Delivery ledger

```text
Outcome: post-MVP packet documented
Commits: planning only
Functional evidence: pending
Real-app E2E: pending
Performance evidence: pending
Roadmap/documentation updates: task packet + Phase 8/track links
Open blockers and non-goals: native WebKit page menus remain out of BBR Portal ownership
Worktree state: no implementation lane active
```
