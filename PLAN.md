# BBR implementation plan

> This document explains the architectural sequence and target behavior. The live, hierarchical checklist is [ROADMAP.md](ROADMAP.md); implemented facts are recorded in [ARCHITECTURE.md](ARCHITECTURE.md).

## Product target

A macOS-first, keyboard-oriented desktop browser with native WebKit content and GPUI browser chrome. The first daily-driver loop is:

1. Launch and restore the previous window.
2. Switch profile-backed spaces from the vertical sidebar.
3. Open, select, pin, reorder, move, and close tabs.
4. Navigate or switch existing content through one command bar.
5. Place two tabs in a split view.
6. Quit and restore the same organization without losing profile isolation.

The existing Swift implementation at `/Users/patwoz/dev/Personal/macos/PWBrowser` is the behavioral reference. Port behavior deliberately; do not mirror its SwiftUI/notification architecture.

## Architectural decisions

### 1. GPUI owns chrome; native views own pages

GPUI renders the window shell, sidebar, vertical tab list, command bar, split controls, settings, and internal pages. WKWebView remains a real AppKit child view layered into content slots. Do not capture web pages into GPUI textures: that would impair input, accessibility, video, text selection, energy use, and WebKit process isolation.

The first technical spike must prove that native WKWebViews can be positioned, clipped, focused, hidden, and resized in sync with GPUI layout. This is the highest-risk integration and gates feature work.

### 2. Portable core with platform adapters

Product logic must not import macOS frameworks. Platform-specific modules implement native browsing and window integration.

```text
crates/
  bbr-app/             # process entry, composition root
  bbr-domain/          # Profile, Space, Tab, Split, IDs, invariants
  bbr-browser/         # tab/space/profile orchestration and commands
  bbr-command/         # command registry, search, ranking, shortcuts
  bbr-persistence/     # SQLite repositories and schema
  bbr-ui/              # GPUI views, components, focus, themes
  bbr-web/             # platform-neutral WebEngine contracts/events
  bbr-web-macos/       # WKWebView, WKWebsiteDataStore, AppKit bridge
```

Later adapters may be `bbr-web-windows` and `bbr-web-linux`. Their engine choice is deliberately deferred.

### 3. Core data model

- `Profile`: browsing identity and isolated website data store.
- `Space`: ordered workspace attached to exactly one profile.
- `Tab`: persistent page identity attached to exactly one space.
- `WebSession`: live, disposable native engine instance for a tab.
- `Split`: ordered tree or two-pane layout referencing tabs; v1 supports two panes only.
- `WindowSession`: selected space, active tab/split, sidebar state, and geometry.

Use stable UUIDs and explicit ordering keys. Persistent state never owns native pointers. Runtime engine state is kept in a separate registry keyed by `TabId`.

### 4. Profile isolation

On macOS, map every `ProfileId` deterministically to a persistent `WKWebsiteDataStore` identifier. Tabs moved between spaces sharing a profile retain their web session. Moving across profiles destroys and recreates the WKWebView in the destination store; cookies and credentials must never cross the boundary.

### 5. Resource lifecycle

A browser that retains one live WKWebView per tab will eventually become expensive. Introduce lifecycle states early:

- `Active`: visible pane.
- `Warm`: hidden live WKWebView, immediately restorable.
- `Suspended`: persisted URL/title/history state, native view destroyed.
- `Discarded`: recoverable navigation entry after WebKit process termination.

Initial policy: active split panes stay active; recently used tabs stay warm; background tabs suspend after memory pressure or an inactivity threshold. Pinned status alone must not force a permanently live web view.

## Delivery phases

### Phase 0 — reproducible environment

Deliverables:

- `flake.nix` with stable Rust, rust-analyzer, rustfmt, clippy, build tools, libclang, nextest, and watch tooling.
- `.envrc` using the flake.
- `.gitignore` for Nix and Cargo artifacts.
- Xcode remains a host prerequisite because GPUI uses Metal and WKWebView comes from the macOS SDK.

Acceptance:

