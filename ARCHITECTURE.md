# BBR — Architecture (current implementation)

Status: **describes the implemented workspace through the Phase-4 shell slices, the integrated Phase-5 command core/app composition, and the Phase-6 split-view shared contract/core**: native GPUI +
WKWebView embedding, portable browser orchestration, controller-backed real
tabs, profile-isolated multi-session WebKit, the adjustable/collapsible
sidebar, tab organization, the dense-500 list, the pointer-state +
internal-semantic-model slice, dependency-free command parsing/ranking, the
keyboard-complete real-app command result surface, and the per-`WebViewId`
native geometry contract with the authoritative pane layout plus the typed
`SetSplit`/`CloseSplit` testkit seam. Verified against the actual
GPUI crate (`gpui 0.2.2`), the `objc2-web-kit 0.3.2` bindings, the Swift
reference at `PWBrowser`, and green bounded verification runs. Historical
real-app split-geometry and single-pane sidebar evidence remains green. The
current Renderer-Isolation candidate has automated A1–A4 evidence and passed
whole-candidate review plus focused remediation recheck; Orchestrator commit
and serial integration remain. A5 (the external WebKit → callback/error hop)
remains an unpassed Apple platform contract.
Performance was not executed for this slice, and the latest controlled startup
observation remains red at `535.866375 ms > 500 ms`; no current canonical E2E
or performance pass is claimed.

---

## 1. Decisions locked in this phase

| Decision | Choice | Why |
|---|---|---|
| Workspace shape | Cargo workspace, nine crates: seven shell/product/test crates, implemented portable `bbr-command`, and the `bbr-persistence` landing zone with its exact bundled-SQLite dependency contract | Phase-5 parsing/ranking is integrated without app coupling; the Phase-7 repository remains isolated until its review findings settle |
| Chrome vs. pages | GPUI owns chrome; `WKWebView` is a native AppKit child view | Product principle; never texture-capture web content |
| Embedding strategy | WKWebView is a **subview of GPUI's root `NSView`**, sized by a bounds-reporting GPUI element | Keeps the responder chain, key equivalents, accessibility, video/IME/process isolation intact |
| Native-view slot | A custom public GPUI `Element` (`NativeWebViewSlot`) publishes its own window-relative bounds each frame | Uses only public GPUI APIs; no GPUI fork. **Fulfils the "no forking GPUI" stop-condition path** |
| Focus choreography | Explicit `makeFirstResponder` calls driven from the host bridge, plus a transparent `ChromeResponder` overlay | GPUI's `Window::focus` only mutates GPUI's internal focus tree; AppKit refuses `makeFirstResponder(gpui_view)` because `GPUIView` does not accept first responder (see §5) |
| Profile isolation | `nonPersistentDataStore` per profile in test mode; identifier-backed persistent store in production | E2E hermeticity; no WebKit container leakage between tests |
| Main thread | All `bbr-web-macos` calls on the main thread (enforced + debug-asserted) | `WKWebView` and the GPUI mac platform are main-thread-only |
| Engine adapter | `bbr-web` defines the platform-neutral `WebEngine` contract; `bbr-web-macos` implements it; a `NullEngine` covers non-macOS targets | Cross-platform requirement; portable crates never import macOS frameworks |
| Chrome above native pages | App-local `PortalHost::{GpuiDeferred, NativeAboveWeb}` with exactly one transparent, non-activating native child portal per browser window | GPUI deferred painting cannot cross the AppKit `WKWebView` subview boundary; the child host preserves native WebKit while GPUI owns portal content/state |

---

## 2. Workspace & crate layout

```
bbr/
  Cargo.toml            # [workspace] members = the crates below; resolver = "2"
  flake.nix             # dev shell for aarch64/x86_64 darwin + linux
  justfile              # bounded local verification: fmt/check/test/clippy/verify/e2e-smoke
  crates/
    bbr-web/            # lib: platform-neutral contracts (no objc, no gpui, no unsafe)
    bbr-web-macos/      # lib: WKWebView + AppKit bridge (target_os = "macos" only)
    bbr-app/            # bin `bbr`: composition root, GPUI window, E2E gateway glue
    bbr-ui/             # lib: portable presentation authority — Chassis tokens
                        #   (Theme/Palette/Density/elevation/space tint), §15 state
                        #   vocabulary, plain (Model, Theme, Callbacks) components;
                        #   exact direct deps: gpui + bbr-domain only
    bbr-domain/         # lib: portable browser core model (Phase 2 slice)
    bbr-browser/        # lib: portable controller: tabs ↔ disposable web sessions
    bbr-testkit/        # lib: typed E2E protocol, gateway server + client (unix sockets)
    bbr-fixtures/       # lib+bin: deterministic loopback HTTP fixture server
    bbr-command/        # portable Phase-5 parser, corpus, action registry, ranking + benchmarks
    bbr-persistence/    # portable Phase-7 landing zone; bundled SQLite contract, repository pending integration
```

### Dependency direction (strict, acyclic)

```text
bbr-web-macos  ──uses──▶  bbr-web        (implement the WebEngine trait)
bbr-domain     ──uses──▶  bbr-web        (ProfileId for profile identity)
bbr-ui         ──uses──▶  gpui, bbr-domain (exact direct set; no direct bbr-web)
                 └──────▶  bbr-web only through bbr-domain's existing ProfileId edge
bbr-browser    ──uses──▶  bbr-domain, bbr-web (orchestration + engine contract)
bbr-testkit    ──uses──▶  bbr-web        (serde feature: WebEvent/WebAction/ids)
bbr-app        ──uses──▶  bbr-web, bbr-web-macos (macOS), bbr-testkit (testkit
                          feature), bbr-fixtures (dev-dep)
bbr-fixtures   ──uses──▶  (std only)
bbr-command    ──uses──▶  (std only; provider-neutral command data and ranking)
bbr-persistence──uses──▶  rusqlite (exact 0.38.0, bundled; repository implementation not yet on main)
```

- `bbr-web` **must not** depend on `gpui`, `objc2*`, `raw-window-handle`, or any
  AppKit/WebKit symbol. It is a pure contract crate and carries
  `#![forbid(unsafe_code)]`. Current portable engine-aware crates
  (`bbr-domain`, `bbr-browser`) may depend on it. `bbr-ui` may not: its exact
  direct dependencies are `gpui` and `bbr-domain`, and its only allowed
  first-party transitivity is the existing `bbr-domain → bbr-web` `ProfileId`
  edge. Any direct `bbr-ui → bbr-web` dependency, import, or API use is
  forbidden.
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
- The G1a Foundations/Primitives cutover made `bbr-ui` the one design-system
  authority: central tokens/theming (Theme/Palette/Density/elevation/space
  tint) and dumb `(Model, Theme, Callbacks)` components live there, and
  `bbr-app` consumes the same production symbols. The app keeps only
  composition: the space strip, the controller-snapshot DTO adapters, and
  the portal geometry/native-boundary code stay app-owned. Sidebar and
  Command Palette are real compositions, not copied Gallery anatomy: the G3
  Sidebar cutover moved the rail composition to `bbr_ui::sidebar` and the
  G3 Command cutover moved the floating palette/overlay presentation to
  `bbr_ui::surfaces::command` (plain DTO + exact renderer + stable ids),
  each with `bbr-app` as the thin assembler/mapping owner and no local
  duplicate renderer. Generic Button/Input/Badge-style primitives are not
  added speculatively; extraction requires repeated same-intent product
  anatomy.
- `bbr-command` is fully portable and dependency-free. It owns typed command
  modes/identities/activations, strict deterministic URL-vs-search parsing, an
  owned provider-neutral corpus, the 13-action catalog, standalone tab/space/
  history/action/fallback ranking, deterministic score keys, and its 500-tab +
  10,000-history distribution benchmark. It performs no GPUI composition,
  filesystem/SQLite access, testkit transport, or synchronous I/O; those remain
  serial integration surfaces.

### Version pins (workspace `[workspace.dependencies]`)

