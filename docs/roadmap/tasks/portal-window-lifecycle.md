# Task packet — keep the Portal with its browser window

Parents: [Phase 4](../phases/04-ui-shell.md) · [Native integration](../tracks/native-integration.md) · [Portal hardening](native-portal-hardening.md).

## Outcome

The Command Palette and every consumer of its native Portal stay with their
own browser window, never above another app or independently on every Space.

## Scope

Reset GPUI PopUp defaults at the existing AppKit bridge; reconcile commanded
visibility with the owning window/application lifecycle. Add measured native
policy diagnostics and real-app regression coverage for activation, window
switching, hide/show, minimize/restore and hidden parents. The user-authorized
integration follow-up repairs the existing Titlebar reload E2E synchronization:
require a fresh load/commit/completion for each shortcut/click, then the expected
settled title, without weakening title, availability or session assertions.

## Non-goals

No new command state, renderer, overlay host, focus authority, dependencies,
profile changes, desktop automation or changes to other delivery lanes.

## Constraints

Keep one non-key/non-main child per browser window and real native WKWebViews.
Preserve draft text and session identity when temporarily hiding a presentation.
Use public AppKit APIs, no polling or render-time window ordering. Deferred
work must read current visibility intent so stale show work cannot resurrect a
hidden or closed host. Hidden test parents must not expose standalone Portals.

## Parallel-lane contract

One exclusive writer in a sibling worktree from a committed base. Production
territory is the native Portal bridge; tests/diagnostics touch the app's Portal
snapshot and testkit routing. Expected conflicts with ongoing drag/internal-page
work are additive request/snapshot fields, roadmap pointers and architecture
text. Main's existing work integrates first; this patch follows serially with
semantic unions and fresh gates. No other worktree is changed or merged here.

## Evidence

- [x] Real-app regression fails on the old PopUp policy and passes on the fix.
- [x] App/window switches hide only the owning host, preserve its draft and
      restore the same child without changing web sessions.
- [x] App hide, minimize, hidden parent, dismissal and teardown never leave a
      visible/input-intercepting orphan; dismissal while inactive stays closed.
- [x] Native policy has the parent's level, no independent all-Spaces flag,
      a measured child relation while shown and non-key/non-main behavior.
- [x] Existing consuming suites and stable functional aggregate remain green.

Verified in the isolated lane and locally integrated with fresh combined-tree
Tart gates. The Titlebar synchronization repair preserves every final assertion
and requires a fresh native reload before checking the settled title. Runtime
evidence and exact logs are recorded in [Current delivery](../../delivery/current.md).
Cross-app deactivation uses two actual BBR processes, preserving both real AppKit
and WebKit boundaries.

## Completion gates

Bounded `mbx` unit/contract tests and Clippy for the touched crates; `just verify`;
Portal, Command, Find, Context Menu, Auth, Sidebar, Zoom, Spaces, Shutdown and
Windows real-app suites; stable canonical `just e2e` without the explicitly deferred performance
recipe, plus Peek and Gallery. Existing performance code/budgets remain intact;
execution is deferred by Current delivery and is not claimed passed. GUI gates
run only in a disposable Tart macOS guest, never on the host desktop; host-built
mbx artifacts and source are hashed and copied at their compiled absolute paths.
The guest executes the recipe-derived canonical functional order sequentially.
Actual desktop Space switching and native fullscreen completion remain outside
this lane's proven evidence.

## Iteration policy

First add typed diagnostics and a failing real-app regression. Then apply the
smallest parent-owned native policy, prove lifecycle races and widen the gates.
Classify environmental activation failures separately from product failures.
Review locally; no subagent was requested.

## Blocked stop condition

Stop with failing commands, native policy/visibility observations, preserved
diff and next required input if the GUI host cannot provide activation evidence,
a real dependency is unavailable, or ownership/integration is ambiguous. Never
weaken assertions, bypass the GUI lock or modify another writer's worktree.
