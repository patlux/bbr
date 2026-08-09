# Task packet — Deterministic process shutdown

Parents: [Phase 9 — quality and release hardening](../phases/09-quality-release.md) · [Testing track](../tracks/testing.md) · [Post-MVP delivery queue](../../delivery/2026-08-08-post-mvp-delivery.md)

Activation condition: complete F2a must be integrated and lifecycle/testkit/E2E-registry territory writer-free. Runtime readiness and ownership live in [Current delivery](../../delivery/current.md).

## Outcome

BBR fully terminates through both user shutdown paths: terminal `Ctrl-C`
(`SIGINT`) stops a development-run application, and closing the owning browser
window stops the complete process. Success is verified by real process exit,
not merely by hidden windows or internal shutdown state.

## Scope

- Add one app-lifetime shutdown seam that routes `SIGINT` onto GPUI's
  foreground/main thread and performs graceful application teardown.
- Observe closure of the owning browser window and call the same application
  quit path. Closing or tearing down an implementation-owned portal child
  window must not terminate BBR by itself.
- Extend the testkit only as needed to drive the real browser-window close path
  without racing the successful response against process/socket teardown.
- Add a bounded real-app E2E suite covering both shutdown paths and register it
  in the canonical sequential E2E gate.
- Update implemented-fact and E2E documentation after the behavior is proven.

## Non-goals

- Persistence flush, warm restore, or crash-recovery composition; those retain
  their existing Phase-7/F3 owners.
- General multi-browser-window lifetime policy. This slice proves the current
  single owning browser-window product shape without inventing future window
  restoration or reopening behavior.
- `SIGTERM`, `SIGHUP`, force-kill recovery, Cmd-Q/menu construction, login-item
  behavior, or signed/notarized release automation.
- Performance optimization or a new latency budget. Shutdown is terminal and
  is accepted by bounded completion rather than presentation timing.

## Parallel-lane contract

```text
Base condition:           complete verified F2a integration on main; exact SHA in Current delivery
Depends on:               F2c → F2b → F2a serial integration complete
Provides/API:             deterministic owning-window and SIGINT shutdown;
                          real-app process-exit regressions
Exclusive paths:          a focused bbr-app lifecycle module or window_seam;
                          shutdown-specific testkit request/dispatch only;
                          crates/bbr-app/tests/e2e_shutdown.rs;
                          shutdown recipe/docs/task packet
Shared integration points: bbr-app main composition, testkit wire/server,
                           justfile canonical E2E list, architecture docs
Integration order:        F2c → F2b → F2a → this slice → F2 closeout gates
```

This is a serial kernel slice. It may use an isolated worktree, but must not
start while another writer owns app composition, testkit wire/server, the
canonical E2E recipe, or browser/portal window lifecycle.

## Constraints

- Native AppKit/WebKit objects tear down on GPUI's application/main thread. A
  signal handler must perform no direct native mutation and no
  async-signal-unsafe application work.
- Use graceful GPUI application termination; do not use `process::exit`,
  `_exit`, abort, or a kill fallback as product behavior.
- Identify the owning browser window explicitly. Portal/find/auth/context-menu
  child-window closure must not accidentally satisfy the process-exit rule.
- The close-window E2E must exercise the production GPUI/AppKit close path, not
  a direct `cx.quit()` shortcut. The SIGINT E2E must deliver a real signal to
  the spawned BBR process.
- A shutdown-driving testkit response must be flushed before application
  teardown can close the socket, preserving the existing `Quit` race
  invariant.
- Tests assert successful process exit within an explicit deadline and clean
  up the process tree on failure. No unbounded waits and no product sleeps.
- Do not weaken existing portal teardown, shutdown, timeout, or process-exit
  assertions.

## Evidence

### Unit and contract

- [x] Lifecycle/helper tests distinguish the owning browser window from portal
      or unrelated window ids where a pure seam is introduced.
- [x] Testkit server coverage proves a close-window response is flushed before
      the shutdown handler runs, matching the existing graceful-`Quit`
      guarantee.
- [x] Default and testkit builds compile and Clippy remains warning-free.

### Real-app E2E

- [x] `Ctrl-C` regression: spawn the real BBR binary, wait for typed readiness,
      deliver real `SIGINT`, and assert successful full process exit within 10
      seconds.
- [x] Window-close regression: close the real owning browser window through
      the production GPUI/AppKit path and assert successful full process exit
      within 10 seconds, with no portal child keeping the process resident.
- [x] Existing gateway `Quit`, portal teardown/close, smoke, and packaging
      launch behavior remain green.

### Performance