| Crate | Version | Notes |
|---|---|---|
| `gpui` | `0.2.2` | GPUI-CE git `568271c01c5a9d640d7b091e5bfd136b04035f28`; `default-features = false`, `features = ["font-kit"]`; crates.io `gpui` consumers patched to the same source (one type world) |
| `gpui_platform` | `0.1.0` | same GPUI-CE rev; provides `gpui_platform::application()` (crates.io's `Application::new` is gone) |
| `gpui-component` | `=0.5.1` | `default-features = false` (webview/Wry disabled); source vendored under `vendor/gpui-component` with mechanical GPUI-CE API-compatibility deltas only (see `vendor/gpui-component/BBR_PATCHES.md`) |
| `objc2` | `0.6.4` | satisfies `objc2-web-kit`'s `>=0.6.2,<0.8.0` |
| `objc2-web-kit` | `0.3.2` | `WKWebView`, `WKWebsiteDataStore`, `WKNavigationDelegate`, `WKUIDelegate`, … |
| `objc2-app-kit` | `0.3.2` | `NSView`, `NSWindow`, `NSResponder`, `NSEvent` |
| `objc2-foundation` | `0.3.2` | `NSString`, `NSURL`, `NSURLRequest`, `NSError`, `NSUUID`, KVO |
| `block2` | `0.6.2` | available for delegate `decisionHandler` blocks (not yet exercised) |
| `raw-window-handle` | `0.6` | must match GPUI 0.2.2's `raw-window-handle 0.6` |
| `serde` / `serde_json` | `1` | protocol framing (feature-gated in `bbr-web`) |
| `smol` | `2` | channels for the main-loop event/command pump |
| `rusqlite` | `=0.38.0` | `default-features = false`, bundled SQLite only; consumed solely by `bbr-persistence`. Pinned to the newest release whose `libsqlite3-sys` (0.36) still compiles on the devshell stable toolchain: 0.40.1 pulls `libsqlite3-sys` 0.38.1 whose build script requires the unstable `cfg_select!` (E0658 on rustc 1.97.1) |

Toolchain: workspace `rust-version` is `1.89` on macOS and `1.92` for the
Linux-target stack (the pinned dependency manifests' highest declared
floors: `smol_str 0.3.6` on macOS, `oo7` via ashpd/zbus on Linux); the
dev shell provides `rustc 1.97.1` via `rust-overlay`. `Xcode 26.2.0` is
the installed host prerequisite (WebKit SDK, plus the Xcode "Metal
Toolchain" component for shader compilation); the repository default is
`DEVELOPER_DIR=/Applications/Xcode-26.2.0.app/Contents/Developer` (see
the `justfile` `xcode_developer_dir` variable). With that developer dir
set, the system `/usr/bin/xcrun` resolves `metal` and `metallib` through
the Metal Toolchain component (`com.apple.MobileAsset.MetalToolchain`,
mounted by cryptexd) rather than from files inside the Xcode developer
dir. The flake devshell prepends `scripts/xcode-shims` (a forwarder to
`/usr/bin/xcrun`) so the real Apple `xcrun` wins over the xcbuild fake
`xcrun` in the nix closure; no session-local shim is required.

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
3. **Position & resize via bounds-reporting elements.** `NativeWebViewSlot`
   (`bbr-app/src/slot.rs`), a custom public GPUI `Element`, fills one content
   pane. Each slot is **keyed by its `WebViewId`** (stable element id
   `bbr-native-web-slot-<n>`); in `prepaint` GPUI hands it
   `Bounds<Pixels>` relative to the window and the element forwards the rect
   as a `bbr_web::WebRect` to the engine's `set_web_view_rect(id, rect, …)`,
   so each visible native session is framed independently. The bridge
   converts the rect to an AppKit frame via the **y-flip**
   (`bbr_web::flip_rect`, unit-tested) using the root view's actual height
   (AppKit origin bottom-left, GPUI origin top-left).
   - Every slot has a stable keyed `ElementId`
     (`bbr-native-web-slot-<n>`), and the semantic snapshot's `visible_ids`
     exposes exactly those rendered keyed ids.
   - The shell renders one slot per visible pane from the controller's
     authoritative [`PaneLayout`]: one full-width slot in single-pane mode
     (geometry identical to the pre-split contract) and, for a shown split
     group (blueprint D23), one absolutely placed slot per pane at the rect
     `bbr_split::tiling::project` gives the group's tree over the card's
     inner box (`BrowserController::split_layout` names every pane's view;
     2 pt GPUI dividers sit in the gaps). A group that cannot give every
     pane 320 × 200 pt renders only the focused pane and hides the others
     natively (`engine.show(view, false)`), never destroying them. Split
     groups live in `bbr_domain::Space::splits` and persist in the app's
     `splits.txt`, not in the store's two-pane table. The focused/active slot additionally
     feeds the legacy `content_rect` and the presentation-commit path; a
     secondary pane never satisfies an existing benchmark.
   - `set_web_view_rect` writes the frame only when that view's rect
     changed, so an idle window produces no `setFrame:` traffic (no polling
     loop) and a per-view rect update never re-frames a sibling pane.
4. **No polling loop.** GPUI is display-link driven; the slot publishes bounds
   only on change.

### Fullscreen / scale / occlusion / resize

- Window resize → GPUI layout re-runs → every visible slot's bounds change
  → the bridge updates each pane's frame independently. Verified by the E2E
  smoke resize step (`SetWindowSize` → snapshot shows new geometry; the
  native view re-frames next frame) and the split-geometry suite (a resize
  reframes both pane ids independently).
- Scale-factor change / fullscreen / occlusion: WKWebView handles these
  natively; nothing extra beyond the frame update.

### Desktop-facing browser chrome

Each browser window owns one click-through `NSVisualEffectView` behind its
GPUI root (`bbr-web-macos::chrome_material`). It uses public Sidebar material,
behind-window blending and native activation, with an explicit light/dark
appearance. Installation happens during window setup, never inside render;
window close removes the view and its owned layers. Settings and Gallery
Roots keep their existing opaque background through a per-instance component
Root override, not a shared-theme mutation.

`app::chrome_glass` paints one compensated translucent space tint across rail
and chassis. A cached public Core Animation even-odd mask removes only the
inverse content-card corner wedges; matching corner tint sits above the native
material. Native WKWebView clipping, page opacity, content geometry and session
ownership are unchanged. Reduce Transparency, Increase Contrast or a failed
native installation select the original opaque chrome; the accessibility
fallback hides the effect and removes the owned mask.

Typed testkit observations distinguish actual native attachment, effect/mask
state and WKWebView opacity from requested policy. They do not prove the final
WindowServer composite. The official visual route captures GPUI paint only;
desktop blur, inactive appearance and native edge antialiasing remain open in
[the glass packet](docs/roadmap/tasks/chrome-desktop-glass.md).

### Native portal above WebKit

`bbr-app::portal` owns portable portal identity, placement, priority,
dismissal, and the active GPUI presentation. `GpuiDeferred` is for chrome that
only needs to escape GPUI layout; `NativeAboveWeb` drives one implementation-
owned child `NSWindow` attached above the browser window's native views.

`bbr-web-macos::portal` owns only AppKit attachment, z-order, frame sync,
visibility, diagnostics, and teardown. The borderless non-activating panel is
created once per browser window, starts ordered out, remains non-key/non-main,
and is pinned to the parent content rect. It has no tab, profile, space,
session, history, or independent product lifetime. GPUI renders the portal
content; WebKit remains a real native `WKWebView` with no screenshot, texture,
reparenting, or replacement path.

Native visibility is parent-owned: an armed presentation is shown only while
BBR is active/unhidden and its browser owner is visible, key, non-minimized and
on the active macOS Space. The bridge resets GPUI's PopUp level to the parent's
level and replaces independent all-Spaces behavior with `FullScreenAuxiliary`.
App/key-window/Space/lifecycle notifications coalesce into deferred reconciliation
that reads current facts and intent, so dismissal/drop supersedes a queued show.
Temporary hiding keeps the logical presentation, draft and native identity.
AppKit can remove `parentWindow()` on `orderOut`; restoration reattaches only to
the retained original owner, then orders relative to it (never `orderFront`).
Diagnostics distinguish retained ownership from the measured child relation.

The resize/move observer stores a raw pointer to a `NativePortal` that is
heap-pinned in a `Box` **before** observer installation. Moving the bridge moves
only the box pointer, never the observer target. Teardown removes the observer
from both the application and workspace notification centers and the child
relation, orders out the panel, and closes it on a bare run-loop turn.
Typed diagnostics expose attachment, native visibility, frame equality,
non-activating/key/main policy, and real Window Server hit-test queries. The
real-app portal suite proves the child host stays above the parent's WKWebViews
without changing `WebViewId`, session count, profile, space, or focus.

U1's centered palette and field-anchored command overlay are the first product
consumers of this host. `BbrApp` remains the sole authority for command text,
mode, ranking, selection, focus origin, and browser mutations. The one
`PortalRoot` observes the same `Entity<BbrApp>`, maps the app-owned
`CommandSurface`/state/rows onto the plain `bbr_ui::surfaces::command`
model during render (`app/palette.rs` — the one app-owned DTO mapping that
strips every `Activation` payload), and paints only GPUI chrome inside the
child window; it owns no tab/profile/space/session lifetime. The G3 Command
cutover moved the floating card renderer and the stable §11.4/§15 ids into
`bbr-ui` (the exact render-only anatomy) and deleted the app-local
`portal/command.rs`; `portal/probe.rs` remains the app-owned portal builder
(placement/dismissal/backdrop) that resolves the rect and invokes the
production `command_card`. Ranking, activation, focus, and every
testkit/semantics/performance authority stay in `bbr-app`. Opening U1
presents a stable `NativeAboveWeb` request (`PresentationId(0xC1)` palette
or `0xC2` overlay), and every close/commit/cancel/navigation context path
dismisses the same host. The browser root no longer paints a duplicate
floating command surface underneath WebKit.

The find-in-page bar (task packet `find-in-page-portal-bar`) is the next
consumer: `Cmd-F` presents a `NativeAboveWeb` request (`PresentationId(0xC3)`)
anchored top-right of the content area with the §11.4 command-surface compact
clamp (360 pt) at `density:row` height — settled S1 tokens, no invented sizes.
Exactly one active find presentation per browser window; tab/space/session
switches discard it (`find_discard_if_unbound`), while a navigation of the
bound view invalidates the reported results in place (the bar stays open,
`find_navigation_invalidated`). The bar reads the app's live `FindSurface` per
render, so the WebKit-reported count/ordinal can never drift from the app. The
portal window is non-activating and never key by design; keystrokes are handled
by the main window's find-focus handle, and Escape/close restore the focus
domain captured at open (`origin_chrome_focused`). The real-app find suite
proves the same owner-classified Window Server hit-test evidence for the bar
over the live `WKWebView`, and the `find_open`/`find_query_result`/`find_step`
performance scenarios measure the warm presentation path against the
command-surface budget.

The floating rail (blueprint D02) is the Portal's interactive layer, beneath
any foreground presentation and the transient overlays. `PortalRoot` builds
the unchanged rail composition through `BbrApp::floating_rail_layer`, inside a
`with_app` update of the **browser** window, so the rail model reads that
window's focus. Every rail callback is retargeted to the browser window
(`rail_float_view::retarget`); the Portal only delivers the pointer. The
Portal stays non-key: the browser window keeps the keyboard, and a keyboard
twin there tracks the rail's focus handles with the same key listeners. The
Portal captures the pointer over the panel minus the notch around the native
traffic lights, and over the whole window while a drag that began in the rail
runs. The reveal is a pure model (`app::rail_reveal`) fed by a local AppKit
pointer monitor (`bbr-web-macos::pointer_observer`, observe-only, drained
through a channel by `rail_observer`) and by the testkit `RailPointer` seam.
The card and the web views keep the collapsed geometry; nothing reflows. The
Downloads fade retains one interpolation-state entity, with a fresh GPUI
transition wrapper evaluated in each browser render. Docked and floating panel
composition read that presented value without evaluating again: Portal builds
the rail against the browser window outside its draw phase, where requesting
animation frames is invalid. Goals, continuous retargeting and `dur:fine` remain
owned by the same production transition.

### App preferences and the Settings window

Blueprint D16: user preferences (appearance, contrast and motion overrides,
a pinned density, the default paper for new spaces, Peek for new-window
links, the ⌘T space scope) live in `preferences.txt` in the profile
directory (`bbr-app::preferences`), a plain `key=value` file written
atomically; unknown keys and bad values fall back to the defaults. They are
not browser state, so they never enter the SQLite store, whose schema stamp
has no migrations (a bump would discard the store). The search engine stays
in the store's settings row. The Settings window (`settings_window`) is a
separate GPUI window that observes `BbrApp` and renders
`BbrApp::settings_model`; every control reports one `SettingsEvent`
applied by `BbrApp::on_settings_event`, the one authority. The Peek choice
also reaches the native new-window handler, installed before the app entity
exists, through `preferences::new_window_decision`.

### WebKit find adapter (content-world isolated, non-destructive)

The portable `bbr-web::find_in_page` session owns the L7 request/result
lifecycle (start/report/find_next/find_previous/finish/invalidate). The macOS
adapter (`bbr-web-macos::find_adapter` + `find_mapping` + `find_web_kit` +
`js/find_strategy.js`) binds once at open to the active view's retained
`WKWebView` and drops at close; opening, stepping, and closing never create,
destroy, move, or reselect a web session. The adapter echoes portable request
ids and drops stale completions; the portable session rejects stale ids
independently — count/ordinal always come from WebKit's content process, never
synthesized in GPUI.

The strategy executes through
`evaluateJavaScript_inFrame_inContentWorld_completionHandler` in
[`WKContentWorld::defaultClientWorld`] — BBR's own client world, never the
page's main content world. The client world shares the DOM (the strategy can
read text runs and move the selection) but has a separate JavaScript global
object, so the `__bbr*` globals and the MutationObserver are unobservable by
and cannot collide with page scripts; the `find-isolation.html` fixture proves
this in the real app (the page world records every unexpected global it can
see and finds none after a find run). The strategy is non-destructive: it may
read the DOM and move the selection, but never persistently mutates page
structure, attributes, styles, or globals — highlights (CSS Custom Highlight
registry) and the selection are transient and cleared on
finish/invalidate/zero-match.

A portal-window testkit draw must happen after the mutable `BbrApp` lease is
released: the portal render reads that same entity. The gateway therefore
copies the portal `AnyWindowHandle` during `with_app`, then calls
`Window::draw` outside that lease. This prevents GPUI entity-map reentrancy
while preserving a single state authority.

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
| Native portal observer | `bbr-web-macos::portal` | `NativePortal` is boxed before observer installation; the observer raw pointer targets the stable heap allocation. Install/remove and every AppKit mutation are main-thread-only; teardown removes observation before close |
| New-window UI delegate | `bbr-web-macos::delegate` / `new_window` | The live `WKNavigationAction` and `WKWindowFeatures` are copied into portable values on the main thread. The coordinator applies portable policy and publishes a plain delivery; the delegate always returns `None`, so no native view escapes the controller registry |

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
`cmd-]`, `cmd-r`, `cmd-s`, `cmd-t`). They run in the bubble phase and are
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
    fn set_web_view_rect(&mut self, id: WebViewId, rect: WebRect, content_height: f32);
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

