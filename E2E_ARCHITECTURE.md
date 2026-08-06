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
| Workspace compiles and all unit/contract tests pass | `just verify` green: bbr-app 2, **bbr-domain 40**, **bbr-browser 47**, bbr-fixtures 4, bbr-testkit 13, bbr-web 9, bbr-web-macos 1 passed |
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

- Client → `Hello { protocol_version: 6, token, capabilities }`. Wrong version
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
| `WebAction(Back/Forward/Reload/Stop)` | Engine action on the active view |
| `ActivateCommandBar` / `FocusWeb` | The E2E readers of Cmd-L / Escape |
| `SetCommandBarText { text }` | Set command-bar text directly (avoids per-keystroke round trips) + broadcasts `CommandBarChanged` |
| `DispatchKeystroke { keystroke }` | GPUI `Keystroke::parse` + `Window::dispatch_keystroke` (real input path) |
| `BenchmarkSelectTab` / `BenchmarkToggleSidebar` / `BenchmarkDispatchKeystroke` / `BenchmarkResizeSidebar` / `BenchmarkRestoreSidebar` | Start typed in-process production-path measurements and finish only after the scenario's semantic presentation milestone |
| `DragSidebarDivider { dx }` / `FocusSidebarDivider` | Drive the exact production pointer/focus listener methods the divider element handlers call (never a controller mutation) |
| `FlushFrame` | Force one synchronous real GPUI draw (`Window::draw`) so paint-driven native geometry callbacks fire under the hidden harness (no display link) |
| `SetWindowSize { width, height }` | `window.resize`; native view re-framed next frame |
| `GetSnapshot` | Full typed `Snapshot` |
| `Ping` | Liveness |
| `Quit` | `cx.quit()` (graceful shutdown, clean exit) |

### Events (implemented)

`AppReady { pid, app_version }` (once, after the gateway is up) ·
`Web(bbr_web::WebEvent)` (every engine event: url/title/loading/canGoBack/
canGoForward/page-crash/navigation-failure) · `ChromeFocusChanged {
chrome_focused }` (emitted by the app's focus sink after every focus mutation,
so it always agrees with the snapshot) · `CommandBarChanged { mode, text }` ·
`BrowserChanged` with typed profile/space creation and activation plus tab
open/select/close/move/suspend outcomes from the shared app mutation path.
`Event::Quitting` is declared in the protocol but **not yet emitted** (the
harness asserts clean exit by process status instead).

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
  active_tab_index: Option<usize>, active_web_view: Option<WebViewId>,
  engine_sessions: usize,       // native destruction / warm-session evidence
  engine_kind: String,          // "native-wkwebview" (macOS) | "fake" elsewhere
  chrome_focused: bool,
  shell_modes: ShellModes,      // theme/contrast/density (§15)
  tab_states: Vec<TabVisualState>, row_titles: Vec<String>,
  space_states: Vec<SpaceVisualState>, space_titles: Vec<String>,
  visible_ids: Vec<String>,     // rendered bbr-* ids (§15)
  command_bar_mode: Inactive | Location | NewTab | NewSpace,
  command_bar_text: String,
  sidebar_visible: bool,
  sidebar_width: f32,        // effective rail width (0 when collapsed)
  sidebar_min / sidebar_default / sidebar_max: f32,  // DESIGN §5 adjustable range
  content_min: f32,          // native-content floor (never zero/negative geometry)
  divider_dragging: bool,    // §15 divider drag state
  divider_label: String,     // stable label shared with future Accessibility
  content_rect: [f32; 4],    // last engine-received WKWebView frame (x,y,w,h)
  window_size: [f32; 2],     // points
}
```

FUTURE (planned protocol growth, not yet implemented): remaining tab/split manipulation
(`PinTab/ReorderTab/ReopenTab/SplitTabs/CloseSplit`),
`WebEvaluate` (whitelisted DOM helpers), `AdvanceClock`/
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
    relative to the screen-clamped starting geometry and polls typed snapshots
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
7. `SetWindowSize` with a target relative to the current screen-clamped size
   → bounded snapshot polling → geometry changed (width delta > 50 pt).
8. `DispatchKeystroke("cmd-l")` → `ChromeFocusChanged{true}` → snapshot
   `chrome_focused`; `SetCommandBarText(page2)` → snapshot text; then
   `DispatchKeystroke("escape")` → `ChromeFocusChanged{false}` (focus returned
   to the web view).
9. `Quit` → process exits with success status within 10 s.

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
gates build and run on this host via the Xcode 16.4.0 Metal toolchain
(`DEVELOPER_DIR` set by the just recipes) in a display-attached session.

FUTURE: visible spaces/splits/dialogs/routing/persistence scenarios,
persisted-profile warm startup (depends on the absent persistence repository),
naming a dedicated canonical runner so `canonical_host_model` is populated and
the >15% relative rule activates, and the `bbr-e2e` crate with `manifest.toml`
per suite, `scripts/run_e2e.py` runner, `nextest.toml` `[profile.e2e]`, and
`docs/e2e-protocol.md` remain planned.

## 7d. The shell-spaces suites (`bbr-app/tests/e2e_shell_spaces{,_keyboard}.rs`)

Two `#[ignore]`d macOS-only suites (the shared harness lives in
`tests/e2e_common/`) prove the visible profile/space surface over real
WKWebViews:

