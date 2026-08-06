# BBR — Architecture (current implementation)

Status: **describes the implemented workspace through the Phase-4 shell slices**: native GPUI +
WKWebView embedding, portable browser orchestration, controller-backed real
tabs, profile-isolated multi-session WebKit, the adjustable/collapsible
sidebar, tab organization, the dense-500 list, and the pointer-state +
internal-semantic-model slice. Verified against the actual
GPUI crate (`gpui 0.2.2`), the `objc2-web-kit 0.3.2` bindings, the Swift
reference at `PWBrowser`, and green `just verify` + sequential `just e2e` runs.

---

## 1. Decisions locked in this phase

| Decision | Choice | Why |
|---|---|---|
| Workspace shape | Cargo workspace, nine crates: the seven implemented product/test crates plus dependency-free portable landing zones `bbr-command` and `bbr-persistence` | The implemented shell remains unchanged while Phase-5/7 portable cores can evolve in isolated worktrees behind explicit integration contracts |
| Chrome vs. pages | GPUI owns chrome; `WKWebView` is a native AppKit child view | Product principle; never texture-capture web content |
| Embedding strategy | WKWebView is a **subview of GPUI's root `NSView`**, sized by a bounds-reporting GPUI element | Keeps the responder chain, key equivalents, accessibility, video/IME/process isolation intact |
| Native-view slot | A custom public GPUI `Element` (`NativeWebViewSlot`) publishes its own window-relative bounds each frame | Uses only public GPUI APIs; no GPUI fork. **Fulfils the "no forking GPUI" stop-condition path** |
| Focus choreography | Explicit `makeFirstResponder` calls driven from the host bridge, plus a transparent `ChromeResponder` overlay | GPUI's `Window::focus` only mutates GPUI's internal focus tree; AppKit refuses `makeFirstResponder(gpui_view)` because `GPUIView` does not accept first responder (see §5) |
| Profile isolation | `nonPersistentDataStore` per profile in test mode; identifier-backed persistent store in production | E2E hermeticity; no WebKit container leakage between tests |
| Main thread | All `bbr-web-macos` calls on the main thread (enforced + debug-asserted) | `WKWebView` and the GPUI mac platform are main-thread-only |
| Engine adapter | `bbr-web` defines the platform-neutral `WebEngine` contract; `bbr-web-macos` implements it; a `NullEngine` covers non-macOS targets | Cross-platform requirement; portable crates never import macOS frameworks |

---

## 2. Workspace & crate layout

```
bbr/
  Cargo.toml            # [workspace] members = the seven crates below; resolver = "2"
  flake.nix             # dev shell for aarch64/x86_64 darwin + linux
  justfile              # bounded local verification: fmt/check/test/clippy/verify/e2e-smoke
  crates/
    bbr-web/            # lib: platform-neutral contracts (no objc, no gpui, no unsafe)
    bbr-web-macos/      # lib: WKWebView + AppKit bridge (target_os = "macos" only)
    bbr-app/            # bin `bbr`: composition root, GPUI window, E2E gateway glue
    bbr-domain/         # lib: portable browser core model (Phase 2 slice)
    bbr-browser/        # lib: portable controller: tabs ↔ disposable web sessions
    bbr-testkit/        # lib: typed E2E protocol, gateway server + client (unix sockets)
    bbr-fixtures/       # lib+bin: deterministic loopback HTTP fixture server
    bbr-command/        # portable Phase-5 landing zone; behavior not implemented yet
    bbr-persistence/    # portable Phase-7 landing zone; behavior not implemented yet
```

### Dependency direction (strict, acyclic)

```text
bbr-web-macos  ──uses──▶  bbr-web        (implement the WebEngine trait)
bbr-domain     ──uses──▶  bbr-web        (ProfileId for profile identity)
bbr-browser    ──uses──▶  bbr-domain, bbr-web (orchestration + engine contract)
bbr-testkit    ──uses──▶  bbr-web        (serde feature: WebEvent/WebAction/ids)
bbr-app        ──uses──▶  bbr-web, bbr-web-macos (macOS), bbr-testkit (testkit
                          feature), bbr-fixtures (dev-dep)
bbr-fixtures   ──uses──▶  (std only)
```

- `bbr-web` **must not** depend on `gpui`, `objc2*`, `raw-window-handle`, or any
  AppKit/WebKit symbol. It is a pure contract crate — the only crate portable
  crates (`bbr-domain`, `bbr-browser`, and later `bbr-ui`) may depend on. It
  carries `#![forbid(unsafe_code)]`.
- `bbr-web-macos` is gated `#[cfg(target_os = "macos")]` at the crate and module
  level (`bbr-web-macos/src/lib.rs` is empty elsewhere), so the workspace still
  evaluates on Linux. `bbr-app` uses a portable `NullEngine` on non-macOS
  targets and pulls in `gpui` with the `wayland` feature there.
