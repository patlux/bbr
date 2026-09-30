# Task packet — Several browser windows (blueprint D19, part 3)

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[menus and windows](blueprint-menus-windows.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.13, §19 (D19) ·
[PRODUCT.md](../../../PRODUCT.md).

Status: decided 2026-09-30 by Patrick (answer A to the D19 explainer
`~/Downloads/bbr-d19-mehrere-fenster.html`): several windows are allowed,
and tabs move between windows by drag and drop. Paper boards: L-13 (Dock
menu New Window), L-14 (new, close, quit), W-9 (several windows), PR-21
(new, move, close). Implementation in serial slices below.

## Outcome

BBR runs any number of browser windows over one shared browser model.
Every window is a full shell with its own active space, rail width,
selection and native Portal; spaces, profiles, tabs, folders, favorites,
history, downloads and settings are shared. ⌘N opens a window on the
current space with New Tab ready; ⇧⌘W closes a window without closing any
tab, and ⇧⌘T (or the toast's Reopen) brings it back; a tab row dragged from
one window's rail onto another window's rail or card moves the tab there.
Quit restores every window.

## Model (decisions of this packet)

1. **Shared spaces, per-window selection.** A space may be shown in several
   windows at once (board W-9: ⌘N opens on the current space). Each window
   remembers its own selected tab per space. A tab's page shows in one
   window at a time: selecting a tab another window shows moves it to this
   window, and the other window shows that space's empty state (New Tab
   ready). A Peek preview belongs to the window that opened it and sits
   in that window's root; it closes when that window loses its origin tab
   or closes.
2. **⌘N** (menu, Dock menu): a new window on the active space's profile and
   space, empty, with the ⌘T palette open (L-14 note 2).
3. **⇧⌘W** closes the window; its tabs stay in their spaces. The remaining
   key window shows a toast “Window closed · Reopen”; ⇧⌘T reopens the most
   recently closed window when nothing was closed after it, otherwise the
   last closed tab. The window keeps its geometry, active space and
   selections.
4. **Last window closed** (its close button or ⇧⌘W): BBR saves everything
   and quits, as closing its one window did before; relaunch restores
   every window. (Patrick 2026-09-30 rejected keeping BBR running with the
   window hidden.)
5. **Drag between windows:** a tab row dragged out of one window's rail
   onto another window's rail lands at that insertion slot in the other
   window's active space and shows there; onto the other window's card it
   uses the D23 split zones. A different profile recreates the session in
   the destination store, exactly as a drop on a space chip does. Dropping
   outside every window does nothing (tear-off into a new window is a
   later proposal).

## Architecture

- `bbr-browser`: one `BrowserController` owns the model and the sessions;
  it gains window contexts (`WindowKey` → active space, per-space
  selection). Operations run in the *current* window context; entering
  another window swaps its context in. Tabs visible in other windows count
  as visible for lifecycle and session policy, so no window's page is
  suspended or hidden by another window's operations.
- `bbr-web` / `bbr-web-macos`: the engine hosts several root views; a
  session lives in the root of the window that shows it and is re-parented
  (never reloaded) when it moves; geometry flips against its own root.
- `bbr-app`: one `BbrApp` entity per window, the root view of that
  window: its UI state, focus handles, Portal and pointer observer. The
  shared state (controller, organization, store, downloads, permissions,
  media, certificates, error surfaces, zoom, preferences, …) lives behind
  shared handles every window clones; state keyed by a tab, view or site
  is shared, presentation state (what is open, hovered, dragged, where it
  was drawn) is per window. A window's controller access first switches
  the controller to that window's context, so the window is explicit at
  every access. GPUI routes each window's input, timers and async results
  to its own entity; engine events are routed to the window that shows
  their view. A change of shared state re-renders the other windows.
  Rejected (2026-09-30): one `BbrApp` swapping every window's UI state in
  and out — it hides the window behind an implicit “current window” that
  every callback, timer and event would have to switch correctly.
- Persistence: the store's `windows` table already keys rows by window;
  per-window active space and selections persist in `windows.txt` beside
  the store (no schema change).
- Testkit: window-addressed requests and a window list in the snapshot.

## Scope

Serial slices:

1. Controller window contexts (model, visibility union, move-on-select),
   with fake-engine tests.
2. Multi-root engine hosting and re-parenting (fake + macOS).
3. Per-window app shells over shared state: (a) shared handles for the
   shared state and window-scoped controller access, one window, no
   behavior change, every suite green; (b) ⌘N / New Window (menu, Dock),
   ⇧⌘W, window focus, event routing, last-window quit, testkit window
   addressing.
4. Persistence and restore of every window.
5. Reopen: toast and ⇧⌘T.
6. Tab drag and drop between windows (rail slots, card zones, preview in
   the target window).

## Non-goals

- Tear-off of a tab into a new window.
- Different spaces per window being exclusive (see Model 1).
- Performance evidence (deferred with every blueprint delta).

## Parallel-lane contract

Not parallelized.

## Constraints

- One model and one session registry; no window owns a copy of shared
  state. Profile isolation unchanged.
- Web content is never sampled; overlays over web content use the one
  Portal of their window.
- No SQLite schema change; app-side state in plain files.
- Existing single-window behavior and every existing suite stay green at
  every slice.

## Evidence

### Unit and contract

- [x] Window contexts: per-window active space and selection, moving a tab
      between windows by selection, visibility and lifecycle union, closed
      and moved tabs repairing other windows' selections. *(Slice 1,
      2026-09-30: `controller/windows.rs`, the model's selected-elsewhere
      set, `WebEngine::place_web_view`; `controller_window_tests` (6).)*