- `nix flake check` evaluates on macOS.
- `nix develop --command rustc --version` succeeds.
- `nix develop --command cargo --version` succeeds.
- `direnv allow` loads the same toolchain.

### Phase 1 — native-view integration spike

Build one GPUI window with a sidebar placeholder and one WKWebView content slot.

Prove:

- Page loads and receives mouse, keyboard, scrolling, selection, drag, context menu, and IME input.
- GPUI command bar can take focus and return it to WKWebView.
- Native view follows resize, fullscreen, scale-factor changes, occlusion, and window movement.
- GPUI overlays do not accidentally route input to the web view beneath them.
- WKWebView accessibility remains visible to VoiceOver.
- No continuous redraw or polling loop while idle.

Stop condition: if a robust child-view bridge cannot be achieved without forking GPUI, decide explicitly between a small maintained GPUI platform patch and a native AppKit host window embedding GPUI plus WKWebView siblings. Do not proceed with screenshot compositing.

### Phase 2 — portable browser core

Implement `bbr-domain`, `bbr-browser`, and an in-memory fake `WebEngine`.

Features:

- Profiles and spaces.
- Ordered tabs and pinned state.
- Active tab per space.
- Open, close, reopen, select, reorder, move, pin/unpin.
- Back/forward/reload command routing.
- Two-pane split model.
- Deterministic command actions rather than global notifications.

Acceptance:

- Unit tests cover invariants, ordering, last-tab behavior, moving across spaces/profiles, pinned close semantics, recently closed tabs, and split teardown.
- Domain tests run without macOS frameworks or a display server.

### Phase 3 — macOS WebKit adapter

Implement the native engine behind `bbr-web`.

Capabilities:

- Create/destroy/show/hide/resize WKWebViews.
- Profile-specific persistent `WKWebsiteDataStore`.
- Navigation events: URL, title, loading, progress, history availability, favicon candidate, crash/termination, errors.
- Back, forward, reload, stop, zoom, find-in-page.
- New-window and target-blank handling.
- Download and permission events modeled, even if UI follows later.
- Web process termination recovery.

Acceptance:

- Two profiles logged into the same site remain isolated.
- Moving a tab across profiles recreates it without data leakage.
- Common sites support login, media, downloads, file inputs, clipboard, WebAuthn/passkeys where WebKit allows them, and popup/new-tab flows.

### Phase 4 — vertical sidebar and spaces

Implement the first complete browser shell.

Sidebar hierarchy:

1. Window controls and navigation actions.
2. Compact location/status surface.
3. Pinned tabs for the active space.
4. Scrollable regular tabs.
5. Space switcher and new-tab action.

Behavior:

- Adjustable sidebar width.
- Collapse/hide shortcut.
- Drag reorder within a space.
- Drag tabs between spaces.
- Pointer and keyboard space switching.
- Preserve last active tab per space.
- Clear profile identity whenever changing spaces.

Acceptance:

- All actions are available by keyboard.
- Lists remain responsive with at least 500 persisted tabs and 50 live/warm tabs.
- Sidebar changes do not trigger needless web-view recreation.

### Phase 5 — command bar

One transient command surface for URL entry, web search, tab switching, history, spaces, and actions.

Ranking groups:

- Exact/open tab matches.
- Pinned tabs and current-space results.
- Other spaces, labeled with profile identity.
- History.
- Browser commands.
- Navigate/search fallback.

Behavior:

- `Cmd-L`: location mode, current URL selected.
- `Cmd-T`: new-tab mode.
- Command shortcut: action mode.
- Arrow selection, Enter activation, modified Enter variants, Escape restore previous focus.
- Debounced cancellable search; no synchronous database scan on the GPUI render path.

Acceptance:

- First result appears within 50 ms for a local corpus of 10,000 history entries and 500 tabs on the development Mac.
- URL/search parsing has table-driven tests.
- Focus returns to the originating web pane after dismissal.

### Phase 6 — split view

Start with exactly two panes. Avoid a general tiling tree until real use requires it.

Features:

- Split active tab left/right.
- Choose an existing or new tab for the second pane.
- Resizable divider.
- Independent navigation and focus.
- Close one pane without closing the underlying tab unless explicitly requested.
- Persist split composition per window and space.