- `bbr-app` is the only place allowed to hold the `gpui::Application` and to
  stitch the native engine to the GPUI window. Exactly one `unsafe` call site
  lives in `bbr-app`: `build_engine(ns_view, …)` (macOS), fed by a raw-window-
  handle `NSView*` extraction.
- `bbr-domain` is fully portable: `#![forbid(unsafe_code)]`, no GPUI, no macOS
  frameworks, no SQLite, no display server. Runtime engine state (live
  WKWebViews) is deliberately kept outside it, keyed by `bbr_web::WebViewId`.

### Version pins (workspace `[workspace.dependencies]`)

| Crate | Version | Notes |
|---|---|---|
| `gpui` | `0.2.2` | `default-features = false`, `features = ["font-kit"]` |
| `objc2` | `0.6.4` | satisfies `objc2-web-kit`'s `>=0.6.2,<0.8.0` |
| `objc2-web-kit` | `0.3.2` | `WKWebView`, `WKWebsiteDataStore`, `WKNavigationDelegate`, `WKUIDelegate`, … |
| `objc2-app-kit` | `0.3.2` | `NSView`, `NSWindow`, `NSResponder`, `NSEvent` |
| `objc2-foundation` | `0.3.2` | `NSString`, `NSURL`, `NSURLRequest`, `NSError`, `NSUUID`, KVO |
| `block2` | `0.6.2` | available for delegate `decisionHandler` blocks (not yet exercised) |
| `raw-window-handle` | `0.6` | must match GPUI 0.2.2's `raw-window-handle 0.6` |
| `serde` / `serde_json` | `1` | protocol framing (feature-gated in `bbr-web`) |
| `smol` | `2` | channels for the main-loop event/command pump |

Toolchain: workspace MSRV `1.85`; the dev shell provides `rustc 1.97.1` via
`rust-overlay`. `Xcode 16.4` is the host prerequisite (Metal + WebKit SDK) and
is encoded in the justfile as `BBR_XCODE_DEVELOPER_DIR`.

---

## 3. Native-view embedding (implemented)

### What GPUI 0.2.2 actually provides (verified in source)

- `gpui::Window` implements `raw_window_handle::HasWindowHandle`; on macOS it
  returns an `AppKitWindowHandle { ns_view: NonNull<c_void> }` pointing at the
  GPUI root view (`GPUIView`). The window's `contentView` is the standard AppKit
  content view; `GPUIView` is added as its subview, layer-backed, with a Metal
  layer.
- GPUI has **no public native-child-view API**, so we embed via raw window
  handle + AppKit `addSubview:`.
- `Window::focus(handle)` only changes GPUI's internal focus tree; it does
  **not** call `makeFirstResponder`. Confirmed by inspection of `window.focus()`.

### The implemented strategy (`bbr-web-macos/src/host.rs`, `webview.rs`; `bbr-app/src/slot.rs`)

1. **Capture the GPUI root `NSView` pointer once** at startup, in the window
   open callback (main thread), from `window.window_handle()` via
   `raw-window-handle`. `bbr-app::main::extract_ns_view` returns the
   `AppKitWindowHandle.ns_view` pointer; `build_engine` hands it to
   `WebEngineWk::attach`.
2. **Construct the `WKWebView`** with `initWithFrame_configuration` and
   `addSubview:` it onto the retained root view. Because it is a **subview**:
   - AppKit hit-testing sends mouse/scroll/gesture events to the WKWebView
     where it is frontmost; GPUI's `GPUIView` methods are simply not invoked for
     those regions.
   - The responder chain for key equivalents runs firstResponder → superviews →
     window → menu, so `Cmd-L`/`Cmd-T`/… still reach GPUI's
     `performKeyEquivalent:` while the web view has keyboard focus.
   - Accessibility, video, IME, selection, and process isolation are preserved
     because we never screenshot/composite.
3. **Position & resize via a bounds-reporting element.** `NativeWebViewSlot`
   (`bbr-app/src/slot.rs`), a custom public GPUI `Element`, fills the content
   area right of the sidebar. In `prepaint` GPUI hands it `Bounds<Pixels>`
   relative to the window; the element forwards the rect as a `bbr_web::WebRect`
   to the engine's `attach_slot`. The bridge converts it to an AppKit frame via
   the **y-flip** (`bbr_web::flip_rect`, unit-tested) using the root view's
   actual height (AppKit origin bottom-left, GPUI origin top-left).
   - The slot has a stable `ElementId` (`"native-web-slot"`).
   - `attach_slot` writes the frame only when the rect changed, so an idle
     window produces no `setFrame:` traffic (no polling loop).
4. **No polling loop.** GPUI is display-link driven; the slot publishes bounds
   only on change.

### Fullscreen / scale / occlusion / resize

- Window resize → GPUI layout re-runs → slot bounds change → bridge updates the
  frame. Verified by the E2E smoke resize step (`SetWindowSize` → snapshot shows
  new geometry; the native view re-frames next frame).
- Scale-factor change / fullscreen / occlusion: WKWebView handles these
  natively; nothing extra beyond the frame update.

---

## 4. Unsafe-code boundaries

