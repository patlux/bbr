# BBR — Architecture (current implementation)

Status: **describes the implemented workspace** (Phase 1 native-view spike + the
portable domain slice from PLAN.md Phase 2). Verified against the actual GPUI
crate (`gpui 0.2.2`), the `objc2-web-kit 0.3.2` bindings, the Swift reference at
`PWBrowser`, and a green verification run (`just verify` + `just e2e-smoke`,
2026-08-05).

---

## 1. Decisions locked in this phase

| Decision | Choice | Why |
|---|---|---|
| Workspace shape | Cargo workspace, seven crates: `bbr-web`, `bbr-web-macos`, `bbr-app`, `bbr-domain`, `bbr-browser`, `bbr-testkit`, `bbr-fixtures` | Minimal set that exercises the full integration (chrome, native engine, portable model, orchestration, E2E control) |
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
| `Cmd-L` (FocusCommandBar) | `window.focus(chrome_handle)` **and** `engine.focus_host_chrome()` → `makeFirstResponder(ChromeResponder)`; `command_bar_active = true` |
| `Escape` / Enter-after-navigate (FocusWeb) | `window.blur()` **and** `engine.focus_native_web()` → `makeFirstResponder(web_view)` |
| Click in the sidebar | `focus_chrome`: `window.focus(chrome_handle)` + `focus_host_chrome()`, `command_bar_active = false` |

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
  webview.rs        # WebEngineWk: WKWebView cache, profile→data-store map, actions
  delegate.rs       # define_class! WKNavigationDelegate + WKUIDelegate + KVO title
```

- **Profile stores:** `dataStoreForIdentifier` (persistent, identifier-backed,
  via `NSUUID`) or `nonPersistentDataStore` when `EngineOptions.non_persistent`
  is set — the app enables non-persistent mode whenever a `--bbr-test-token`
  is present (E2E hermeticity). Stores cached in a `HashMap<ProfileId, …>`.
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
2. Build the id generator; allocate ProfileId + WebViewId.
3. gpui::Application::new().run: open one window (centered, options from flags).
4. In the open_window callback: capture the GPUI root NSView, build the engine
   (WebEngineWk on macOS, NullEngine elsewhere), create BbrApp.
5. create_web_view(initial_url) after the first frame (the first prepaint frames it).
6. install_actions: bind Cmd-L/Escape/Cmd-[ /Cmd-]/Cmd-R/Cmd-B/Cmd-T actions.
7. Under testkit: start the gateway on the profile dir socket, pump the
   event/command channels on the main thread, broadcast AppReady + focus events.
```

The chrome (`bbr-app/src/app.rs`) renders a fixed-width sidebar
(`SIDEBAR_WIDTH_PX = 260`): brand, nav dots (◀ ▶ ⟳ with enabled-state
styling), a command bar (mirrors the live URL; inline cursor; accepts typing,
backspace, enter), a status line (`NavigationState.status_line()`, incl.
"⚠ Web content process terminated"), and a sidebar toggle. A `key_context`
("bbr_chrome") routes printable input to the command bar only while
`command_bar_active`. The content area is entirely the `NativeWebViewSlot`.

The `shell.rs` module holds `ShellState` (one profile + one active view —
the whole Phase-1 identity) and `NavigationState`, which mirrors `WebEvent`s
(including crash/failure) into what the chrome renders and the E2E snapshot
reads.

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

The slice ships **34 unit tests** (crate-level `tests.rs` + lib docs): creation
and ordering, close-focus semantics (next sibling, else previous, else none),
pinned close, reopen position/clamping/pinned-region, reorder bounds and
boundary rejection, pin/unpin round-trips, move across spaces and profiles,
split set/close/duplicate rejection/dissolution on pane close or move,
lifecycle transition/visibility policy, and tamper detection through
`check_invariants`. All run without macOS frameworks
or a display server.

Not yet present (per PLAN.md Phase 2, next milestones): command routing was
shipped in the `bbr-browser` controller (§9a below); still missing are the
`bbr-ui` layer that renders domain state and the real-app composition path
that hosts the controller behind the multi-session native adapter (Phase 3).

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
drives the engine through the same `dyn` seam the app uses, the 46 crate tests
(17 fake/registry contract tests + 28 controller behavior tests + one snapshot test) prove policy
without a display server.

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
just e2e-smoke  # cargo test -p bbr --features testkit --test e2e_smoke -- --ignored --nocapture
```

All recipes are wrapped in GNU `timeout --kill-after=10s` with
`BBR_CHECK_TIMEOUT`/`BBR_TEST_TIMEOUT` (5m) and `BBR_E2E_TIMEOUT` (2m)
overrides, and pin `DEVELOPER_DIR` to Xcode 16.4. On headless CI,
`BBR_SKIP_GUI_TESTS=1` skips the smoke.

**Verified green on 2026-08-05** (evidence, not claims):

```
just fmt     → exit 0
just check   → exit 0 (workspace, all targets)
just test    → exit 0  — bbr-app 4 passed · bbr-domain 40 passed ·
                        bbr-browser 46 passed · bbr-fixtures 3 passed ·
                        bbr-testkit 10 passed · bbr-web 9 passed ·
                        e2e_smoke 1 ignored (GUI, run explicitly)
just clippy  → exit 0 (-D warnings)
just roadmap → exit 0 (hierarchical roadmap links valid)
just e2e-smoke → exit 0 — "SMOKE PASSED", 1 passed (real WKWebView, 1.53s)
```

Post-controller remediation (controller + fake-engine commit group):

```
just verify    → exit 0 (fmt + check + test + clippy + roadmap)
just e2e-smoke → exit 0 — "SMOKE PASSED", 1 passed (real WKWebView, 1.25s)
```

### Run the app

```bash
cargo run -p bbr -- --bbr-initial-url https://example.com
```

---

## 11. Known limitations (current implementation)

1. **Exactly one profile + one live web view.** No tab list, spaces, splits,
   or multi-view UI yet — `ShellState` is the whole identity. `Cmd-T` is a
   placeholder that focuses the command bar (Phase 5 new-tab mode).
2. **Persistent store identity is not yet deterministic.** Production mode
   creates a fresh `NSUUID` identifier per profile per launch
   (`dataStoreForIdentifier`), so browsing data persists but the PLAN §4
   "deterministic per-profile mapping" is deferred; E2E always uses
   `nonPersistentDataStore`. Cookies/credentials cross profile boundaries only
   via WebKit store isolation (engine layer), not domain policy.
3. **`--bbr-clock` is a no-op**; the `TimeSource` seam is intentionally not
   added until time-based logic exists (debounces in Phase 5/7).
4. **Popups/new windows are suppressed** (`createWebViewWithConfiguration` →
   `None`); wired popups land in a later phase.
5. **No find-in-page, zoom, downloads, permission prompts, WebAuthn, or
   process-termination recovery UI yet** (Phases 3/8). `PageCrashed` is
   observable in the status line and snapshot, but there is no recovery
   surface.
6. **The release-exclusion failsafe is not implemented**: under a non-testkit
   build the `--bbr-*` test flags are silently ignored rather than rejected at
   startup (E2E §12's exit-64 gate is future work).
7. **The controller is not composed into the real app yet.** `bbr-browser` is
tested by its fake-engine contract suite (46 tests, no display server), but
`bbr-app` still hosts a single view: wiring `BrowserController` behind the
multi-session native adapter and adding the real-app tab E2E vertical slice is
the Phase 3 path (task packet: browser-controller.md "Tests and evidence").
