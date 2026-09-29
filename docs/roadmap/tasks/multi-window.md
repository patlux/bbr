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
   ready).
2. **⌘N** (menu, Dock menu): a new window on the active space's profile and
   space, empty, with the ⌘T palette open (L-14 note 2).
3. **⇧⌘W** closes the window; its tabs stay in their spaces. The remaining
   key window shows a toast “Window closed · Reopen”; ⇧⌘T reopens the most
   recently closed window when nothing was closed after it, otherwise the
   last closed tab. The window keeps its geometry, active space and
   selections.
4. **Last window closed:** BBR keeps running without a window (macOS
   convention); the Dock icon or ⌘N opens one. ⌘Q quits; relaunch restores
   every window.
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
- `bbr-app`: one `BbrApp` per window (UI state, focus handles, Portal)
  sharing the controller, store and the app-global subsystems; GPUI routes
  each window's events to its own entity.
- Persistence: the store's `windows` table already keys rows by window;
  per-window active space and selections persist in `windows.txt` beside
  the store (no schema change).
- Testkit: window-addressed requests and a window list in the snapshot.

## Scope

Serial slices:

1. Controller window contexts (model, visibility union, move-on-select),
   with fake-engine tests.
2. Multi-root engine hosting and re-parenting (fake + macOS).
3. Per-window app shells over shared state; ⌘N / New Window (menu, Dock),
   ⇧⌘W, window focus, last-window behavior; testkit window addressing.
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
- [ ] Multi-root engine: sessions re-parent between roots without reload;
      per-root geometry.

### Real-app E2E

- [ ] `just e2e-windows`: ⌘N, two windows on one space with separate
      selections, a tab moving by selection, ⇧⌘W and Reopen, last-window
      behavior, drag between windows (rail and card), restore of every
      window after a restart.

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
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:
```