- [x] Not applicable: this terminal lifecycle slice introduces no presented
      interaction and no reusable latency class. Bounded successful process
      exit is the acceptance criterion; existing startup and interaction
      budgets remain unchanged.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-testkit
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just verify
just e2e-shutdown
just e2e-shell-portal
just e2e-smoke
just e2e
just package-verify
```

GUI suites run sequentially through the host-wide GUI lock. Use
`CARGO_BUILD_JOBS=4` when concurrent delivery activity would otherwise create
host pressure.

## Delivery sequence

1. Start from the verified post-F2a `main`; confirm exclusive ownership of app
   composition, testkit wire/server, window lifecycle, and the canonical E2E
   recipe.
2. Add the two real-app regressions first and demonstrate that each fails for
   the reported behavior rather than for harness setup.
3. Implement the smallest shared graceful-shutdown seam for owning-window close
   and SIGINT.
4. Run one independent read-only review mapping both user paths to process-exit
   evidence and checking portal-child isolation plus response-flush safety.
5. Use one focused remediation writer only for concrete findings, followed by
   a focused recheck.
6. Integrate serially, then run the full F2 closeout and packaging gate.

## Iteration policy

Diagnose lifecycle failures by separating signal delivery, GPUI scheduling,
browser-window identification, AppKit close callbacks, portal-child ownership,
gateway response flush, and final process status. After each change rerun the
smallest failing shutdown scenario. Run the canonical sequential E2E and
package gate only after both focused scenarios are stable. Do not substitute a
force-kill or a test-only direct quit for a missing production lifecycle path.

## Blocked stop condition

If no defensible graceful path remains, stop with:

- the exact signal/window-close event observed;
- whether GPUI received the event and whether `App::quit` ran;
- surviving browser/portal window and process evidence;
- response/socket ordering evidence;
- APIs and approaches attempted;
- preserved worktree state and current owner;
- the smallest GPUI/AppKit contract decision or environment input required.

## Delivery ledger

```text
Outcome: Deterministic process shutdown proven through both user paths by
        real process exit. BBR fully terminates on terminal Ctrl-C (real
        SIGINT delivered to the spawned binary) and on owning-browser-window
        close (testkit `CloseWindow` -> production GPUI `window.remove_window()`
        observer path; real AppKit `performClose:` covered by the portal
        hardening parent-close scenario). An implementation-owned portal child
        window never keeps the process resident. The gateway flushes the
        `Quit`/`CloseWindow` response through the connection's sole writer
        before the shutdown handler runs (no response/socket EOF race).

Commits: none; settlement candidate is uncommitted for serial integration by
        the Orchestrator (base e9f5d30214e7efc46586cb3ae11c6472e0b1801c).

Functional evidence:
- `is_owning_window` pure predicate + 3 unit tests (portal/unrelated ids never
  match; owning id matches): `cargo test` green (window_seam tests ok).
- Testkit flush ordering test `close_window_response_is_flushed_before_
  shutdown_handler_runs`: `cargo test -p bbr-testkit` 27 passed.
- Real SIGINT: `kill -INT` to the spawned process; graceful exit success
  within 10 s (a default-disposition signal death would fail `success()`).
- Owning-window close: `window.remove_window()` -> window-closed observer ->
  `flush_browser_state()` + `cx.quit()`; process exits within 10 s.
- Portal-child isolation: close with the native portal probe presented and
  attached still exits the complete process.

Real-app E2E:
- `just e2e-shutdown`: 3/3 green (sigint, close_window, close_with_portal).
- `just e2e-smoke`: 1/1 green. Portal parent-close scenario
  (`host_tears_down_when_parent_closes_with_open_presentation`) green.
- Canonical `just e2e`: NOT green. Green through portal position, then
  BLOCKED by `hardening::host_hides_on_minimize_and_restores` (AppKit
  hit-test at the probe point does not converge: foreign window 42993 sits
  below the probe, owns true) -- the documented PORTAL-MINIMIZE host
  blocker (see docs/delivery/current.md). Identical failure pre-exists this
  lane (8 Aug suite-e2e-shell-portal.log) and reproduced identically in the
  stopped run and one settlement diagnostic retry. Do not infer a green
  canonical gate from individually green remainder suites (find, dense,
  downloads, errors, external-url, zoom, context-menu, auth all green).

Performance evidence: not applicable; terminal bounded-exit contract. The two
        unchanged `startup` p95 failures (563.908209 ms and 517.736041 ms vs
        the 500 ms ceiling, no budget changes) remain explicit F2-closeout
        evidence and are not erased by this slice.

Roadmap/documentation updates: ARCHITECTURE.md and E2E_ARCHITECTURE.md
        document the lifecycle seam, response-flush guarantee, and
        e2e_shutdown suite; stale portal parent-close expectation corrected
        to the owning-window process-exit contract.

Open blockers and non-goals: PORTAL-MINIMIZE host/window-server state blocks
        one portal hardening test (pre-existing, not lane-caused); two
        startup p95 budget failures block the canonical perf gate (unchanged
        budgets, preserved as evidence). Non-goals per packet: persistence
        flush, warm restore, multi-window policy, SIGTERM/SIGHUP, Cmd-Q/menu,
        packaging automation, performance work.

Worktree state: /Users/patwoz/dev/Personal/macos/bbr-shutdown-lifecycle on
        branch parallel/shutdown-lifecycle, base e9f5d30214e7efc46586cb3ae11c6472e0b1801c;
        seven paths
        (ARCHITECTURE.md, E2E_ARCHITECTURE.md, window_seam.rs, hardening.rs,
        new e2e_shutdown.rs, server.rs, this packet); uncommitted candidate
        preserved for independent read-only review.
```
