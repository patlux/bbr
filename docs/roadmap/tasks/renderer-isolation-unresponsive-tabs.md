# Task packet — Renderer Isolation & Unresponsive Tabs (first-MVP vertical lane)

Planning basis: an independent read-only research run (planning output kept
in /tmp only; repository unmodified; no run-identifier hex preserved here).
Parents: [Phase 8 — Daily-driver essentials](../../../ROADMAP.md) ·
[native-integration track](../tracks/native-integration.md) ·
[testing track](../tracks/testing.md) ·
[accessibility track](../tracks/accessibility.md) ·
[performance contract](../../product/performance-contract.md) ·
[interaction contract](../../product/browser-interactions.md)

Status: **Slice 2 settled ready and is under independent read-only review.**
PEEK and B3 are integrated through control checkpoint
`492615b571c98a14f9132a6bfd6f4c53395cdc3b` / tree
`4bc6a6ed6906ce01fe96d620ef1220c92eb94a63`. The Orchestrator created branch
`parallel/renderer-isolation` and worktree
`/Users/patwoz/dev/Personal/macos/bbr-renderer-isolation` from committed
readiness authority `a264cd4575e2f2cbe85907f14b8f65abb168bf18` / tree
`0b0f49386e18589262ae8e5580bce18f5633f554`. The first writer settled with a
contradictory `CANDIDATE_READY`: its report explicitly completed only the
portable/controller slice and named the macOS adapter, product UI, testkit,
fixtures, real-app E2E, performance code, and documentation as still absent.
That token therefore authorizes no review, commit, or integration.

The Orchestrator rebound the preserved process-free 18-path forward state:
HEAD and real index remain `a264cd4575e2f2cbe85907f14b8f65abb168bf18` /
`0b0f49386e18589262ae8e5580bce18f5633f554`; temporary-index tree
`6f7d2dc930b6fa0b3580b222afd7972a2c58557d`; binary diff SHA-256
`fbd1a53879b89bc10c904482cf7ad2044237e70379d004b0fd0754be955c2dcf`;
porcelain-v1 status SHA-256
`12cddf5f528223f96d8e38f0fc0d15f1cab512eab4c94ab8aa751f881beb0731`;
sorted 18-path SHA-256
`7ac8ffbe20360e82f1a72ffc2a49d332fb13c8961773156fe2219cbb904a5b12`;
diff-check clean, no worktree process, GUI lock free. The 648-line new
`bbr-browser/src/responsiveness.rs` is an explicit unresolved module-budget
violation and must be split below ~500 lines without weakening its tests.
The first continuation split the 648-line portable monitor into 386-line
implementation plus 287-line tests and advanced a focused 270-line macOS probe
adapter, then returned direct `BLOCKED` before compilation or any later packet
layer. Exact latest process-free state: temporary-index tree
`f8d520d5aaa58b00c51ba673e80b56da0ee61c2b`; binary diff SHA-256
`63dab1fea7445a24e283a185e0c0dac456c332a7aab11a9a06ccc09b2a19617a`;
porcelain-v1 status SHA-256
`a1fd4f201897bbbd4c144edec3a40b05ff9b760a311c2fb3c41b64d3500f94e0`;
sorted 24-path SHA-256
`b8de2ebe125a5007f1fda38ce97a1c64f1b7c8126eed8e2afbda4b5905d90a1e`;
real index remains the base tree, no process survives, and the GUI lock is
free. Four known compile repairs remain: merge the probe method into the one
existing `WebEngine` impl; import `AnyThread` for `NSUUID::alloc`; enable or
replace the unavailable public `NSTimer` block API; clone the shared sink.
`webview.rs` is 532 lines and must return to at most its audited 530-line
no-grow bound. The full temporary-tree diff check also flags one trailing blank
line in the new portable module. UI, testkit/process observation, fixtures,
real-app E2E, performance code, docs, and all final gates remain absent.

The next recovery repaired all four macOS compiler findings, returned
`webview.rs` to 519 lines, kept the split portable modules bounded, and began
the product UI/snapshot/semantics wiring. It returned direct `BLOCKED` only
because its shell tool disappeared; no product/platform blocker exists.
Independently rebound latest state: temporary-index tree
`6065c57360a5da7c1e0be420f79588977e684a55`; binary diff SHA-256
`bf2e686b3250d5ab922aa4efaf160b343713e5bcee8a9314a13d135b83d8776f`;
porcelain-v1 status SHA-256
`1752208170bbe311691e5c32dd5a0891519d0797095cc3429e24f03fd2bc44c6`;
sorted 41-path SHA-256
`d7ede995795a735e6481482cd657d8bd09595620a197464a0ffbc5badcd77f34`;
real index remains base, no process survives, GUI lock free. Full temporary
-tree diff-check still flags only the trailing blank line in the portable
module. Product app render/actions/cadence/event handling and dialog boundary,
testkit/process observation, fixtures, E2E, performance code, docs, and final
gates remain.

