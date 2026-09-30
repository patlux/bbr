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

### Real-app E2E

- [x] A: `e2e_shell_persistence::default_location` launches without
      `--bbr-profile-dir` under an isolated `HOME`, finds the store and the
      gateway in `~/Library/Application Support/dev.patwoz.bbr.dev`, quits,
      relaunches and gets the open tab back. Before the fix it failed: the
      gateway was not there and the store log read "persistence store open
      failed … continuing ephemeral" (the line from Patrick's `just run`).
- [ ] B: real challenge → sheet → correct credentials load the page; wrong
      credentials challenge again; cancel shows the 401 page.
- [ ] C: real events select and close tab rows, open Downloads and `+`, press
      a titlebar control, click the space strip, click a web link, and run
      ⌘L/⌘T/⌘W/⇧⌘T with the page focused.
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
Outcome:         A implemented.
Commits:         A (this commit: default profile directory).
Functional evidence: profile_location unit tests; a one-off dev bundle with
                 bundle id dev.patwoz.bbr.probe under an isolated HOME
                 created ~/Library/Application Support/dev.patwoz.bbr.probe
                 (the id comes from the main bundle, not the build fallback).
Real-app E2E:    A: default_location red before the fix, green after;
                 e2e-shell-persistence green.
Performance evidence: not applicable to A.
Roadmap/documentation updates: ARCHITECTURE §8 (--bbr-profile-dir default).
Open blockers and non-goals: see Non-goals.
Worktree state:  main; another writer's uncommitted tab-row pin-control
                 change was present during A and is not part of this packet.
```
