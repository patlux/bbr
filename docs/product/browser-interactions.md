# Browser interaction contract

Status: **canonical product behavior** for tabs, spaces, splits, and browser
focus. This document describes what users should observe. Implementation facts
remain in [ARCHITECTURE.md](../../ARCHITECTURE.md), while delivery status and
acceptance evidence remain in [ROADMAP.md](../../ROADMAP.md). Perceived response
time for these interactions is governed by the
[performance contract](performance-contract.md); correctness and latency must
both pass.

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
- **Opener tab** — the concrete existing `TabId` from whose web content a
  user-originated new-window/link request created a regular child tab.
- **Child tab** — a regular tab with session-local opener provenance. This is
  not inferred from adjacency, URL, title, or navigation history.
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

### Link-created regular child tabs — Contract

- The canonical popup/new-window policy first decides whether a user-originated
  request becomes Peek, a regular tab, external handling, or rejection.
- If it becomes a regular tab, the controller records a session-local relation
  from the child `TabId` to the exact opener `TabId` and browser window.
- The relation is identity-based and survives navigation, reload, reorder,
  same-window moves, suspension, crash, and native-session recreation. Closing
  either tab or ending the process invalidates it; v1 does not persist it.
- Cmd-T, Saved Page activation, reopen, external URLs, and tabs without a
  concrete source tab do not invent opener provenance.

### Background link-created tabs — Contract

- `Cmd` + primary click and middle click on a web link resolve to one regular
  child tab opened in the background. The opener remains selected, visible,
  and web-focused; no temporary foreground activation or focus bounce occurs.
- The first background child inserts immediately after its opener. Further
  background children from the same opener form one contiguous group after it
  in click order. Unrelated tabs retain their relative order.
- Manual reorder or move remains authoritative afterward. The child-to-opener
  relation stays identity-based and is never reconstructed from adjacency.
- Plain primary click keeps normal page navigation. Shift-modified link
  behavior remains a separate decision and is not inferred here.
- Canonical implementation packet:
  [background link tabs](../roadmap/tasks/background-link-tabs.md).

The opener lifecycle and Back-at-root implementation remain owned by
[web-content navigation gestures and opener return](../roadmap/tasks/web-content-navigation-gestures.md).
The request/disposition authority remains owned by
[Link Preview / Peek](../roadmap/tasks/link-preview-peek.md).

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

### Unresponsive renderer — Contract

- A missed bounded liveness deadline marks only the owning live session
  `unresponsive`; tab identity, ordering, profile, and session remain intact.
- Browser chrome never waits for WebContent. Cmd-W, row close, focused-list
  Backspace, neighbor selection, Cmd-T/Cmd-L, sidebar, spaces, and split focus
  use their normal authorities while the active renderer is hung.
- The active hung slot shows the closed recovery set: **Wait** keeps the same
  session under bounded monitoring; **Reload** destroys and recreates a fresh
  session for the same tab/profile; **Close tab** uses ordinary neighbor
  selection. There is no automatic reload or process kill.
- A completing probe clears `unresponsive`; a process crash takes precedence.
  Late probe/dialog/termination observations for a closed or recreated view
  are ignored.
- Chrome-initiated close destroys the WKWebView immediately and does not run
  page scripts. Public WKUIDelegate in the pinned SDK exposes no beforeunload
  callback; BBR does not use deprecated legacy WebUIDelegate or private SPI.
- JavaScript `alert`, `confirm`, and `prompt` are auto-dismissed in v1
  (confirm = false, prompt = null). They never block chrome and no dialog state
  survives close or recreation. A user-facing dialog portal is explicitly
  deferred.

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

### Drag and drop — Contract

- Pressing a tab row (not one of its controls) begins a drag of that row — the
  dragged tab, which may be the active tab but is never mutated by the grab.
  The drag is a chrome interaction: chrome focus stays in the tab list
  throughout, and Escape cancels the drag without committing.
- While the pointer stays over the tab list, the drag resolves to an insertion
  slot inside the source tab's region and renders the §10.9 tick there. A slot
  outside the region (the pinned/regular boundary crossing) is illegal: the
  tick clamps to the region edge and renders the invalid variant, and the
  release commits nothing (the drop is rejected whole). Dropping back on the
  source slot is a no-op that never emits a reorder event.
