# BBR — E2E Architecture (current implementation)

Status: **describes what is implemented and verified** — the testkit gateway
protocol, the fixture server, and executable smoke + multi-tab tests that drive
the real `bbr` binary with real WKWebViews. Planned-but-unbuilt parts are
explicitly marked **FUTURE**; nothing here claims green checks without the
evidence in §9.

Inputs studied (still the design basis): Zen Browser's custom test stack at
`thirdparty/zen-browser-desktop/src/zen/tests` and the actual `gpui 0.2.2`
input-dispatch APIs (verified: `Window::dispatch_keystroke`, `dispatch_event`;
no public screen readback; `Window::focus` does not touch the AppKit first
responder).

---

## 0. Facts verified today (with evidence)

| Fact | Evidence |
|---|---|
| Workspace compiles and all unit/contract tests pass | `just verify` green: bbr-app 64, **bbr-domain 40**, **bbr-browser 75**, bbr-fixtures 4, bbr-testkit 18, bbr-web 10, bbr-web-macos 1 passed |
| Real-app GUI suites pass sequentially | `just e2e` → `SMOKE PASSED`, `TABS E2E PASSED`, `PROFILES E2E PASSED`: navigation/focus/resize, warm-session reuse and close destruction, cookie isolation, profile-boundary recreation, and suspend cleanup |
| All previously-missing files exist | `crates/bbr-app/Cargo.toml`, `crates/bbr-app/src/webview|delegate…` under `bbr-web-macos` are present; the workspace memberships are complete |
| `bbr-web` contract crate | `#![forbid(unsafe_code)]`, unit tests incl. `flip_rect`, `normalize_url`, object-safety proof |
| GPUI 0.2.2 public synthetic input | `Window::dispatch_keystroke(…, cx)` (window.rs) and `Window::dispatch_event(PlatformInput, cx)`; the smoke drives `cmd-l`/`escape` through the real input path |
| WebKit hermeticity hazard | `WKWebsiteDataStore::dataStoreForIdentifier` persists outside temp dirs → E2E uses `nonPersistentDataStore` per profile (set whenever `--bbr-test-token` is given); loopback `http://127.0.0.1` is ATS-exempt (no TLS) |

---

## 1. Design constraints (kept from the plan, now partially realized)

- **Real app, real WKWebView.** E2E drives the actual `bbr` binary with a real
  WebKit view; the mock engine (`NullEngine`) is only for non-macOS builds and
  platform-free unit tests.
- **Hermetic temp profile per test.** The smoke uses
  `target`-independent temp dirs (`$TMPDIR/bbr-e2e-smoke-<pid>/profile`); WebKit
  website data is non-persistent (see §0).
- **Deterministic local HTTP fixtures.** `bbr-fixtures` binds
  `127.0.0.1:<ephemeral>` and serves deterministic pages — ATS-exempt, no TLS.
- **Privileged test-only API.** The in-app *testkit gateway* (unix socket)
  drives navigation, input, focus, window size, snapshot, and graceful quit.
- **Synthetic keyboard input.** `DispatchKeystroke` goes through GPUI's real
  `dispatch_keystroke` path — no accessibility permission, no OS event posting.
- **Event waits/state snapshots, no sleep-based product assertions.** Web and
  focus state use sequence-numbered events with deadlines. Bounded short sleeps
  exist only for socket/process polling; resize uses bounded typed-snapshot
  polling until a semantic window-resized event is added.
- **No production network listener.** Gateway = loopback-unix-socket only,
  token-authenticated, `testkit`-feature-gated.
- **One host-wide GUI gate across worktrees.** Every bounded real-app and
  performance recipe runs through `scripts/with_gui_lock.py`; the lock key is
  Git's common directory, so sibling worktrees cannot overlap AppKit windows.
- **Harness windows are never shown.** Every `--bbr-hidden` harness run opens
  the window with `show: false` (only the interactive `just run` path shows a
  window). A shown harness window is visible to the host's window manager —
  on this host yabai BSP tiled the test window to the current space's tile
  frame (1213×1063, exactly the KeePassXC tile) a few frames after spawn,
  which the E2E geometry suites read as a late, order-dependent resize; the
  earlier ledger misattributed that to AppKit screen clamping. A never-shown
  window keeps the requested bounds deterministically; WebKit and GPUI
  rendering work without display (frames are driven by the `FlushFrame`
  paint path), and the `wait_for_window_settled` contract still waits for 4
  consecutive unchanged window-bounds + native-content observations so any
  remaining late AppKit geometry settles before absolute assertions.

---

## 2. Implemented E2E topology

```text
bbr-app/tests/e2e_smoke.rs + e2e_tabs.rs + e2e_profiles.rs (three #[ignore]d macOS tests)
  │  1. fixture server in-process (bbr-fixtures) on 127.0.0.1:<ephemeral>
  │  2. temp profile dir + artifacts dir
  │  3. spawn app: bbr --bbr-initial-url <page1> --bbr-profile-dir <dir>
  │        --bbr-test-token <tok> --bbr-ids=sequential --bbr-hidden
  │        --bbr-clock=system
  │  4. connect client → Hello → Attach(replay) → run steps → graceful Quit
  │  5. success: delete profile dir; failure: keep profile + artifacts
  ▼
app process (bbr binary, --features testkit)
  bbr-app (GPUI chrome)          bbr-fixtures (in-process, harness side)
  bbr-web-macos (WKWebView)
  bbr-testkit gateway ◄──unix──► bbr-testkit client (same host process)
```

The fixture server is in the harness process for the smoke (not a separate
process); the `bbr-fixtures` **binary** exists for the FUTURE runner topology
(prints `port` + `base_url`, then serves until killed).

---

## 3. The testkit protocol (implemented: `bbr-testkit`, version 4)

Single source of truth: `crates/bbr-testkit` (portable: unix sockets + serde
JSON; no macOS, no GPUI, no WebKit).

### Transport

- One `SOCK_STREAM` unix socket per app: `{profile_dir}/bbr-test.sock`, dir and
  socket created mode `0700`. A stale socket from a crashed run is removed on
  start. Frames are length-prefixed (`u32 LE` JSON), debuggable with `nc -U`.
- The server binds in `bbr-app` only under the `testkit` cargo feature
  (`bbr-app/Cargo.toml` `[features] testkit = ["dep:bbr-testkit", …]`; never in
  release profiles).

### Handshake & auth

- Client → `Hello { protocol_version: 7, token, capabilities }`. Wrong version
  → `Error("protocol_mismatch")`; wrong token → connection dropped. Correct →
  `Welcome { app_version, pid, capabilities, next_seq }`.
- Token comes from `--bbr-test-token` (random per spawn, in the harness).
- Every other request before a successful Hello is refused (connection dropped).

### Requests (implemented)

| Request | Effect |
|---|---|
| `Attach { since: Option<u64> }` | Replays seq-numbered events from the server ring with seq > `since`; live delivery continues after |
| `Navigate { url }` | Load the controller's active tab (same path as the command bar) + broadcasts `CommandBarChanged` |
| `CreateProfile` / `CreateSpace` / `ActivateSpace` | Create isolated identities/workspaces and switch the controller's active space |
| `OpenTab` / `SelectTab` / `CloseTab` / `MoveTab` / `SuspendTab` | Semantic controller mutations for setup and non-input scenarios |
| `SetSplit { primary, secondary }` / `CloseSplit` | Test-only seam into the production controller split path: open/close the two-pane layout over the two active-space tab indices (both tabs keep live sessions; the domain active tab stays the focused pane). The user-facing split-creation UI belongs to the later split-core slice |
| `WebAction(Back/Forward/Reload/Stop)` | Engine action on the active view |
| `ActivateCommandBar` / `FocusWeb` | The E2E readers of Cmd-L / Escape |
| `SetCommandBarText { text }` | Mutate the production UTF-8 command text and restart ranking; broadcasts `CommandBarChanged` |
| `FlushCommandRanking` | Functional-E2E-only deterministic seam: cancel the pending command debounce and publish the current generation synchronously; optimized command timing never uses it |
| `DispatchKeystroke { keystroke }` | GPUI `Keystroke::parse` + `Window::dispatch_keystroke` (real input path) |
| `BenchmarkSelectTab` / `BenchmarkToggleSidebar` / `BenchmarkDispatchKeystroke` / `BenchmarkResizeSidebar` / `BenchmarkRestoreSidebar` | Start typed in-process production-path measurements and finish only after the scenario's semantic presentation milestone |
| `BenchmarkCommandType { operation_id, text, expected_first_key }` | Measure production text entry through the real cancellable debounce/rank path; commit only after a render constructs the exact expected result palette (generation, first/selected key, state, mode, focus) |
| `BenchmarkCommandActivate { operation_id, expected_view }` / `BenchmarkCommandActivateNewTab { operation_id }` | Measure Enter/Cmd-Enter activation with exact destination, focus, mode, tab/session-count, and presentation guards armed before dispatch |
| `HoverTabRow { index }` / `UnhoverTabRow` / `PressTabRow { index }` / `ReleaseTabRow` | Drive the exact production hover/press app methods the row `on_hover` and mouse-down/up element handlers call — the §10.2 hover wash + close reveal and the §10.3 pressed wash (never a snapshot mutation) |
| `BenchmarkHoverTab { index }` / `BenchmarkPressTab { index }` | Measure the production pointer hover/press entry transitions; the sample commits only when the derived row state actually shows the §10.2 `hover` / §10.3 `active` wash |
| `FindOpen` / `FindClose` / `FindSetQuery { query }` / `FindNext` / `FindPrevious` | Route to the production find seams (`find_open` / `find_close` / `find_set_query` / `find_next` / `find_previous`); the portal frame is flushed after the mutable `BbrApp` lease is released |
| `BenchmarkFindOpen { operation_id }` | Measure the production Cmd-F open; the sample commits only from a render of the portal window that drew the find bar open with the field focused (fast-but-wrong guard) |
| `BenchmarkFindQueryResult { operation_id, query, expected_count, expected_ordinal }` / `BenchmarkFindStep { operation_id, next, expected_count, expected_ordinal }` | Measure the production query/step paths; the sample commits only when WebKit's content process reported the armed count/ordinal through the portable contract and the bar re-rendered it (pending/stale/mismatched reports reject in the app) |
| `DragSidebarDivider { dx }` / `FocusSidebarDivider` | Drive the exact production pointer/focus listener methods the divider element handlers call (never a controller mutation) |
| `FlushFrame` | Force one synchronous real GPUI draw (`Window::draw`) so paint-driven native geometry callbacks fire under the hidden harness (no display link) |
| `SetWindowSize { width, height }` | `window.resize`; native view re-framed next frame |
| `PortalOpenProbe` / `PortalCloseProbe` / `PortalOutsideDismiss` | Drive the general production portal API and its dismissal policy over the one `NativeAboveWeb` host |
| `PortalTeardown` | Exercise deterministic observer removal, child-window detachment, order-out, and close while preserving the browser session |
| `ActivateApp` | Re-activate the shown browser window for native Window Server diagnostics; the portal remains non-activating |
| `GetSnapshot` | Full typed `Snapshot` |
| `Ping` | Liveness |
| `CloseWindow` | Flush the successful response, then remove the real GPUI browser window; the production browser-window lifecycle observer calls `cx.quit()` and the process exits cleanly |
| `Quit` | Gateway flushes `Response::Ok`, then emits `Quitting` and calls `cx.quit()` (graceful shutdown, clean exit without a response/EOF race) |