1. **Pointer suite** (`just e2e-shell-spaces`): complete typed profile/space
   collections with §12 round-robin hues and stable `bbr-space-row-<n>` /
   `bbr-profile-chip-<n>` ids, active/selected `space_states`, pointer row
   activation through the production listener (chrome focus domain preserved),
   per-space active-tab restoration with the same WebViewId and **zero load
   events in the replay window**, empty-space pane naming from
   `active_profile_title`, cross-profile view-only switches that keep cookie
   isolation (each store reads only its own cookie) without recreating warm
   sessions, and a cross-profile tab move that destroys + recreates the
   session in the destination store (`recreated: true`, fresh WebViewId
   reading the destination store's cookie).
2. **Keyboard suite** (`just e2e-shell-spaces` second binary): `Cmd-Shift-N`
   staging (Escape cancels, Enter commits + activates, `SpaceCreated` is
   broadcast), real `↑/↓` traversal of the focused space list, and same-
   profile moves that keep the WebViewId (`recreated: false`) versus
   cross-profile moves that recreate it.

The `space_switch` interaction benchmark (in `tests/perf_common/benchmarks.rs`)
measures alternating warm-space activation through `BenchmarkActivateSpace`,
asserting the destination space, its own active view, the unchanged live-
session count, chrome focus, and the rendered selected space row + chip ids;
it is part of `just e2e-perf` and the full gate.

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

## 9. Commands (implemented)

```bash
just e2e           # canonical sequential GUI gate: smoke, tabs, profiles, shell-tabs, shell-spaces, shell-sidebar, performance
just e2e-smoke     # baseline navigation/focus/resize
just e2e-tabs      # multi-tab/session behavior
just e2e-profiles  # native profile isolation, moves, and suspension
just e2e-shell-tabs    # visible vertical-tab surface (pointer/keyboard/empty)
just e2e-shell-spaces  # visible profile/space surface (pointer + keyboard suites)
just e2e-shell-sidebar # adjustable/collapsible sidebar surface (drag, keys, collapse, clamp)
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
| macOS CI headless constraints | WKWebView needs a window + GUI session; `BBR_SKIP_GUI_TESTS=1` for headless; real smoke requires a self-hosted/GUI Mac runner. Verified locally on the Xcode 16.4.0 Metal toolchain in a display-attached session |
| Event-ring memory growth | Fixed cap (4096) + client cursors; snapshot on demand |
| Protocol evolution | `PROTOCOL_VERSION = 5` added command-bar modes, typed benchmark operation IDs/presentation measurements, a deterministic `PageReady` milestone, and request types. `PROTOCOL_VERSION = 6` adds per-tab `pinned`/`loading`/`crashed` flags and per-row `tab_states` + resolved `shell_modes` to snapshots, the pointer-path `ClickTab`/`ClickTabClose` and `ReopenTab`/`PinTab` requests, deterministic `InjectWebEvent` (rides the real event transport), and the `TabSelectPointer`/`TabReopen` benchmark kinds. Additive growth after v6 (still version 6): snapshots expose the rendered `bbr-*` `visible_ids`, per-row `row_titles` (after §4 truncation), `active_profile_title`, the typed sidebar geometry (`sidebar_width`/`sidebar_min`/`sidebar_default`/`sidebar_max`, `content_min`, `divider_dragging`, `divider_label`) and the last engine-received native `content_rect`; `TabActivateKeyboard`, `SidebarResize`, and `SidebarRestore` join the benchmark kinds; `DragSidebarDivider`/`FocusSidebarDivider` join the pointer seams and `FlushFrame` the paint drivers; `ServerFrame::Response` is boxed internally (JSON wire shape unchanged); future additive growth stays typed |
| Dedicated-runner / external gaps | No dedicated stable macOS runner or public trend publication is provisioned yet. `canonical_host_model` is therefore empty, so the >15% relative baseline rule stays report-only until a runner is named; display refresh is recorded `null` with an explicit reason (window is offscreen and the OS refresh API is not linked). Absolute ceilings are enforced on every machine. Persisted warm startup stays open until the persistence repository lands — a test-only seed that bypasses the future persistence path is not acceptance evidence. |

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
   `sidebar_resize`/`sidebar_restore` benchmarks; splits, dialogs, routing,
   and complete command-bar results land with their phases; remaining WebKit
   essentials, persistence, accessibility, and broader performance scenarios
   follow Phases 8/7/9 on top of the implemented performance foundation.
5. ⏳ Release-exclusion gate in CI (exit 64 + symbol/socket checks).

Principle: **BBR's first executable ships with testkit sockets, sequential
ids, an event ring, cursor-based waits, and a real-WKWebView smoke — every
later feature's testability is inherited, not retrofitted.**