- Dragging onto a space presentation targets the space identity chip in the
  footer strip (DESIGN §11.3/§11.4) that the pointer hovers; the chip renders
  the §10.9 drag-target state with its destination profile companion.
  Releasing there moves the dragged tab into that space through the
  controller's move policy (same-profile keeps the live session; cross-profile
  recreates it once in the destination store). Dropping on the tab's own space
  is a no-op.
- Every pointer operation above has a keyboard equivalent: the Tab menu's
  Pin/Unpin row and the Pinned ghost row ↔ `Cmd-Shift-P`, drag reorder ↔
  `Cmd-Shift-↑/↓`, drag-into-space ↔ staged `Cmd-Shift-M` + Enter.

## Web-content navigation gestures — Planned contract

### History Back/Forward

- A horizontal trackpad gesture beginning over native web content targets the
  exact pane under the pointer. A gesture over the sidebar remains the existing
  space swipe; chrome, Portal, palette, find, and menus keep their own input.
- The macOS adapter normalizes raw platform direction into logical Back or
  Forward. Portable code receives no device-specific deltas.
- A committed Back/Forward gesture dispatches exactly once through the same
  navigation authority as titlebar controls and shortcuts. Before the threshold,
  on cancel, under vertical dominance, or at an unavailable boundary, it commits
  nothing.
- Back/Forward state comes from the real bound WebKit session. GPUI and the
  gesture layer own no second history-state copy.
- In split mode the gesture targets only its pane and follows the existing
  focused-pane command contract.

### Back at a child tab's root

When a committed Back gesture targets a regular child tab whose real session
cannot go back, BBR closes the child and returns to its opener only if the
initial document is settled and the exact session-local opener `TabId` still
exists in the same browser window.

- Closing uses the normal tab/session teardown and recently-closed authority.
- The exact opener is activated by identity at its current Space/Profile; a
  warm session is reused and a suspended/discarded one is recreated only under
  existing lifecycle policy.
- The opener receives native web focus. In split mode closing the child
  dissolves the split under the existing close contract before presenting the
  opener.
- If the child has older page history, ordinary Back navigation happens first;
  only a later Back at root may return.
- No opener, stale opener, provisional load, cancelled gesture, or unavailable
  history is a no-op. An unrelated root tab is never closed.
- This gesture-specific opener return does not alter ordinary Cmd-W/close-button
  neighbor selection.

Canonical packet:
[web-content navigation gestures and opener return](../roadmap/tasks/web-content-navigation-gestures.md).

## Spaces and profiles

### Active tab per space — Planned

- Each space remembers its own active tab.
- Switching away from and back to a space restores that selected tab without
  needless recreation of warm sessions.
- The UI always communicates the destination profile when switching spaces or
  moving tabs across profile boundaries.

### Create and configure a space — Contract

- Creating a Space immediately creates and activates one real Space through the
  controller's single creation authority.
- The Space receives deterministic valid default name, theme, and curated
  identity so it is usable before configuration completes.
- A production configuration surface then permits changing the exact Space's
  name and one curated emoji or BBR glyph identity. Escape or cancellation
  keeps the created Space and its valid defaults; it never rolls creation back.
- The footer presents icon-only Space identities. Accessible names include the
  Space and Profile names, so icon and color never carry identity alone.
- Canonical implementation packet:
  [Space creation and appearance UI](../roadmap/tasks/space-creation-appearance-ui.md).

### Switch a space — Contract

- Spaces are permanently presented as identity chips in the sidebar footer
  strip; there is no reveal toggle and no separate space-row region. Pointer
  activation, keyboard activation, and a committed horizontal trackpad swipe
  over the sidebar are three routes into the identical space-switch authority
  and produce identical state, events, and session lifecycle.
- A horizontal swipe moves to the previous/next space in canonical list order.
  It is a direct-manipulation gesture: while the gesture is in progress nothing
  is committed, releasing below the commit threshold restores the source space
  with no switch event, and releasing beyond it commits exactly one switch.
  Chrome tint may follow gesture progress; the native web view is never
  animated, sampled, or interpolated.
- Switching to another space of the same profile never announces a profile
  transition.
