# Browser interaction contract

Status: **canonical product behavior** for tabs, spaces, splits, and browser
focus. This document describes what users should observe. Implementation facts
remain in [ARCHITECTURE.md](../../ARCHITECTURE.md), while delivery status and
acceptance evidence remain in [ROADMAP.md](../../ROADMAP.md).

## How to read this document

- **Contract** — decided behavior. Implementations and tests must follow it.
- **Planned** — product direction already present in the roadmap, but not yet a
  completed user-visible capability.
- **Open decision** — intentionally unresolved. Do not infer behavior from a
  temporary UI or implementation detail; resolve and update this document
  before shipping the affected feature.

The current portable domain/controller behavior is the initial contract unless
an open decision below explicitly calls for product review.

## Terminology

These concepts must not be conflated:

- **Selected tab** — the tab selected in a space. In single-pane mode it is the
  visible tab.
- **Focused pane** — the split pane that receives browser commands such as
  Back, Reload, or command-bar navigation.
- **Chrome focus** — keyboard focus is in BBR controls such as the sidebar or
  command bar.
- **Web-content focus** — keyboard focus is inside a native web view.
- **Warm tab** — hidden tab retaining its live native web session.
- **Activation** — selecting a tab or pane as the browser command target. It
  does not by itself define whether native keyboard focus moves to chrome or
  web content.

## Tabs

### Open a tab — Contract

- A newly opened regular tab is appended to the regular-tab region of the
  active space.
- The new tab becomes selected and active.
- The previously selected tab becomes hidden and warm; opening a tab does not
  destroy its live session.
- Opening a normal single-pane tab dissolves an existing split.
- The new tab belongs to the active space and therefore uses that space's
  profile.

### Select a tab — Contract

- Selecting a tab makes it the active tab of its space.
- In single-pane mode, the previously selected tab becomes hidden and warm.
- A warm tab reuses its existing native session.
- A suspended or discarded tab is reactivated with a newly created native
  session.
- Selecting a member of the current split preserves the split and focuses that
  pane.
- Selecting a tab outside the current split dissolves the split and returns to
  single-pane mode.
- Browser navigation commands target the selected tab in single-pane mode and
  the focused pane in split mode.

### Close a tab — Contract

Closing an inactive tab:

- removes only that tab;
- preserves the currently selected tab;
- destroys the closed tab's native session; and
- records the tab in the owning space's recently-closed stack.

Closing the active tab chooses the next active tab from the ordering that
existed immediately before removal:

1. Select the next sibling.
2. If there is no next sibling, select the previous sibling.
3. If there is no sibling, leave the space empty with no selected tab.

Examples:

```text
Before: [A] [B*] [C]
Close:      B
After:  [A] [C*]
```

```text
Before: [A] [B] [C*]
Close:          C
After:  [A] [B*]
```

```text
Before: [A*]
Close:      A
After:  empty space
```

Additional rules:

- Pinned and regular tabs follow the same neighbor-selection algorithm.
- Closing a pinned tab also removes it from the pinned region.
- Closing either member of a split dissolves the split.
- The surviving split member remains open; no unrelated tab or native session
  is orphaned.
- Closing a tab does not implicitly close its space, profile, or window.

### Reopen a closed tab — Contract

- Recently closed tabs are stored per space.
- Reopen uses last-in, first-out order.
- The tab keeps its persistent identity, URL, title, and pinned state.
- It returns as close as possible to its former index.
- If the list has changed, its restore position is clamped to a valid index.
- A pinned tab is restored inside the pinned region; a regular tab is restored
  outside it.
- The reopened tab becomes selected and active.
- It receives a fresh native session initialized with its preserved URL.
- Reopening a tab dissolves an existing split.

### Pin and unpin — Contract

- Pinned tabs form one contiguous region before regular tabs.
- Pinning moves a tab into the pinned region.
- Unpinning moves it into the regular region.
- Reordering cannot silently cross the pinned/regular boundary; pin or unpin is
  the explicit operation for crossing it.
- Pinned status does not guarantee a permanently live native session.

### Reorder — Contract

- Reordering changes only the tab's position within its current pinned or
  regular region.
- The selected tab remains selected.
- Reordering does not recreate the tab's native session.
- Invalid indices or an attempted pinned-boundary crossing reject the complete
  operation rather than partially mutating state.

### Move between spaces — Contract

- Moving a tab between spaces preserves its persistent tab identity and pinned
  status.
- A move between spaces of the same profile preserves the live native session.
- A move across profiles destroys the old native session and creates a new one
  in the destination profile's isolated data store.
- Moving a split member dissolves the source split.
- Source-space selection is repaired using the same deterministic neighborhood
  principles as tab removal.

### Empty space — Contract

- Closing the final tab leaves a valid empty space.
- The domain does not automatically create an `about:blank` replacement tab.
- The UI must provide an obvious way to open a new tab from the empty state.

## Spaces and profiles

### Active tab per space — Planned

- Each space remembers its own active tab.
- Switching away from and back to a space restores that selected tab without
  needless recreation of warm sessions.
- The UI always communicates the destination profile when switching spaces or
  moving tabs across profile boundaries.

### Profile isolation — Contract

- Every space belongs to exactly one profile.
- Tabs derive their profile from their owning space.
- Cookies, cache, credentials, and other website data must not leak between
  profiles.