Every `objc2`/WebKit call is `unsafe`. It is concentrated **inside
`bbr-web-macos`** plus the single engine-construction call site in `bbr-app`
(section 2); it never crosses into `bbr-web`, `bbr-domain`, `bbr-testkit`, or
`bbr-fixtures`.

| Boundary | Where | Rules |
|---|---|---|
| Raw root-view pointer | `bbr-web-macos::host::HostBridge` | Captured once on the main thread from `window.window_handle()`; retained (`Retained<NSView>`). Never dereferenced off-main; `debug_assert!(MainThreadMarker::new().is_some())` on every entry |
| `WKWebView` object graph | `bbr-web-macos::webview` | `Retained<WKWebView>`; created/destroyed on main thread only |
| Delegate protocol objects | `bbr-web-macos::delegate` | `define_class!` implementing `WKNavigationDelegate` + `WKUIDelegate`; kept alive for the view's lifetime (WebKit does not retain delegates). KVO `title` observation is paired `observe_title`/`unobserve_title` before view dealloc (avoids `NSKVODeallocateException`) |
| `raw-window-handle` | `bbr-app::main` | `ns_view` → `objc2` `id` cast is sound only because the caller controls lifecycle and thread; gated `#[cfg(target_os = "macos")]` |

**Safety invariant:** `bbr-web` and `bbr-domain` are `#![forbid(unsafe_code)]`.
`bbr-app`'s only unsafe is the engine build on macOS; `bbr-testkit` and
`bbr-fixtures` are pure std.

---

## 5. Focus hand-off (implemented, incl. the responder overlay)

The implementation goes one level deeper than the original plan: `GPUIView`
does **not** override `acceptsFirstResponder`, so `makeFirstResponder(gpui_view)`
is silently refused by AppKit. The fix is a transparent `NSView` subclass,
`ChromeResponder` (`host.rs`):

- Its only behaviour is `acceptsFirstResponder → YES`. Every other
  `NSResponder` method uses the default implementation, which forwards up the
  responder chain to its `nextResponder` — the GPUI root view. Key events
  therefore arrive at GPUI exactly as if `GPUIView` were the first responder,
  with no GPUI fork.
- `hitTest:` returns `nil`, so the overlay never steals clicks from the
  WKWebView or from GPUI chrome.
- It is pinned to the root view bounds with width/height autoresizing masks.

Focus movements (both AppKit **and** GPUI focus tree are kept in sync):

| Action | Effect |
|---|---|
| Click into the WKWebView | AppKit auto-focuses the web view; GPUI blurs its chrome |
| `Cmd-L` (FocusCommandBar) | Enter `Location` mode, focus `chrome_handle`, and call `engine.focus_host_chrome()` → `makeFirstResponder(ChromeResponder)` |
| `Cmd-T` | Enter `NewTab` mode with empty text; no controller tab or WKWebView is created before Enter |
| Escape from `NewTab` | Restore prior command-bar text/mode and chrome/web focus domain without creating a tab |
| Enter from `NewTab` | Open and select the destination tab, create its WKWebView, then focus the page |
| `Cmd-W` | Close the active tab; preserve chrome versus web focus, or expose empty `NewTab` mode after the final tab closes |
| Click in the sidebar | Enter `Inactive` command-bar mode, focus `chrome_handle`, and call `focus_host_chrome()` |

GPUI `Action`s are bound with `cx.bind_keys` (`cmd-l`, `escape`, `cmd-[`,
`cmd-]`, `cmd-r`, `cmd-b`, `cmd-t`). They run in the bubble phase and are
dispatched through `performKeyEquivalent:`, so they fire even while the
WKWebView is the AppKit first responder. Because handlers are invoked during
`Window::dispatch_keystroke` (window borrowed), the actual `win.update` is
deferred to the effect flush (`cx.defer`) — the same pattern GPUI's own
`dispatch_action` uses.