To avoid repeating transport failures on one oversized writer, remaining
forward recovery is serial and reviewable in the same worktree, never
concurrent: (1) macOS + complete product surface/actions/cadence/dialog slice;
(2) testkit process observation + deterministic fixtures; (3) regression E2E
+ performance code + docs + complete gates. Each slice preserves all prior
work and receives independent read-only review before ownership advances.
The first product-surface writer reproduced its exact authority, cleared the
diff finding, completed compiling macOS dialog auto-dismiss plus most
production app rendering/actions/cadence/event/visibility wiring, and passed
both testkit and release `bbr` checks. Its turn ended before snapshot mirror,
focused tests, and final slice gates, and it explicitly returned neither
`PRODUCT_SLICE_READY` nor `BLOCKED`; therefore review is not authorized.
Independent process-free rebind: temporary-index tree
`54f89f2cd01a39968879564648f93ef558671b59`; binary diff SHA-256
`5d45cee85287e95ae4172d6273622ccd68a55c850e2f71c68678ed4ce14326d8`;
porcelain-v1 SHA-256
`deabdd374f1d7d97a26b6521fb1a3c5d47968d6c07c18456ac77876030024b79`;
sorted 53-path SHA-256
`fa6c4dfd071b527f560cca38c576266955e3cfae5c6532066c7f03b09819d987`;
real index remains base, temp-tree diff-check clean, no process, GUI lock free.
`webview.rs` is 519 lines, fixtures `lib.rs` remains 839, and all new listed
modules remain below 500.

The product-surface continuation returned direct `PRODUCT_SLICE_READY` after
completing snapshot/visible-id/semantic/accessibility mirrors, fixing exact
native-view hide/restore precedence, auditing recovery/cadence/dialog/stale
behavior, and adding focused tests. It reports green fmt/diff, Web 322, Browser
214, macOS 107, UI 75, testkit app 273, release app 215, release/testkit checks,
and warning-denied Clippy. Independent rebind confirms exact temporary-index
tree `e7c8ab85f5f62a5822e3291ecf9b03ec9f1ee130`; binary diff SHA-256
`f1ceb6fecf0a3543f019873e3a5986b896f7a894fff52dd6d52f35073015a07c`;
porcelain-v1 SHA-256
`d109949bbea4847ece4db80f5fc3b84b1b9b1d3623c98acb9a155008d5e54fc2`;
sorted 54-path SHA-256
`357209b2ee28797e58268954211fd5d36a01217ef7cfa83c4078f35200c5f9c6`;
real index remains base, temp-tree diff-check clean, no process, GUI lock free.
No-grow surfaces remain 519/839 lines and all new files are below 500.

Independent read-only review reproduced the exact 54-path candidate before and
after review, accepted all Slice-1 product, architecture, authority, stale /
crash / probe / dialog / pane / focus / cadence, scope, test and evidence
requirements, and reran a broad critical non-GUI set green. It returned direct
`PRODUCT_SLICE_FINDINGS` for exactly three LOW mechanical module-budget
crossings introduced by this lane: `bbr-browser/src/controller.rs` 516 lines,
`bbr-ui/src/icons.rs` 510 lines, and
`bbr-app/src/shell/snapshot/tests.rs` 519 lines. No behavior, assertion, test,
evidence, scope or authority finding exists. Candidate fingerprints remain
exact `e7c8ab85…` / `f1ceb6fe…` / `d109949b…` / 54 paths
`357209b2…`, real index base, diff clean, no process, GUI lock free.

The sole bounded remediation writer returned direct
`PRODUCT_SLICE_REMEDIATED`. Independent Orchestrator rebind proves exact new
temporary-index tree `6abd88e049262254d10c6fe75957f21d74a099ae`;
binary diff SHA-256
`2c89eb2154249a55c59b655dd32ab56597df573387f9feb3416e66eb78d65549`;
porcelain-v1 `-z` SHA-256
`06b34a39a3a4b1f1096328915e4f3dc5553c0af70c47e9c3bcd32b01a253ff21`;
sorted 56-path SHA-256
`69353ec4489749120cd21d969818929f823d9d544ce6843267049fab2a1ba7b5`.
The delta from reviewed tree `e7c8ab85…` is exactly six files: controller
crash handling moved into its existing responsiveness module, five private
icon paint primitives moved into `icons/paint.rs`, and the unresponsive
visible-id test moved into `tests/unresponsive.rs`. Resulting module counts are
498/160, 424/104 and 393/135 lines. Real index remains exact base
`0b0f4938…`; worktree equals the temp tree; staged diff and diff-check are
empty; no process remains and the exact GUI lock is free. Focused gates pass:
Browser 214, UI 75, snapshot 10, testkit app 273, warning-denied Clippy for all
three affected crates, release checks, fmt and diff-check. Evidence log
`/tmp/remediation-evidence.log` has SHA-256
`af47ee7f5dbb260fb67b0fff95a93dbdf75a5f893fec9c01c153c5a23b6d1781`.

