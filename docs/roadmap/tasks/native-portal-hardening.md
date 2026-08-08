# Task packet — native Portal host hardening

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[Phase 9](../phases/09-quality-release.md) ·
[native portal host](native-portal-host.md) ·
[native-integration track](../tracks/native-integration.md) ·
[Testing track](../tracks/testing.md).

Status: **post-MVP follow-up after the base Portal host and U1 migration.**

## Outcome

The one-per-browser-window native Portal host has an explicit, tested contract
for focus/key routing, multiple BBR windows, display/screen changes, compact
geometry, presentation stacking, teardown, and packaged-app behavior. It stays
an implementation detail of its parent browser window and cannot become an
orphan, unexpected key/main window, or cross-window presentation owner.

## Scope

- Define focus/key routing for Escape, Cmd-W, command shortcuts, text input,
  pointer activation, and focus restoration for nonactivating and explicitly
  focus-requesting presentations.
- Define one-host-per-browser-window identity and isolation across multiple BBR
  windows; presentations never attach to or inspect another browser window.
- Harden movement/resizing across displays, scale changes, Spaces/fullscreen,
  minimization, occlusion, parent close, app hide/show, activation, and compact
  screen clamping.
- Define deterministic Portal stack priority, replacement, dismissal, and
  teardown for palette, find bar, and BBR-owned menus.
- Extend typed snapshots/ids/events so E2E can identify browser-window owner,
  parent/child numbers, frames, host visibility, key/main state, active
  presentation, and frontmost hit-test owner.
- Add packaged-app/manual and Appium-ready acceptance notes after automated
  evidence is stable.

## Non-goals

- No new Portal visual identity; consuming surfaces remain owned by their task
  packets and `DESIGN.md`.
- No independent browser lifetime for the native child window.
- No screenshot/texture composition, WebView reparenting, GPUI fork, or one
  window per popup.
- No cross-platform native host implementation in this slice.

## Parallel-lane contract

Not parallelized. Browser-window ownership, AppKit attachment, focus, testkit
wire, and all consuming Portal surfaces are shared serial contracts.

## Constraints

- Exactly one native Portal host per BBR browser window; zero cross-window
  attachment or presentation leakage.
- The host remains borderless, implementation-owned, and excluded from Dock,
  Mission Control, independent movement/minimization, and browser state.
- Nonactivating presentations never become key/main. A future presentation that
  needs text focus must use one explicit focus request/restore contract rather
  than accidental window activation.
- Parent close/teardown detaches and closes the host deterministically on the
  main thread.
- Geometry uses AppKit points with explicit screen/content conversion and
  clamps to the owning browser window/display.
- Keep native/unsafe code in `bbr-web-macos`, files below roughly 500 lines,
  and preserve host-wide GUI serialization and deadlines.

## Evidence

### Unit and contract

- [ ] Focus/key-routing matrix covers nonactivating, text-focus, Escape, Cmd-W,
      replacement, outside dismissal, and restoration.
- [ ] Window registry/state tests cover two parents, unique hosts, no
      cross-attachment, stack priority, display conversion, and teardown.

### Real-app E2E

- [ ] `just e2e-shell-portal` covers two BBR windows, distinct child hosts,
      independent presentations, parent close, hide/show, minimize/restore,
      movement/resize, and no orphan window/session.
- [ ] Display/compact-frame tests prove clamping and frame synchronization; key/
      main/frontmost diagnostics prove focus and hit-test ownership.
- [ ] Palette, find, and context-menu suites share the hardened host contract
      without weakening their own assertions.

### Performance

- [ ] Measure only user-visible open/switch paths that remain in the hot path;
      host construction/attachment stays once per browser-window lifetime.
- [ ] Multiple-window idle state adds no continuous frame invalidation and is
      included in Phase-9 energy/idle evidence.

## Completion gates

```bash
just verify
cargo clippy -p bbr-web-macos --all-targets -- -D warnings
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-portal
just e2e-shell-command
just e2e-shell-find        # when integrated
just e2e-shell-context-menu # when integrated
just e2e-perf
just e2e
just roadmap
```

