# Task packet — Link Preview / Peek

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[Native integration track](../tracks/native-integration.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Accessibility track](../tracks/accessibility.md) ·
[Testing track](../tracks/testing.md) ·
[browser interaction contract](../../product/browser-interactions.md) ·
[daily-driver capability inventory](../../product/daily-driver-capability-inventory.md).

Status: **planned for the first feature-complete MVP.** Peek is a standalone
Delivery lane after one accepted typed new-window request/disposition seam. It
may proceed independently of regular-child navigation gestures only after that
shared contract lands; it never duplicates regular-tab opener provenance.

## Outcome

Open eligible link-created windows in a temporary, focused preview above the
current page instead of immediately adding a regular tab. The originating tab
and its native session stay intact underneath. The user can dismiss the preview
without changing tab state or promote it to a regular tab without reloading or
losing the preview's navigation state.

## Scope

- Define one portable Peek disposition and transient preview lifecycle after the
  shared typed new-window request/disposition seam is integrated.
- Add controller/native/app composition for source-profile-isolated temporary
  sessions, exact focus restoration, and same-session promotion.
- Add the required production UI, accessibility semantics, deterministic
  fixtures, typed testkit evidence, real-app E2E, performance scenarios, Gallery
  stories after production integration, and roadmap/architecture truth.

## Product behavior

- [x] A user-initiated eligible `target="_blank"` or `window.open` request opens
      Peek by default instead of creating a regular tab immediately.
- [ ] Provide an explicit keyboard/pointer action to open an ordinary new tab
      directly and an explicit action to request Peek for a normal link.
- [x] Cover the complete current web-content area with the preview while keeping
      browser chrome and preview controls operable.
- [x] Keep the originating tab selected and its WKWebView warm underneath; it
      must not receive pointer, keyboard, scroll, drag, or Accessibility focus
      while Peek is open.
- [x] Give the preview its own back/forward/reload/navigation state and route
      browser commands to it while it owns focus.
- [x] Escape and the close control dismiss Peek, destroy its temporary native
      session, and restore focus to the exact originating control or web view.
- [x] “Open as Tab” adopts the existing preview session into the originating
      space and profile without reload, preserving URL, history, form state,
      scroll position, and same-document state.
- [x] If the origin tab/space/window disappears, apply a deterministic policy:
      close the preview unless the user has already promoted it.
- [x] Support at most one Peek per pane initially; a new eligible request from
      inside Peek replaces or navigates the existing preview according to the
      documented policy rather than stacking unbounded previews.

## Delivery dependency and lane contract

```text
Base commit:          fresh verified integrated checkpoint chosen by the
                      delivery orchestrator
Depends on:           one accepted typed new-window request/disposition seam;
                      native transient-session/z-order APIs; free controller,
                      app, testkit, Portal/window, and bbr-ui ownership
Provides/API:         portable Peek disposition/lifecycle; transient preview
                      identity/session; dismiss and no-reload promotion
Exclusive paths:      focused Peek policy/controller/native/UI/E2E/perf modules
Shared integration points: bbr-web request/event contract, controller facade,
                      macOS WKUIDelegate/session registry, native z-order,
                      bbr-app focus/composition, testkit protocol, justfile,
                      canonical E2E/performance registries, Gallery/roadmap
Integration order:    serial new-window request/disposition seam → Peek and N1
                      may proceed in isolated lanes → production Gallery stories
```

No concurrent writer may own new-window disposition, transient native-session
adoption, the same native z-order/composition surface, or the same testkit and
performance registry entries. The
[web-content gesture/opener-return packet](web-content-navigation-gestures.md)
owns only regular-tab provenance and Back-at-root behavior after disposition;
Peek owns transient preview policy/lifecycle.

## Eligibility and safety policy

- [ ] Add a typed portable new-window request containing source tab/session,
      target URL, disposition, user-gesture state, and relevant WebKit metadata.
      When policy resolves the request to a regular tab, this same authority
      establishes the session-local child→opener provenance consumed by the
      [web-content Back-at-root contract](web-content-navigation-gestures.md);
      the gesture layer never independently interprets WebKit popup metadata.
- [x] Keep policy decisions outside the macOS delegate: the adapter reports the
      request, while portable orchestration chooses Peek, regular tab, external
      handling, or rejection.
- [x] Create Peek in the source profile's `WKWebsiteDataStore`; never copy
      cookies, credentials, or session state across profile boundaries.
- [x] Do not route downloads, non-web schemes, authentication/challenge windows,
      permission flows, or explicitly sized/tool-style popups into Peek unless a
      later documented policy marks them safe.
- [ ] Preserve WebKit opener/security semantics deliberately; do not expose a
      stronger JavaScript opener relationship than an ordinary browser-created
      tab would. BBR's internal `TabId` provenance is browser orchestration data
      for deterministic focus/close behavior, not page-script access.
- [x] Surface blocked or unsupported popup requests without silently losing the
      user's action.

## Constraints