- After a successful **cross-profile** space activation, the destination
  profile appears at the top of the command field for **one interaction**:
  the profile's §12 chip plus a `type:label` "in Profile · <title>" hint, so
  the cross-profile destination is unmistakable without relying on color
  (DESIGN §11.3).
- The hint's lifetime is a deterministic user-interaction boundary, never a
  timer: the next chrome context mutation — another space activation, a tab
  mutation (open/select/close/reopen/move), or a command-bar mode entry
  (`Cmd-L`/`Cmd-T`/`Cmd-Shift-M`/New-Space staging) — clears it. A
  same-profile activation also clears any prior hint.
- The hint is a distinct transient from the staged-move `command_bar_hint`;
  the two never coexist.

### Profile isolation — Contract

- Multiple profiles and their spaces coexist inside one browser window. A
  profile is an isolated website-data/container identity, never an implicit
  separate-window boundary.
- Every space belongs to exactly one profile.
- Tabs derive their profile from their owning space.
- Cookies, cache, credentials, and other website data must not leak between
  profiles.
- Activating a space of another profile switches the active container inside
  the same window and presents that space's retained tab/session.
- A tab crossing a profile boundary is loaded in a newly created web session
  backed by the destination profile's data store.

### Favorites — Contract

- The favorites grid is global across every space and profile represented in
  the browser window; changing spaces never filters or replaces its tiles.
- A favorite is an additional presentation of one concrete persistent
  `TabId`, not a URL shortcut, copied tab, or independent web session.
- Its owning `SpaceId` and `ProfileId` are recorded/exposed with the favorite
  and remain consistent with the tab's canonical owner. If the tab moves, the
  favorite follows the same `TabId` and resolves the new space/profile.