### Events (implemented)

`AppReady { pid, app_version }` (once, after the gateway is up) ·
`Web(bbr_web::WebEvent)` (every engine event: url/title/loading/canGoBack/
canGoForward/page-crash/navigation-failure) · `ChromeFocusChanged {
chrome_focused }` (emitted by the app's focus sink after every focus mutation,
so it always agrees with the snapshot) · `CommandBarChanged { mode, text }` ·
`BrowserChanged` with typed profile/space creation and activation plus tab
open/select/close/move/suspend and split open/close outcomes
(`SplitOpened { primary, secondary }` / `SplitClosed`) from the shared app
mutation path · `Quitting` immediately before `cx.quit()` after the gateway
writer has flushed the successful Quit response. Harnesses still assert the
process exit status as the final shutdown evidence.

### Snapshot (implemented)

```rust
Snapshot {
  url, title: Option<String>, loading: bool,
  can_go_back, can_go_forward: bool,
  web_view_hosted: bool,        // active tab has a live native session
  active_profile: ProfileId, active_profile_title: Option<String>,
  active_profile_hue: Option<String>,   // DESIGN §12 hue name
  active_space: TestSpaceId,
  profiles: Vec<ProfileSnapshot>,       // complete collection, creation order
  spaces: Vec<SpaceSnapshot>,           // each with own active tab + live session
  tabs: Vec<TabSnapshot>,       // active-space index/url/title/active/current WebViewId
  all_tabs: Vec<GlobalTabSnapshot>, // complete persistent tab/space/profile identity
  active_tab_index: Option<usize>, active_web_view: Option<WebViewId>,
  engine_sessions: usize,       // native destruction / warm-session evidence
  engine_kind: String,          // "native-wkwebview" (macOS) | "fake" elsewhere
  chrome_focused: bool,
  shell_modes: ShellModes,      // theme/contrast/density (§15)
  tab_states: Vec<TabVisualState>, row_titles: Vec<String>,
  space_chips: Vec<SpaceChipSnapshot>, // one chip per space, canonical display order
  space_strip_scroll: SpaceStripScrollSnapshot, // typed viewport/offset/max/chip rects (§11.3)
  visible_ids: Vec<String>,     // rendered bbr-* ids (§15)
  command_bar_mode: Inactive | Location | NewTab | NewSpace | MoveTab,
  command_bar_text: String,
  command_result_state: Inactive | Empty | Loading | Results | ActivationError,
  command_results: Vec<CommandResultSnapshot>, // stable key/group/title/selection/destructive
  command_cursor: usize, command_selection: Option<[usize; 2]>,
  sidebar_visible: bool,
  sidebar_width: f32,        // effective rail width (0 when collapsed)
  sidebar_min / sidebar_default / sidebar_max: f32,  // DESIGN §5 adjustable range
  content_min: f32,          // native-content floor (never zero/negative geometry)
  divider_dragging: bool,    // §15 divider drag state
  divider_label: String,     // stable label shared with future Accessibility
  content_rect: [f32; 4],    // last engine-received WKWebView frame (x,y,w,h)
  pane_layout: Option<PaneLayoutSnapshot>, // authoritative pane layout
                               // (primary/secondary/focused WebViewIds); None
                               // when no live pane renders
  pane_rects: Vec<PaneRectSnapshot>, // keyed per-pane native frames in pane
                               // order (primary, then secondary); the focused
                               // pane's entry equals content_rect
  window_size: [f32; 2],     // points
  portal: PortalSnapshot {   // one per browser window; closed/open/teardown evidence
    presentation, host, visible, rect, ids,
    child_attached, native_visible, frame_synced, nonactivating_panel,
    parent_window_number, portal_window_number,
    frontmost_at_probe_point, below_portal_at_probe_point,
    portal_owns_probe_point, portal_is_key, portal_is_main,
  },
  semantics: Vec<SemanticNode>, // §15 internal semantic tree (roles/names/values/
                               // enabled/selected/focused + hierarchy); internal model
                               // only — GPUI 0.2.2 has no platform Accessibility layer,
                               // the macOS bridge is Phase 9
}
```

The split surface is implemented as the **test seam** above (`SetSplit` /
`CloseSplit` requests, `SplitOpened`/`SplitClosed` events, and the keyed
`pane_layout` / `pane_rects` snapshot evidence), covered by the split-geometry
suite (§7h). The **production** split-creation UI and its interaction /
performance benchmark kinds belong to the later split-core slice and are not
part of this anchor.

FUTURE (planned protocol growth, not yet implemented): `WebEvaluate`
(whitelisted DOM helpers), `AdvanceClock`/
`ResetIds`, `TerminateWebProcess`, `Screenshot`, and the richer event set
(`TabAdded/TabClosed/…`, `PersistenceFlushed`, …). `--bbr-clock` is parsed but
a no-op until the `TimeSource` seam lands.

---

## 4. Replay / queue semantics (implemented)

- The gateway keeps a cap-first ring of the last **4096** events
  (`VecDeque`, `RING_CAPACITY`) with monotonically increasing seq numbers
  (`AtomicU64`).
- `Attach { since }` replays `seq > since`; the client keeps its own cursor
  (`since`), so a stale queued event can never satisfy a later `_since` wait —
  the smoke relies on this for reload/canGoBack asserts.
- Each connection has a dedicated writer thread; responses are serialized with
  live event pushes. Events that arrive between a request and its response are
  queued client-side (`pending_events`) and consumed by later waits — fast app
  events cannot be lost.
- Slow clients drop live events (best-effort push); state lives in
  `GetSnapshot` anyway.
- Broadcast sites: `AppReady` (main), `Web` (event pump), `ChromeFocusChanged`
  (focus sink), `CommandBarChanged` (navigate/set-text dispatch), and typed
  `PresentationCommitted` samples from the in-process performance probe.

---

## 5. Process deadlines & cleanup (implemented)

- **Server side:** socket worker threads never touch the app; they forward
  each request through a channel to the main-thread pump and block on the
  reply with a **30 s timeout** (`dispatch_from_worker`) — a wedged app fails
  the test visibly instead of hanging. The main pump races `cmd_rx`/`evt_rx`
  via `smol::future::race`.
- **Keystroke dispatch note:** `DispatchKeystroke` goes through the untyped
  `AnyWindowHandle` path, not `WindowHandle::update` — dispatching a keystroke
  re-renders the root view (double-lease panic otherwise).