The first independent read-only recheck was started before Patrick's subagent
model override and later failed with `Subagent produced no assistant output`.
It emitted no verdict, finding, mutation, or authority. After the override was
committed, replacement reviewer
`renderer-product-budget-recheck-replacem-msnkcwtv-n3ogmj` used
`homelab-llm/gpt-5.6-sol-fast:high`, independently reproduced the exact
56-path candidate, six-file extraction delta, module budgets, focused evidence,
empty staged/diff-check state, zero processes and free GUI lock, and returned
direct `PRODUCT_SLICE_RECHECK_PASS` with no mutation. Slice 1 is therefore
accepted.

Slice-2 writer `renderer_testkit_fixtures_writer` returned direct
`TESTKIT_FIXTURES_SLICE_READY`. The Orchestrator independently reproduced the
unchanged temporary-index tree
`9b6624faed76ab7ec1da9df81533172e3662faad`; HEAD-to-tree binary diff SHA-256
`3f5d2b4c6566c6b4a4eb2106078e7c5dec45bbd86a024c2c1fb15f85b1c4e0cf`;
porcelain-v1 `-z` SHA-256
`700f4f20865149a23a40e27a582c0f2d37c93be3827282d5666112d9e5354853`;
sorted 70-path SHA-256
`c474afccce09d81e646c868723380ceddc1256fc0fda1eb6725a454cd2aea7f2`;
and exact Slice-1-to-Slice-2 binary-diff SHA-256
`bea06769a99c4f6d1ece6081de32505bf213e971ac15ce0c7aa344f67d1331ac`.
The delta is exactly 17 additive testkit, fixture, and manifest paths. It adds
typed protocol-v7 responsiveness/helper requests, structured observation,
public `NSRunningApplication` enumeration, public libproc parent-PID proof,
deterministic lowest-PID selection with immediate ownership recheck and at
most one normal termination request, plus modular busy/heavy/dialog/
beforeunload fixtures. Release builds exclude the observation module through
the new macOS `testkit` feature. Writer evidence reports fixtures 11, testkit
29, default/testkit macOS 107+9 and 111+9, default/testkit app checks,
warning-denied app Clippy, Web 322+18+1 and Browser 214 green. Real index and
staged state remain base/empty; diff-check passes; no process or Git lock
remains; GUI lock is free. `webview.rs` remains 519 lines, fixture `lib.rs` is
652, and all new modules remain below 500. No real-app E2E, performance code,
broad documentation, commit, merge, or integration entered Slice 2.

Exactly one independent read-only Slice-2 reviewer now owns review of the
immutable candidate. It must verify public-API and parent-PID safety,
fail-closed exactly-one semantics, release/testkit gating, typed wire and
fixture behavior, Slice-1 preservation, module budgets, scope and evidence.
Only a direct Slice-2 pass authorizes Slice 3. Only the Orchestrator transfers
ownership or commits/integrates after the final whole-candidate pass.

---

## Outcome

A hung or dead renderer never blocks the BBR chrome. While the active page's
WebKit WebContent process is unresponsive, the user can still close the tab
(Cmd-W, row close button, focused-list ⌫), switch to the neighbor tab, operate
the sidebar, command bar, spaces, and splits, and choose a bounded recovery —
all on the next practical frame and inside the existing interaction budgets.
Unresponsiveness is detected deterministically and fail-closed (never a
blocking wait, never an unbounded poll), surfaced as a distinct
`unresponsive` tab state plus a dedicated pane with `Wait` / `Reload` /
`Close tab`, and cleared when the renderer answers again. No stale renderer
event, liveness-probe completion, JS-dialog completion, or termination
callback can mutate a closed or recreated session. The lane records how
WebKit actually shares WebContent processes on the target OS and designs the
policy so BBR never relies on a one-process-per-tab guarantee that public API
does not make.

## Scope

- **Portable renderer-availability model** in `bbr-browser`: per-session
  `RendererAvailability ∈ {Unknown, Responsive, Unresponsive, Crashed}`
  plus a monitor generation; the liveness probe lifecycle
  (arm → in-flight → complete/timeout → backoff), the deadline policy, and
  the closed recovery-action set. Crashed stays owned by the existing
  `on_page_crash` authority (§11.8 crash panel); `Unresponsive` is a new
  sibling state that shares the same "destroy session, keep tab" recovery
  seam.
- **Minimal `bbr-web` contract growth** (serial kernel, additive only):
  - `WebEvent::UnresponsiveChanged(WebViewId, bool)` — `true` when the
    renderer stopped answering, `false` when it answers again;
  - `WebEngine::probe_responsiveness(&mut self, id: WebViewId)` — refresh the
    adapter's bounded liveness probe for one live view; main-thread only,
    never blocks, results arrive as events; unknown/destroyed ids are no-ops.
  No new `WebAction`: recovery is the existing controller destroy-session →
  recreate-session authority (the §11.8 crash-Reload path), because a hung
  renderer may never honour `Reload`.
