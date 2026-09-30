# Task packet — Tests that run what users run (reality gaps)

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[testing track](../tracks/testing.md) ·
[native-integration track](../tracks/native-integration.md) ·
[E2E_ARCHITECTURE.md](../../../E2E_ARCHITECTURE.md) ·
[AGENTS.md](../../../AGENTS.md) (browser-feature rule).

Status: decided 2026-09-30 by Patrick ("ja mach alles davon") after a debug
build aborted on the first HTTPS page while every suite was green
(`be35936c`). The audit found paths that users run and tests only simulate.

## Outcome

Every gap below is closed on the production path, with a regression test on
that path that failed before its fix wherever that is automatable:

- **A — default profile directory.** A launch without `--bbr-profile-dir`
  keeps its profile in `~/Library/Application Support/<bundle id>` (release
  and dev bundles separate) and never runs without a store.
- **B — HTTP authentication.** Basic/Digest origin and proxy challenges from
  the real `WKNavigationDelegate` open the credential sheet; the answer goes
  back to WebKit as an `NSURLCredential`. Server trust and other challenges
  keep WebKit's default handling.
- **C — real input.** Testkit requests build real `NSEvent`s (mouse, keys
  with modifiers) and deliver them in-process through AppKit, aimed from
  snapshot geometry; the chrome's pointer and key paths and the web page's
  own input run end to end.
- **D — background links.** ⌘-click and middle-click on ordinary links (not
  only links that ask for a new window) open one background child tab after
  the opener.
- **E — pop-ups from script clicks.** Real and script clicks on
  `target=_blank` links are told apart where public WebKit metadata allows
  it; otherwise the limit is recorded with evidence.
- **F — persistent website data.** E2E runs use identifier-backed WebKit
  data stores in a per-run namespace: a cookie survives a restart, stays in
  its profile and leaves with a deleted profile; every run removes its
  stores.

## Scope

- Composition root, native adapters and testkit seams named per item.
- New fixture routes and E2E scenarios on the production path.
- DESIGN / ARCHITECTURE / E2E_ARCHITECTURE and roadmap corrections where a
  claim overstated what was tested.

## Non-goals

- Camera/microphone grants and live TLS client certificates: macOS privacy
  prompts and the user's Keychain stay hand checks (D11/D12 packets).
- Posting OS-level events (`CGEventPost`, event taps) or private WebKit SPI.
- Replacing every seam-based test; a seam test stays unless a real-path test
  covers the same behavior at least as strongly.

## Parallel-lane contract

Not parallelized (serial on `main`).

## Constraints

- No private WebKit SPI, OS event posting or DOM-injected input presented as
  real input; no sleeps or pixel sampling as primary synchronization; never
  draw over or sample a WKWebView.
- Profile isolation; no SQLite schema bump; no test touches the user's real
  profile or website-data directories.
- Budgets, assertions and validators are never weakened; a test changes only
  with its contract in the same commit.

## Evidence

### Unit and contract