Note the trait shape: `set_web_view_rect` carries `content_height` (so the
flip never drifts from the true container height and can be unit-tested
without macOS) and addresses geometry **per live `WebViewId`** — the split
shared contract. Unknown ids are no-ops, a destroyed id loses its cached
geometry, an unchanged per-view rect produces no repeated native frame
update, and a newly created view keeps a zero/default frame until its keyed
slot publishes. Zoom/warm/suspend/lifecycle methods are deferred to later
phases — the trait remains deliberately narrow. The event sink is
 deliberately *not* `Send` (the concrete engine and its delegates are
`MainThreadOnly`). An object-safety test proves the trait works behind
`Box<dyn WebEngine>`.

`bbr-web` also exposes `serde` derives on the ids/events/actions behind the
`serde` feature (enabled by `bbr-testkit`) so the wire format never leaks
framework types.

The productive new-window seam is also portable. `NewWindowRequest` carries the
exact source `WebViewId`, source `ProfileId`, verbatim URL, user-activation
classification (a link activation with a nonzero `buttonNumber`: WebKit
fills it only from a trusted click or its own click for Enter, so a script's
`click()` on a `target=_blank` link is not the user's), explicit input disposition, target, and window-feature
metadata. `default_decision` is the single MVP policy: a background input
(Command-primary / middle click; `WKNavigationAction.buttonNumber` is WebKit's
button mask, middle `4`) becomes `OpenBackgroundTab` for a direct
activation — for a link that asks for a new window and, through the
navigation-action policy, for any other link, whose own navigation is then
cancelled; an eligible direct activation becomes `OpenTab` (or `Preview` for
the Peek policy); script-created requests become `Block`. `decide` produces a
typed `NewWindowOutcome`, and `NewWindowDelivery` keeps the request beside that
outcome so the app/controller can fail closed on stale source identity or
profile mismatch. No adapter may normalize the URL or choose a second
disposition policy.

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
- **Per-view geometry cache:** `HashMap<WebViewId, WebRect>` holds the last
  host-layout rect published for each live view. `set_web_view_rect` frames
  only the addressed view, skips unchanged rects (no repeated `setFrame:`),
  removes the cached rect on destroy, and ignores unknown ids. A newly
  created view starts with a zero/default frame until its keyed slot
  publishes. Never frames a sibling view; the y-flip math is unchanged.
- **Profile stores:** `dataStoreForIdentifier` uses a stable UUID derived from
  `ProfileId` (`store_kind`: `42425200-0000-4000-…`), or
  `nonPersistentDataStore` when `EngineOptions.non_persistent` is set — the
  app enables non-persistent mode whenever a `--bbr-test-token` is present
  (E2E hermeticity). A testkit run with `--bbr-web-data-run <hex>` gets
  identifier-backed stores in its own namespace (`42425e57-<run>-4000-…`)
  instead, so E2E can prove persistence without touching a production
  identifier. Stores are cached per profile and outlive views. Deleting a
  profile removes its store after its views are closed; WebKit reports the
  store in use until it has released their pages, so the removal is retried
  on that error from the main run loop (40 × 250 ms).
- **WebView creation:** `initWithFrame_configuration`, per-view delegate
  (`WKNavigationDelegate` + `WKUIDelegate`), KVO `title` observation
  (`observe_title` registers with `Initial` so the shell learns startup state
  even mid-load; the callback re-emits full navigation state when title lands).
- **Delegates:** `didStartProvisionalNavigation` (loading=true + state),
  `didCommitNavigation` (state), `didFinishNavigation` (loading=false + state),
  `didFailProvisionalNavigation`/`didFailNavigation` (`NavigationFailed` with the
  localized description), `webViewWebContentProcessDidTerminate`
  (`PageCrashed`). `createWebViewWithConfiguration…` extracts the live request,
  runs the shared `NewWindowCoordinator`, and publishes one decided delivery to
  the app. It still always returns `None`: neither the delegate nor WebKit
  creates an untracked BBR session. The coordinator reports blocked outcomes
  and delivers every request/outcome pair exactly once.
- **Authentication challenges:** WebKit hands the delegate a
  `WKNSURLAuthenticationChallenge`, an `NSProxy` that forwards to the real
  challenge; `auth_challenge::real_challenge` messages the forwarding target
  (objc2's debug message check finds no methods on the proxy class). A
  client-certificate request goes to `client_identities` (blueprint D12).
  HTTP Basic, Digest and NTLM for an HTTP(S) origin or proxy go to
  `http_auth`: the handler waits in the shared `HttpAuthTable`, and
  `WebEvent::HttpAuthRequested` carries the portable `ProtectionSpace`,
  scheme and failure count. The app records the challenge in its
  `AuthCoordinator` bound to that request, presents the credential sheet,
  and answers through `WebEngine::answer_http_auth`: the submitted
  credential leaves the app's memory with the answer
  (`NSURLCredentialPersistence::ForSession`, kept by the profile's own
  network session, so another profile is asked again); Cancel, a superseded
  challenge and a destroyed view answer without a credential, so the
  server's 401 page loads. Every other challenge — server trust above all,
  Negotiate, HTML forms — keeps WebKit's default handling.
- **Actions:** `loadRequest`, `goBack`/`goForward` (guarded by `canGoBack`/
  `canGoForward`), `reload`, `stopLoading` + explicit state re-emit (`Stop`).
- **Main thread:** all entry points re-assert with `MainThreadMarker`; the
  `MainThreadOnly` thread-kind is enforced at bind time.
- **Real input (testkit only):** `input_events` builds real mouse and key
  `NSEvent`s for the harness and sends them through
  `NSApplication.sendEvent:` (never to the OS), refusing a window that is
  not key; see E2E_ARCHITECTURE §7s.
- **Accessibility display options** (`accessibility_display.rs`, v2 S1):
  `current_options()` reads `NSWorkspace`'s public Increase Contrast and
  Reduce Motion values; `AccessibilityDisplayObserver` registers one
  receiver for `NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification`
  on the workspace notification center and removes it on drop. The app's
  appearance authority (`bbr-app/src/app/appearance.rs`) drains the change
  signal through a smol channel into a foreground task, re-reads the options
  and composes `Theme.contrast` and `Theme.motion_mode`; nothing polls. The
  testkit `SetAccessibilityOverride` request forces either option, and the
  snapshot's `shell_modes` reports `contrast` and `reduced_motion`.

### Renderer availability and public process boundary

`bbr-browser::ResponsivenessMonitor` is the single portable availability
policy. Every live session owns `Unknown | Responsive | Unresponsive |
Crashed`, a monotonic probe generation, bounded deadline, cadence, and timeout
backoff. Only visible sessions are probed. Adapter completions and deadlines
arrive as `WebEvent::UnresponsiveChanged`; unknown, destroyed, recreated, or
stale-generation observations are dropped. `PageCrashed` remains the existing
crash authority and triggers a bounded sibling reconciliation sweep because
WebKit may share renderer processes.

`bbr-web-macos::responsiveness` evaluates only `1 + 1` in
`WKContentWorld::defaultClientWorld` and arms a 2 s app-process `NSTimer`; both
paths are main-thread and nonblocking. A timeout hides the exact active native
slot under the GPUI §11.8a pane without destroying it. Wait retains the
session, Reload uses the sole controller destroy→recreate path, and Close uses
ordinary tab close. `WKUIDelegate` alert/confirm/prompt callbacks are answered
immediately; the pinned public SDK provides no WKUIDelegate beforeunload hook,
so chrome close destroys the WKWebView without executing page scripts.

The public lifecycle boundary is a split evidence chain (renderer lifecycle
acceptance decision). BBR's own retained `WKNavigationDelegate`
implementation is tested by direct invocation of the documented
`webViewWebContentProcessDidTerminate:` selector and by mapping the
documented `WKErrorDomain`/`WKErrorWebContentProcessTerminated` `NSError`
through both navigation-failure callbacks to the same portable termination
authority — a `PageCrashed` event plus the classified `ProcessCrash`
incident, never a misleading `NavigationFailed`. The portable controller is
tested with sequential multi-view `PageCrashed` bursts in varying orders
(no one-process-per-tab or event-count/order assumption): each affected
session is destroyed exactly once, persistent tab/profile identity survives,
a split never publishes a false live pane after a member crash, every
still-live sibling is re-probed independently, and duplicate/unknown/
destroyed/old-incarnation events fail closed. Real-app recovery enters
through the same productive typed event pump as native events
(`InjectWebEvent(PageCrashed)`), which proves only BBR-owned transport. The
external WebKit → callback/error hop remains an explicitly unpassed Apple
platform contract (A5): WebKit decides that a content process terminated and
invokes its documented callback or returns its documented error; no
acceptance gate enumerates, selects, terminates, or infers ownership of a
`com.apple.WebKit.WebContent` helper, and no private selector, KVC, helper
PID, or process operation is used. `WKProcessPool` is
deprecated/behavior-neutral and is never used to promise one renderer per
tab.

---

## 8. `bbr-app` composition root

`crates/bbr-app/src/main.rs` (package/binary name `bbr`)

```text
1. Parse CLI flags (--bbr-*):
     --bbr-initial-url    initial load (default "about:blank")
     --bbr-profile-dir    profile dir; default ~/Library/Application Support/
                          <bundle id> ($HOME, then the main bundle's id;
                          an unbundled binary takes dev.patwoz.bbr.dev in a
                          debug build, dev.patwoz.bbr otherwise), created
                          before the store opens (`profile_location`)
     --bbr-test-token     secret; presence ALSO enables non-persistent stores
                          and (with the testkit feature) starts the gateway
     --bbr-hidden         E2E: never show, focus, or activate the window
                          (the harness window is invisible to the host's
                          window manager; see E2E_ARCHITECTURE.md)
     --bbr-ids=sequential|random   IdGen mode (SequentialIdGen/RandomIdGen)
     --bbr-clock=system|fake       parsed; still a no-op (the command debounce
                                   uses GPUI's platform timer and a testkit flush
                                   seam, not a product-wide TimeSource)
     --bbr-window-size=WxH         initial window size (default 1080x700)
2. Build the id generator and inject it as the controller's WebViewId allocator.
3. `gpui_platform::application().run`: call `gpui_component::init(cx)` first, then
   open one window (centered, options from flags). The window root is
   `gpui_component::Root`; the `BbrApp` entity is built first in the
   `open_window` callback and retained separately
   (`Entity<BbrApp>` + `WindowHandle<Root>` seam, see `crate::with_app`).
4. In the open_window callback: capture the GPUI root NSView, build the engine,
   create `BrowserController(engine.clone(), allocator)`, create `BbrApp`,
   retain its entity, and wrap it in `Root::new`.
5. Open the initial controller-backed tab after the window exists (via the seam).
6. install_actions: bind Cmd-L/Escape/Cmd-[ /Cmd-]/Cmd-R/Cmd-S, Cmd-T staging, the focus-preserving Cmd-W, command-result arrows/Enter/Cmd-Enter, and the tab-organization commands (Cmd-Shift-P pin toggle, Cmd-Shift-↑/↓ reorder, Cmd-Shift-M staged move) — every handler routes through `with_key_window` so actions address the key window's shell with `&mut Window` access from its Root window.
7. Install process-lifecycle ownership: closing the last browser window
   triggers `cx.quit()` even on macOS (where GPUI otherwise stays resident
   after the last window), and terminal `SIGINT`/Ctrl-C is received
   asynchronously then routed onto GPUI's foreground/main thread for graceful
   `cx.quit()` teardown.

   Blueprint D19: `crate::browser_windows` owns the browser windows. Its
   `Registry` (a shared handle every shell holds) maps each window's key to
   its `WindowHandle<Root>` and `BbrApp` shell and remembers the window that
   last took input. `install_lifecycle` registers the first window and one
   `on_window_closed` observer: only registered browser windows count, so the
   native Portal host, Settings and About windows never quit the app (and the
   Portal never keeps it alive). A closed browser window's shell releases its
   controller context (its tabs stay) and engine root; closing the last one
   flushes the structural browser model and the Saved Page provider, then
   quits. `install_sigint_shutdown` installs an `async_signal` listener on
   GPUI's foreground executor whose handler performs no direct native
   mutation; the find bar and auth sheet are in-window elements and never
   produce an independent window-closed event.
   Under the testkit feature the gateway flushes the `Quit`/`CloseWindow`
   response through the connection's sole writer before the shutdown handler
   runs, so a successful response can never race process/socket teardown
   (unit-tested in `bbr-testkit`; the real-app `e2e_shutdown` suite proves
   both user paths through real process exit).
8. Compose productive new-window handling before opening sessions: install
   `bbr_web::default_decision` in `WebEngineWk`, forward each
   `NewWindowDelivery` over the main-loop channel, and drain it into the same
   `BbrApp::on_new_window_request` method used by typed testkit injection. That
   method records request/open/block counters and delegates all tab/session
   creation to `BrowserController`; native and test paths therefore cannot
   acquire separate execution authorities.
9. Under testkit: start the gateway, pump engine events into the controller,
   and broadcast AppReady, focus, WebEvents, and semantic BrowserChanged events.
```

All global actions, the event/ranking pumps, testkit dispatch, snapshots,
focus sinks, and presentation hooks address the app through the Root-window
plus app-entity contract (`crate::with_app`): the root type is
`WindowHandle<gpui_component::Root>` everywhere; the separately retained
`Entity<BbrApp>` is updated inside the Root window's update closure. The
testkit keystroke/draw arms still use the untyped `(**window).update` path
because a forced draw re-borrows the Root entity during the render.

Several windows (blueprint D19, task packet `multi-window`): every browser
window is a `Root` over its own `BbrApp` shell, built by `BbrApp::assemble`
from `SharedParts` (the first window from the composition root, every
further one by `browser_windows::new_window` from the current shell's parts,
with its own engine root, Portal host, pointer observer and find channel).
`assemble` lists every field, so each new field decides whether it is
shared or per window. Keyboard and menu actions run in the key window
(`with_key_window`; the current window under the hidden harness), engine
events in the window whose root holds their view (`with_view_window`), the
responsiveness cadence in the current window (the others re-assert their
unresponsive panes), and the testkit in the current window (`SelectWindow`
changes it). Each shell observes the others and re-renders only when the
shared generation moved since its last frame: repeated engine reports
(URL, title, history flags, probe answers — told apart by
`BrowserController::view_fingerprint`), cadence counters and derived-state
repair use bookkeeping borrows that do not move it.

Shared state (blueprint D19, task packet `multi-window`): `BbrApp` is a
window's shell. Everything that belongs to the browser rather than to one
window — the controller, organization, store, media, error surfaces,
unresponsive panes, auth challenges, preferences and Settings state, menu
state, favicon mirror and its disk cache (`crate::favicon_cache`, the
controller's per-profile records kept in `favicons/` of the profile
directory and seeded back at launch), external-URL adapter, and the shared parts of
downloads (attempt store, reveal cache), permissions (policies, requests,
per-view marks), client certificates (choices, requests, load marks) and
zoom (site levels, applied levels) — sits behind a `crate::shared::Shared`
handle that every window's shell clones. `read()` borrows for reading (reads
may overlap), `get()` for changing (exclusive for the statement); a guard
lives to the end of its statement, so a statement that changes a handle
never touches it again, and Clippy's `significant_drop_in_scrutinee` keeps
guards out of `if let`/`match` scrutinees. The controller is reached through
`ControllerHandle`, which switches the controller to its window's context
(`BrowserController::view_window`) before each borrow, so every controller
access names its window. A changing borrow bumps the shared generation the
other windows re-render from.

Window persistence (blueprint D19): each shell publishes its presentation
row (content rect, rail) to the `browser_windows::Registry` when it draws;
any window's save writes every open window's row to the store's `windows`
table by position (ids above every domain id), and every window's active
space and per-space selections to `windows.txt` beside the store
(`crate::window_layouts`; no schema change). At launch the first saved
window's selections replace the model's per-space active tabs in the
restore intent and its space is the hydrated one (the policy default only
without the file); `browser_windows::restore_windows` then reopens the
others through `BrowserController::restore_window`, giving each its native
root before `refresh_sessions` moves its pages in (the engine also parks a
page placed in a root that is not attached yet until it is). The file is
written synchronously and the store in the background, so a selection the
store never committed keeps the store's own active tab. The last window's
close flushes while it is still registered, so its row is written.
A window closed while others stay open is kept in the registry with its
layout and row (`push_closed`); the window that takes over shows the
Reopen toast, and `browser_windows::reopen_window` brings it back through
the same `restore_window` path. ⇧⌘T reopens that window only while the
domain's monotonic `closed_tab_count` has not moved since it closed.

A tab drag stays with the window it began in (the pointer events keep
coming to it, in its coordinates). `app::window_drag` turns every move into
a screen point and hit-tests the browser windows front to back (the window
server's stack, else the order they last took input): over another window
the source drops its own target and that window previews the drop — a rail
slot resolved against its rendered tab rows (one rect recorder per row) or
a D23 split zone over its card — and the release commits there through
`BrowserController::drop_tab_here` (move into that window's active space,
cross-profile recreation included, then the slot and the selection), plus
the split insert for a card zone.

The chrome (`crates/bbr-app/src/app.rs` + `app/render.rs` + the split
`app/sidebar.rs`/`app/sidebar_view.rs`/`app/spaces.rs`/`app/tab.rs`/
`app/interaction.rs` + the `shell/` presentation modules) renders the
Chassis-tokenized sidebar (tokens/ramps/components from `bbr-ui` — never
local magic values): brand block, command bar (mirrors/selects the live
URL; owns a UTF-8-safe cursor/selection model; accepts
typing/backspace/arrows/Enter/Cmd-Enter), the Design §11.5 local result
surface, the controller-backed tab surface, a status line
(`shell::status_line`, incl. "⚠ Web content process terminated"), and the
sidebar toggle. The G3 Sidebar composition cutover made `bbr-ui::sidebar`
the one rail authority: `SidebarModel` + the one `Callbacks` set + the
pure `geometry` width model + the `sidebar_surface` composition live in
`bbr-ui`, while `bbr-app`'s `app/sidebar_view.rs` is the thin assembler
(builds the model from the controller snapshot, fills every callback slot
with the production app methods, and passes the three app-owned child
elements — the §11.4 command field anchor, the §11.5 in-rail command
results, the §11.12 downloads panel — at the composition boundary).
`app/sidebar.rs` keeps only the production input wiring (pointer drag,
focused-divider keyboard equivalents, the shared width mutation).
`CommandBarMode` separates `Inactive`, current-tab `Location`,
and staged `NewTab` intent; `NewSpace`/`MoveTab` keep their existing staged
paths. Location/NewTab ranking rebuilds a provider-neutral corpus from the
controller's complete persistent tab/space/profile read model, waits on a
cancellable 30 ms GPUI platform timer, and publishes only the latest generation.
History is intentionally an empty real collection until Phase-7 composition;
plain text receives the settled encoded DuckDuckGo fallback. Activations route
through app/controller seams, including current-space re-resolution by
persistent tab identity, so warm sessions and profile isolation remain
controller-owned. Escape restores the explicitly captured origin text/mode and
chrome/web focus domain. The content area renders one
`NativeWebViewSlot` per visible pane from the controller's authoritative
`pane_layout()` (a single full-width slot, or two equal adjacent horizontal
slots for a split — no divider/polish in this anchor), or the empty-space
pane (§11.8) and crash panel (§10.7) when no live pane exists.
The sidebar rail itself is the adjustable, collapsible `SidebarGeometry`
(`bbr_ui::sidebar::geometry`): DESIGN §5/§11.4 min/default/max widths
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
active tab's pin state (the Tab menu's Pin/Unpin row and the §11.2 drag
offers are the pointer halves; rows carry no pin control),
`Cmd-Shift-↑/↓` step the active tab within its region (a boundary
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
warm. Pinned tabs are kept places (DESIGN §10.12): `pin_tab` captures the
tab's `PinnedPage` (URL and title), and `close_tab` on a pinned tab closes it
in place as `TabLifecycle::Closed` — same index, no session, URL/title reset to
the pinned page — and pushes an *in-place* `ClosedTab` record carrying the page
it closed on (`closed_pins.rs`). `reopen_tab` pops records in close order: a
regular record re-inserts the tab, an in-place record restores that page in the
same row and selects it. A selection reopens the row at its pinned page; every
path that opens, unpins or moves a closed row drops its in-place record
(`forget_reopened_in_place`, run by `sync_lifecycles` and `unpin_tab`), and a
space deletion closes the row onto the fallback's stack as an ordinary record.
Neighbor selection skips closed tabs and the session budget never counts them;
`unpin_tab` forgets the pinned page and turns a closed tab into a suspended
regular one; `unpin_tab_to` (`unpin_to.rs`) does the same and places the tab
at a regular-region position in one transition — the drop of a pinned row or
a Favorite tile dragged back among the regular tabs (task packet
`drag-back-out-of-pins`). A public `check_invariants()` verifies profile/space/tab
cross-references, pinned ordering, active-tab and lifecycle visibility
agreement, `closed => pinned`, `pinned page <=> pinned`, split membership and
distinctness, recently-closed exclusion (an in-place record instead names a
closed pinned member, once), and unique ids. The companion
`FolderStore` keeps folders and the Favorites grid beside the `Browser` with
`folder member => pinned`, `favorite => pinned` and `favorite => no folder`:
favoriting pins a regular tab and ungroups a member in one transition, so the
presentation lists a Favorite as its tile only.

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
reset parity, Cmd-S collapse/expand with last-expanded-width restoration, and
compact-window clamping that keeps the native WKWebView geometry strictly
positive — proven by `just e2e-shell-sidebar` and the `sidebar_resize` /
`sidebar_restore` performance scenarios. The tab-organization slice adds the
keyboard commands (`Cmd-Shift-P` pin toggle, `Cmd-Shift-↑/↓` region-bounded
reorder, staged `Cmd-Shift-M` move), the Tab menu's Pin/Unpin row and §10.9
drag tick, real pointer drag through the production press/move/release listeners,
regular → pinned drops rejected whole (the ghost row pins), pinned rows and
Favorite tiles dragged back out of the pinned area (`DragTarget::Unpin`,
`DragOrigin::Favorite`, resolved against the recorded row and New Tab rects;
`app::drag_back`), same-profile moves that keep the
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
  `Suspended`/`Discarded`/`Closed` tabs own none and recreate one on
  activation (a closed pinned tab at its pinned URL).
- Closing, suspending, or crashing a tab destroys its session; moving across
  profiles destroys and recreates it in the destination profile's store, while
  same-profile moves keep it.
- Only visible tabs (the single-pane selection or both split panes) are shown.

`BrowserController` exposes the first vertical slice (open/select/close/reopen),
navigation routing to the focused tab or split pane, suspend, profile moves,
split open/close with no-session-orphan teardown, productive new-window tab
creation, and `apply_event` for `WebEvent`s (URL/title map to the owning tab;
loading/history/crash/failure are tracked per session in `NavState`; unknown or
stale `WebViewId`s are reported as `ControllerEvent::UnknownWebView` without
corrupting state). `open_tab_from_new_window` resolves the exact source session,
validates its owning profile against the decided session, and opens one regular
tab in the source Space/Profile. An inactive source Space stays inactive and
its fresh warm session stays hidden; stale sources and profile mismatches fail
before domain or engine mutation. `open_background_tab_from_new_window` is the
background-link sibling (Command-primary / middle click, DD-LINK): it inserts
one regular child through the domain `insert_tab` primitive (positional,
never selection/split-mutating) at the identity-bound grouped index — the
first child lands immediately after the exact opener and later children after
the last surviving member of the opener's contiguous child group in click
order, computed from `OpenerProvenance` (never from URL/title/render index or
hidden adjacency). The opener stays selected, visible, and web-focused; the
child is a warm hidden session in the source Space/Profile. Every successful
mutation returns exactly one typed `ControllerEvent`, and a read-only
`snapshot()` publishes the tab list, active view, and live-session count for
UI and E2E.

Testability: the deterministic `FakeEngine` (in `crate::fake`,
`#[cfg(test)]`-only) records every engine call into an op log — including
keyed geometry ops (`SetRect { view, rect }`) — owns fake sessions, rejects
duplicate creation and missing-session actions, and emits controlled
`WebEvent`s through the registered sink. Because the controller drives the
engine through the same `dyn` seam the app uses, 230 crate tests prove
registry, controller, event-routing, active-space visibility, profile move,
split, lifecycle, snapshot policy, and the authoritative `pane_layout()`
query without a display server.

The controller exposes a public plain-value [`PaneLayout`]
(`primary`, `secondary: Option<WebViewId>`, `focused`) via
`BrowserController::pane_layout() -> Option<PaneLayout>`, derived **only**
from the active space's domain split, its active tab, and the session
registry: no active tab/session → `None`; single pane → one view; split →
two ordered views with the domain-active member as focus. A split layout
**requires both live sessions**: a missing primary or secondary session
never degrades an open split into a false single pane — it publishes no
layout at all until the split dissolves or the session is recreated. The
app owns no independent focused-pane state — the focused pane is the
browser command target because it is the domain active tab's view. This is
the serial composition surface the split-core lane consumes.

## 9b. `bbr-persistence` — schema v3 global Saved Pages (B1 packet)

B1 adds the portable Saved Page contract and its durable schema-v3 store
(`bbr-domain::saved_pages` + `bbr-persistence`). A Saved Page is a **global
installation URL record**: no `ProfileId`, space, tab, session, cookie,
credential, history, snapshot, or favicon blob — it survives closing tabs,
spaces, windows, and restart, and B2 opens it as exactly one fresh regular
tab in the currently active Space/Profile.

**Canonical URL seam (one authority).** `bbr-domain::saved_pages::canonical_url`
reuses `bbr_web::normalize_url` for scheme defaulting, then ASCII-case-folds
only the scheme and host. Path, query, fragment, explicit port, trailing
slash, `www.`, and percent-encoding are preserved byte-for-byte; default
ports are never elided and no bookmark-only rewrite exists. Supported schemes
are exactly `http`, `https`, and `about`; userinfo credentials in the
authority reject whole, so records are credential-free by construction and
`Debug` is privacy-safe. Save/lookup/remove/search share this one typed
`CanonicalUrl` seam; persistence stores the canonical key **opaquely**
(bounded `valid_url` only) and enforces at-most-one-active-record-per-key
through the `UNIQUE` column plus write-side validation.

**Schema v3.** (Superseded by v4, §9e.) The v3 DDL adds `collections` and
`saved_pages` (canonical key `UNIQUE`, `collection_id` a strict foreign key
into `collections`, dense `position` order keys). Per the standing development
rule there is **no migration and no backward compatibility**: a store whose
stamp or shape does not match is discarded and recreated empty (`StoreLoad`
records the disposition), and matching valid reads perform zero writes.
Same-version corruption — a broken shape, duplicate canonical keys, or a
dangling collection reference — is refused read-only and discarded on write.

**Repository contract.** `Repository::store_saved_page` is idempotent
(re-save returns the existing `BookmarkId`, no second row, preserves
`created_at`, never touches a user label, and updates the captured title only
when no user label exists and the title changed; a byte-identical re-save
performs no write). `saved_page_by_id`/`saved_page_by_url`/`load_saved_pages`/
`load_collections` are read-only; `search_saved_pages` is a deterministic
case-insensitive literal-substring search over title/label/URL. `update_saved_page`
applies field updates, `remove_saved_page` is an idempotent no-op on unknown
ids and re-compacts dense positions, `replace_collections` rewrites the flat
collection set whole (removal while referenced rejects; id-preserving renames
use deferred foreign keys), and `move_saved_page_collection` moves a record
into or out of one flat collection. The `Snapshot` aggregate carries the new
collections; browser-state writes never touch the Saved Pages tables.
Deterministic order is `(updated_at, id)` ascending in the domain
(`SavedPage::sort_key`), with persistence preserving caller order in dense
`position` keys.

Evidence: 117 `bbr-domain` tests and the focused `repository_saved_pages`
contract target; checked-in `schema_v3`, `schema_v3_corrupt`,
`schema_v3_duplicate_url`, and `schema_v3_collection_integrity` fixtures;
release `repository_benchmark` with a ≥1,000-record Saved Pages
write/list/search distribution. B2 owns app composition, Cmd-D, the command
provider, menus, and fresh-tab E2E; B3 owns the Library.

## 9c. App-owned Saved Page command + Palette provider (B2 packet)

B2 composes B1's global snapshot into the one AppStore and adds one typed
app-owned command authority (`SavedPageCommand`) that Cmd-D, the Command
Palette, and the later Z1/Z3 menu rows share. The provider is the **single
runtime authority**: loaded once at startup from the schema-v3 repository
into an in-memory snapshot (`SavedPagesProvider` in
`crates/bbr-app/src/store/saved_pages.rs`), updated only after accepted
save/remove mutations, and never queried from SQLite during ranking or
render. Persistence flows through the same bounded nonblocking writer
discipline (capacity-1 poke, background coalesce, one
`replace_saved_pages` transaction; the testkit flush and the production
owning-window-close quit flush write synchronously through the identical
repository call, so a normal quit never loses a committed save).
`bbr-command` stays provider-neutral — the corpus carries the loaded
snapshot and never imports persistence.

**Semantics (settled B2 contract).** Saving is canonical-URL-idempotent
(duplicate returns the existing `BookmarkId`, never a second record);
removing never mutates a tab and closing the source tab never removes the
record (a Saved Page carries no `TabId`/session link); missing, malformed,
dangerous, internal-unsupported, or non-navigable URLs reject or omit the
action with the canonical seam's typed reason. `Cmd-D` toggles the active
page's saved state. Opening revalidates the record in the in-memory
provider, then appends exactly one **fresh regular tab** to the currently
active Space/Profile through the existing NewTab/navigation authority — it
never silently selects an existing matching tab, switches Space/Profile,
pins, favorites, splits, or reuses a session; the Space's current Profile
supplies the native website-data store.

**Distinct result kind.** The corpus carries a `SavedPageEntry` with
`BookmarkId`, display title (label → captured title → URL ladder), the
canonical URL, the optional flat collection label, and a stable
`saved_page:<id>` key. Ranking places the `SavedPage` group exactly between
standalone Spaces and History: `ExactTab < CurrentPinned < CurrentTab <
OtherSpace < Space < SavedPage < History < Action < Fallback`; no existing
group's relative order changes. Deterministic search consumes only the
loaded in-memory snapshot. `Cmd-D` and the availability/mutation seams are
what Z1 (Tab/Favorite menus) and Z3 (page-background menu) consume without
duplicating the authority.

Evidence: `just e2e-shell-saved-pages` 4/4 real-app scenarios (idempotent
Cmd-D toggle with typed saved-state/repository agreement; save → close
source tab → restart → Palette find with no original `TabId`/session;
exactly-one-fresh-tab activation in the active Space/Profile; production
quit durability — out-of-band store clear + owning-window close re-persists
the provider and a same-profile restart restores exactly one record); the
optimized 30-sample scenarios `saved_page_search_first_result` (p95
41.0 ms / max 41.43 ms against the warm command-text 50/100 budget) and
`saved_page_open` (p95 27.13 ms / max 40.93 ms against the
session-recreation 100/150 budget) over the seeded ≥1,000-record provider
with `just e2e-perf` green (`regressions: []`); the shared ranking contract
suite in `bbr-command`. Z1/Z3 own their menu placement/source/focus/Portal
behavior; B3 owns the Library.

## 9d. App-owned Saved Pages Library surface (B3 packet)

B3 composes the settled chrome into one ordinary BBR-owned GPUI Library
surface for the global Saved Page collection — hundreds or thousands of cold
URL records, never a permanent Sidebar region and never a web page, Portal,
WebKit view, or copied Gallery UI. `bbr-ui` owns only presentation
(`crates/bbr-ui/src/components/saved_pages_library/`: `model.rs` =
`SavedPagesLibraryModel` presentation data plus callbacks, `rows.rs`,
`header.rs`, `toolbar.rs`, `collection_surface.rs`, `surface.rs`, `ids.rs`);
it sees no repository connection, BrowserController,
WebEngine, native session, or network client. `bbr-app` owns snapshot building
and command binding (`crates/bbr-app/src/app/library/`, the AppStore
`store/saved_pages_store.rs` + `store/saved_pages_policy.rs`), so there stays
exactly one Saved Page identity/persistence authority (B1) and one command
authority (B2 `SavedPageCommand`) — the Library adds no second authority.

**Production surface.** The integrated entry point is Cmd-Shift-L; Escape
dismisses the internal chrome surface, which covers the native view while
open and restores it exactly on close (hidden-view bookkeeping). Search,
deterministic sort (fixed title/comparison orders), inline pointer rename with
draft confirm/cancel, multi-select, a clear destructive remove confirmation
(for single and multi-select, never closing open tabs), and one-level flat
collections with move-in/move-out ride the in-memory snapshot and persist
through the one AppStore/repository writer (`replace_collections` and
`move_saved_page_collection`, B1 contracts). Opening any row routes through B2
and creates exactly one fresh regular tab in the currently active
Space/Profile; it never silently selects a matching tab.

**Bounded rendering and semantics.** At least 1,000 records use bounded/
windowed rendering: `visible_window(total, viewport_rows, scroll_offset)`
clamps a typed window, `library_visible_ids` (the test/testkit mirror, in
`crates/bbr-app/src/app/library/model/visible_ids.rs`) publishes the
visible-id set, and
no full-list layout, synchronous repository scan, or favicon network work
happens in render. Favicons resolve through the production favicon path with
the deterministic monogram fallback; no favicon blob is stored in records.
Typed shell semantics (`crates/bbr-app/src/shell/semantics/library.rs`,
row-level nodes in `library/row.rs`) and
the testkit `SavedPagesLibrarySnapshot` expose open flag, search/sort, the
bounded row window, selection/collection membership, and the remove-confirm
state for UI and E2E. Accessibility follows DESIGN §11.16: keyboard
focus/order, semantic names, contrast, reduced motion, and the stable
`bbr-saved-pages-*` ids.

Evidence: the final semantic-union runtime tree
`b4ab98113c6582caad2150764815d3885589dfa7` passed
`just e2e-shell-saved-pages-library` 12/12 real-app scenarios: entry/Escape,
Cmd-T/Cmd-L replacing an open Library, deterministic search/sort, inline
rename confirm/cancel, multi-select + destructive remove with tabs untouched,
flat collection lifecycle, pointer selection-toolbar clear/remove/move,
pointer collection create/rename/remove surface, production scroll-wheel
path, restart persistence, exactly-one-fresh-tab activation, and the 1k bounded
window/scroll/selection with long/missing fallbacks (log
`/tmp/bbr-b3-review-remediation-e2e-shell-saved-pages-library-20260810T144111Z.log`,
SHA-256 `06838f3556a34bb46d7dfe20f8b5f6f3bdd3b6d7ce91a19a59b633dfffa11673`).
The same tree passed `just e2e-shell-saved-pages` 4/4 (log
`/tmp/bbr-b3-review-remediation-e2e-shell-saved-pages-20260810T144301Z.log`,
SHA-256 `440317455fbf499fada4515d404c1f460d0a5854a817490314e5ac65763273cc`)
and the integrated Gallery 22/22 with real Tab Row id 22 plus Library ids
23–30 (log `/tmp/bbr-b3-review-remediation-e2e-gallery-20260810T144301Z.log`,
SHA-256 `bc41dd1df11d887dea03a10fa30949a33be05edb916462c59fd9cbd25a2ff7e9`).
The command surface regression suite also passed 25/25 (log
`/tmp/bbr-b3-review-remediation-e2e-shell-command-20260810T144111Z.log`,
SHA-256 `953138116c7ad011402cba6a8269c75c9fed690cb4545df52d940a132bcc7b6f`). The
`saved_pages_library_search`/`saved_pages_library_scroll` scenario drivers,
fast-but-wrong guards, and budget classes exist but have **zero executions** —
no optimized sample accepted, performance unpassed under the active delivery
override.

---

## 9e. Space appearance persistence — schema v4 (v2 S10)

`SCHEMA_VERSION` is `4`: `spaces` gains `tint_intensity` (closed
`subtle|standard|rich`) and `grain` (boolean) beside the v2 `tint`/`glyph`
identity columns. `bbr-persistence` mirrors the closed sets
(`TintIntensity`, `SpacePaper`) without depending on `bbr-domain`; an unknown
value is corruption, never a fallback. Older stamps are discarded under the
closed version-bump policy (no migration).

The whole curated appearance now persists. The domain keeps it in the
`SpaceAppearanceStore` beside the `Browser` (theme, identity and the
`SpacePaper` intensity + grain), so the portable `RestoreIntent` stays
appearance-free: the app's `store::appearance` module maps the rows
variant-for-variant (a new token on either side fails to compile), the
browser-state write takes the controller's effective appearance per space,
and startup applies the persisted rows after `BrowserController::restored`
through `restore_space_appearance`, which keeps a record only where it
differs from the derived default ("defaults never create records"). The
presentation resolves the intensity in `bbr-ui::theme::space_tint(token,
intensity, theme)` and paints the static grain from `components::grain` on
the rail and chassis.

## 9e2. Kept places and organization persistence — schema v5

`SCHEMA_VERSION` is `5` (task packet `closed-pins-and-favorites`): `tabs`
gains `pinned_url`/`pinned_title` (the pinned page; both or neither, only on a
pinned tab) and `closed`; `recently_closed` gains `in_place` (the record names
a closed pinned live tab of the same space and reserves no identity of its
own); and three organization tables follow — `folders` (id, space, title,
per-space order), `folder_members` (tab key, folder, per-folder order) and
`favorites` (tab key, global order), each with strict foreign keys. Write
validation (`validation_organization`) and restore
(`repository_organization`) enforce the same rules: a member is a pinned tab
of its folder's space in one folder, a favorite a loose pinned tab listed
once, a closed tab is never a selection or split pane (restore projects such
a soft reference away with a typed warning). Both browser writes share
`repository_browser_rows`. Older stamps are discarded (no migration).

The `RestoreIntent` carries the tab facts (`pinned_page`, `closed`,
`in_place`): `HydrationPolicy::apply` calls
`RestoreTarget::restore_kept_place` after each pinned tab, a closed tab never
hydrates a session, and in-place records restore through
`Browser::restore_in_place_closed`. The organization stays beside the
controller: the app maps the persisted folders/favorites through
`store::browser::persisted_organization`, restores them after the controller
with `OrganizationController::restored` (`FolderStore::restored` keeps folder
ids and checks every invariant whole; a rejected organization starts empty),
and writes the reconciled export `OrganizationController::persisted` with
every structural persist. Folder, membership and favorite commands persist
like any structural mutation; the folder disclosure state stays session-local.

## 9f. Motion core and transient moments (v2 S6–S8)

**One timing authority.** `bbr-ui::theme::motion` owns the §6 easing curves
and the `Moment` descriptors (`Moment::ALL`, §18.5 order). `Moment::plan
(MotionMode)` returns the one `MomentPlan` (duration, easing, loop or once,
opacity-only reduced flag) or `None` — "render the static counterpart and
request no frames". Every animated element is built through
`MomentPlan::animation()`, which also counts it (`animated_elements_built`),
so a frame's animation work is observable render-side. `Theme.motion_mode`
comes from the appearance authority (§S1 accessibility display options or
the testkit override); components read it from the theme, never from AppKit.

**After-commit rule.** A moment only decorates a state that is already
committed and presented: descriptors carry no delay and the committed state
paints on the commit frame whether or not anything animates. The app's
`moments.rs` registry starts one-shot moments (Favorite pop, download check,
⌘T entrance and glide, chrome tint fade) with a unique key per start, keeps
one instance per kind — a newer commit supersedes the older decoration even
when the newer one is static — and ends each with a timer, after which the
render passes no key and builds no animation element. Loops exist only for
loading (the favicon arc, the D5 address progress sweep while the active
load runs).

**Pane illustrations (D10).** The render decides which content branch wins,
so it also observes the presented illustration kind (`content_illustration`)
and keys its appearance; a new kind restarts it, and after the two 1400 ms
cycles the render passes `None`, so the rest frame builds no animation
element.

**Sidebar slide (DESIGN §6 1a, Patrick 2026-10-17).** The one layout
transition. `bbr-ui::theme::motion::Transition::SidebarSlide` is its timing
authority (`dur:emphasis`, `ease:standard`, `None` under Reduce Motion), and
`bbr-ui::sidebar::transition` is the pure model: a `SidebarTransition` eases
the rail's *presence* (1 expanded, 0 collapsed) from the share on screen to
the committed share — a reversal starts at the presented share and covers
only the remaining distance — and `SidebarPresentation::resolve` maps a
presence to the rail edge and a `CardInsets` blended between the collapsed
and expanded frames, on whole points between the settled ends.
`toggle_sidebar` commits `sidebar_visible`, persistence and focus exactly as
before and then starts the slide (`app::sidebar_motion`). Each render
resolves the presentation, lays the full-width rail into a strip as wide as
the presented edge, and hands the card frame to the chassis, the native
chrome material and the Peek wash; GPUI then gives every
`NativeWebViewSlot` its interpolated bounds, so the WKWebViews are re-framed
every frame (`setFrame:`, never captured). While the slide travels the
render calls `request_animation_frame`; a timer for the slide's duration
notifies once more, so the committed geometry renders even when no
display-link frame arrives. A leaving rail carries an occluding veil and
blocks the edge reveal; `float_rail` ends a running exit. Semantic ids,
the semantic tree and every command path read the committed state, never
the presentation. The testkit reports `sidebar_motion` (variant, presence,
presented edge, settle ms and a bounded per-frame trace with the native
slot x) and `motion.transition_frames`, the frames the slide requested, for
the idle proof.

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
just e2e-shutdown # deterministic process shutdown: real SIGINT + owning-window close (portal-child isolation)
just e2e-tabs   # real multi-tab/session behavior
just e2e-profiles # real profile isolation, moves, and suspend lifecycle
just e2e-shell-tabs    # real vertical-tab surface: open/select/close/reopen
just e2e-shell-spaces  # real profile/space switching (pointer + keyboard)
just e2e-shell-sidebar # real sidebar drag/keyboard/collapse/geometry clamp
just e2e-shell-split-geometry # keyed per-WebViewId panes, focus, resize, close (split anchor)
just e2e-perf   # optimized startup and production-path interaction budgets
just e2e        # canonical sequential GUI gate (smoke, shutdown, tabs, profiles,
                 # shell-tabs, shell-spaces, shell-sidebar, shell-split-geometry,
                 # performance)
```

All recipes are wrapped in GNU `timeout --kill-after=10s` with
`BBR_CHECK_TIMEOUT`/`BBR_TEST_TIMEOUT` (5m) and `BBR_E2E_TIMEOUT` (2m)
overrides, and pin `DEVELOPER_DIR` to the installed Xcode 26.2.0
(`BBR_XCODE_DEVELOPER_DIR`). On headless CI, `BBR_SKIP_GUI_TESTS=1`
skips the smoke.

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
just run
```

`just run` invokes `mbx run -p bbr`, so development builds use the host's
managed, self-pruning Rust target storage. The recipe sets
`scripts/package/dev_app.py` as Cargo's runner: it wraps the debug binary in
`target/debug/BBR.app` (bundle id `dev.patwoz.bbr.dev`) and execs the bundle
executable in the terminal's foreground. A bare Mach-O has no bundle
identity: macOS shows a generic Dock icon, names the app menu `bbr` and, since
GPUI makes the process a regular app only in `didFinishLaunching`, does not
activate it when launched from a terminal, so the terminal keeps the menu
bar. The bundle supplies the name, the Phosphor-b icon and the BBR menu bar.
The first bundled launch copies the bare binary's WebKit site data
(`~/Library/WebKit/bbr`) to the new identifier once and refuses to start while
another `bbr` process could still be writing it. Extra arguments pass through:

```bash
just run --bbr-initial-url https://example.com
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
3. **`--bbr-clock` is a no-op**; the app now has one bounded time-based path:
   the cancellable 30 ms command-ranking debounce uses GPUI's platform-driven
   background-executor timer. Functional E2E can cancel and publish the current
   generation through the typed `FlushCommandRanking` seam; optimized command
   evidence measures the real timer and commits only after the result palette is
   rendered. A product-wide `TimeSource` remains deferred until persistence and
   broader scheduling need one.
4. **The MVP new-window seam and transient Peek preview are wired; richer
   disposition remains deferred.** Direct user activations open one regular
   tab in the exact source Space/Profile, or one eligible transient
   source-profile Peek preview, through `BbrApp` and `BrowserController`;
   script-created `window.open` requests are blocked and reported. The native
   delegate still returns `None`. Only the regular-tab path records
   session-local identity-based child→opener provenance. The N1 web-content
   gesture path dispatches exactly one real Back/Forward per committed gesture
   or runs the atomic Back-at-settled-root close-child/activate-opener
   transaction; Preview never records that provenance. Peek owns one transient
   session per pane and promotes that same live WebView session without reload.
   Modifier/background insertion policy, named-window reuse, popup allow-lists,
   and second browser windows remain later bounded work. Functional evidence is
   green for gestures 9/9, Peek 6/6, NW-SEAM 4/4, and Portal 7/7. Required
   optimized gesture and Peek scenarios remain explicitly unexecuted and
   unpassed under the active delivery override.
5. **No find-in-page, zoom, downloads, permission prompts, WebAuthn, or
   process-termination recovery UI yet** (Phase 8). `PageCrashed` is
   observable in the status line and snapshot, but there is no recovery
   surface.
6. **The release-exclusion failsafe is not implemented**: under a non-testkit
   build the `--bbr-*` test flags are silently ignored rather than rejected at
   startup (E2E §12's exit-64 gate is future work).
7. **The Saved Pages Library production surface is in with unit/E2E/Gallery
   evidence; its optimized performance remains unexecuted and unpassed.**
   `saved_pages_library_search` and `saved_pages_library_scroll` drivers,
   fast-but-wrong guards, and budget classes exist, but no sample has ever
   been accepted under the active delivery override (Patrick deferred
   current-MVP performance execution); the canonical `just e2e`/`just
   e2e-perf` gates were not run for the B3 slice.