- **Harness (e2e_smoke.rs):**
  - Connect loop with a 60 s overall deadline (100 ms retries).
  - Event waits: 30 s deadlines via the client. Resize chooses a target
    relative to the current starting geometry (the `--bbr-hidden` harness
    window is never ordered front — `show: false` — so its bounds stay the
    requested ones; see the harness note below) and polls typed snapshots
    until the width changes or the same deadline expires.
  - Exit: `Quit` then up to 10 s for a clean `success()` status; otherwise the
    test fails.
  - `Drop` guarantee: the harness `Drop` runs regardless of assertion result —
    `SIGKILL` fallback (`child.kill()`), stderr drained on a background thread
    (tail-capped at 128 KiB to avoid a full-pipe deadlock).
  - Failure diagnostics: profile dir **kept**, plus `artifact/event-ring.txt`
    (last replayed events via `Attach(None)`, seq-prefixed) and
    `artifacts/app-stderr.txt`; fixture hit counts printed. Success removes the
    profile dir.
  - `catch_unwind(AssertUnwindSafe)` ensures the diagnostics path runs even
    when a step panics on an assertion.

---

## 6. `bbr-fixtures` (implemented)

- `FixtureServer::start(root)`: `TcpListener::bind("127.0.0.1:0")`, ephemeral
  port, per-connection thread. GET-only; `Connection: close`; path components
  must be `Normal` (traversal → 404). `Hits` counting per path.
- Dynamic deterministic routes: `/__fx/hello`, `/__fx/robots`.
- `bbr-fixtures` bin: prints `port` then `base_url`, serves until killed
  (FUTURE external-process runner).