- **macOS probe adapter** in `bbr-web-macos`: a per-view bounded deadline
  timer plus `evaluateJavaScript_inFrame_inContentWorld_completionHandler`
  in `WKContentWorld::defaultClientWorld` (the same client-world seam the
  find adapter uses — page scripts cannot observe or collide with the probe).
  Deadline expiry marks `Unresponsive(true)`; a completing probe marks
  `Unresponsive(false)`; completions are generation-checked so a late answer
  from a previous probe never clears a newer timeout, and completions for
  destroyed/recreated views are dropped (stale-id discipline mirrors
  `ControllerEvent::UnknownWebView`).
- **Chrome surface**: a new §10.11 `unresponsive` row state (lifering + clock
  companion, distinct from §10.7 `error`), a §11.8-adjacent unresponsive pane
  (`bbr-unresponsive-panel`, `bbr-unresponsive-wait`,
  `bbr-unresponsive-reload`, `bbr-unresponsive-close`) with focus order
  Wait → Reload → Close tab → sidebar, and typed snapshot + semantic nodes.
- **Chrome-responsiveness proof**: close (Cmd-W / pointer close / focused ⌫),
  neighbor activation, tab switching, sidebar, spaces, and command bar all
  remain responsive while the active renderer hangs; the hang never enters
  the main thread (probe, destroy, show, and focus are non-blocking AppKit
  calls; the WebContent process teardown after destroy is WebKit-side and
  asynchronous).
- **beforeunload & JS-dialog boundary** (documented + regressed): chrome-
  initiated close destroys the WKWebView immediately and never runs page
  scripts; `beforeunload` is **not interceptable** through public WKWebView
  API in the pinned SDK (the only `runBeforeUnloadConfirmPanel…` binding is
  the deprecated legacy `WebUIDelegate`, not `WKUIDelegate` — verified in
  `objc2-web-kit 0.3.2`); JS dialogs (`alert`/`confirm`/`prompt` in
  `WKUIDelegate`) remain auto-dismissed in v1 with the boundary documented in
  the interaction contract, and no dialog state survives close/recreate.