## Delivery sequence

1. Freeze the focus/key and multi-window ownership contracts.
2. Implement registry/geometry/teardown hardening with lower-level diagnostics.
3. Extend real-app multi-window/display/visibility evidence.
4. Re-run consuming Portal suites and optimized measurements.
5. Run independent AppKit-boundary and packaged-app review, remediation, and
   final recheck.

## Iteration policy

First prove two browser windows cannot share or steal hosts. Then harden
focus/key routing, teardown, display movement, minimization/activation, and
stacking. Diagnose failures as owner registry, AppKit parent relation, frame
conversion, key/main policy, focus choreography, or GPUI lifecycle before
changing architecture.

## Blocked stop condition

If the pinned GPUI/AppKit APIs cannot preserve ownership or focus across these
lifecycle cases, stop with the base one-window Portal contract intact. Report
APIs tried, parent/child/key/main/frame observations, affected packaged-app
behavior, and the smallest GPUI-CE patch or product limitation required. Never
silently accept an orphan/independent Portal window.

## Delivery ledger

```text
Outcome: hardening slice 2 (queue items 10 + 14, native window boundary):
        the credential sheet (item 10) was already landed by the preceding
        lane (sheet UI, redacted surface, coordinator, E2E); this lane
        hardened the one NativeAboveWeb host (item 14) against the window
        lifecycle events that reliably break native overlays:
- fullscreen enter/leave: the host observes
  NSWindowDidEnter/ExitFullScreenNotification on the parent, re-pins the
  frame to the (fullscreen or restored) content rect, and re-orders front
  when a presentation is armed — the same portal window, no recreation.
  **E2E cannot drive this scenario on this host and is therefore NOT
  shipped**: on the macOS 27.0 Window Server a programmatic
  `toggleFullScreen:` starts the transition (WillEnter fires, mask flips)
  but never completes — DidEnter/DidExit never post, for the BBR window
  AND for a plain standalone NSWindow in an independent Swift probe on the
  same host — and the transition is silently cancelled (mask flips back),
  so a mask-based wait converges hollow on the cancelled state or stalls
  (the stuck transition drops the exit toggle). The lane's earlier
  mask-based fullscreen E2E was therefore hollow/flaky here, not
  deterministic evidence; it was removed and the scenario is reported as
  environment-blocked (see Blockers), with the host-side hardening kept
  and the observer pipeline proven by the minimize/close scenarios.
- minimize/restore: NSWindowDidMiniaturize/DeminiaturizeNotification;
  the portal is ordered out while minimized (never occluding/intercepting)
  with the presentation paused (not dismissed), and re-shown/re-pinned on
  restore; E2E drives real miniaturize:/deminiaturize: and asserts
  native_visible false while minimized and frontmost ownership after
  restore.
- space/display: isOnActiveSpace diagnostics prove the child always agrees
  with its parent on active-space membership; the E2E drives the real
  CanJoinAllSpaces collection-behavior transition (Window Server space
  assignment) and the portal always follows its parent's space
  membership. NSWindowDidChangeScreenNotification re-pins on display
  changes; portal_scale_factor documents the point-based frame invariant
  (scale changes pixels, never points).
- window close with an open presentation: NSWindowWillCloseNotification on
  the parent tears the host down deterministically through the same
  teardown path the testkit PortalTeardown proves (observer removed, child
  relation dropped, window closed); E2E closes through the real
  performClose: path and proves via a Window Server query from the test
  process (CGWindowListCopyWindowInfo) that zero BBR windows remain on
  screen — never orphaned, never occluding, never intercepting input.
Blockers (reported, not weakened): (1) a real desktop space switch and a
        display change with a different backing scale factor cannot be
        driven deterministically on this single-display host without
        private CGS APIs, admin display changes, or user input; the host is
        hardened for both (space-membership agreement evidence,
        screen-change observer, point-based frame invariant) and the
        available real-AppKit transitions are E2E-covered, but the two
        exact events themselves are documented as environment-blocked on
        this host. (2) fullscreen enter/leave: on macOS 27.0 the Window
        Server never completes a programmatic `toggleFullScreen:` —
        `NSWindowWillEnterFullScreenNotification` fires and the mask flips,
        but `NSWindowDidEnterFullScreenNotification` never posts and the
        transition is cancelled (verified for the BBR window with global
        unfiltered notification observers AND for a plain standalone
        `NSWindow` in an independent Swift probe on this host: willEnter=1,
        didEnter=0, mask flips then back). There is no deterministic
        completion signal, so the scenario is environment-blocked here;
        the host-side DidEnter/DidExit observers and re-pin/re-order
        hardening remain correct code, and the same observer pipeline is
        proven by the minimize/restore and close scenarios.
Audit/cleanup (this lane): the AUTH-DIAG eprintln leftover in
        crates/bbr-app/src/app/auth.rs was removed (orphaned diagnostic,
        nothing depended on it); coordinator.rs was split (core + unit
        tests into coordinator_tests.rs, and the redacted store/records
        and wire/render shapes into the sibling auth/records.rs),
        the portal hardening tests were split out of e2e_shell_portal.rs
        into e2e_shell_portal/hardening.rs, and the native lifecycle
        methods AND the portal notification observer moved out of
        portal.rs into portal_lifecycle.rs with the diagnostics builder
        next to its struct in portal_frame.rs. File sizes after the
        complete split: crates/bbr-app/src/app/auth/coordinator.rs 474,
        crates/bbr-app/src/app/auth/records.rs 136,
        crates/bbr-app/src/app/auth.rs 333, crates/bbr-web-macos/src/
        portal.rs 418, portal_lifecycle.rs 249, portal_frame.rs 453,
        window_server.rs 104 — every new/changed file stays under the
        ~500-line budget (hard rule). (The intermediate state shipped in
        this lane's first delivery claim had coordinator.rs at 589 and
        portal.rs at 509, both over budget; the remediation split
        corrected that claim.) A clippy `items_after_test_module`
        and an unbalanced-brace regression introduced during the split were
        fixed and the full gate set re-run.
Commits: none (lane rule: changes stay uncommitted in this worktree)
Functional evidence: cargo fmt --all -- --check; cargo clippy -p
        bbr-web-macos --all-targets -- -D warnings; cargo clippy -p bbr
        --features testkit --all-targets -- -D warnings; nix develop -c
        just verify; nix develop -c just e2e-shell-portal; nix develop -c
        just e2e-auth; nix develop -c just e2e-perf. All green on base
        a24321d.
Real-app E2E: e2e_shell_portal gained host_hides_on_minimize_and_restores,
        host_follows_window_server_space_assignment, and
        host_tears_down_when_parent_closes_with_open_presentation (real
        AppKit drivers + Window Server queries, deadline-bounded polling,
        no sleeps); the fullscreen scenario is environment-blocked (see
        Blockers) and its hollow mask-based test was removed. The auth
        suite (item 10) delivers its three flow scenarios
        (present/submit, cancel-fails-never-retries, per-view identity +
        rechallenge) with redaction assertions.
Performance evidence: just e2e-perf green; hardening adds no hot-path
        work (host construction stays once per browser-window lifetime;
        the new observers fire only on lifecycle events).
Open blockers and non-goals: see blockers above; no second host, no web
        view reparenting, no composite; exactly one NativeAboveWeb host per
        browser window remains the rule.
Worktree state: uncommitted changes in this lane's exclusive territory
        (portal hardening in crates/bbr-app/src/portal/**, the auth sheet
        files, crates/bbr-web-macos/src/portal*.rs + window_server.rs, the
        portal/auth E2E suites, and the additive testkit wire requests/
        snapshot fields); the merged crates/bbr-web/src/auth_challenges/**
        contract is untouched.
```