Acceptance:

- Both WKWebViews remain interactive and correctly clipped during divider drag.
- Keyboard shortcuts target the focused pane.
- Closing/moving a split tab leaves no orphan native view.

### Phase 7 — persistence and recovery

Use SQLite with versioned migrations and repository interfaces. Prefer `rusqlite` unless async database access becomes demonstrably necessary; serialize writes away from the UI render path.

Persist:

- Profiles, spaces, tabs, pinned state, ordering.
- Window geometry and sidebar state.
- Last active tab per space and split layouts.
- History and recently closed tabs.
- Per-site zoom/settings.

Recovery:

- Batch/debounce transient metadata writes.
- Write structural mutations transactionally.
- Restore chrome immediately, then hydrate only visible web sessions.
- Recover from corrupt optional session state without deleting profile browsing data.

Acceptance:

- Force-quit during navigation or tab reorder restores a valid model.
- Schema migration tests use checked-in fixture databases.
- Startup does not eagerly instantiate every persisted tab.

### Phase 8 — daily-driver browser essentials

Before calling the result an alpha:

- Downloads shelf/history and destination handling.
- Find in page.
- Permission prompts: camera, microphone, location, notifications, clipboard where applicable.
- Authentication challenges and client certificates where supported.
- File upload/save panels.
- Page crash and TLS/navigation error surfaces.
- History UI and clear-data controls per profile.
- Default-browser registration and external URL opening.
- Native menus, services, window lifecycle, and updates.

Extensions, cloud sync, mobile clients, AI features, reader mode, ad blocking, and theme marketplace are post-alpha unless reprioritized.

### Phase 9 — performance and energy hardening

Measure before tuning. Track:

- Cold and warm startup.
- Time to first usable chrome and first visible page.
- Command-bar latency.
- Sidebar scroll/frame time.
- Idle CPU and wakeups.
- Memory per active/warm tab.
- Suspend/resume latency.
- Energy impact during browsing and background idle.

Rules:

- Event-driven native callbacks; no metadata polling.
- Debounce title/favicon/history persistence.
- Bound concurrent background loads.
- Respond to memory pressure by suspending least-recently-used tabs.
- Avoid animations that continuously invalidate native-view geometry.

Compare equivalent workflows against Safari, the Swift reference, Arc, and Zen. Treat Safari as the realistic WebKit energy baseline, not Chrome: WKWebView uses WebKit, not Chromium/Blink.

## Cross-platform contract

Keep these portable:

- IDs and domain models.
- Browser commands and keyboard intent.
- Persistence schema and repositories.
- Search/ranking.
- Resource lifecycle policy.
- UI structure and most GPUI components.

Keep these platform-specific:

- Native web engine and data stores.
- Child-view embedding.
- Menus, titlebar, default-browser registration, keychain/credential integration.
- Permission, download, file-panel, drag/drop, and accessibility bridges.

Do not promise identical engine behavior across operating systems. Promise consistent product behavior over the best practical native engine adapter.

## Verification strategy

- Pure unit tests for domain and URL/command parsing.
- Fake engine contract tests for orchestration.
- macOS integration harness for WKWebView events, profiles, popup handling, and process recovery.
- GPUI tests for commands, focus, selection, and sidebar state.
- Manual smoke suite against representative sites: authentication, video/audio, WebGL/WebGPU where supported, editors, conferencing, downloads/uploads, payments, and passkeys.
- Instruments measurements for Time Profiler, Allocations, Energy Log, and hangs.

## Immediate next milestone

Build only the native-view spike before the full workspace:

1. Minimal Cargo workspace with `bbr-app`, `bbr-web`, and `bbr-web-macos`.
2. GPUI window and fixed sidebar.
3. WKWebView mounted as a sibling native view in a measured content rectangle.
4. URL load plus back/forward/reload.
5. Focus and overlay test.
6. Document the integration boundary and measured idle behavior.

This milestone retires the primary architectural risk and provides a thin end-to-end product on which every later capability can grow.
