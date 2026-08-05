# BBR — E2E Architecture (current implementation)

Status: **describes what is implemented and verified** — the testkit gateway
protocol, the fixture server, and the first executable smoke test that drives
the real `bbr` binary with a real WKWebView. Planned-but-unbuilt parts are
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
| Workspace compiles and all unit/contract tests pass | `just verify` green: `cargo check --workspace --all-targets` exit 0; `cargo test --workspace` exit 0 — bbr-app 4, **bbr-domain 40**, **bbr-browser 46**, bbr-fixtures 3, bbr-testkit 10, bbr-web 9 passed |
| Real-app E2E smoke passes against a real WKWebView | `just e2e-smoke` → `SMOKE PASSED`, 1 passed (1.53s), including navigation, back/forward/reload, resize, focus round-trip, graceful quit |
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
bbr-app/tests/e2e_smoke.rs (harness, one #[ignore]d test, macOS-only)
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

## 3. The testkit protocol (implemented: `bbr-testkit`, version 1)

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

- Client → `Hello { protocol_version: 1, token, capabilities }`. Wrong version
  → `Error("protocol_mismatch")`; wrong token → connection dropped. Correct →
  `Welcome { app_version, pid, capabilities, next_seq }`.
- Token comes from `--bbr-test-token` (random per spawn, in the harness).
- Every other request before a successful Hello is refused (connection dropped).

### Requests (implemented)

| Request | Effect |
|---|---|
| `Attach { since: Option<u64> }` | Replays seq-numbered events from the server ring with seq > `since`; live delivery continues after |
| `Navigate { url }` | Load the active web view (same path as the command bar; keeps command-bar text consistent) + broadcasts `CommandBarChanged` |
| `WebAction(Back/Forward/Reload/Stop)` | Engine action on the active view |
| `ActivateCommandBar` / `FocusWeb` | The E2E readers of Cmd-L / Escape |
| `SetCommandBarText { text }` | Set command-bar text directly (avoids per-keystroke round trips) + broadcasts `CommandBarChanged` |
| `DispatchKeystroke { keystroke }` | GPUI `Keystroke::parse` + `Window::dispatch_keystroke` (real input path) |
| `SetWindowSize { width, height }` | `window.resize`; native view re-framed next frame |
| `GetSnapshot` | Full typed `Snapshot` |
| `Ping` | Liveness |
| `Quit` | `cx.quit()` (graceful shutdown, clean exit) |

### Events (implemented)

`AppReady { pid, app_version }` (once, after the gateway is up) ·
`Web(bbr_web::WebEvent)` (every engine event: url/title/loading/canGoBack/
canGoForward/page-crash/navigation-failure) · `ChromeFocusChanged {
chrome_focused }` (emitted by the app's focus sink after every focus mutation,
so it always agrees with the snapshot) · `CommandBarChanged { text }`.
`Event::Quitting` is declared in the protocol but **not yet emitted** (the
harness asserts clean exit by process status instead).

### Snapshot (implemented)

```rust
Snapshot {
  url, title: Option<String>, loading: bool,
  can_go_back, can_go_forward: bool,
  web_view_hosted: bool,        // a live native WKWebView is hosted
  engine_kind: String,          // "native-wkwebview" (macOS) | "fake" elsewhere
  chrome_focused: bool,
  command_bar_text: String,
  sidebar_visible: bool,
  window_size: [f32; 2],        // points
}
```

FUTURE (planned protocol growth, not yet implemented): model manipulation
(`CreateProfile/CreateSpace/AddTab/ActivateTab/CloseTab/PinTab/MoveTab/
SplitTabs/CloseSplit`), `WebEvaluate` (whitelisted DOM helpers), `AdvanceClock`/
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
  (focus sink), `CommandBarChanged` (navigate/set-text dispatch).

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

FUTURE: the 12-suite plan from the original design (smoke/tabs/spaces/splits/
dialogs/routing/commandbar/webkit/persistence/perf), the `bbr-e2e` crate with
`manifest.toml` per suite, `scripts/run_e2e.py` runner, `nextest.toml`
`[profile.e2e]`, and `docs/e2e-protocol.md` are the planned growth path; none
exist yet. Each product phase is expected to ship its suite in the same PR
(every phase's coverage is a definition of done).

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
just e2e-smoke     # the smoke above (CI fast lane), DEVELOPER_DIR=Xcode 16.4, 2m timeout
just verify        # fmt + check + test + clippy (unit/contract gate)
BBR_SKIP_GUI_TESTS=1 just e2e-smoke   # skip on headless CI
```

```text
just e2e-smoke  → exit 0 — "SMOKE PASSED", 1 passed, 1.53s (real WKWebView)
just verify     → exit 0 — fmt/check/test/clippy all green, 2026-08-05
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
| macOS CI headless constraints | WKWebView needs a window + GUI session; `BBR_SKIP_GUI_TESTS=1` for headless; real smoke requires a self-hosted/GUI Mac runner |
| Event-ring memory growth | Fixed cap (4096) + client cursors; snapshot on demand |
| Future model-driven events vs. current single-view snapshot | The protocol types are versioned (`PROTOCOL_VERSION = 1`); growth is additive (`Setup`-style new requests/events) until a breaking bump |

---

## 11. Implementation order (status)

1. ✅ Restore the build + Phase-1 spike (bbr-app, bbr-web, bbr-web-macos;
   `NativeWebViewSlot`, `ChromeResponder`, navigation, focus round-trip).
2. ✅ Testkit protocol + gateway + client (`bbr-testkit`), `bbr-fixtures`,
   smoke test with attach-replay, `_since` cursors, resize + focus asserts,
   graceful-quit assert (this doc's §3–§7).
3. ▶ portable domain slice (`bbr-domain`, 40 tests) — done; `bbr-browser`
   orchestration (controller + in-memory fake `WebEngine`) — done at the
   framework-free contract level (46 tests, no display server); the real-app
   multi-session composition and its tab E2E are Phase 3.
4. ⏳ Suite growth: tabs/spaces/splits/dialogs/routing/commandbar model events
   land with their phases; webkit/persistence/perf suites follow Phases 3/7/9.
5. ⏳ Release-exclusion gate in CI (exit 64 + symbol/socket checks).

Principle: **BBR's first executable ships with testkit sockets, sequential
ids, an event ring, cursor-based waits, and a real-WKWebView smoke — every
later feature's testability is inherited, not retrofitted.**