Under the `testkit` feature, a **focus sink** reports the actual chrome-focus
state (queried from GPUI's focus tree after each mutation) to the gateway as
`ChromeFocusChanged`, so E2E assertions match the snapshot's `chrome_focused`.

---

## 6. `bbr-web` contract (portable, no frameworks)

```rust
// bbr-web/src/lib.rs  (pure types; no objc, no gpui, no unsafe)
pub struct ProfileId(pub u64);   // maps 1:1 to a WKWebsiteDataStore identity
pub struct WebViewId(pub u64);   // live web view / content slot
pub struct WebRect { x, y, width, height: f32 }   // host layout: top-left origin, points

pub fn flip_rect(rect: WebRect, container_height: f32) -> WebRect;  // AppKit y-flip
pub fn normalize_url(input: &str) -> String;                        // scheme defaulting (https://)

pub enum WebAction { Load { url: String }, Back, Forward, Reload, Stop }
pub enum WebEvent {
    UrlChanged(WebViewId, String), TitleChanged(WebViewId, String),
    LoadingChanged(WebViewId, bool), CanGoBackChanged(WebViewId, bool),
    CanGoForwardChanged(WebViewId, bool),
    PageCrashed(WebViewId), NavigationFailed(WebViewId, String),
}

pub trait WebEngine {            // object-safe; main-thread-only
    fn attach_slot(&mut self, rect: WebRect, content_height: f32);
    fn create_web_view(&mut self, id: WebViewId, profile: ProfileId, initial_url: &str);
    fn destroy_web_view(&mut self, id: WebViewId);
    fn show(&mut self, id: WebViewId, visible: bool);
    fn action(&mut self, id: WebViewId, action: WebAction);
    fn set_events(&mut self, sink: Box<dyn FnMut(WebEvent)>);
    fn focus_native_web(&mut self, id: WebViewId);
    fn focus_host_chrome(&mut self);
    fn live_session_count(&self) -> usize;
}
```

Note the trait shape differs from the early blueprint: `attach_slot` carries
`content_height` (so the flip never drifts from the true container height and
can be unit-tested without macOS), and zoom/warm/suspend/lifecycle methods are
deferred to later phases — the trait remains deliberately narrow. The event
sink is deliberately *not* `Send` (the concrete engine and its delegates are
`MainThreadOnly`). An object-safety test proves the trait works behind
`Box<dyn WebEngine>`.

`bbr-web` also exposes `serde` derives on the ids/events/actions behind the
`serde` feature (enabled by `bbr-testkit`) so the wire format never leaks
framework types.

---

## 7. `bbr-web-macos` organization

```
src/
  lib.rs            # re-exports WebEngineWk; modules cfg(target_os="macos")-gated
  host.rs           # HostBridge: retained root view, y-flip frame math,
                    #   first-responder hand-off, ChromeResponder overlay class
  webview.rs        # WebEngineWk: WebViewId→NativeWebSession registry, profile stores, actions
  delegate.rs       # define_class! WKNavigationDelegate + WKUIDelegate + KVO title
```

- **Native sessions:** `HashMap<WebViewId, NativeWebSession>`; each entry owns
  the WKWebView, its retained delegate, profile identity, and visibility. Every
  create/show/action/focus/destroy operation resolves the requested id first;
  teardown removes KVO before detaching and releasing the view.
- **Profile stores:** `dataStoreForIdentifier` uses a stable UUID derived from
  `ProfileId`, or `nonPersistentDataStore` when `EngineOptions.non_persistent`
  is set — the app enables non-persistent mode whenever a `--bbr-test-token`
  is present (E2E hermeticity). Stores are cached per profile and outlive views.
- **WebView creation:** `initWithFrame_configuration`, per-view delegate
  (`WKNavigationDelegate` + `WKUIDelegate`), KVO `title` observation
  (`observe_title` registers with `Initial` so the shell learns startup state
  even mid-load; the callback re-emits full navigation state when title lands).
- **Delegates:** `didStartProvisionalNavigation` (loading=true + state),
  `didCommitNavigation` (state), `didFinishNavigation` (loading=false + state),
  `didFailProvisionalNavigation`/`didFailNavigation` (`NavigationFailed` with the
  localized description), `webViewWebContentProcessDidTerminate`
  (`PageCrashed`). `createWebViewWithConfiguration…` returns `None` — popups are
  suppressed in Phase 1 for deterministic tests.
- **Actions:** `loadRequest`, `goBack`/`goForward` (guarded by `canGoBack`/
  `canGoForward`), `reload`, `stopLoading` + explicit state re-emit (`Stop`).
- **Main thread:** all entry points re-assert with `MainThreadMarker`; the
  `MainThreadOnly` thread-kind is enforced at bind time.

---

## 8. `bbr-app` composition root

`crates/bbr-app/src/main.rs` (package/binary name `bbr`)

```text
1. Parse CLI flags (--bbr-*):
     --bbr-initial-url    initial load (default "about:blank")
     --bbr-profile-dir    per-test profile dir (default temp/pid)
     --bbr-test-token     secret; presence ALSO enables non-persistent stores
                          and (with the testkit feature) starts the gateway
     --bbr-hidden         do not focus/activate the window (E2E)
     --bbr-ids=sequential|random   IdGen mode (SequentialIdGen/RandomIdGen)
     --bbr-clock=system|fake       parsed; NO-OP in Phase 1 (no time-based logic)
     --bbr-window-size=WxH         initial window size (default 1080x700)
2. Build the id generator and inject it as the controller's WebViewId allocator.
3. gpui::Application::new().run: open one window (centered, options from flags).
4. In the open_window callback: capture the GPUI root NSView, build the engine,
   create `BrowserController(engine.clone(), allocator)`, then create BbrApp.
5. Open the initial controller-backed tab after the window exists.
6. install_actions: bind Cmd-L/Escape/Cmd-[ /Cmd-]/Cmd-R/Cmd-B, Cmd-T staging, the focus-preserving Cmd-W, and the tab-organization commands (Cmd-Shift-P pin toggle, Cmd-Shift-↑/↓ reorder, Cmd-Shift-M staged move).
7. Under testkit: start the gateway, pump engine events into the controller,
   and broadcast AppReady, focus, WebEvents, and semantic BrowserChanged events.
```

The chrome (`crates/bbr-app/src/app.rs` + `app/render.rs` + the split
`app/sidebar.rs`/`app/sidebar_view.rs`/`app/spaces.rs`/`app/tab.rs`/
`app/interaction.rs` + the `shell/` presentation modules) renders the
Chassis-tokenized sidebar (`crates/bbr-app/src/theme.rs` tokens, `palette.rs`
ramps, `components.rs` reusable typed components — never local magic values):
brand block, command bar (mirrors the live URL; inline cursor; accepts typing,
backspace, enter), the controller-backed tab surface, a status line
(`shell::status_line`, incl. "⚠ Web content process terminated"), and the
sidebar toggle. `CommandBarMode` separates `Inactive`, current-tab
`Location`, and staged `NewTab` intent. The content area is the
`NativeWebViewSlot`, the empty-space pane (§11.8), or the crash panel (§10.7).
The sidebar rail itself is the adjustable, collapsible `SidebarGeometry`
(`crates/bbr-app/src/sidebar.rs`): DESIGN §5/§11.4 min/default/max widths
(200/260/360 pt) with a 64 pt native-content floor, narrow-window clamping
that never yields zero or negative native geometry, collapse memory that
expand restores, and a 4 pt divider surface (`bbr-sidebar-divider`) carrying
pointer drag and the focused-divider keyboard equivalents (`←`/`→`/`0`).
`crates/bbr-app/src/shell.rs` was split into `shell/` (presentation, tab,
space, snapshot, semantics modules) so the touched presentation code stays
below the ~500-line budget.

The tab surface (DESIGN.md §11.1/§11.2) renders the active space's pinned
region above the regular region, both derived from the controller snapshot
(`shell.rs` owns presentation-only derivation: title/URL fallbacks §11.9,
deterministic monogram favicons §7.2, per-row visual states §10). Pointer
selection/close run the production row listeners (`select_row_via_pointer` /
`close_row_via_pointer`) which keep the chrome focus domain (the chrome domain
is the union of the command-bar and tab-list GPUI handles). Keyboard
traversal (`↑`/`↓`, `⌫`) lives on the focused tab list; `Cmd-T`/Enter opens,
`Cmd-W` closes, and `Cmd-Shift-T` reopens (fresh session, focus domain
preserved). A crashed tab keeps its identity and its §10.7 error state until a
fresh session loads; closing the final tab opens the empty New Tab surface.

Pointer-state polish (§10.2/§10.3) is wired through the production listeners:
the row's `on_hover` handler drives `set_row_hover`/`clear_row_hover` and the
mouse-down/up handlers drive `set_row_pressed`/`clear_row_pressed` (with the
root's mouse-up/up-out/hover listeners clearing both when the pointer leaves
chrome), and `row_visuals` composes the §10 state so the more specific state
wins (error > loading > pressed > selected > hover > default). The §15 typed
semantic model (`shell/semantics`, testkit-gated) derives roles, names,
values, enabled/selected/focused state, and the parent/child hierarchy from
the same row/space visuals the render consumes (bounded by the dense window,
so a dense collection never exposes rows it did not render). **Scope honesty:**
this is the internal contract shared with the testkit snapshot and the future
platform Accessibility bridge — GPUI 0.2.2 implements no platform
accessibility layer (verified in its source: no NSAccessibility, no
accesskit), so the nodes are not exposed to macOS Accessibility / VoiceOver
today; the platform bridge and the packaged-app VoiceOver/Appium/manual
acceptance are Phase 9.

Tab organization (the Phase-4 drag/reorder slice) is routed through the same
controller mutations as every other input path. `Cmd-Shift-P` toggles the
active tab's pin state (the row's `bbr-tab-pin-<n>` control is the pointer
half), `Cmd-Shift-↑/↓` step the active tab within its region (a boundary
crossing is refused before the controller rejects it — the pinned-first
invariant stays domain-owned), and `Cmd-Shift-M` stages a destination space
name in the command bar (`CommandBarMode::MoveTab`; exact-title match then
prefix; Enter commits through `move_active_tab_to_space`, Escape cancels
without moving). The pointer drag is a typed state machine
(`app/drag.rs`, pure resolution in `app/drag_target.rs`): a row press begins
the drag, mouse-move resolves an insertion slot inside the source region or a
visible space row (space-surface offsets mirror `shell::space_groups`), and
mouse-up commits region-bounded reorder or move-between-spaces through the
controller. Resolution is anchored at the press: the recorded press offset
within the source row makes the space-surface hit exact (`press_in_row`),
and callers that cannot observe element geometry (GPUI 0.2.2 exposes none)
anchor at the row center. Boundary crossings clamp the tick to the region
edge and report invalid so the drop is rejected whole; the §10.9 tick
(accent, muted when invalid) and `drag-target` space rows render from that
state, and Escape cancels. Same-profile moves keep the WebViewId and warm
session; cross-profile moves recreate exactly once in the destination
profile's store (`recreated` reported on the wire). The pinned region
boundary and drag state are part of the typed E2E snapshot.