- Peek owns transient preview policy/lifecycle only; it does not duplicate the
  shared new-window disposition seam or N1's regular-tab opener provenance and
  Back-at-root behavior.
- Preserve source-profile isolation, the exact warm origin session, one preview
  per pane, native webpage rendering, public platform APIs, and deterministic
  focus/session teardown.
- No private SPI, global event tap, screenshot/texture-composed webpage, second
  navigation authority, or synchronous persistence in render/ranking paths.
- Keep files below roughly 500 lines and do not weaken tests, semantic guards,
  budgets, sample counts, deadlines, or canonical GUI serialization.

## Architecture and lifecycle

- [x] Model Peek as transient browser state, not as a persisted `Tab`, recently
      closed entry, sidebar row, or history item merely because it was opened.
- [x] Give each preview a typed identity and disposable `WebViewId` owned by the
      controller/session registry until dismissal or promotion.
- [x] Extend `WebEngine` with the minimal create/show/hide/focus/destroy/adopt
      operations or outcomes needed for transient sessions without leaking
      AppKit/WebKit types into portable crates.
- [x] Stack the preview WKWebView as a native child view above the originating
      WKWebView; never screenshot or texture-compose page content into GPUI.
- [x] Define explicit z-order and clipping between GPUI chrome, Peek controls,
      the preview WKWebView, and the covered originating WKWebView.
- [x] Destroy delegates, KVO observations, native views, and registry mappings on
      every dismissal, failed creation, window close, and process-termination
      path.
- [x] Promotion transfers ownership atomically so the existing native session is
      neither recreated nor temporarily orphaned.

## UI, input, and accessibility

- [x] Design Peek as an intentional BBR surface with clear temporary status,
      close, “Open as Tab”, URL/origin context, loading, failure, and crash states.
- [ ] Support light/dark, increased contrast, reduced motion, compact windows,
      and safe-area/titlebar geometry.
- [ ] Use a bounded entrance/exit transition; reduced motion uses an immediate or
      non-spatial state change.
- [ ] Trap browser-chrome focus within Peek and its page while open, while still
      allowing predictable transfer between preview controls and preview content.
- [ ] Expose semantic role/name/state for the preview and announce opening,
      navigation failure, promotion, and dismissal to VoiceOver.
- [ ] Make every pointer action keyboard-operable; document shortcuts and avoid
      conflicts with page-reserved key handling.

## Evidence

- [x] Portable controller tests cover request policy, transient identity,
      command routing, replacement, dismissal, promotion, origin removal, and
      profile isolation.
- [ ] macOS adapter tests prove new-window metadata reaches the portable policy
      and that preview sessions use the correct website data store.
- [x] Real-app E2E opens a fixture `target="_blank"` link and proves Peek appears
      above the current native view without creating a regular tab.
- [x] E2E proves the covered page receives no input and retains the same
      `WebViewId`, URL, and page state after Peek closes.
- [x] E2E navigates inside Peek, promotes it, and proves the same preview
      `WebViewId` becomes a regular tab without another page load.
- [ ] E2E covers Escape/focus restoration, replacement policy, popup exclusions,
      failure/crash cleanup, and cross-profile isolation.
- [ ] Add a bounded `just e2e-peek` suite and include it in canonical sequential
      `just e2e` once stable; GUI suites remain sequential.
- [ ] Extend performance E2E with request-to-present, dismiss-to-origin-present,
      and promotion-to-tab-present milestones. Request-to-present is a fresh
      native-session class (p95 ≤ 100 ms; max ≤ 150 ms); dismissal back to the
      warm origin and no-reload promotion use the warm interaction ceiling
      (p95 ≤ 50 ms; max ≤ 100 ms), with at least 30 accepted samples each in
      the full gate and typed fast-but-wrong guards.
- [ ] Run `just verify`, `just e2e-peek`, all affected suites, optimized
      performance, and canonical sequential `just e2e` before checking this
      task complete.

## Iteration policy

Start with one eligible user-originated request, one source pane, and a fresh
source-profile preview that can dismiss back to the exact warm origin. Add
same-session promotion only after that end-to-end path works, then exclusions,
replacement, split/profile, crash/failure, accessibility, and performance
matrices. After each failure, identify whether the cause is disposition,
session ownership, native z-order, focus, teardown, or evidence before editing.
Use targeted gates while iterating and run canonical sequential E2E once the
complete slice is stable; do not broaden into N1 or unrelated Phase-8 work.

## Blocked stop condition

If public AppKit/WebKit APIs cannot provide deterministic transient-session
z-order/adoption without reloading, crossing profiles, stealing origin input,
or duplicating the new-window authority, stop with:

- exact API/source and runtime evidence;
- attempted public approaches and observed failure mode;
- preserved worktree/branch/fingerprints and current owner;
- affected contract/E2E/performance claims still open; and
- the smallest serial platform, architecture, or product decision required.