- Activating a favorite switches to its owning space in the same window (and
  therefore to that space's profile/container), then selects that exact tab.
  It does not open, copy, or move a tab and never reuses the tab's session in a
  different profile.
- Closing the concrete tab removes its favorite tile. Toggling the tab into or
  out of Favorites never changes its space membership, order, selection, or
  live session.

### Saved Pages — Planned contract

- A Saved Page is a durable global browser-installation URL record with a
  stable `BookmarkId`; it is not a `TabId`, Favorite, pin, Space/Profile
  member, native session, cookie/data-store container, history record, or page
  snapshot.
- Pin, Favorite, and Saved Page have different lifetimes: a pin and Favorite
  die with their concrete tab; a Saved Page survives closing tabs, spaces,
  windows, and restarting the browser.
- Saving a tab, Favorite, or focused page captures only its current canonical
  navigable URL and title as an independent record. No `TabId`/session link is
  required afterward. Canonical-URL save is idempotent.
- Removing a Saved Page never closes, navigates, unpins, unfavorites, moves, or
  otherwise mutates an open tab. Closing a tab never removes its Saved Page.
- Activating a Saved Page creates exactly one fresh regular tab in the
  currently active Space, and therefore uses that Space's current Profile/data
  store. It never silently switches context, selects/reuses a matching live
  tab, or restores capture-time cookies/session state.
- Saved Page search/ranking consumes one loaded global in-memory snapshot; no
  synchronous repository query occurs from command ranking or GPUI render.
- `Cmd-D`, Palette activation, and BBR-owned Tab/Favorite/page-background menu
  rows route through the same app-owned save/remove/open authority. Menu
  surfaces retain their own source-binding/focus/Portal contracts and do not
  duplicate Saved Page behavior.
- Canonical packets: [B1 persistence](../roadmap/tasks/saved-pages-portable-persistence.md),
  [B2 actions/provider](../roadmap/tasks/saved-pages-actions-command.md), and
  [B3 Library](../roadmap/tasks/saved-pages-library.md).

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

### Focus after tab mutations — Contract

Tab mutations preserve the focus domain that initiated them unless the command
itself explicitly targets another surface:

- `Cmd-T` enters the command bar's **New Tab** mode. It creates neither a tab
  nor a native web session until Enter commits a non-empty destination.
- Escape from New Tab mode creates nothing and restores the previous command-
  bar text, mode, and chrome/web focus domain.
- Enter from New Tab mode appends and selects the new tab, creates its native
  session, navigates it, and moves focus into the new page.
- `Cmd-W` from web content closes the active tab and gives the deterministic
  neighbor web focus. From browser chrome it keeps chrome focus while updating
  the selected-tab context.
- Closing the final tab leaves the space empty and opens an empty New Tab
  command surface rather than creating `about:blank` automatically.
- Pointer and keyboard tab selection preserve their originating focus domain:
  web stays web; sidebar/command interactions stay in browser chrome.
- Reopen selects the restored tab but preserves the focus domain of the command
  that invoked it.

Split routing, pointer controls, keyboard tab traversal, and reopen commands
must assert both selected-tab state and actual chrome/web native focus as their
visible UI slices land.

## Context menus — Planned contracts

- Opening a tab, Favorite, folder, or page-background menu binds it to the
  exact source identity that invoked it. Opening, traversing, Escape, and
  outside-click dismissal do not implicitly activate, select, expand, move,
  reload, close, suspend, recreate, or change that source's profile/space/
  session.
- A Favorite menu targets the same persistent `TabId` as the corresponding tab
  row. It is not a URL shortcut or second session.
- Sidebar tab/Favorite/folder menus remain GPUI-owned. A page-background menu
  uses the existing one-per-window NativeAboveWeb Portal; link, image,
  selection, editable, media, page-JavaScript, and unclassified WebKit menus
  remain native.
- Menu actions route through typed portable commands. Capability-dependent rows
  are omitted or use a deterministic disabled reason; no render callback
  invents behavior.
- Same-profile menu moves preserve the live session. Cross-profile moves
  recreate exactly once in the destination store and retain the persistent
  tab identity.
- Keyboard operation follows native macOS expectations: Up/Down traversal,
  Right/Left submenus, Return commit, Escape dismissal. Focus returns to the
  invoking domain after dismissal unless the committed command explicitly
  targets another surface.
- Detailed action/omission matrices live in the bounded
  [Tab/Favorite](../roadmap/tasks/tab-favorite-context-menus.md),
  [Folder](../roadmap/tasks/folder-context-menu.md), and
  [Page-background](../roadmap/tasks/page-background-context-menu.md) packets.

## Titlebar navigation — Planned contract

- The unified macOS titlebar will expose one sidebar collapse/expand control
  and Back/Forward/Reload controls. The collapse control and Cmd-S are the same
  authority; the titlebar placement replaces, rather than duplicates, the
  pre-G1a status-row control.
- Back/Forward/Reload target the selected tab in single-pane mode and the
  focused pane in split mode. Disabled actions do not dispatch.
- Empty safe-titlebar space remains a native window-drag region. Fullscreen,
  multiple displays, narrow windows, and sidebar collapse do not change
  navigation target or tab/session identity.
- Canonical packet: [titlebar navigation controls](../roadmap/tasks/titlebar-navigation-controls.md).

## Media sessions — Planned, capability-first

- Media observation and commands exist only for an actual session reported by
  the portable [media capability contract](../roadmap/tasks/media-session-capability-contract.md).
  Public WebKit APIs or an isolated content-world bridge may supply facts; SPI
  and HTML heuristics alone may not.
- Every command carries and revalidates opaque media identity plus owning tab/
  space/profile/session generation. A stale command rejects and never controls
  a newer or different tab.
- Play, Pause, Seek, Previous, Next, and Mute are independent capabilities.
  Unsupported commands are absent/disabled; unknown/live duration has no fake
  total or scrubber.
- Ordinary media commands do not select or focus the owning tab. Explicit
  source activation selects the exact persistent TabId through its owning
  space/profile and follows the normal cross-profile identity contract.
- Dismissing the sidebar card hides that media identity until it changes; it
  does not pause or stop playback. End, navigation, reload, crash, close, or
  session replacement invalidates the card/commands deterministically.
- Canonical UI packet: [sidebar media control](../roadmap/tasks/sidebar-media-control.md).

## Commands

### Existing and planned shortcuts

- `Cmd-L` — focus the command bar in location mode. **Contract**
- `Cmd-T` — enter New Tab command-bar mode without creating a tab or web
  session. Enter commits the destination; Escape cancels it. **Contract**
- `Cmd-W` — close the active tab or focused split target. **Contract; the
  single-pane action exists, while split routing and exact focus aftermath
  remain open above.**
- `Cmd-Shift-P` — pin or unpin the active tab (the pinned/regular boundary
  toggle). The selected tab stays selected and its live session is preserved;
  the invoking focus domain is untouched. **Contract** (the Tab menu's
  Pin/Unpin row and the Pinned ghost-row drag offer are the pointer
  equivalents; rows carry no pin control).
- `Cmd-Shift-↑` / `Cmd-Shift-↓` — move the active tab one step within its
  current pinned or regular region. A step that would cross the boundary is
  rejected whole (no partial mutation); the session and selection stay
  untouched. **Contract**
- `Cmd-Shift-M` — stage a move of the active tab into a destination space
  typed by name; nothing moves until Enter commits it, Escape cancels it and
  restores the previous command-bar state and focus domain. **Contract**
- Back, Forward, Reload, and Stop — target the active tab or focused pane.
  **Contract**
- `Cmd-Shift-T` — reopen the most recently closed tab of the active space.
  **Contract** The restored tab keeps its identity/URL/title, receives a fresh
  native session, becomes active, and preserves the focus domain of the
  invoking command (chrome stays chrome, web stays web).
- `Cmd-D` — toggle the focused/active page's Saved Page state through B2.
  **Planned contract.** Saving is canonical-URL-idempotent and creates no tab;
  removing never mutates the current tab. Missing/unsupported URLs reject
  truthfully.
- Tab-list keyboard: `↑`/`↓` move selection through the visible rows, `⌫`
  closes the active tab, Enter activates. **Contract** (rows stay in browser
  chrome while traversed).
- Sidebar-divider keyboard: with the divider focused (pointer entry or
  keyboard focus on the resize hairline), `←`/`→` decrease/increase the
  sidebar width by the 16 pt step and `0` resets to the §8 default. The
  divider is part of the browser-chrome focus domain, and pointer drag and
  the keyboard equivalents mutate the same requested width, so parity holds
  by construction. **Contract**

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

1. **Non-link new-tab insertion:** Cmd-T and other source-less regular tabs
   currently append. Background link children are separately resolved above as
   opener-relative grouped insertion and do not silently change Cmd-T.
2. **Close selection history:** retain the current visual-neighbor contract or
   introduce most-recently-used/opener-aware selection. Current contract uses
   the visual neighbor and must not change accidentally.
3. ~~**Background opening**~~ **Resolved:** Cmd-primary and middle click open
   one grouped background child immediately after the opener while preserving
   opener selection and web focus. Shift-modified behavior remains open.
4. **Last-tab window behavior:** an empty space is currently valid; decide
   separately whether closing the last tab of the last window ever closes the
   window.
5. **Recently closed scope:** current storage is per space; decide whether the
   user-facing command should also search across spaces or windows.
6. **Bulk close commands:** close other tabs, tabs above/below, tabs to an edge,
   and whether pinned tabs are protected.
7. ~~**Opener relationships**~~ **Resolved for explicit Back-at-root gesture
   return:** a regular child tab records session-local `TabId` provenance and a
   committed Back gesture at its settled history root closes it atomically and
   activates the surviving opener. Ordinary close remains visual-neighbor based;
   insertion/background policy remains open in items 1 and 3.
8. **Cross-space activation:** whether moving the active tab selects the source
   neighbor, follows the moved tab into the destination, or varies by command.
9. **Split close command:** distinguish “close pane layout” from “close the
    pane's underlying tab” in labels and shortcuts.

When resolving an item, move its decision into the relevant Contract section,
add automated coverage, and remove it from this list.

## Required verification

Any implementation change to this contract requires:

- a portable domain/controller regression test where applicable;
- real-app E2E for user-visible browser behavior;
- assertions for selected tab, session lifecycle, visible native view(s), and
  focused pane;
- native chrome/web focus assertions when focus behavior changes;
- a real-app performance scenario for the affected visible transition, measured
  through semantic presentation commit rather than command receipt; and
- corresponding roadmap and architecture updates after evidence passes.

Do not change interaction behavior only by changing a UI component. The domain,
controller, native composition path, accessibility semantics, and tests must
agree with this contract.