- Fixture pages: `page1.html`/`page2.html` (deterministic titles "BBR Fixture
  One"/"BBR Fixture Two", cross links).
- 3 unit tests (static serving + hits, 404/traversal, deterministic routes).

FUTURE: upstream of `bbr-fixtures` into a hyper/axum server with `redirect/`,
`slow?ms=`, `setcookie`, `auth/basic`, `form`, `popup`, `iframe`, `ws` routes
is not needed by the smoke and was not built.

---

## 7. The smoke test (`bbr-app/tests/e2e_smoke.rs`)

One `#[ignore]`d macOS-only test that spawns the real binary with
`--features testkit` and drives product state via events and snapshots (no
sleep-based WebKit or focus assertions; bounded polling is limited to process
lifecycle and typed resize snapshots):

1. `AppReady` (from replay, or waited).
2. Initial load warm-up (title "BBR Fixture One" + `LoadingChanged(false)`) —
   cursor-free waits, because WebKit may finish before the harness attaches.
3. Snapshot asserts: `title`, `url`, `web_view_hosted`, `engine_kind ==
   "native-wkwebview"`.
4. `Navigate` → `page2`: wait `TitleChanged` → `CanGoBackChanged(true)` (with
   `since` cursor), snapshot asserts `can_go_back`.
5. `Back` → page1 (+ `CanGoForwardChanged(true)`), `Forward` → page2.
6. `Reload` → fresh `LoadingChanged(true)` → `LoadingChanged(false)` (`since`
   cursor forbids leftovers from the forward step).
7. `SetWindowSize` with a target relative to the current starting geometry
   (the harness window keeps its requested bounds — see the harness note)
   → bounded snapshot polling → geometry changed (width delta > 50 pt).
8. `DispatchKeystroke("cmd-l")` → `ChromeFocusChanged{true}` → snapshot
   `chrome_focused`; `SetCommandBarText(page2)` → snapshot text; then
   `DispatchKeystroke("escape")` → `ChromeFocusChanged{false}` (focus returned
   to the web view).
9. `Quit` → process exits with success status within 10 s.

## 7 — Process shutdown addendum (`bbr-app/tests/e2e_shutdown.rs`)

One `#[ignore]`d macOS-only suite (shared harness in `tests/e2e_common/`;
`just e2e-shutdown`, canonical `just e2e` position 2) proves both user
process-lifecycle paths through real process exit, not hidden windows or
internal shutdown state:

1. **Owning-window close** (`Request::CloseWindow` → the production GPUI
   `window.remove_window()` path): BBR's browser-window observer runs
   `flush_browser_state()` and calls `cx.quit()`, and the complete process
   exits successfully within 10 seconds. The gateway flushes the successful
   `CloseWindow` response to the client before the shutdown handler runs (no
   response/socket EOF race), matching the graceful-`Quit` guarantee.
2. **Owning-window close with an open portal child**: the one
   implementation-owned native portal host is presented and attached while
   the owning browser window closes; the complete process still exits
   successfully within 10 seconds — a portal child never owns process
   lifetime.
3. **Real `SIGINT`** (the terminal Ctrl-C signal): a real `SIGINT` is
   delivered to the spawned BBR process; the async signal listener schedules
   `cx.quit()` on GPUI's foreground/main thread and the complete process
   exits successfully within 10 seconds (a default-disposition signal death
   would fail the success assertion).

The suite waits on process state under explicit deadlines and uses the same
failure artifacts as the other real-app E2E harnesses. The native portal
hardening suite's parent-close scenario asserts the same owning-window
contract through the real AppKit `performClose:` path: zero BBR windows on
screen after the close and the process exit within the standard deadline.

## 7a. The tabs test (`bbr-app/tests/e2e_tabs.rs`)

One `#[ignore]`d macOS-only test proves the production controller/native path:

1. Initial tab owns `WebViewId(1)` and one native session.
2. `DispatchKeystroke("cmd-t")` emits `TabOpened`; two native sessions exist.
3. Command-bar Enter navigates tab 2; Back/Forward/Reload events all carry tab 2's id.
4. Select tab 1 and tab 2 through the semantic command; both original ids remain.
5. `DispatchKeystroke("cmd-w")` closes tab 2; native count falls to one and tab 1 keeps its id.
6. Quit exits cleanly; failures preserve event-ring and stderr artifacts.

## 7b. The profiles test (`bbr-app/tests/e2e_profiles.rs`)

One `#[ignore]`d macOS-only test proves real profile and lifecycle boundaries:

1. Set and read cookie `alpha` in the default profile's native data store.
2. Create a second profile/space, set cookie `beta`, and prove each store reads only its own value.
3. Move the default tab between two spaces of one profile and retain its original `WebViewId`.
4. Move the private tab across profiles and observe a fresh `WebViewId` backed by the destination store.
5. Suspend a hidden warm tab, assert its native session disappears, then select it and assert fresh-session recreation.
6. Quit exits cleanly; failures preserve event-ring and stderr artifacts.

## 7c. The performance test suite (`bbr-app/tests/`)

The performance foundation is split into a shared support module
(`tests/perf_common/`) and three `#[ignore]`d macOS-only suites. All measure
through the optimized real app with testkit transport excluded (`Benchmark*`
operation IDs start inside the process; startup uses the shared macOS
monotonic clock to include process spawn).

`tests/e2e_performance.rs` (short gate):

1. Process spawn to the first presentation-ready state containing the sidebar,
   selected `WebViewId`, and non-zero native content geometry.
2. A distinct deterministic-page-ready milestone (`BenchmarkKind::PageReady`),
   measured from process spawn to the fixture title/loading on the expected
   view — separate from shell/native-view presentation.
3. Thirty measured warm-tab switches after discarded warm-up iterations.
4. Thirty sidebar toggles through the real `Cmd-B` GPUI input/action path.
5. Ten staged `Cmd-T` command-bar activations, ten Enter/new-tab creations, and
   ten active-tab closes through the real shortcuts.
6. Semantic state assertions for active view/tab, sidebar, native session count,
   focus, command-bar mode, and geometry before accepting each sample.
7. Machine metadata (build revision, build profile, host model, arch, OS,
   fixture mode, binary/engine identity, display refresh or `null`+reason) and
   canonical-vs-report-only classification stamped on the report.
8. Baseline evaluation: absolute ceilings always enforced; the >15% p95
   relative rule only when `host_model` matches `canonical_host_model` and the
   entry source is `accepted`. On success a trend frame is appended to
   `target/bbr-perf-trend/`; on failure `performance.json` + `raw_samples.csv`
   + `event-ring.txt` + `app-stderr.txt` are retained under
   `target/bbr-perf-artifacts/<run>`.

`tests/e2e_performance_full.rs` (`just e2e-perf-full`): a bounded repeated
canonical gate — `COLD_STARTUP_SAMPLES=10` cold fresh-profile launches build
`startup`/`page_ready` distributions and the 30-sample interaction matrix; it
asserts the contract-required repetition counts.

`tests/e2e_performance_inject.rs` (`just e2e-perf-inject`): a deterministic
self-test that arms the testkit-only `--bbr-bench-inject-delay-ms` flag,
asserts the budget gate detects the injected latency (warm-tab p95 exceeds the
50 ms ceiling), then runs without injection and asserts green — without
changing any budget or baseline. The injection is compiled only under
`#[cfg(feature = "testkit")]`, so it is impossible in
release-without-testkit builds.

GPUI 0.2.2 exposes no public platform-presentation completion callback. Startup
and geometry-changing chrome operations therefore commit at the end of the
`NativeWebViewSlot` paint path, after native WKWebView geometry attachment and
immediately before GPUI presents the frame. Warm-tab switching and the other
interaction benchmarks commit synchronously against the authoritative
post-mutation controller snapshot (active view, sidebar, focus, command-bar
mode): the harness window runs offscreen, where GPUI's frame/`on_next_frame`
callback does not reliably fire, so a frame-dependent commit would silently drop
interaction samples. The suite does not claim post-scanout timing. The real GUI
gates build and run on this host via the installed Xcode 26.2.0 Metal
toolchain (`BBR_XCODE_DEVELOPER_DIR` set by the just recipes; the Metal
compiler is the downloadable Xcode "Metal Toolchain" component — see
ARCHITECTURE.md) in a display-attached session. Since the GPUI-CE anchor,
the app under test starts through `gpui_platform::application()`, roots
every window in `gpui_component::Root`, and all gateway mutations reach
the separately retained `Entity<BbrApp>` through the Root-window seam
(`crate::with_app`); snapshots still expose the production browser state
— no synthetic Root-only proxy substitutes for app evidence.

FUTURE: the remaining split/dialog/routing/persistence scenarios, naming a
dedicated canonical runner so `canonical_host_model` is populated and the
>15% relative rule activates, resource instrumentation, and the `bbr-e2e`
crate with `manifest.toml` per suite, `scripts/run_e2e.py` runner,
`nextest.toml` `[profile.e2e]`, and `docs/e2e-protocol.md` remain planned.
F3 integrated the real repository-backed persisted-profile warm-startup
scenario with visible-set-only hydration and at least ten accepted samples.

## 7d. The shell-spaces suites (`bbr-app/tests/e2e_shell_spaces{,_hint,_keyboard}.rs`)

Three `#[ignore]`d macOS-only binaries (the shared harness lives in
`tests/e2e_common/`) run under `just e2e-shell-spaces`; the
`e2e_shell_spaces` binary additionally compiles the typed `appearance`
(tint) and `swipe/` modules. Together they prove the visible profile/space
surface over real WKWebViews:

1. **Pointer suite** (`just e2e-shell-spaces`): the permanent §11.3/§11.4
   footer `[Downloads][Space Strip][+]` over complete typed profile/space
   collections with §12 round-robin hues, stable `bbr-space-chip-<persistent-
   space-id>` / `bbr-space-identity-<persistent-space-id>` ids derived from
   the persistent `SpaceId` alone (never render position or creation-order
   index), the typed `space_chips` with exactly one selected chip (the
   active space), pointer chip activation through the production listener
   (chrome focus domain preserved),
   per-space active-tab restoration with the same WebViewId and **zero load
   events in the replay window**, empty-space pane naming from
   `active_profile_title`, cross-profile view-only switches that keep cookie
   isolation (each store reads only its own cookie) without recreating warm
   sessions, and a cross-profile tab move that destroys + recreates the
   session in the destination store (`recreated: true`, fresh WebViewId
   reading the destination store's cookie).
2. **Keyboard suite** (`just e2e-shell-spaces` second binary): `Cmd-Shift-N`
   staging (Escape cancels, Enter commits + activates, `SpaceCreated` is
   broadcast), real `↑/↓`/Enter traversal of the focused space strip through
   the one strip focus/activation authority (exact active space, exactly-
   one-selected chip `selected == active`, warm WebViewId, unchanged session
   count, no reload, chrome focus, and the cross-profile hint), and same-
   profile moves that keep the WebViewId (`recreated: false`) versus
   cross-profile moves that recreate it.
3. **Hint suite** (`just e2e-shell-spaces` third binary,
   `e2e_shell_spaces_hint.rs`): the DESIGN §11.3 cross-context rule over real
   WKWebViews — a cross-profile space activation renders the destination
   profile chip + `"in Profile · <title>"` at the top of the command field
   for one interaction, same-profile switching never announces a profile
   transition, the hint clears at a deterministic user-interaction boundary
   (a tab mutation or a command-bar mode entry), chrome focus and the warm
   session count/WebViewIds are untouched, the staged-`MoveTab`
   `command_bar_hint` is a separate surface that never coexists, and
   activating an *empty* space of another profile announces the destination
   (hint + chip + semantic node) while the empty pane names the owning
   profile and renders no web view, with the hint re-set on a reverse
   cross-profile activation and cleared by the command-bar boundary.
4. **Appearance/tint** (`e2e_shell_spaces/appearance.rs`): the active
   space's **effective appearance** — the one controller authority, an
   explicit curated theme record or the derived profile-hue default —
   resolves the chrome/chip tint, and production chip activation across
   spaces and profiles flips the typed `active_appearance` (theme/identity
   names plus the resolved chrome/chip tint hex from the one central
   presentation model, never raw pixels) exactly with the active space
   while warm-session counts, per-space WebViewIds, the native content
   rect, the pane layout, and chrome focus stay untouched.
5. **Swipe commit/cancel** (`e2e_shell_spaces/swipe/`): every phase drives
   the exact production `on_sidebar_scroll_wheel` handler through the typed
   `SwipeScroll` request, with the release always a real **zero-delta**
   `Ended` resolved independent of horizontal dominance (what a real macOS
   `Ended` carries). Above-threshold release commits exactly one switch in
   both display directions through the existing `activate_space` authority
   — warm WebView/session identity, unchanged content/pane geometry, chrome
   focus; below-threshold release and explicit cancel emit no switch and
   restore the exact source presentation; boundary (no adjacent space) and
   vertical-dominant deltas are no-ops; the typed snapshot carries
   active/source/destination/direction/progress plus the interpolated
   chrome/chip tints while the semantic selected chip stays on the source
   until commit; and a cross-profile adjacent swipe keeps the same
   authority (no session recreation) with the one-interaction command-field
   profile hint.

Both activation paths (pointer rows and keyboard traversal) assert the
§11.3 rule; the hint suite covers the pointer path plus the empty-space
branch, the keyboard suite the `↑/↓` traversal path.

The `space_switch` interaction benchmark (in `tests/perf_common/benchmarks.rs`)
measures alternating warm-space activation through `BenchmarkActivateSpace`,
asserting the destination space, its own active view, the unchanged live-
session count, chrome focus, and the rendered selected space row + chip ids;
the `space_switch_cross_profile` benchmark measures the same warm view-only
class across profile boundaries and additionally asserts the hint copy in
the accepted snapshot while the in-app presentation probe rejects any
presentation that misses or invents it. The `swipe_commit` interaction
benchmark measures the production swipe entry — the real Started/Moved
gesture path plus the zero-delta Ended resolving into exactly one existing
`activate_space` commit of a warm adjacent space — with 30 render-inclusive
samples against the same 50/100 ms warm budget and sessions never recreated.
All three are part of `just e2e-perf` and the full gate.

## 7e. The shell-sidebar suite (`bbr-app/tests/e2e_shell_sidebar.rs`)

One `#[ignore]`d macOS-only suite (shared harness in `tests/e2e_common/`)
proves the adjustable/collapsible sidebar surface over real WKWebViews. GPUI
0.2.2 exposes no public mouse-event dispatch, so the testkit
`DragSidebarDivider` seam walks the exact app methods the divider/root
element handlers call — pointer drag, drag-move, drag-end — never a
controller mutation; `FlushFrame` forces the real production `Window::draw`
path so the paint-driven native geometry callbacks fire under the hidden
harness (no display link, no sleeps, no pixels):

1. **Pointer drag** (`just e2e-shell-sidebar`): relative deltas resize within
   the DESIGN §5 range (clamps at 200/360), the engine-received native
   content rect tracks the rail exactly (x == rail width, width == window
   minus rail, stable height — no drift), the divider focus reports the
   chrome domain, and the selected WebViewId + session count never change.
2. **Keyboard parity**: real `←`/`→` keystrokes on the focused divider step
   the same requested width the pointer path mutated, `0` resets to the §8
   default, and the native content follows every step.
3. **Collapse/reopen + compact window**: real Cmd-B yields an instant
   full-bleed slot (width 0) and expand restores the last expanded width
   (not the default); in compact windows the rail yields below the requested
   width so the native content keeps exactly the 64 pt content floor —
   positive geometry in every state, and the full requested width returns
   when the window widens.

The `sidebar_resize` and `sidebar_restore` interaction benchmarks (in
`tests/perf_common/benchmarks.rs`) measure the width-commit and
collapse→expand paths through `BenchmarkResizeSidebar` /
`BenchmarkRestoreSidebar`: the resize sample commits only at the exact
clamped geometry (typed min/default/max + content floor let the drivers
compute the same expectation the render enforces, so a fast-but-wrong sample
is rejected in the app before it leaves); the restore sample asserts the
last-expanded width after an unmeasured production collapse. Both join the
30-sample matrix with the 50 ms p95 / 100 ms max next-frame budget.

---

## 7f. The shell-organize and drag-to-chip suites (`bbr-app/tests/e2e_shell_organize.rs` + `e2e_shell_drag_chip/`)

Two `#[ignore]`d macOS-only binaries run under `just e2e-shell-organize`:
the organize suite (`e2e_shell_organize.rs`, shared harness in
`tests/e2e_common/`) proves the tab-organization slice over real WKWebViews,
and the dedicated drag-to-chip suite (`e2e_shell_drag_chip/`) proves the
production tab drag onto the §11.3 space chips. Keyboard scenarios of the
organize suite dispatch **real shortcuts** through the GPUI keybinding path
(`DispatchKeystroke` → `window.dispatch_keystroke`); pointer scenarios drive
the production mouse-down/move/up listener methods with synthetic events
(`BeginTabDrag`/`MoveTabDrag`/`EndTabDrag`/`CancelTabDrag` seams, exactly
like the sidebar-divider drag seam) — never a direct reorder or move command:

1. **Keyboard suite**: `Cmd-Shift-P` pins and unpins the active tab (region
   boundary + selected row asserted via the typed `pinned_count` and row
   order, live session preserved), `Cmd-Shift-↑/↓` reorder within the region
   with boundary-crossing steps rejected whole (no `TabReordered` event, no
   partial mutation), `Cmd-Shift-M` stages a destination space by name
   (same-profile hint `→ Space`, cross-profile hint `in Profile · … → Space`),
   Escape cancels and restores mode/focus, Enter commits a same-profile move
   that keeps the WebViewId (source selection repaired to the deterministic
   neighbor), and a cross-profile move destroys + recreates the session
   exactly once in the destination store — proven by cookie isolation (the
   recreated tab reads the destination store's cookie, never the source's).
2. **Pointer suite**: the row pin control through the production listener,
   drag reorder within the pinned and regular regions with the typed
   `drag_source`/`drag_slot`/`drag_valid` snapshot fields and the §15
   `bbr-drag-tick-<n>` id (accent when legal, muted at the clamped region
   edge when illegal), an illegal boundary drop rejected whole, and Escape
   cancellation. The pointer drag-to-chip move (§10.9/§11.3, dragging a tab
   row onto a strip chip) is deliberately not part of this binary: it is
   delivered as the dedicated `e2e_shell_drag_chip` suite below, which the
   same `just e2e-shell-organize` recipe runs after this one.

The `tab_pin`, `tab_reorder`, `tab_move_same_profile`, and
`tab_move_cross_profile` interaction benchmarks (in
`tests/perf_common/benchmarks.rs`) keep their existing typed kinds, report
keys, budgets, and guards. The pin and reorder scenarios drive the
production `Cmd-Shift-P` toggle and `Cmd-Shift-↑/↓` step through
`BenchmarkPinTab` / `BenchmarkReorderTab`. The two move scenarios
(`tab_move_same_profile`, `tab_move_cross_profile`) now enter through the
delivered drag-to-chip production path (`BenchmarkDragMoveTab`): the
concrete row is pressed, the pointer is moved onto the destination space's
rendered **visible clipped chip** (the driver brings the chip into view
through the one tracked `space_strip_scroll` handle plus a settled
main-window draw, so the pointer never targets hidden hit geometry), and
the release commits through the drag drop → the existing single move
authority — never a direct move command. Each sample commits from the
authoritative post-mutation snapshot (active view, focus, mode) and the
drivers assert the region boundary, the source-space repair view, the wire
`recreated` flag, and — via the destination activation after each measured
move — the session identity (same WebViewId for same-profile, fresh
incarnation for cross-profile) and an unchanged net session count. The
cross-profile move is the session-recreation class with its own 100/150 ms
budget; the other three share the 50/100 ms warm ceiling.

### The drag-to-chip suite (`bbr-app/tests/e2e_shell_drag_chip/`)

The dedicated drag-to-chip binary — one `#[ignore]`d macOS-only suite run
by the same `just e2e-shell-organize` recipe after `e2e_shell_organize` —
proves the production tab drag onto a §11.3 strip chip over real WKWebViews.
Every drag drives the production request/handler path
(`BeginTabDrag`/`MoveTabDrag`/`EndTabDrag`/`CancelTabDrag`), every commit
goes through the single existing tab-move authority, and the footer
`[Downloads][Space Strip][+]` composition, the active theme tint, the
tracked strip scroll, and the downloads panel stay invariant:

1. **Same-profile move** (`same_profile`): the concrete pressed row — not
   the implicit active row — moves into the exact persistent chip under the
   pointer; the typed during-drag chip target + §12 destination-profile
   companion are reported on the snapshot and in the semantic tree; the
   moved tab keeps its live session/WebViewId and the source selection
   repairs.
2. **Cross-profile move** (`cross_profile`): the destination-store
   recreation happens exactly once (`recreated: true`), the recreated view
   reads only the destination store's cookie, the net session count is
   unchanged, and the source selection repairs.
3. **No-op contract** (`noops`): own-space drops, gap drops, drops on the
   fixed `[Downloads]`/`[+]` controls, and Escape cancel all commit nothing —
   no `TabMoved`/`TabReordered` event leaves the app.
4. **Invariants** (`invariants`): a completed same-profile drag-to-chip move
   leaves the footer composition, the active tint, the strip scroll
   geometry, and the downloads panel byte-for-byte unchanged.

The drag entry is deliberately distinct from the organize keyboard suite's
staged `Cmd-Shift-M` + Enter move: the drag scenarios exercise the production
pointer geometry over the visible clipped chips, while the keyboard scenarios
exercise the staged command path.

---

## 7g. The shell-hover and semantic suites

Three focused `#[ignore]`d macOS-only binaries —
`bbr-app/tests/e2e_shell_semantics.rs`, `e2e_shell_hover.rs`, and
`e2e_shell_semantic_branches.rs` — share `tests/e2e_common/` plus the small
`tests/e2e_semantics_common/` assertion module. The sequential
`just e2e-shell-hover` recipe proves the pointer-state slice (§10.2/§10.3)
and the internal §15 semantic tree over real WKWebViews:

1. **Hover/press through the production listener paths**: `HoverTabRow` /
   `PressTabRow` mutate the exact app state the row `on_hover` and
   mouse-down/up element handlers drive — the snapshot's `tab_states` show
   `hover` / `active` and the composed §10 order holds (error > loading >
   pressed > selected > hover > default; hovering the selected row keeps
   `selected`). Hover and press never select.
2. **Typed semantic tree**: one root node (`bbr-root`, role `Root`); the
   sidebar (`Sidebar`) with brand, command field (`Field`, typed mode/value),
   space strip (`List` → `Row` chips with `Title` identity children),
   tab list (`List` → section
   labels (`Section`) + rows with `Favicon`/`Title`/`Button` pin + `State` +
   `Button` close children), status (`Status`), toggle, and divider
   (`Divider` with the width value); the glass content branch (`Content` =
   native slot, crash panel, or empty pane naming the profile).
3. **Focus clarity**: after pointer selection the owning list and its
   selected row expose `focused: true` (the space strip and command field do
   not); web focus renders the chrome at ease (nothing focused); Cmd-L
   focuses the field and exposes the `bbr-command-cursor` node; section
   labels never own focus.
4. **Disabled coherence**: a dragged row's close/pin nodes flip `enabled`
   false and back after cancel.
5. **Identity and edge states**: profile chips carry the §12 monogram, the
   long-title row name is the truncated §4 title, the hostless row falls back
   to the §11.9 scheme title and §7.2 monogram letter, the crash branch and
   the empty-space branch render their typed panes, and a collapsed sidebar
   leaves only root + glass.

The `tab_hover`/`tab_press` interaction benchmarks (in
`tests/perf_common/benchmarks.rs`) measure the entry transitions through
`BenchmarkHoverTab`/`BenchmarkPressTab`; each sample commits only when the
derived row state shows the wash, joins the 30-sample matrix with the 50 ms
p95 / 100 ms max warm budget, and is asserted in the full-gate sample counts.

---

## 7h. The shell-split-geometry suite (`bbr-app/tests/e2e_shell_split_geometry.rs`)

One `#[ignore]`d macOS-only suite (shared harness in `tests/e2e_common/`;
`just e2e-shell-split-geometry`) proves the Phase-6 serial anchor over real
WKWebViews: the keyed per-`WebViewId` geometry contract and the authoritative
controller `pane_layout()`. It synchronizes on typed events and paint-driven
keyed snapshot rects under deadlines (no sleeps/pixels) and preserves the
post-resize native content-height baseline (the pane itself, not the window
bounds):

1. **SetSplit**: the typed `SplitOpened { primary, secondary }` event is
   awaited; two simultaneous native panes publish distinct, adjacent, strictly
   positive keyed rects in pane-layout order with unchanged session
   identities, the focused pane is the domain-active pre-split tab, its rect
   is the legacy `content_rect`, and the keyed slot ids
   (`bbr-native-web-slot-1/2`) replace the single unkeyed one.
2. **Member selection**: selecting the other split member switches the
   focused pane (and `content_rect`) without dissolving the split.
3. **Resize**: `SetWindowSize` reframes both pane ids independently; both
   panes share the post-resize native content height (the corrected
   baseline).
4. **CloseSplit**: the typed `SplitClosed` event is awaited; one full-width
   focused rect returns (exact single-pane sidebar contract), both tabs stay
   open and warm, no orphan keyed geometry or hidden slot id remains.

The single-pane regression (exact sidebar/content contract before any split)
rides the same suite and the independently green `just e2e-shell-sidebar` run.
No new benchmark kind is introduced: this anchor adds no production user
invocation; split-creation/divider interactions belong to the later
split-core slice.

---

## 7i. The native portal suite (`bbr-app/tests/e2e_shell_portal.rs`)

`just e2e-shell-portal` launches the real app with a real WKWebView and proves
the one-host `NativeAboveWeb` boundary through typed production seams:

1. **Initially hidden:** the child panel already exists and is attached/frame-
   synchronized, but remains ordered out, non-key, non-main, and non-activating;
   no portal presentation or `bbr-portal-*` ids are exposed.
2. **Open over WebKit:** the production probe resolves through the general
   portal state, renders stable ids, orders the child host in, and preserves the
   exact active `WebViewId`, session count, profile, space, and chrome focus.
3. **Native z-order/hit testing:** `portal_owns_probe_point` proves the attached,
   visible, frame-matched child mechanism. At least one real AppKit Window
   Server query corroborates it: the absolute query reports the BBR portal/
   parent, or the portal-relative query reports the BBR parent immediately
   below. Requiring both would incorrectly assert global desktop state because
   foreign always-on-top windows and all-spaces/system overlays can appear in
   the other raw query.
4. **Resize without recreation:** parent resize re-pins the same portal window
   and keeps the same web session. This also regresses the former observer
   use-after-move crash.
5. **Dismiss and teardown:** outside-dismiss and explicit close order the host
   out without browser mutation; `PortalTeardown` removes observation and the
   child relation and closes the host deterministically.

The suite uses typed snapshots/deadlines, no pixel assertions or product sleeps.
After remediation it passed three focused 3/3 runs, the command and sidebar
regressions, and two complete sequential `just e2e` runs.

## 7j. U1 command surfaces over native WebKit

`just e2e-shell-command` contains 17 real-app scenarios after U1 portal
migration. The existing palette/overlay behavior tests now assert from the
same snapshots that the active command surface is hosted by
`native-above-web`: logical/native visibility, child attachment, parent-frame
synchronization, non-activating/non-key/non-main policy, stable portal-window
reuse after dismissal, and unchanged `WebViewId`/session ownership.

One focused shown-window scenario adds the required native-boundary evidence.
It opens Cmd-T over a real `native-wkwebview`, flushes the separate portal
window only after releasing the mutable `BbrApp` lease, and requires
`portal_owns_probe_point` plus at least one real AppKit Window Server
corroboration (absolute BBR portal/parent, or the BBR parent immediately below
the portal). Escape then orders the same host out without recreating the web
session. This scenario also regresses the GPUI double-lease failure that would
occur if a portal draw tried to read `BbrApp` while the gateway still held its
mutable update lease.

Patrick's intermittent manual report from the U1 adaptation worktree added a
separate idle-host regression before packaging. The shown real-app scenario
`idle_portal_never_occludes_or_intercepts_repeated_web_presentations` performs
three independent cold launches. Each launch opens a second real WKWebView,
navigates it, and switches between both live sessions four times. Every step
requires the portal's real AppKit window to remain ordered out, the deterministic
portal hit-test ownership flag to remain false, and a raw Window Server query at
the content probe to name the browser parent rather than the portal. WebKit
title events and stable WebView/session identities corroborate content
presentation. This closes the gap where typed logical visibility alone could
miss a stale native child window that still occluded or intercepted content.

## 7k. The find-in-page suite (`bbr-app/tests/e2e_shell_find/`)

`just e2e-shell-find` contains 6 real-app scenarios for the find-in-page Portal
bar (task packet `find-in-page-portal-bar`):

1. **Open over a real WKWebView:** the shown-window scenario drives `Cmd-F`
   through GPUI's real input path, then requires the `0xC3` `NativeAboveWeb`
   presentation, the stable `bbr-find-*` ids, child attachment, frame sync,
   non-activating/non-key/non-main policy, `portal_owns_probe_point`, and at
   least one real AppKit Window Server corroboration at the portal frame
   center over the live web view — plus the exact WebKit-reported count 3 /
   ordinal 1 for the deterministic repeated fixture (hidden text excluded).
2. **Navigation:** Enter/Shift-Enter wrap at both boundaries, a replacement
   query restarts the search, and zero matches report count 0 with no ordinal.
3. **Focus restoration:** Escape and the explicit close path restore the
   captured origin focus domain (web → page; chrome/Cmd-L → command bar with
   the origin mode intact) and clear highlights; identity is untouched.
4. **Stale navigation and page mutation:** a navigation of the bound view
   invalidates in place (bar stays open, old report kept); a fixture that
   mutates its own DOM is observed by the strategy and reported fresh+stale.
5. **Resize + tab-switch isolation:** a resize while open proves the same
   portal window follows; a tab switch discards the presentation with both
   sessions alive and profile/space untouched.
6. **Page-world isolation:** the `find-isolation.html` fixture records every
   unexpected global and CSS-highlight entry the page world can observe on
   every selection change; after a full open + start + next + previous +
   close run the title proves the page world never sees the client-world
   find machinery (the strategy runs in
   `WKContentWorld::defaultClientWorld`, never the page's main world).

The suite uses typed snapshots/deadlines only — no sleeps or pixel
comparisons. The optimized `find_open`/`find_query_result`/`find_step`
scenarios run in the `e2e_performance` matrix (30 accepted samples each,
commit only from a portal render that drew the armed WebKit-reported report,
command-surface warm budget).

## 7l. The Saved Pages suite (`bbr-app/tests/e2e_shell_saved_pages.rs`)

`just e2e-shell-saved-pages` contains 4 real-app scenarios for the B2 packet
(`saved-pages-actions-command`) — the one app-owned Saved Page command
authority driven over real WKWebViews:

1. **Idempotent Cmd-D toggle:** the first `Cmd-D` saves the active fixture
   page, the availability matrix flips (save hidden, remove offered), a
   second `Cmd-D` removes it, and repeated save/remove cycles never create
   duplicates; the typed `saved_pages_count` and the repository effects
   agree after the deterministic flush.
2. **Survives tab close and restart:** save, open a second tab, close the
   source tab (no `TabId` link), flush, restart the real app on the same
   profile — the record restores while no original tab/session exists. The
   restart restores the persisted browser state, so the restored tab is
   closed before the Palette search: the Saved Page ranks as the distinct
   `saved_page:<id>` row (CurrentTab < SavedPage is fixed) and the record
   is findable even with zero live tabs.
3. **Exactly-one-fresh-tab activation:** the source tab is navigated to a
   non-matching page first so the CurrentTab row never beats the Saved Page
   row; activating the result opens exactly one new regular tab + one live
   session in the currently active Space/Profile, with the fresh view never
   the source, the surface closed in web focus, and the destination the
   record's canonical URL.
4. **Production quit durability:** same-profile Cmd-D save; the bounded
   background writer's persisted state is observed deterministically
   out-of-band (`load_saved_pages` == 1); a foreign
   `Repository::replace_saved_pages(&[])` empties the store and the
   persisted zero is observed; the production `CloseWindow` (no testkit
   flush) exits the complete process successfully; the profile root is
   preserved and a same-profile restart restores exactly one record with
   the availability matrix offering remove — the real owning-window-close
   quit flush, never the testkit seam, carried the record across the exit.

No sleeps: every step synchronizes on typed snapshot state or semantic
events under an explicit deadline, and the suite takes the serial GUI lock.

The optimized matrix (`just e2e-perf`) adds the B2 scenarios
`saved_page_search_first_result` (warm command-text class 50/100) and
`saved_page_open` (session-recreation class 100/150), each with 30 accepted
samples over the pre-spawned ≥1,000-record provider with fast-but-wrong
identity/one-tab guards; raw artifacts land in
`target/bbr-perf-artifacts/short`.

## 7m. The web-content navigation-gestures suite (`bbr-app/tests/e2e_web_navigation_gestures.rs`)

One `#[ignore]`d macOS-only suite (`just e2e-web-navigation-gestures`)
driving the real `bbr` process over real WKWebViews and deterministic
history fixtures (N1 packet `web-content-navigation-gestures`). Committed
logical gestures enter through the exact production transport: the typed
`ContentGestureCommit` request pushes into the same channel the native
content-gesture bridge reporter uses, and the pump drains it into
`BbrApp::on_content_gesture_commit` — the identical method a real committed
horizontal trackpad swipe reaches. The hidden harness cannot synthesize a
trustworthy NSEvent scroll gesture, so committed commits are injected typed
(the accepted seam pattern; native phase/direction/cancel mapping stays
unit-proven in `bbr-web-macos`).

Scenarios, all real-history and sleep-free:

1. Back/Forward navigate the exact pane by `WebViewId` through real
   `canGoBack()`/`canGoForward()` state while an unrelated active tab stays
   untouched (one dispatch per committed gesture, sessions preserved).
2. Back at a settled regular child-tab root closes only that child through
   the normal close path and presents the exact opener (warm session reused,
   one causal `OpenerReturn`, relation invalidated, native web focus).
3. A child with older history navigates first (history dispatch, child
   stays open); the second committed Back at its root returns to the opener.
4. The no-op matrix: disabled Forward, root Back without provenance, stale
   view, and a child whose opener was closed — all preserve tabs, sessions,
   and focus (an arbitrary tab is never closed).
5. A provisional child load (the deterministic slow fixture) is a whole
   no-op; once the document commits the same child returns at its root.
6. Return from a split child pane dissolves the split and presents the
   opener in single-pane mode.
7. Return activates an opener that moved across a Space/Profile boundary at
   its current isolated profile/session (move-time recreation reported by
   `TabMoved`; the warm destination session is reused at return).
8. `Cmd-W` stays the ordinary close seam — it closes the active child via
   `TabClosed` and never runs the gesture opener-return transaction.
9. A settled navigation-failure root still returns to the opener.

The suite takes the serial GUI lock; the optimized scenarios
`web_history_swipe_back` and `child_tab_swipe_return` are implemented with
production-path drivers (perf_common/benchmarks.rs + app arms through the
exact native bridge commit method, deterministic pre-loaded history, real
`canGoBack()`/title semantic milestones, ≥30-sample guards, and in-app
fast-but-wrong rejection) and registered in the contract baseline with the
unchanged warm ceiling (50/100, 30 samples); execution remains
unexecuted/unpassed under the active delivery override. The nine E2E
scenarios passed 9/9 under the host-exclusive GUI lock on the remediation
tree (log SHA-256 `0d51390b…`).
## 7n. The Peek suite (`bbr-app/tests/e2e_shell_peek.rs`)

`just e2e-peek` runs 6 real-app scenarios for the Link Preview / Peek packet
(`link-preview-peek.md`). One honest seam: the hidden real-app harness cannot
synthesize a trustworthy native `WKNavigationType::LinkActivated` click (the
settled NW-SEAM evidence split), so the user-initiated Peek path enters through
the exact `BbrApp::on_new_window_request` production method the native
coordinator delivery pump calls, with the default intent; the script-created
`window.open` block path stays fully native (the real `popup.html` fixture
executes `window.open`, WebKit invokes the real WKUI delegate, portable policy
blocks).

1. **Eligible request → Peek above the warm origin, no tab:** the preview
   appears with the origin's exact `WebViewId`/URL, no regular tab
   (`tabs.len()` unchanged, `created_tabs == 0`), the bar's stable ids
   presented, and truthful native z-order (`preview_index > source_index` in
   the AppKit subview order).
2. **Dismissal restores the exact warm origin:** the production Escape path
   destroys the preview session and leaves the origin with the same
   `WebViewId`, URL, engine-session count, and tab set; while open the origin
   never becomes the focus target and the preview is raised above it, so
   pointer/keyboard/scroll input lands on the preview.
3. **Navigation + same-session promotion:** the preview page loads its real
   fixture target, Reload routes to the preview session, and “Open as Tab”
   promotes the SAME `WebViewId` into a regular tab with its live
   navigation state (title) preserved and `engine_sessions == 2` — no reload,
   no recreated session.
4. **One preview per pane:** a new eligible request replaces the existing
   preview (the old session is destroyed; no stacking).
5. **Exclusions:** an ineligible direct activation (non-web scheme) with the
   default intent falls back to the ordinary regular-tab path (the user's
   action is never silently lost or forced into a preview); script requests
   stay blocked.
6. **Crash teardown:** a deterministic `PageCrashed` drives the production
   crash path — the crashed preview refuses promotion and dismissal still
   destroys the session cleanly with the warm origin untouched.

No sleeps: every step synchronizes on typed snapshot state or semantic events
under an explicit deadline, and the suite takes the serial GUI lock. Evidence:
PEEK 6/6 (`/tmp/bbr-peek-closeout-e2e-peek.log`, SHA-256
`85d3b8c49b2d1230a84f9ea7ff8780df3174d01b3e21a4114e74024c0dc83b94`); the portal
hardening suite re-ran 7/7 on the exact unchanged source with Peek in the tree
(`/tmp/bbr-peek-unlocked-portal-20260810T111056Z.log`, SHA-256
`766da3b1838414dc1c659612a17c1125e4a8813aa64f81e12b0a0e8fe8936d35`), confirming
the shared `NativeAboveWeb` portal host stays green. The suite is not yet part
of canonical sequential `just e2e`; the optimized Peek milestones
(`peek_request_to_present`, `peek_dismiss_to_origin`, `peek_promote_to_tab`)
are coded with budgets and 0 accepted samples — unexecuted and unpassed.

## 7o. The Saved Pages Library suite (`bbr-app/tests/e2e_shell_saved_pages_library.rs`)

`just e2e-shell-saved-pages-library` contains 12 real-app scenarios for the B3
packet (`saved-pages-library`) — the integrated production Library surface,
split across focused modules under `e2e_shell_saved_pages_library/` so no file
outgrows the ~500-line budget:

1. **Production entry and dismissal:** Cmd-Shift-L opens the internal chrome
   surface, which covers the native view while open; Escape dismisses and
   restores the native view exactly (hidden-view bookkeeping).
2. **Symmetric command-surface exclusion:** Cmd-T and Cmd-L replace an open
   Library, restore the exact hidden native view, and present only the selected
   command surface; no invisible Library authority or trapped focus survives.
3. **Deterministic search and sort:** keyboard and pointer search plus all
   deterministic sort orders expose typed semantics from the typed
   `SavedPagesLibrarySnapshot`.
4. **Inline pointer rename:** the draft is confirmed or cancelled through the
   production control, and the rename persists.
5. **Multi-select and destructive remove:** one or many records are selected;
   the remove confirmation is explicit; every open tab/session stays
   untouched and closing tabs leaves Library records untouched.
6. **Flat collection lifecycle:** create, rename, move selected records in and
   out, and remove a one-level collection; every mutation persists.
7. **Pointer selection toolbar:** the Clear / Remove / Move controls through
   their production pointer paths — Clear empties the set, Move opens the
   move-to sheet whose options move the selection in/out of a collection,
   and Remove stages the destructive confirmation for exactly the selection;
   the one AppStore authority's counts stay exact and tabs stay untouched.
8. **Pointer collection surface:** the create entry, the staged create/rename
   input with Confirm/Cancel, the collection-row rename→remove→none cycle,
   and the destructive collection remove confirmation (members move out
   first, records never lost) — all through the pointer paths.
9. **Production scroll-wheel path:** the bounded list's wheel clamp math
   feeding the one production scroll entry over the seeded 1,000-record
   provider; windows/anchors are exact and the wheel path is never a typed
   `SetSavedPagesLibraryScroll` substitute.
10. **Restart persistence:** rows, labels, and collections restore after a real
   app restart on the same profile.
11. **Exactly-one-fresh-tab activation:** opening a Library row routes through
    the B2 command authority — exactly one fresh regular tab in the currently
    active Space/Profile, never a silent selection of a matching tab.
12. **1k bounded window:** the seeded ≥1,000-record fixture proves bounded
    rows, deterministic scroll/selection with stable anchors, and the long
    title/URL + missing title/favicon fallbacks.

Every mutation routes through the one app-owned Library composition and the
one AppStore/B2 authority the typed snapshot exposes — never a second
persistence or identity policy. No sleeps: every step synchronizes on typed
snapshot state or semantic events under an explicit deadline, and the suite
takes the serial GUI lock.

Protocol (additive, still version 6): the `SavedPagesLibrarySnapshot` carries
the open flag, search/sort state, the bounded row window, selection/collection
membership, and the remove-confirm state; `library_set_search` (shared with
the keyboard path) and `library_set_scroll` (the tracked offset the wheel
listener drives) are the production seams; the benchmark kinds
`SavedPagesLibrarySearch` and `SavedPagesLibraryScroll` join the interaction
matrix in the B2 warm command-text / interaction classes — no new budget
class. The B3 optimized scenarios (`just e2e-perf`) exist with drivers,
fast-but-wrong guards, and budgets but have **zero executions**: no optimized
sample has been accepted, the boxes stay unchecked, and performance remains
unpassed under the active delivery override.

**Final semantic-union evidence (exact runtime tree
`b4ab98113c6582caad2150764815d3885589dfa7`):** the full 12/12 suite above —
log `/tmp/bbr-b3-review-remediation-e2e-shell-saved-pages-library-20260810T144111Z.log`,
SHA-256 `06838f3556a34bb46d7dfe20f8b5f6f3bdd3b6d7ce91a19a59b633dfffa11673`;
`just e2e-shell-saved-pages` 4/4 — log
`/tmp/bbr-b3-review-remediation-e2e-shell-saved-pages-20260810T144301Z.log`,
SHA-256 `440317455fbf499fada4515d404c1f460d0a5854a817490314e5ac65763273cc`;
Gallery 22/22 — log
`/tmp/bbr-b3-review-remediation-e2e-gallery-20260810T144301Z.log`, SHA-256
`bc41dd1df11d887dea03a10fa30949a33be05edb916462c59fd9cbd25a2ff7e9`;
and the command regression suite 25/25 — log
`/tmp/bbr-b3-review-remediation-e2e-shell-command-20260810T144111Z.log`, SHA-256
`953138116c7ad011402cba6a8269c75c9fed690cb4545df52d940a132bcc7b6f`.
All runs used Xcode 26.2 and the serial repository GUI lock. The final staged
tree differs from this runtime tree only in evidence/documentation wording.

## 8. Security & release exclusion

- The gateway module compiles **only** under `bbr-app`'s `testkit` feature
  (`#[cfg(feature = "testkit")]`); release builds exclude it.
- Socket path must be inside the profile dir (it is constructed there);
  mode `0700`; token required before any request is honoured; loopback/unix
  transport only — nothing externally reachable.
- Test-mode data stores: `nonPersistentDataStore` per profile (no WebKit
  container leakage between tests).
- FUTURE (not implemented): the exit-64 fail-safe — non-testkit builds
  currently **silently ignore** `--bbr-*` test flags instead of rejecting them
  at startup; the `strings`/socket-appearance release gate in CI is also
  future work.

---

## 8a. Worktree-safe GUI serialization

A sequential `just e2e` inside one process does not prevent another worktree
from starting a second AppKit suite. `scripts/with_gui_lock.py` closes that gap:

- all `just e2e-*` and performance recipes acquire one advisory `flock` derived
  from `git rev-parse --git-common-dir` before starting their existing bounded
  command;
- holder metadata records PID, Pi session id, and cwd for diagnostics;
- the OS releases the lock when a process exits, so no stale lock-file cleanup
  or destructive recovery is needed;
- lock waiting is bounded (default 1800 seconds, configurable through
  `BBR_GUI_LOCK_TIMEOUT_SECONDS`) and exits 75 with holder metadata on timeout;
- unit contract tests prove command exit forwarding, cross-process
  serialization, and timeout behavior before real-app validation.

The gate serializes execution only; it does not alter fixtures, presentation
milestones, samples, budgets, testkit transport, or product assertions.

## 9. Commands (implemented)

```bash
just e2e           # canonical sequential GUI gate: smoke, shutdown, tabs, profiles, shell-tabs, shell-spaces, shell-sidebar, shell-split-geometry, shell-organize, shell-hover, shell-dense, performance
just e2e-smoke     # baseline navigation/focus/resize
just e2e-shutdown  # deterministic process shutdown: SIGINT + owning-window close (incl. portal-child isolation)
just e2e-tabs      # multi-tab/session behavior
just e2e-profiles  # native profile isolation, moves, and suspension
just e2e-shell-tabs    # visible vertical-tab surface (pointer/keyboard/empty)
just e2e-shell-spaces  # visible profile/space surface (pointer + keyboard suites)
just e2e-shell-sidebar # adjustable/collapsible sidebar surface (drag, keys, collapse, clamp)
just e2e-shell-split-geometry # keyed per-WebViewId split panes, focus, resize, close (split anchor)
just e2e-shell-command  # command engine + U1 NativeAboveWeb palette/overlay + AppKit boundary evidence
just e2e-shell-saved-pages # Saved Pages command authority: Cmd-D toggle, restart survival, exactly-one-fresh-tab, production quit durability (B2)
just e2e-peek         # transient Link Preview / Peek: eligible request, dismissal/focus, same-session promotion, one-per-pane, exclusions, crash teardown (PEEK)
just e2e-shell-saved-pages-library # Saved Pages Library: entry/Escape, search/sort, rename, remove, multi-select, collections, pointer toolbar/collection surface, scroll-wheel path, restart, exactly-one-fresh-tab activation, 1k bounded window (B3)
just e2e-shell-hover    # pointer hover/press states + the internal §15 semantic tree
just e2e-perf      # optimized startup and interaction latency budgets
just e2e-perf-full # bounded repeated canonical gate (COLD_STARTUP_SAMPLES launches + 30-sample interactions)
just e2e-perf-inject # deterministic injected-delay self-test (fails under injection, green after removal)
just verify        # fmt + check + test + clippy + roadmap
BBR_SKIP_GUI_TESTS=1 just e2e   # skip GUI suites on headless CI
```

```text
just e2e-perf  → exit 0 — startup p50 ≈ 393 ms (< 750 max ceiling); page-ready p50 ≈ 520 ms; warm-tab p95 0.07 ms; space-switch p95 ≈ 0.07 ms; sidebar-toggle p95 0.82 ms; sidebar-resize p95 ≈ 0.002 ms; sidebar-restore p95 ≈ 0.002 ms (both against the 50 ms next-frame budget); Cmd-T p95 ≈ 1.0 ms; new-tab open p95 ≈ 48 ms; active-close p95 ≈ 1.3 ms (2026-08-06, released-testkit build on this host)
just verify    → exit 0 — fmt/check/test/clippy/roadmap all green, 2026-08-06
```

FUTURE: `just e2e [suite [::case]]`, `cargo nextest run -p bbr-e2e`, and
`python3 scripts/run_e2e.py --suite … --tag …` will map onto the planned
runner + suites.

---

## 10. Risks & mitigations (updated)

| Risk | Mitigation |
|---|---|
| WebKit identifier stores leak across tests | Test mode forces `nonPersistentDataStore` (any `--bbr-test-token` run); E2E never depends on WebKit's on-disk stores |
| Events racing between requests and asserts | Seq cursor + `wait_for_event_since`; server ring for replay; events queued client-side during round trips |
| Wedged app hangs the suite | 30 s main-thread reply timeout on the gateway; 60 s connect deadline; `SIGKILL` in harness Drop |
| Synthetic keystroke fidelity drift | `DispatchKeystroke` goes through GPUI's real `dispatch_keystroke` path; FUTURE: tier-2 NSEvent posting in the webkit suite |
| macOS CI headless constraints | WKWebView needs a window + GUI session; `BBR_SKIP_GUI_TESTS=1` for headless; real smoke requires a self-hosted/GUI Mac runner. Verified locally on the installed Xcode 26.2.0 Metal toolchain in a display-attached session |
| Event-ring memory growth | Fixed cap (4096) + client cursors; snapshot on demand |
| Protocol evolution | `PROTOCOL_VERSION = 5` added command-bar modes, typed benchmark operation IDs/presentation measurements, a deterministic `PageReady` milestone, and request types. `PROTOCOL_VERSION = 6` adds per-tab `pinned`/`loading`/`crashed` flags and per-row `tab_states` + resolved `shell_modes` to snapshots, the pointer-path `ClickTab`/`ClickTabClose` and `ReopenTab`/`PinTab` requests, deterministic `InjectWebEvent` (rides the real event transport), and the `TabSelectPointer`/`TabReopen` benchmark kinds. Additive growth after v6 (still version 6): snapshots expose the rendered `bbr-*` `visible_ids`, per-row `row_titles` (after §4 truncation), `active_profile_title`, the typed sidebar geometry (`sidebar_width`/`sidebar_min`/`sidebar_default`/`sidebar_max`, `content_min`, `divider_dragging`, `divider_label`) and the last engine-received native `content_rect`; `TabActivateKeyboard`, `SidebarResize`, and `SidebarRestore` join the benchmark kinds; `DragSidebarDivider`/`FocusSidebarDivider` join the pointer seams and `FlushFrame` the paint drivers; the hover/press seams (`HoverTabRow`/`UnhoverTabRow`/`PressTabRow`/`ReleaseTabRow`) plus the `BenchmarkHoverTab`/`BenchmarkPressTab` kinds and the typed §15 `semantics` tree join the protocol; the split shared contract adds the `SetSplit`/`CloseSplit` requests, `SplitOpened`/`SplitClosed` outcomes, and the keyed `pane_layout`/`pane_rects` snapshot fields (test seam only — production split UI/perf kinds stay future); command composition adds `FlushCommandRanking`, typed command result/cursor/selection plus complete `all_tabs` snapshot evidence, and the `CommandType`/`CommandActivate`/`CommandActivateNewTab` benchmark kinds while retaining protocol v6; `ServerFrame::Response` is boxed internally (JSON wire shape unchanged); future additive growth stays typed |
| Dedicated-runner / external gaps | No dedicated stable macOS runner or public trend publication is provisioned yet. `canonical_host_model` is therefore empty, so the >15% relative baseline rule stays report-only until a runner is named; display refresh is recorded `null` with an explicit reason (window is offscreen and the OS refresh API is not linked). Absolute ceilings are enforced on every machine. F3 integrated persisted warm startup through the real repository; canonical-runner distributions, resource metrics, and measured dense 500/50 frame-time evidence remain open. |

---

## 11. Implementation order (status)

1. ✅ Restore the build + Phase-1 spike (bbr-app, bbr-web, bbr-web-macos;
   `NativeWebViewSlot`, `ChromeResponder`, navigation, focus round-trip).
2. ✅ Testkit protocol + gateway + client (`bbr-testkit`), `bbr-fixtures`,
   smoke test with attach-replay, `_since` cursors, resize + focus asserts,
   graceful-quit assert (this doc's §3–§7).
3. ✅ Portable domain (`bbr-domain`, 40 tests) and browser orchestration
   (`bbr-browser`, 47 tests), real-app controller composition, Protocol v5,
   multi-session tabs, and native profile-isolation/lifecycle E2E.
4. ▶ Suite growth: visible space switching landed with the shell-spaces
   suites and the `space_switch` benchmark; the adjustable/collapsible
   sidebar landed with the shell-sidebar suite and the
   `sidebar_resize`/`sidebar_restore` benchmarks; the real-app command surface
   landed with `just e2e-shell-command` plus the render-inclusive
   `command_type` / `command_activate` scenarios and, with the Phase-7
   command-history provider, the history-rich `command_bar_first_result` /
   `command_bar_full_search` scenarios (the complete limit-16 result window
   plus the truthful total, queue item 16); production split UI, dialogs,
   routing, remaining WebKit essentials, persistence, accessibility, and
   broader performance scenarios follow Phases 6/8/7/9 on the implemented
   foundation.
5. ⏳ Release-exclusion gate in CI (exit 64 + symbol/socket checks).

Principle: **BBR's first executable ships with testkit sockets, sequential
ids, an event ring, cursor-based waits, and a real-WKWebView smoke — every
later feature's testability is inherited, not retrofitted.**