- [x] Window contexts, part 2: reading a window's context without moving
      sessions (`view_window`), a window's selection, Peek previews per
      window. *(2026-09-30: `controller_window_tests` (10), full
      `bbr-browser` suite.)*
- [x] Shared state behind window-scoped handles (slice 3a, one window, no
      behavior change): `crate::shared` (`Shared`, `ControllerHandle`),
      shared parts of downloads, permissions, certificates and zoom.
      *(2026-09-30: clippy with `significant_drop_in_scrutinee` and a
      statement-level guard audit clean; all 52 functional real-app suites
      of `just e2e` green on the committed tree. `e2e-perf` failed in the
      context-menu bookmark benchmark — reproduced 5/5 on `68be5cef` and
      2/2 on `89c97cdf`, so it predates this work; its cause is open and
      tracked below — and in the startup ceiling under a host load average
      of 30–130.)*
- [ ] Multi-root engine: sessions re-parent between roots without reload;
      per-root geometry.
      *(Slice 2, 2026-09-30: `bbr-web-macos` `window_roots.rs` — further
      windows' roots with their own menu and gesture bridges,
      `place_web_view` re-parents the page and its cover, geometry and
      focus use the session's own root. Proven by `just e2e-windows` in
      slice 3.)*
- [x] Several windows (slice 3b): ⌘N / New Window (File, Dock, ⌘T action
      26), ⇧⌘W / Close Window (action 27), per-window shells over the
      shared handles (`BbrApp::assemble`, `browser_windows::Registry`),
      input in the key window, engine events in the window of their view,
      re-render on another window's change only, last-window quit.
      *(2026-09-30: `browser_windows` registry unit test;
      `controller_window_tests` (12) incl. `window_of_view` and the
      repeated-report fingerprint; `just e2e-windows` (4 tests: a new
      window on the space with New Tab ready, a page moved between windows
      without reload into the other root, ⇧⌘W keeping tabs and the last
      close quitting, menu bar and Dock, a quiet idle window and a
      redraw on another window's change) — 6/6 idle-test reruns green after
      the fingerprint fix.)*

### Real-app E2E

- [ ] `just e2e-windows`: ⌘N, two windows on one space with separate
      selections, a tab moving by selection, ⇧⌘W and Reopen, last-window
      behavior, drag between windows (rail and card), restore of every
      window after a restart. *(Slice 3b covers ⌘N, separate selections, a
      tab moving by selection, ⇧⌘W, last-window quit and the menus; Reopen,
      drag and restore follow with slices 4–6.)*

### Performance

- [ ] Deferred (blueprint deltas, Patrick 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-windows
just e2e
```

## Iteration policy

One slice at a time; each leaves `main` green with the single-window
suites unchanged. Diagnose failures as window context, session hosting,
per-window UI state or input routing.

## Blocked stop condition

If a slice needs a product decision the model above does not cover, stop
with the evidence and the question (HTML explainer in `~/Downloads`).

## Delivery ledger

```text
Outcome:         slices 1–3 implemented (several windows over shared state);
                 slices 4–6 (restore of every window, reopen toast/⇧⌘T,
                 drag between windows) open.
Commits:         89a78c75 (controller window contexts), 676e3c45 (engine
                 roots), 68be5cef (window view + previews per window),
                 4e596c99 (shared handles), slice 3b (this commit).
Functional evidence: unit/contract tests per slice (see Evidence).
Real-app E2E:    just e2e-windows (4); affected suites green on the 3b tree:
                 e2e-shell-spaces (incl. e2e_shell_menus), e2e-shutdown,
                 e2e-shell-settings, e2e-shell-command. e2e-shell-portal 6/7:
                 hardening::host_hides_on_minimize_and_restores fails at the
                 AppKit hit-test probe because a foreign window (id 53534)
                 is frontmost there; the same failure reproduces on 4e596c99
                 (host GUI state, owner not identified).
Performance evidence: deferred (Patrick). e2e-perf fails in the context-menu
                 bookmark-commit benchmark on 89c97cdf, 68be5cef and later;
                 e2e-perf had not run during the blueprint wave; cause open.
Roadmap/documentation updates: PRODUCT (several windows), DESIGN §11.13 and
                 §19 D19, ARCHITECTURE (windows, shared handles, lifecycle).
Open blockers and non-goals: restore of every window, Reopen, drag between
                 windows (slices 4–6); tear-off is a non-goal.
Worktree state:  /tmp/bbr-base is a detached comparison worktree
                 (4e596c99) created for the perf/portal comparisons.
```