The space surface (DESIGN.md §11.3/§12) renders every space grouped under its
profile with a per-row §12 chip (round-robin hue + monogram) and a hairline
between profile groups, plus a New Space row at the foot. Pointer activation
runs the production row listener (`select_space_via_pointer`), which keeps
the chrome focus domain and routes through the controller's `activate_space`;
the space list owns a GPUI focus handle with `↑`/`↓`/Enter traversal, and
`Cmd-Shift-N` / the New Space row stage a name in the command bar (Escape
cancels, Enter creates the space in the active profile and activates it).
Activation restores each space's own active tab while warm sessions stay
warm — including across profile boundaries, where a view-only switch never
recreates a session (recreation stays reserved for tabs that actually move
across profiles, per `ensure_session`'s profile check).

Cross-context identity (DESIGN.md §11.3) is a transient in `BbrApp`
(`command_profile_hint: Option<String>`): `activate_space` compares the
source and destination profile and, only on a cross-profile activation, sets
the hint to `"in Profile · <title>"` (via `shell::profile_line`) with the
destination profile's §12 chip identity derived from the controller snapshot
(`shell::space::active_profile_chip`). Same-profile activation clears any
prior hint. The hint renders at the leading edge of the production command
field (`sidebar_view::command_bar`) under the stable
`bbr-command-profile-hint` id and is exposed in the typed snapshot
(`command_profile_hint`) and the §15 semantic tree as a `Chip` node whose
child carries the `bbr-profile-chip-<n>` monogram. Its lifetime is a
deterministic user-interaction boundary, never a timer: the next chrome
context mutation — another space activation (which re-evaluates it), a tab
mutation (`open`/`select`/`close`/`reopen`/`move` clear it), or a
command-bar mode entry (`Cmd-L`/`Cmd-T`/`Cmd-Shift-M`/New-Space staging
clear it) — ends the one-interaction display via the single
`clear_command_profile_hint` seam. It is a distinct transient from the
staged-`MoveTab` `command_bar_hint`; the two never coexist (staging a move
clears the space-switch transient). The presentation probe records and
asserts the hint state (`observe_profile_hint`/`expect_profile_hint`) so the
gate-enforced `space_switch_cross_profile` benchmark rejects any
presentation that misses or invents it (fast-but-wrong guard).

`BrowserController` is the sole owner of product tab/navigation state. The
GPUI root renders a controller snapshot; `shell.rs` contains only formatting
and the app-local typed E2E snapshot adapter, so tab switching cannot leave a
second navigation-state copy stale.

---

## 9. `bbr-domain` — portable core model (PLAN.md Phase 2 slice)

`crates/bbr-domain` implements the owning persistent model: `Profile`,
`Space`, `Tab`, `ClosedTab`, and the two-pane `Split`, plus the `Browser`
container that enforces invariants through every mutation.

Ownership rules:

- `Profile` is a browsing identity; it reuses `bbr_web::ProfileId` (the same
  type the engine maps to a data store). New `SpaceId`/`TabId` are stable,
  sequential, and never collide across kinds.
- `Space` belongs to exactly one profile; the first `pinned_count` entries of
  `tabs` are pinned (pinned-first ordering).
- `Tab` belongs to exactly one space (its profile is derived, never stored).
- `Split` references two distinct member tabs of one space; v1 supports at most
  one split per space (PLAN Phase 6).

`Browser` API: `create_profile`, `create_space`, `open_tab`, `close_tab`,
`reopen_tab` (LIFO + clamped restore honouring the pinned region),
`select_tab`, `reorder_tab` (splice semantics; crossing the pinned boundary is
rejected), `pin_tab`/`unpin_tab`, `set_split`/`close_split`, `move_tab` (across
spaces and profiles; source active tab and split are repaired; pinned state
survives; profile-crossing is a domain-level no-op — the engine layer recreates
the web view in the destination store), and `transition_tab_lifecycle` for the
explicit `Active → Warm → Suspended → Discarded` resource policy. Selecting a
tab or showing it in either split pane reactivates it; hidden active tabs become
warm. A public `check_invariants()` verifies profile/space/tab cross-references,
pinned ordering, active-tab and lifecycle visibility agreement, split
membership and distinctness, recently-closed exclusion, and unique ids.

`DomainError` is a typed error enum (`UnknownProfile/Space/Tab`,
`TabNotInSpace`, `IndexOutOfBounds`, `ReorderAcrossPinnedBoundary`,
`SplitRequiresDistinctTabs`, `SplitAlreadyExists`, `NothingToReopen`); errors
reject invalid input before any mutation.

The slice ships **40 unit tests** (crate-level test modules + lib docs): creation
and ordering, close-focus semantics (next sibling, else previous, else none),
pinned close, reopen position/clamping/pinned-region, reorder bounds and
boundary rejection, pin/unpin round-trips, move across spaces and profiles,
split set/close/duplicate rejection/dissolution on pane close or move,
lifecycle transition/visibility policy, and tamper detection through
`check_invariants`. All run without macOS frameworks
or a display server.

The controller is composed into the real GPUI app and multi-session adapter.
The first Phase-4 shell slice (visible vertical tabs, pinned/regular regions,
states, empty space, crash recovery, pointer + keyboard interaction) is
implemented with real-app E2E (`just e2e-shell-tabs`) and performance
scenarios. The space slice adds visible profile/space navigation with
round-robin identity hues, pointer + keyboard activation, per-space
active-tab restoration without recreating warm sessions, and cross-profile
view-only switches — proven by `just e2e-shell-spaces` and the
`space_switch` performance scenario. The adjustable/collapsible sidebar slice
adds the DESIGN §5/§11.4 width contract, pointer drag + keyboard resize/
reset parity, Cmd-B collapse/expand with last-expanded-width restoration, and
compact-window clamping that keeps the native WKWebView geometry strictly
positive — proven by `just e2e-shell-sidebar` and the `sidebar_resize` /
`sidebar_restore` performance scenarios. The tab-organization slice adds the
keyboard commands (`Cmd-Shift-P` pin toggle, `Cmd-Shift-↑/↓` region-bounded
reorder, staged `Cmd-Shift-M` move), the visible pin control and §10.9 drag
tick, real pointer drag through the production press/move/release listeners,
illegal boundary drops rejected whole, same-profile moves that keep the
WebViewId and warm session, cross-profile moves that recreate exactly once in
the destination store with cookie isolation, and pin/reorder/move
presentation benchmarks — proven by `just e2e-shell-organize` and the
`tab_pin`/`tab_reorder`/`tab_move_same_profile`/`tab_move_cross_profile`
performance scenarios. The dense-list slice adds the bounded lifecycle
(500 identities, 50 live sessions, 450 suspended through the production
controller paths with a bounded concurrent-session peak during preparation),
windowed regular-region rendering that follows the production scroll offset
(`crate::dense`, ≤ 64 rows + spacers, deterministic scroll anchor), the
pinned + recently-used keep set with exactly-one-session suspended
recreation, and the `dense_scroll`/`dense_select` scenarios wired into the
gate-enforced perf matrix — proven by `just e2e-shell-dense` /
`e2e_shell_dense_edge` and the dense unit/contract suites. Measured
frame-time evidence on the canonical benchmark runner remains open.

---

## 9a. `bbr-browser` — portable controller (Phase 2 slice)

`crates/bbr-browser` owns orchestration between the persistent domain model and
disposable web sessions. It is fully portable: no GPUI, AppKit/WebKit,
filesystem, SQLite, or `unsafe`; it depends only on `bbr-domain`, `bbr-web`,
and the standard library.

Ownership shape:

- The controller drives the engine through `Rc<RefCell<dyn WebEngine>>` — the
  "equivalent testable ownership shape" chosen over a generic `E: WebEngine` so
  the composition root can share one engine handle between the controller and
  the GPUI web-view slot (same engine backs the command path and layout).
- `WebViewId`s come from an injected generator (`ViewIdAllocator`) so E2E runs
  can rely on deterministic sequential ids.
- `SessionRegistry` maintains the forward (tab → view) and reverse (view →
  tab) bijection; `check_invariants` re-verifies registry + engine agreement
  after every mutation.

Session policy (the crate's core invariant):

- Every `Active`/`Warm` tab owns exactly one live session in its profile;
  `Suspended`/`Discarded` tabs own none and recreate one on activation.
- Closing, suspending, or crashing a tab destroys its session; moving across
  profiles destroys and recreates it in the destination profile's store, while
  same-profile moves keep it.
- Only visible tabs (the single-pane selection or both split panes) are shown.

`BrowserController` exposes the first vertical slice (open/select/close/reopen),
navigation routing to the focused tab or split pane, suspend, profile moves,
split open/close with no-session-orphan teardown, and `apply_event` for
`WebEvent`s (URL/title map to the owning tab; loading/history/crash/failure are
tracked per session in `NavState`; unknown or stale `WebViewId`s are reported
as `ControllerEvent::UnknownWebView` without corrupting state). Every mutation
returns exactly one typed `ControllerEvent`, and a read-only `snapshot()`
publishes the tab list, active view, and live-session count for UI and E2E.

Testability: the deterministic `FakeEngine` (in `crate::fake`,
`#[cfg(test)]`-only) records every engine call into an op log, owns fake
sessions, rejects duplicate creation and missing-session actions, and emits
controlled `WebEvent`s through the registered sink. Because the controller
drives the engine through the same `dyn` seam the app uses, 47 crate tests
prove registry, controller, event-routing, active-space visibility, profile
move, split, lifecycle, and snapshot policy without a display server.

---

## 10. Build & verification (bounded recipes, `justfile`)

From repo root in the dev shell (`nix develop` or direnv):

```bash
just fmt        # cargo fmt --all -- --check
just check      # cargo check --workspace --all-targets
just test       # cargo test --workspace
just clippy     # cargo clippy --workspace --all-targets -- -D warnings
just roadmap    # validate hierarchical roadmap Markdown links
just verify     # fmt + check + test + clippy + roadmap
just e2e-smoke  # baseline real-WKWebView navigation/focus/resize
just e2e-tabs   # real multi-tab/session behavior
just e2e-profiles # real profile isolation, moves, and suspend lifecycle
just e2e-shell-tabs    # real vertical-tab surface: open/select/close/reopen
just e2e-shell-spaces  # real profile/space switching (pointer + keyboard)
just e2e-shell-sidebar # real sidebar drag/keyboard/collapse/geometry clamp
just e2e-perf   # optimized startup and production-path interaction budgets
just e2e        # canonical sequential GUI gate (smoke, tabs, profiles,
                 # shell-tabs, shell-spaces, shell-sidebar, performance)
```

All recipes are wrapped in GNU `timeout --kill-after=10s` with
`BBR_CHECK_TIMEOUT`/`BBR_TEST_TIMEOUT` (5m) and `BBR_E2E_TIMEOUT` (2m)
overrides, and pin `DEVELOPER_DIR` to Xcode 16.4. On headless CI,
`BBR_SKIP_GUI_TESTS=1` skips the smoke.

**Verified green on 2026-08-06** (evidence, not claims):

```
just fmt     → exit 0
just check   → exit 0 (workspace, all targets)
just test    → exit 0  — bbr-app 29 passed · bbr-domain 41 passed ·
                        bbr-browser 54 passed · bbr-fixtures 4 passed ·
                        bbr-testkit 13 passed · bbr-web 10 passed ·
                        bbr-web-macos 1 passed · 15 GUI E2E tests ignored
just clippy  → exit 0 (-D warnings)
just roadmap → exit 0 (hierarchical roadmap links valid)
just e2e     → exit 0 — smoke, tabs, profiles, shell-tabs, shell-spaces,
                        shell-sidebar, and optimized performance
                        (sequential real GPUI + WKWebViews)
```

The testkit-feature path also passes `cargo clippy -p bbr --features testkit
--all-targets -- -D warnings`. GUI suites are deliberately sequential: two
AppKit windows resizing concurrently can interfere through screen clamping.
Every bounded GUI/performance recipe additionally uses
`scripts/with_gui_lock.py`, whose advisory key derives from Git's common
directory; sibling worktrees therefore serialize on the same host-wide gate
while retaining each recipe's hard timeout and process-tree cleanup.

### Run the app

```bash
cargo run -p bbr -- --bbr-initial-url https://example.com
```

---

## 11. Known limitations (current implementation)

1. **The vertical tab shell, the dense-500 list, and the pointer-state +
   internal-semantic-model slice are in; the canonical measured frame-time
   evidence remains.**
   `BrowserController` and `WebEngineWk` open, switch, preserve, navigate,
   and close multiple real sessions; Cmd-T stages a tab destination and Cmd-W
   closes through the real product path. The visible vertical tab list with
   pinned/regular regions, pointer + keyboard interaction, drag reorder, and
   move-between-spaces arrived with the Phase-4 shell slices; the dense-list
   slice added the bounded lifecycle, windowed rendering, recency keep set,
   and the gate-wired `dense_scroll`/`dense_select` scenarios; the closeout
   slice added the §10.2 hover / §10.3 press states through the production
   listeners and the internal §15 semantic tree (`tab_hover`/`tab_press`
   benchmark scenarios + `just e2e-shell-hover`). The open acceptances are
   the measured frame-time evidence for 500 tabs / 50 live sessions on the
   canonical runner, persisted-restore acceptance (Phase 7 — the dense state
   is in-memory only), drag auto-scroll (an off-window drag target shows no
   tick today), and the platform Accessibility bridge (Phase 9).
2. **Profile/space visibility is complete; cross-context polish remains.** The
   sidebar renders every profile's spaces with chips, activation works through
   pointer + keyboard, empty spaces name their profile, and the command-field
   destination hint communicates cross-profile moves and space switching; the
   remaining Phase-4 polish is shell craft (e.g., drag auto-scroll in dense
   lists) and the canonical measured frame-time evidence.
3. **`--bbr-clock` is a no-op**; the `TimeSource` seam is intentionally not
   added until time-based logic exists (debounces in Phase 5/7).
4. **Popups/new windows are suppressed** (`createWebViewWithConfiguration` →
   `None`); wired popups land in a later phase.
5. **No find-in-page, zoom, downloads, permission prompts, WebAuthn, or
   process-termination recovery UI yet** (Phase 8). `PageCrashed` is
   observable in the status line and snapshot, but there is no recovery
   surface.
6. **The release-exclusion failsafe is not implemented**: under a non-testkit
   build the `--bbr-*` test flags are silently ignored rather than rejected at
   startup (E2E §12's exit-64 gate is future work).
