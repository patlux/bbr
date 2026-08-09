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

- [ ] Lifecycle/helper tests distinguish the owning browser window from portal
      or unrelated window ids where a pure seam is introduced.
- [ ] Testkit server coverage proves a close-window response is flushed before
      the shutdown handler runs, matching the existing graceful-`Quit`
      guarantee.
- [ ] Default and testkit builds compile and Clippy remains warning-free.

### Real-app E2E

- [ ] `Ctrl-C` regression: spawn the real BBR binary, wait for typed readiness,
      deliver real `SIGINT`, and assert successful full process exit within 10
      seconds.
- [ ] Window-close regression: close the real owning browser window through
      the production GPUI/AppKit path and assert successful full process exit
      within 10 seconds, with no portal child keeping the process resident.
- [ ] Existing gateway `Quit`, portal teardown/close, smoke, and packaging
      launch behavior remain green.

### Performance

- [ ] Not applicable: this terminal lifecycle slice introduces no presented
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
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence: not applicable; terminal bounded-exit contract
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:
```