- **WebKit process-sharing observations** (no one-process-per-tab guarantee):
  a testkit-gated observation seam enumerates the running
  `com.apple.WebKit.WebContent` helper processes via public
  `NSRunningApplication`, filters to child helpers only — those whose public
  parent PID equals this app's PID, read via public libproc `proc_pidinfo` —
  and terminates exactly one app-owned helper per observation run after a
  deterministic real renderer termination. The seam never claims per-view
  targeting (public API exposes no per-view WebContent PID) and explicitly
  never touches foreign helpers (any helper whose parent PID is not this
  app). The run records whether the affected sessions' probes fail
  (shared-process evidence); the E2E asserts at least one affected session
  and independently reconciles siblings. The policy never assumes per-tab
  processes: after any `PageCrashed` or unresponsive timeout, a bounded
  reconciliation sweep re-probes the other live sessions and independently
  marks each. Observations land in the delivery ledger and
  ARCHITECTURE.md; WKProcessPool is explicitly documented as deprecated and
  behavior-neutral (`objc2-web-kit 0.3.2` header: "Creating and using multiple
  instances of WKProcessPool no longer has any effect"; "each web view is
  given its own web content process until an implementation-defined process
  limit is reached, after which web views with the same process pool end up
  sharing web content processes").
- **Deterministic hang fixtures** in `bbr-fixtures`:
  `/__fx/hang/busy` (synchronous renderer main-thread busy loop — never
  finishes, never answers probes), `/__fx/hang/heavy` (interruptible heavy
  work — slow but responsive control), `/__fx/hang/dialog` (page that opens a
  JS dialog), `/__fx/hang/beforeunload` (page with a `beforeunload` handler).
- **Real-app E2E suite** `just e2e-shell-responsiveness` (new bounded suite,
  registered in canonical `just e2e` once stable) covering the sections
  below.
- **Performance obligations** (scenarios + budgets specified, **execution
  deferred** to the canonical runner — see Performance).
- **Documentation/roadmap**: DESIGN §10.11 + §11.8a, interaction-contract
  close/dialog rows, ARCHITECTURE.md, E2E_ARCHITECTURE.md, Phase-8 checklist
  roll-up, delivery-queue entry.

## Non-goals

- **JS-dialog portal surface** (alert/confirm/prompt presented in the
  `NativeAboveWeb` portal): explicitly deferred; v1 auto-dismisses dialogs
  deterministically. The permissions (`webView_requestMediaCapturePermission…`)
  and file-panel (`webView_runOpenPanel…`) delegate families keep their
  existing owners (permissions/file-panels packets).
- **One-process-per-tab enforcement**: impossible through public API
  (WKProcessPool deprecated/no effect; no public per-view WebContent PID).
  The lane observes and designs around sharing; it does not attempt process
  pinning or private SPI.
- **Automatic reload / terminate after a timeout**: no auto-reload, no
  automatic process kill; recovery stays user-initiated and closed (Wait /
  Reload / Close tab), matching the crash panel's "no auto-reload in v1".
- **Background-tab monitoring**: probes run only for visible sessions on a
  bounded cadence with backoff; hidden/warm tabs are not probed (their
  resource policy stays `bbr-domain` lifecycle-owned).
- **Page-level hang diagnostics** (devtools, sampling profiler, renderer
  telemetry), crash reporting, or a "stop page" menu item.
- **Persistence of unresponsive state** across restart (Phase 7 owns
  persisted restoration).
- **Perf execution on this lane**: optimized scenario code may land
  (feature-gated, typed, budgeted), but measured acceptance is deferred to
  the canonical runner; no canonical-baseline claim from a dev host.
- **Multi-window policy**: unchanged single owning browser window.

## Parallel-lane contract

```text
Base commit:              supplied at launch, after the serial integration of
                          the PEEK dependency and after the current
                          integration checkpoint on main settles
Depends on:               PEEK = link-preview-peek task packet integrated
                          FIRST (its transient WebViewId/session machinery and
                          covered-page input isolation define the per-session
                          visibility model the responsiveness monitor reuses;
                          it also extends the WebEngine seam, so the serial
                          bbr-web kernel must be free); P8b error-surface
                          contract (merged) owns crash classification;
                          performance-e2e-foundation (integrated) provides the
                          scenario matrix
Provides/API:             WebEvent::UnresponsiveChanged(WebViewId, bool);
                          WebEngine::probe_responsiveness(id);
                          portable RendererAvailability + monitor policy;
                          §10.11/§11.8a unresponsive surface + stable ids;
                          deterministic hang fixtures + e2e-shell-responsiveness
Exclusive paths:          crates/bbr-browser/src/responsiveness.rs (+ tests);
                          crates/bbr-web-macos/src/responsiveness.rs (new
                          probe file; `webview.rs` at 530 lines is never
                          grown by this lane — existing split obligation);
                          crates/bbr-web-macos/src/webview.rs and
                          crates/bbr-web-macos/src/delegate.rs (lane-owned
                          edits only where the probe/dialog seam demands);
                          crates/bbr-fixtures hang routes (new hang module,
                          never growing the 839-line `src/lib.rs` overall);
                          crates/bbr-app/tests/e2e_shell_responsiveness/**;
                          this task packet + DESIGN §10.11/§11.8a +
                          interaction-contract rows; docs updates listed in
                          Documentation
Shared integration points: bbr-web WebEngine trait/events (serial kernel);
                           bbr-browser controller facade + snapshot;
                           bbr-app composition root + app/render.rs +
                           shell snapshot/semantics; testkit protocol/wire;
                           justfile canonical E2E list; performance baseline
                           registration; ROADMAP.md/Phase-8 roll-ups;
                           docs/product/browser-interactions.md
Integration order:        PEEK first → this lane → subsequent Phase-8
                          capability lanes; serial kernel changes land as the
                          additive bbr-web slice before dependent worktrees
                          rebase
```

This lane may use an isolated worktree but must not start while another
writer owns the `bbr-web` trait/events, the `bbr-browser` controller facade
or snapshot, app composition, testkit wire/server, the canonical E2E recipe,
or `browser-interactions.md`.

## Constraints

- **Public API only**: `WKProcessPool` is deprecated/no-effect and never
  relied upon; no private WebKit SPI, no per-view PID assumptions, no
  `_close`/`_webProcessIdentifier` KVC. Probe, recovery, and observation use
  only `objc2-web-kit 0.3.2` types already depended on (`WKWebView`,
  `WKWebViewConfiguration`, `WKUIDelegate` dialog methods,
  `evaluateJavaScript…inContentWorld…`) plus public macOS process APIs.
  The observation seam is testkit-gated and macOS-only: enumeration via
  public `NSRunningApplication`; the parent-PID filter via public libproc
  `proc_pidinfo` (public, but low-level C FFI — `unsafe` to call from Rust
  and nonportable, which is why the entire seam is testkit-gated and absent
  from release builds). The observation never terminates or otherwise
  touches foreign helpers (helpers whose parent PID is not this app).
- **Main thread, never blocking**: every probe, destroy, show, and focus call
  stays on the AppKit main thread with `MainThreadMarker`; deadlines are
  timers (run-loop/queue timer), never blocking waits; the gateway's existing
  30 s main-thread reply timeout remains the E2E wedged-app guard.
- **Fail-closed detection**: a probe that cannot complete inside its deadline
  marks the view `Unresponsive`; the state is derived, never guessed from
  loading flags or heuristic timing alone (a busy-but-alive page must stay
  Responsive or be distinguishable via the interruptible control fixture).
- **Stale-event discipline**: events, probe completions, dialog completions,
  and termination callbacks for unknown, destroyed, or recreated
  `WebViewId`s are dropped exactly like the existing
  `ControllerEvent::UnknownWebView` path; generation tags make a late probe
  answer unable to clear a newer timeout or mutate a newer session.
- **Chrome responsiveness is a product invariant**: no main-thread work may
  wait on a renderer; the interaction contract's close/neighbor/sidebar rows
  must hold under hang; tests never use sleeps or pixel assertions.
- **No weakened tests, budgets, validators, or acceptance criteria**; no
  migration or backward-compatibility layer.
- **Security/release**: probe and observation code is `testkit`-feature-gated
  where it exposes test-only seams; release builds exclude them (existing
  `--bbr-*` exclusion posture); the probe evaluates only the trivial
  `1+1`-class expression in the client content world and never reads page
  data; the observation terminates at most exactly one app-owned helper per
  run and never touches foreign helpers.
- **Module budget**: each new file stays below ~500 lines; split by
  responsibility (portable policy vs. macOS probe vs. presentation vs. E2E
  scenarios). Existing files already over the budget are explicit no-grow
  surfaces at the audited base: `crates/bbr-web-macos/src/webview.rs` is 530
  lines (the macOS probe lands in a new `responsiveness.rs`; any necessary
  lifecycle extraction must reduce or preserve, never grow, `webview.rs`) and
  `crates/bbr-fixtures/src/lib.rs` is 839 lines (hang routes land in a new
  fixture module; `lib.rs` may only receive the smallest module/route wiring
  and must not increase in line count overall).

## Evidence

### Unit and contract

- [ ] `bbr-browser` responsiveness module: state transitions
      (Unknown→Responsive→Unresponsive→Responsive; Unresponsive→Crashed via
      existing crash authority), probe lifecycle with generation/backoff,
      stale and late completions rejected, deadline policy (bounded,
      configurable constants), recovery action legality (Wait/Reload/Close
      only), close/recreate under an in-flight probe, sibling reconciliation
      sweep after a `PageCrashed` (shared-process tolerance).
- [ ] `bbr-web` contract: `UnresponsiveChanged` serde shape, object-safety
      retained, trait extension compiles for `FakeEngine`/`NullEngine`/mock.
- [ ] `bbr-web-macos` probe unit tests where a seam permits (deadline math,
      generation tagging, stale-id drop, client-world expression constant);
      `webViewWebContentProcessDidTerminate` path stays unchanged.
- [ ] Fixture routes serve deterministically (`busy` never completes,
      `heavy` completes, hit counts recorded).
- [ ] Default and testkit builds compile; Clippy warning-free (both feature
      sets, including `cargo clippy -p bbr --features testkit --all-targets
      -- -D warnings`).

### Real-app E2E (`just e2e-shell-responsiveness`)

- [ ] **Hang detection**: loading `/__fx/hang/busy` in the active tab marks
      the tab `unresponsive` within the probe deadline (typed snapshot +
      §15 semantic node + row state); the tab keeps its identity and session
      count until recovery; `/__fx/hang/heavy` never transitions to
      unresponsive (slow-but-alive control).
- [ ] **Chrome responsiveness under hang** (production input paths, real
      keystrokes/pointer seams, no sleeps): Cmd-W closes the hung tab and the
      deterministic neighbor activates presented; row close button and
      focused-list ⌫ close it; Cmd-T/Cmd-L, sidebar toggle, space switching,
      and split selection stay within the interaction contract while the
      active renderer hangs.
- [ ] **Recovery**: Wait keeps monitoring and a busy→alive fixture (probe
      answers) clears the state; Reload destroys the hung session and
      recreates a fresh one for the same tab (fresh WebViewId, `recreated`
      semantics, cookie isolation preserved in the same profile store); Close
      tab matches the existing close authority.
- [ ] **Stale events after close/recreate**: after closing a hung tab and
      recreating a fresh session in the same tab slot, late probe completions
      and JS-dialog completions from the old session are dropped (no state
      mutation, no `UnknownWebView` corruption, snapshot asserts the fresh
      session's state).
- [ ] **beforeunload / JS-dialog boundary**: a `beforeunload`-registered page
      closes cleanly via Cmd-W without a page prompt and without blocking
      (documented platform constraint: no public WKUIDelegate beforeunload
      hook in the pinned SDK); a `dialog` fixture page's alert/confirm/prompt
      is auto-dismissed deterministically, the chrome stays interactive, and
      closing/recreating leaves no dialog state.
- [ ] **Process-sharing observation** (testkit-gated, real renderer): the
      `TerminateWebProcess`-class seam enumerates
      `com.apple.WebKit.WebContent` helpers via `NSRunningApplication`,
      filters to child helpers whose public parent PID equals this app
      (public libproc `proc_pidinfo`), and terminates exactly one app-owned
      helper — never claiming per-view targeting (no public per-view
      WebContent PID) and never touching foreign helpers. The E2E asserts
      at least one affected session (`PageCrashed`/unresponsive reported
      for ≥1 session), then the reconciliation sweep re-probes every other
      live session and marks each sibling independently; the observation
      (shared vs. independent, with host model/OS) is recorded in the
      ledger — the E2E asserts BBR's policy response, not a specific
      process topology.
- [ ] Existing crash-panel, error-surface, smoke, tabs, profiles, and shell
      suites stay green.

### Performance

- [ ] Obligation (execution deferred to the canonical runner): add the
      render-commit scenario `unresponsive_chrome` covering, under a hung
      active renderer, (a) Cmd-W close → neighbor presented and (b)
      neighbor selection — 30 render-inclusive samples each against the
      unchanged 50/100 ms warm ceiling, with a fast-but-wrong guard that
      asserts the unresponsive state, the exact neighbor view, session
      counts, and focus before accepting a sample.
- [ ] Obligation (execution deferred): a `probe_tick` scenario proving the
      bounded probe cadence adds no frame cost to the warm path (probe
      results are state events, never render inputs; idle windows emit no
      probe traffic).
- [ ] Obligation: detection latency budget recorded as a design constant
      (e.g., unresponsive declared within the configured deadline; measured
      on the canonical runner, never claimed from a dev host).
- [ ] Explicit reason if a scenario is not applicable; no budget relaxation.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-web
cargo test -p bbr-browser --features testkit
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just verify
just e2e-shell-responsiveness
just e2e-shell-errors
just e2e-shell-tabs
just e2e-smoke
just e2e               # canonical sequential gate only after Patrick lifts
                       # the active performance-execution override; the recipe
                       # currently includes e2e-perf, so this lane runs the
                       # required functional recipes individually until then
just roadmap
```

Performance gates (`just e2e-perf`) run when execution is un-deferred;
before that, the scenario code and budgets are registered but no measured
claim is made. The lane itself runs only its individual functional suites
(listed above) — never performance; `just e2e-perf` execution stays deferred
to the canonical runner. GUI suites run sequentially through the host-wide
GUI lock (`scripts/with_gui_lock.py`); use `CARGO_BUILD_JOBS=4` under host
pressure.

## Delivery sequence

1. Promote this packet through the serial roadmap surface; confirm Peek
   integration and a writer-free `bbr-web`/controller/testkit kernel.
2. Add the real-app hang/chrome-responsiveness regressions first and show
   they fail for the reported behavior (hung renderer blocks nothing, but
   the state/surface is absent) before implementing.
3. Land the additive `bbr-web` slice (event + trait method) as its own
   reviewable commit; then the portable `bbr-browser` monitor; then the macOS
   probe; then the surface; then fixtures/E2E; then docs/roadmap.
4. One independent read-only reviewer maps requirements to evidence
   (including the shared-process observation and stale-event cases).
5. One bounded remediation writer for concrete findings, then a focused
   recheck.
6. Integrate serially; run the full gates from the integrated commit.

## Iteration policy

Start with the smallest end-to-end path: a hung active tab whose Cmd-W close
still works and whose state shows `unresponsive`. After each failing check,
identify whether the failure is detection (deadline/generation), policy
(recovery authority), chrome responsiveness (main-thread wait), stale-event
(registry/generation), or presentation (ids/semantics) before changing code.
Use targeted suites while iterating; run the canonical sequential gate only
after the slice is stable. Do not broaden scope to solve unrelated findings.

## Blocked stop condition

If no defensible path remains, stop with:

- evidence gathered (which fixture, which probe outcome, which interaction
  stalled);
- approaches attempted (public-API probe variants, timer sources, recovery
  seams);
- preserved worktree/branch state and current owner;
- exact blocker — e.g., a WebKit version where the client-world
  `evaluateJavaScript` completes even for a busy-loop renderer, an
  unpublishable detection deadline, or a shared-process topology that makes
  the reconciliation sweep unsafe;
- smallest contract decision, environment input, or user decision required
  next (e.g., accept an explicit detection-latency budget, choose
  auto-dismiss vs. portal dialogs, or record the observation and defer the
  policy).

## Source / ownership conflict map

| Shared surface | Current owner / state | Conflict with this lane | Resolution |
|---|---|---|---|
| `crates/bbr-web` trait + `WebEvent` | serial kernel; find/downloads/new-window/auth extensions integrated | `UnresponsiveChanged` + `probe_responsiveness` are additive trait/event changes | Land as one small serial slice first; dependent lanes rebase |
| `crates/bbr-web-macos/webview.rs` + `delegate.rs` | lane-owned webview lifecycle + `WKNavigationDelegate`+`WKUIDelegate` incl. `webViewWebContentProcessDidTerminate` | probe adapter and dialog auto-dismiss touch the same delegate; `webview.rs` is already at 530 lines (existing split obligation) | Probe lands in the new `responsiveness.rs`; extend the existing delegate ivars/sink pattern, no new delegate authority; `webview.rs` is reduced or held flat, never grown by this lane |
| `crates/bbr-browser/controller.rs` `on_page_crash` / `crashed_tabs` / snapshot | merged; crash-panel E2E green | `Unresponsive` reuses the destroy→recreate seam but must not alter crash semantics | Sibling state, shared seam, distinct event path; crash tests stay untouched |
| `crates/bbr-app` composition root, `render.rs`, shell snapshot/semantics | MVP shell integrated | new pane + row state + snapshot fields | App composition is a serial kernel point; presentational files stay lane-owned |
| testkit protocol/wire (`request.rs`, dispatch, snapshot) | protocol v7 (`PROTOCOL_VERSION = 7`); existing synthetic `InjectWebEvent` (testkit-only engine-event injection — pushes a `WebEvent` through the real transport, kills no process) | new `ProbeResponsiveness` request + new real-process termination seam (`TerminateWebProcess`-class; real process kill is new for this lane, distinct from synthetic `InjectWebEvent`) | Additive, feature-gated; top-level wire envelope is serial |
| `justfile` canonical E2E list | current canonical recipe list | new `e2e-shell-responsiveness` recipe + canonical registration | Registered at integration, not by the lane; canonical `just e2e` is not runnable end-to-end until the current recipe-state override lifts (see Completion gates) |
| `performance-baseline.json` + perf matrix | gate-enforced; current baseline settled | `unresponsive_chrome`/`probe_tick` registration | Registered with budgets unchanged; execution deferred |
| `docs/product/browser-interactions.md` | canonical close/focus rows | new beforeunload/dialog/unresponsive rows | Lane edits explicitly and only its rows; reviewed as part of the slice |
| `DESIGN.md` §10/§11.8/§15 | canonical states/ids | new §10.11 + §11.8a ids/states | Additive; `just roadmap`/verify guard links |
| PEEK (link-preview-peek) | integrated at `bccac7ed…`; preview/session visibility authority settled | dependency satisfied; responsiveness must preserve Preview teardown, covered-page input isolation, and same-session promotion | Regress PEEK 6/6 after implementation; do not reopen disposition policy |
| P8b error surfaces / crash panel | merged (`a70e9a3`) | crash class owns §11.8; unresponsive is distinct | No overlap in classification; `e2e-shell-errors` regression gate |
| Current integration checkpoint | B3 semantic union committed as `492615b5…` / tree `4bc6a6ed…`; GUI lock available at audit | shared `justfile`/E2E registry/app composition are now writer-free | Start only from the committed authority commit that records branch/worktree/base/owner; all later shared conflicts serialize through the Orchestrator |
| Foreign uncommitted Phase-09 documentation/research state | preserved in the separate main worktree; dormant and writer-free at audit | never touched or used as authority by this lane | Read-only; excluded from the renderer branch/worktree and every lane commit |

## Canonical filename / link locations / rollups

- **New canonical packet**: `docs/roadmap/tasks/renderer-isolation-unresponsive-tabs.md`
  (this file, once promoted).
- **Phase roll-up**: new Phase-8 checklist item in
  `docs/roadmap/phases/08-daily-driver.md` linking this packet (single
  canonical owner; ROADMAP.md/current-focus gains only a link).
- **Track links**: `docs/roadmap/tracks/native-integration.md` (adapter
  boundary), `docs/roadmap/tracks/testing.md` (deterministic hang E2E),
  `docs/roadmap/tracks/accessibility.md` (unresponsive surface semantics).
- **Delivery queue**: new entry in
  `docs/delivery/2026-08-08-post-mvp-delivery.md` after the current
  integration checkpoint settles, gated on PEEK integration.
- **Implemented facts**: `ARCHITECTURE.md` (new "Renderer availability"
  section: probe adapter, process-sharing observations, stale-event
  discipline, recovery authority) and `E2E_ARCHITECTURE.md` (hang fixtures,
  `e2e-shell-responsiveness`, observation seam) updated in the same
  implementation group.
- **Product contracts**: `docs/product/browser-interactions.md` (close never
  runs page scripts; beforeunload not interceptable via public API; JS
  dialogs auto-dismissed in v1) and `docs/product/performance-contract.md`
  (unresponsive-chrome interaction rows) updated with the slice.
- **Design**: `DESIGN.md` §10.11 `unresponsive`, §11.8a unresponsive pane,
  §15 id/state additions.
- **No duplicated detailed checklists**: higher-level files link; the packet
  stays the sole owner of its detailed checkboxes.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence: obligations registered; execution deferred to the
        canonical runner (no dev-host measured claim)
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:

Planning/read-only:      independent planning run (read-only, /tmp only;
                         repository unmodified)
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```

Use measured timing only when runtime/command timestamps support it;
otherwise label values as estimates.