- [x] A: `profile_location` unit tests (bundles keep separate directories;
      an unbundled or unusable id takes the build's id).
- [x] B: `bbr-web-macos` `http_auth` tests (Basic/Default, Digest, NTLM map;
      server trust, Negotiate, HTML form and client certificates stay
      default; origin and proxy spaces, FTP not mapped); coordinator tests
      (a submitted WebKit challenge hands its credential to the engine once
      and it leaves memory; cancel and supersession answer without one; a
      seam challenge owes nothing); `bbr-web` redacted `HttpCredential`;
      fixture tests for `/__fx/auth/basic`.
- [x] C: `input_events` tests (US-layout key codes and shifted characters,
      AppKit event types per phase and button, modifier flags);
      `rect_probe` test (a new frame forgets only its own window).

### Real-app E2E

- [x] A: `e2e_shell_persistence::default_location` launches without
      `--bbr-profile-dir` under an isolated `HOME`, finds the store and the
      gateway in `~/Library/Application Support/dev.patwoz.bbr.dev`, quits,
      relaunches and gets the open tab back. Before the fix it failed: the
      gateway was not there and the store log read "persistence store open
      failed … continuing ephemeral" (the line from Patrick's `just run`).
- [x] B: `e2e_shell_auth::native` (2 tests): a real challenge opens the
      sheet for the asking view (host, realm, scheme, authority); a wrong
      password is challenged again; the right one loads the protected page
      and appears in no snapshot; the same profile's next tab is not asked;
      another profile is, and Cancel shows the server's 401 page. Both
      failed before the fix ("no new credential sheet"). The earlier
      `e2e_client_certificates` test that asserted the default handling
      (`be35936c`) is replaced by these, which keep the debug-build proxy
      regression covered.
- [x] C: `just e2e-real-input` (4 tests, shown key window, real AppKit
      events through `NSApplication.sendEvent:`): tab row select, hover and
      close, titlebar Back, footer Downloads and a press outside dismissing
      it, footer `+` and Escape, a space chip, a real drag reordering rows,
      a page link click, and ⌘L / ⌘T / ⌘W / ⇧⌘T with the page focused (the
      menu key-equivalent path through WebKit). Found a real bug: a
      Downloads panel opened from the footer ignored the next outside press
      (the inside mark outlived the press because the listener stops
      propagation); red in the first run ("timed out waiting for downloads
      closed"), fixed by clearing the marks in the root's capture phase
      for every surface with that pattern (Downloads, `+`, space
      configuration, Tab menu). The footer and outside-press seams now take
      the listeners' path (`footer_downloads_press`, `begin_root_press`).
      Blocker found and settled: the hidden harness window is never key, so
      AppKit gives its first click to activation and its keys to nobody —
      real input needs a shown window made key.
- [ ] D: a real ⌘-click and middle-click on an ordinary link open one
      background child after the opener.
- [ ] E: script click vs real click on a `target=_blank` link, fixed or
      recorded as a framework limit.
- [ ] F: persistent per-run WebKit stores: cookie survives a restart, stays
      isolated, is removed with its profile; no leftover stores.

### Performance

- [ ] Not applicable to A (no interaction timing). Items B–F name their
      scenario or why none applies; `e2e-perf` execution stays deferred
      (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-persistence
just e2e-auth
just e2e-client-certificates
just e2e
```

Add every suite an item touches. GUI suites run sequentially.

## Delivery sequence

Serial on `main` by Patrick's instruction: A, B, C, D, E, F (D and E build
on C). No subagents.

## Iteration policy

Reproduce each gap on the real path first, then fix the mechanism and rerun
the affected suites before the commit. Record flakes as intermittent with
the cause open; an mbx cache collection that removes the target directory is
an interrupted run, not a result.

## Blocked stop condition

If AppKit refuses in-process events for the hidden harness window with no
allowed alternative, WebKit offers no public API, or a product decision is
missing: record the evidence and attempts here, write an HTML explainer to
`~/Downloads` for any decision, and continue with the other items.

## Delivery ledger

```text
Outcome:         A, B and C implemented.
Commits:         A fc00be97 + 3bc8fb08 (a concurrent commit in the same
                 worktree, 3fd97e37, re-staged two shared files between my
                 staging and commit; 3bc8fb08 adds the dropped hunks);
                 B 48699c2a (native HTTP authentication); C (this commit:
                 real input events).
Functional evidence: profile_location unit tests; a one-off dev bundle with
                 bundle id dev.patwoz.bbr.probe under an isolated HOME
                 created ~/Library/Application Support/dev.patwoz.bbr.probe
                 (the id comes from the main bundle, not the build fallback).
Real-app E2E:    A: default_location red before the fix, green after;
                 e2e-shell-persistence green.
                 B: e2e_shell_auth::native red with the old delegate
                 (reject every HTTP challenge), green after; a one-off
                 debug-build load of https://example.com (network, not
                 committed) still passes server trust with the default
                 handling.
                 C: e2e-real-input 4/4; affected suites green on the C
                 tree: e2e-shell-downloads (12), e2e-shell-sidebar (21),
                 e2e-shell-tab-menu (12), e2e-shell-spaces (27),
                 e2e-shell-context-menu (11), e2e-shell-tabs (4),
                 e2e-shell-organize (13), e2e-shell-titlebar-navigation (7),
                 e2e-shell-hover (4), e2e-windows (9), e2e-shell-command
                 (28), e2e-split (3), e2e-shell-saved-pages-library (12).
Performance evidence: not applicable to A.
Roadmap/documentation updates: ARCHITECTURE §8 (--bbr-profile-dir default),
                 §7 (authentication challenges); E2E_ARCHITECTURE §7s;
                 corrected claims: native-integration track, testing track
                 and phase 8 (the credential sheet was a testkit seam only);
                 E2E_ARCHITECTURE §7s real input, ARCHITECTURE §7.
Open blockers and non-goals: see Non-goals.
Worktree state:  main; another writer's uncommitted tab-row pin-control
                 change was present during A and is not part of this packet.
```
