# Daily-driver browser capability inventory

Status: **canonical discovery backlog** for ordinary browser behavior that must
be deliberately accepted, deferred, or rejected before BBR claims a
feature-complete first MVP. Concrete implementation acceptance remains owned by
linked task packets; this document does not duplicate their checklists.

## Purpose

BBR already has strong architecture and several deep vertical slices. A usable
browser can still feel incomplete when common muscle-memory behavior was never
made explicit. This inventory prevents those gaps from disappearing between
architecture phases or being discovered only during manual daily use.

At every meaningful Phase-8 scheduling checkpoint, audit this complete list and
classify each row as:

- **implemented** — production behavior plus required real-app/performance
  evidence exists;
- **planned** — one canonical task packet owns it;
- **decision needed** — product behavior is unresolved and no implementation
  may invent it; or
- **post-MVP** — explicitly deferred to a linked later owner.

A row is never complete merely because WebKit offers the behavior internally.
BBR must preserve it through native embedding, focus, profiles, tabs, spaces,
splits, Portal hit-testing, deterministic E2E, and performance acceptance.

## First-MVP priority

| Capability family | Status | Canonical owner / required decision |
|---|---|---|
| Trackpad Back/Forward over web content | implemented functionally; performance acceptance deferred | [Web-content navigation gestures](../roadmap/tasks/web-content-navigation-gestures.md): exact-pane WebKit/controller history authority integrated at `542af1a`; real-app gestures 9/9 and affected functional regressions pass, while production performance scenarios remain unexecuted/unpassed under the active override |
| Child-tab Back-at-root returns to opener | implemented functionally; performance acceptance deferred | [Web-content navigation gestures](../roadmap/tasks/web-content-navigation-gestures.md): session-local `TabId` provenance and atomic close-child/activate-opener integrated at `542af1a`; functional lifecycle/split/profile evidence passes and performance acceptance remains unexecuted/unpassed |
| Link-created regular tab vs Peek vs external vs blocked | decision needed / planned seam | [Link Preview / Peek](../roadmap/tasks/link-preview-peek.md) owns typed request/disposition and Peek policy; regular child tabs provide provenance to the gesture packet |
| Foreground/background link opening | decision needed | Resolve modifier-click, middle-click, focus/selection, insertion, and opener retention before implementation; update [browser interactions](browser-interactions.md) |
| Titlebar and shortcut Back/Forward/Reload | planned | [Unified titlebar navigation](../roadmap/tasks/titlebar-navigation-controls.md); same controller/navigation state as gestures and shortcuts |
| Reopen recently closed tabs | implemented baseline | [Browser interaction contract](browser-interactions.md); future cross-space/window history scope remains an explicit decision |
| Downloads, Find, errors, authentication, permissions, file panels | planned/partially implemented | [Phase 8](../roadmap/phases/08-daily-driver.md) linked bounded packets; every surface needs cancellation/failure/restart evidence |
| Saved Pages | in delivery | [B1](../roadmap/tasks/saved-pages-portable-persistence.md) → [B2](../roadmap/tasks/saved-pages-actions-command.md) → [B3](../roadmap/tasks/saved-pages-library.md) |
| Native context menus | planned/partially implemented | BBR-owned page, tab, Favorite, and folder packets; WebKit-owned element menus remain native |
| Split-view daily use | planned | [Phase 6](../roadmap/phases/06-split-view.md): creation, focus, navigation targeting, resize, close, persistence, gesture routing |
| External URLs/default browser | planned | Phase 8 external-URL/default-browser packets |
| Per-site zoom/settings | planned | Phase 8 site-settings/zoom contract and adapter |
| Session restore and crash recovery | partially implemented / planned stability lane | Phase 7 plus error/recovery packets and the first-MVP [renderer isolation and unresponsive tabs](../roadmap/tasks/renderer-isolation-unresponsive-tabs.md) lane; a hung WebContent page must not block chrome close/switch/command behavior, and process sharing must be handled without a false one-process-per-tab guarantee |

## Navigation and tab lifecycle audit

The following product questions must each have one canonical answer and tests:

- Links opened by page content: regular tab, Peek, external, blocked, or a
  user-selected disposition.
- Foreground versus background opening for plain click, Cmd-click,
  Shift-click, middle-click, keyboard activation, and context-menu actions.
- Insertion position for a child tab and whether several children cluster after
  their opener.
- Exact opener provenance lifetime across navigation, reorder, move, suspend,
  crash, profile change, opener close, child close, and process restart.
- Closing/return behavior: ordinary Cmd-W neighbor selection remains distinct
  from explicit child Back-at-root opener return.
- Duplicate tab, move to new window, close others/right/below, reopen across
  spaces/windows, and protection of pinned tabs.
- Back/Forward parity across trackpad, titlebar, shortcuts, context menu, and
  page controls without duplicated history state.
- Navigation at boundaries, redirects, same-document history, provisional
  loads, failures, crashed content, form resubmission, and downloads/non-web
  schemes.
- Exact focused pane and focus-domain result for every split/tab mutation.

Unresolved behaviors belong in `browser-interactions.md` before a product writer
starts. Implementation-specific facts belong in architecture documents only
after verification.

## Native web-content input audit

- Trackpad scrolling, horizontal history gesture, magnification, text selection,
  drag/drop, file drop, and page-owned gestures coexist without a global input
  interceptor.
- Mouse buttons, middle-click, modifier-click, keyboard activation, context
  menus, copy/paste, selection, IME/composition, and browser shortcuts work
  while WKWebView is first responder.
- Portal/find/palette/menu/modal surfaces win hit-testing and cannot accidentally
  perform a second page gesture or click underneath.
- Fullscreen media, picture-in-picture, pointer lock, WebGL/WebGPU, editors,
  conferencing, passkeys, payment flows, and system panels retain native
  behavior where supported.

## Reliability and recovery audit

- Offline, DNS, TLS, HTTP/auth, process-crash, unsupported-scheme, blocked-popup,
  and permission failures present truthful recovery actions.
- Downloads/uploads/panels/auth/permissions can be cancelled without orphaned
  native callbacks or stale UI.
- Force quit and restart restore only committed browser state; transient opener
  and gesture state does not resurrect accidentally.
- Profile isolation is re-proven for every capability that creates or transfers
  a native session.
- Stale `WebViewId`, tab, media, download, permission, popup, Portal, and renderer-liveness events reject without mutating newer state.
- A hung WebContent renderer cannot block Sidebar, tab/space/split switching, command chrome, Cmd-W, pointer close, or keyboard close; deterministic recovery and public-API process-sharing evidence are owned by the [renderer-isolation packet](../roadmap/tasks/renderer-isolation-unresponsive-tabs.md).

## Evidence rules

Every newly implemented row requires, where applicable:

1. portable domain/controller contract tests;
2. native adapter evidence for the real platform boundary;
3. deterministic real-app E2E with real WKWebViews;
4. exact tab/session/profile/space/pane/focus and failure assertions;
5. optimized production-path performance evidence through semantic
   presentation commit; and
6. manual representative-site coverage only as a supplement, never a substitute.

The active delivery orchestrator must revisit this inventory after each Phase-8
integration and add a bounded packet for any newly concrete first-MVP gap.
Avoid one phase-wide "implement all browser features" writer; grow through
independently valuable vertical slices with explicit dependencies and owners.