- A tab crossing a profile boundary is loaded in a newly created web session
  backed by the destination profile's data store.

## Split view

### Open a split — Contract

- V1 supports exactly two panes.
- Both pane tabs become visible and own active native sessions.
- The tab that was selected before opening the split remains the focused pane.
- The second pane is visible but does not implicitly steal command focus.

### Operate a split — Contract

- Exactly one pane is the browser command target.
- Selecting either split member focuses that pane without dissolving the split.
- Back, Forward, Reload, Stop, and Navigate target only the focused pane.
- Both panes remain independently interactive.

### Close a split — Contract

Closing the split layout, rather than a tab:

- returns to single-pane mode;
- keeps both tabs open;
- keeps the focused pane selected and visible; and
- hides the other pane while retaining it as a warm session.

Closing or moving a pane's underlying tab:

- dissolves the split;
- applies normal close/move selection rules; and
- must not leave an orphan native view or ambiguous command target.

## Browser and keyboard focus

### Existing focus contract

- `Cmd-L` moves keyboard focus to browser chrome and activates the command bar.
- Escape or successful command-bar navigation can return keyboard focus to the
  active native web view.
- Browser navigation shortcuts continue to work while native web content owns
  keyboard focus.
- Pointer interaction with web content may give native keyboard focus to that
  web view.
- Pointer interaction with sidebar controls gives focus to browser chrome.
- In split mode, returning focus to web content targets the focused pane.

### Focus after tab mutations — Open decision

The selected tab after open, close, reopen, or pointer selection is already
specified above. The native keyboard-focus destination is not yet fully
specified for every input method.

Before Phase 4 tab UI ships, decide separately for:

- pointer close button;
- `Cmd-W` while web content is focused;
- `Cmd-W` while sidebar or command bar is focused;
- pointer tab selection;
- keyboard tab selection;
- opening a tab with `Cmd-T` versus a background-open action; and
- reopening a tab from a shortcut or menu.

The decision must state whether focus enters the selected page, stays in the
sidebar, or enters the command bar. Tests must assert both selected-tab state
and actual chrome/web native focus.

## Commands

### Existing and planned shortcuts

- `Cmd-L` — focus the command bar in location mode. **Contract**
- `Cmd-T` — create a real new tab and enter the new-tab flow. **Planned**
- `Cmd-W` — close the focused tab or split target. **Planned; exact focus
  aftermath remains open above.**
- Back, Forward, Reload, and Stop — target the active tab or focused pane.
  **Contract**
- Reopen closed tab — restore the most recently closed tab of the relevant
  space. **Planned UI command; domain behavior is Contract.**

Every pointer operation must eventually have a keyboard equivalent, and every
shortcut must expose an understandable menu, tooltip, or help representation.

## Lifecycle and failures

### Session lifecycle — Contract

- `Active` tabs are visible panes.
- `Warm` tabs are hidden but retain their live native sessions.
- `Suspended` tabs retain persistent tab state but no native session.
- `Discarded` tabs retain recoverable navigation identity after termination or
  explicit discard but no native session.
- Selecting a suspended or discarded tab recreates its session and activates
  it.
- A page crash cannot leave a stale tab-to-session mapping.

### Navigation and crash state — Contract

- URL and title events update only the owning tab.
- Loading and Back/Forward availability are transient session state, not
  persistent tab identity.
- Events from unknown or stale native-session IDs are reported and ignored
  without corrupting browser state.
- A crashed page remains represented by its tab and can be reactivated or
  recovered; it is not silently removed.

## Open product decisions

These items require explicit product decisions before their affected UI ships:

1. **New-tab insertion:** always append to regular tabs, insert after the
   current tab, or use opener-aware placement. Current controller behavior
   appends.
2. **Close selection history:** retain the current visual-neighbor contract or
   introduce most-recently-used/opener-aware selection. Current contract uses
   the visual neighbor and must not change accidentally.
3. **Background opening:** modifier/middle-click semantics, selection behavior,
   and insertion location.
4. **Last-tab window behavior:** an empty space is currently valid; decide
   separately whether closing the last tab of the last window ever closes the
   window.
5. **Recently closed scope:** current storage is per space; decide whether the
   user-facing command should also search across spaces or windows.
6. **Bulk close commands:** close other tabs, tabs above/below, tabs to an edge,
   and whether pinned tabs are protected.
7. **Opener relationships:** whether child tabs return focus to their opener
   when closed.
8. **Cross-space activation:** whether moving the active tab selects the source
   neighbor, follows the moved tab into the destination, or varies by command.
9. **Focus aftermath:** complete the input-method matrix in the focus section.
10. **Split close command:** distinguish “close pane layout” from “close the
    pane's underlying tab” in labels and shortcuts.

When resolving an item, move its decision into the relevant Contract section,
add automated coverage, and remove it from this list.

## Required verification

Any implementation change to this contract requires:

- a portable domain/controller regression test where applicable;
- real-app E2E for user-visible browser behavior;
- assertions for selected tab, session lifecycle, visible native view(s), and
  focused pane;
- native chrome/web focus assertions when focus behavior changes; and
- corresponding roadmap and architecture updates after evidence passes.

Do not change interaction behavior only by changing a UI component. The domain,
controller, native composition path, accessibility semantics, and tests must
agree with this contract.