Do not fall back to private SPI, a global event tap, texture composition,
reload-on-promotion disguised as adoption, or weaker assertions.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-browser
CARGO_BUILD_JOBS=4 cargo test -p bbr-web
CARGO_BUILD_JOBS=4 cargo test -p bbr-web-macos
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-smoke
nix develop -c just e2e-tabs
nix develop -c just e2e-profiles
nix develop -c just e2e-shell-portal
nix develop -c just e2e-peek
nix develop -c just e2e-perf
nix develop -c just e2e
nix develop -c just roadmap
```

GUI/performance suites run sequentially under the repository GUI lock.

## Non-goals

- Persisting open previews across app restart.
- Nested or multiple stacked previews in one pane.
- Replacing permission prompts, authentication panels, downloads, or arbitrary
  JavaScript utility windows with Peek.
- Cross-profile preview-session transfer.
- Pixel-composed webpage previews.

## Delivery sequence

1. Re-read Current delivery and settle the shared NW-SEAM plus exact ownership.
2. Implement one source-pane preview path with portable policy and native
   source-profile session creation.
3. Prove dismissal, exact focus restoration, teardown, and origin warmth.
4. Add same-session promotion, then exclusions/replacement and split/profile
   matrices.
5. Add real-app E2E, optimized performance, accessibility, and Gallery stories
   only after production behavior is integrated.
6. Run independent read-only review, bounded remediation only for concrete
   findings, focused recheck, and orchestrator integration.

## Delivery ledger

```text
Outcome: Transient Peek preview lifecycle integrated with the productive new-window seam. An eligible direct activation opens a source-profile preview above the warm origin (no regular tab, same WebViewId/URL/page state underneath); one preview per pane (a request from inside Peek navigates it, any other source replaces it); Escape/close destroys the temporary session and restores the exact warm origin; browser commands route to the preview while it owns focus; "Open as Tab" promotes the SAME session without reload; non-web/sized/tool-style exclusions fall back to the ordinary tab path; script requests stay blocked; origin-removal/staleness and crash teardown clean up. Closeout writer made a doc-only delta — no commit, reset, Rust edit, or functional rerun.
Commits: none by the closeout writer (task forbids commit). Lane sits uncommitted on base `4b77c980c56926fb4662cb05b27606faef9732a2` in the `bbr-peek` worktree (branch `parallel/peek`).
Functional evidence: non-GUI `just verify` green on the candidate tree (fmt, workspace check/test, workspace clippy `-D warnings`, roadmap, ui-deps, script-tests) — `/tmp/peek-verify3.log`, SHA-256 `6d648609eacce474ab2ee134dc2cf6565769dab42298f34066a9d996386d84c8`. No `cargo clippy -p bbr --features testkit` re-run exists in the closeout evidence.
Real-app E2E: PEEK 6/6 `/tmp/bbr-peek-closeout-e2e-peek.log` `85d3b8c4…`; NW-SEAM 4/4 `362a5775…`; Tabs 1/1 `c1b2e2b9…`; Profiles 1/1 `547b4d19…`; Smoke 1/1 `9d713a40…`; Portal 7/7 on the exact unchanged source under the Xcode 26.2 GUI lock with the host unlocked afterwards — `/tmp/bbr-peek-unlocked-portal-20260810T111056Z.log`, SHA-256 `766da3b1838414dc1c659612a17c1125e4a8813aa64f81e12b0a0e8fe8936d35`.
Performance evidence: unexecuted and unpassed. Scenario contracts, baseline entries (`peek_request_to_present` p95 ≤ 100 / max ≤ 150 ms; `peek_dismiss_to_origin` and `peek_promote_to_tab` p95 ≤ 50 / max ≤ 100 ms, `samples: 0`), typed drivers and fast-but-wrong guards exist; no accepted samples exist, so every performance checkbox stays unchecked.
Roadmap/documentation updates: this packet's evidence checkboxes (functional/unit/E2E facts only, with exact evidence); `ARCHITECTURE.md` implemented facts (§8 composition, §9a controller, §11 known-limitations item 4); `DESIGN.md` §11.17 Peek bar spec + §15 stable ids + §16 non-goal revision; `E2E_ARCHITECTURE.md` §7m suite section + §9 command list. Doc-only delta; `just roadmap` and `git diff --check` green.
Open blockers and non-goals: performance acceptance debt; `just e2e-peek` not yet included in canonical `just e2e`; no dedicated macOS-adapter peek unit test (store/z-order proven at controller + real-app level); no real-app cross-profile peek scenario (controller non-default-profile store test covers it); explicit production keyboard/pointer intent actions (ordinary-tab vs Peek) not wired — typed `NewWindowIntent` seams and the preview UI only; VoiceOver announcements, focus-trap, light/dark + reduced-motion peek evidence absent (the bar currently renders the dark theme); opener provenance, modifier/background insertion, named-window reuse, popup allow-lists, and second-window behavior remain outside this slice; no MVP-complete/modifier/insertion claim.
Worktree state: `bbr-peek` @ `4b77c980…`; implementation uncommitted; closeout adds only the four PEEK-owned doc files.

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```
