# Renderer lifecycle acceptance decision

Date: 2026-08-11

Authority scope: Renderer Isolation Slice 3 acceptance contract only

Related evidence:

- [public WebContent ownership-proof research](renderer-webcontent-public-ownership-proof.md)
- [renderer isolation task packet](../roadmap/tasks/renderer-isolation-unresponsive-tabs.md)
- [current delivery authority](../delivery/current.md)

## Decision

**Yes: the blocked helper-ownership test can be replaced by a reviewable,
public-API-only split acceptance contract.**

This changes the test mechanism, not the product guarantee. BBR must still
prove that real unresponsive WebKit content never blocks native chrome and that
all BBR-owned state remains consistent when WebKit reports a content-process
termination. What cannot be required is a deterministic automated real helper
kill: the public platform neither exposes a safe target identity nor provides a
supported API that asks one `WKWebView` content process to terminate.

The accepted boundary is:

1. BBR-owned behavior is automated end to end wherever BBR controls the input.
2. The public native delegate and public termination-error mappings are tested
   directly at the adapter boundary.
3. The productive app/controller transport and multi-session reconciliation are
   tested deterministically with typed events.
4. Only the final external hop — WebKit deciding that a process terminated and
   invoking its documented callback or returning its documented error — rests
   on Apple's public API contract.
5. No acceptance gate enumerates, selects, terminates, or infers ownership of a
   `com.apple.WebKit.WebContent` helper.

## Inputs and independence

The decision was made after separate comparison of:

- `PRODUCT.md`: native engine efficiency, portable product logic, preserved
  browsing context, and daily-driver reliability;
- `ARCHITECTURE.md`: one `WebEngine` boundary, one `BrowserController` session
  authority, native WKWebViews, documented `PageCrashed` delegate mapping, and
  stale-view fail-closed behavior;
- `E2E_ARCHITECTURE.md`: real-app/real-WKWebView evidence, typed event waits,
  deterministic fixtures, production transport, no sleeps or pixel-primary
  assertions;
- the canonical renderer task packet and exact uncommitted Slice-3 state;
- the independently reviewed Path-A negative public-ownership result;
- the full external read-only report from Pi session
  `019fedb3-0649-712f-a035-0469bc8c67aa`, treated as a finding source rather
  than authority.

The external report's core split-evidence proposal is accepted only after the
contract comparison above. Two parts are deliberately not adopted:

- Activity Monitor or any same-name helper kill is not accepted as manual or
  automated evidence because Path A proved that public helper metadata does
  not bind a target to the exact BBR execution.
- `WKWebView.interactionState` is a possible future restoration enhancement,
  not an MVP acceptance dependency and not an authority change in this slice.

## Why the previous gate is invalid

The previous process case required the testkit to discover an app-owned
WebContent helper, terminate one, observe at least one affected session, and
reconcile siblings. That is safe only when the helper is provably owned by the
current BBR execution. On the target host every observed helper had `ppid=1`,
and Path A found no supported ordinary-app replacement for that parent proof.

Keeping the kill case as mandatory would produce one of three invalid outcomes:

- accept zero candidates as success, which proves no termination behavior;
- select by name/path/time/coalition or similar heuristic, which risks a foreign
  process and fails exact-instance ownership;
- use private WebKit/XNU state or a privileged EndpointSecurity architecture,
  which violates the current product boundary.

The gate is therefore not merely unavailable on one host. Its required target
selection is outside the supported ordinary-app API contract. Removing it does
not weaken a controllable product assertion; it removes an unsafe and
unimplementable test mechanism.

## Public platform boundary

The Xcode 26.2 public SDK provides:

- `WKNavigationDelegate.webViewWebContentProcessDidTerminate(_:)`;
- `WKErrorDomain` with `WKErrorWebContentProcessTerminated`;
- no public `WKWebView` PID, audit token, XPC connection, terminate method, or
  deterministic process-crash trigger;
- a documented implementation-defined process-sharing model in
  `WKProcessPool.h`.

Therefore the uncontrollable platform statement is narrowly limited to:

> When WebKit reports a content-process termination through its documented
> callback or documented termination error, it supplies that signal to BBR's
> registered adapter.

Everything after that signal enters BBR-owned code and remains automated.

## Required acceptance chain

### A1 — Real unresponsive-content E2E

Use the real optimized-capable BBR process, real WKWebViews, the production
responsiveness probe, and deterministic first-party fixtures.

Required evidence:

- the permanently busy fixture becomes `unresponsive` through the real bounded
  client-world probe/deadline;
- the heavy-but-answering control remains responsive;
- Cmd-W, row close, focused-list Backspace, warm-neighbor selection, Cmd-L,
  Cmd-T-capable chrome, sidebar, spaces, and split operations remain available
  under typed deadlines while the content renderer is actually busy;
- Wait keeps the session monitored; Reload destroys and recreates the same tab
  in the same profile store; Close uses the ordinary close authority;
- dialog and beforeunload boundaries remain deterministic and nonblocking;
- stale old-view completions after close/recreate mutate nothing.

The existing Slice-3 state already reports these four non-process real-app
scenarios green. Final acceptance re-runs the complete focused suite after the
contract remediation on the unchanged product behavior.

### A2 — Native adapter lifecycle contract

Add public-API-only `bbr-web-macos` tests around BBR's own retained delegate and
mapping helpers.

Required evidence:

- direct invocation of BBR's own registered
  `webViewWebContentProcessDidTerminate:` implementation emits exactly the
  expected `PageCrashed(WebViewId)` and classified process-crash incident for
  the bound session;
- a native `NSError` with `WKErrorDomain` and
  `WKErrorWebContentProcessTerminated` maps to the same portable termination
  authority rather than an invented navigation cause;
- if callback and error signals both arrive for one old incarnation, the second
  signal is stale/idempotent at the controller boundary: no second session
  destroy and no mutation of a recreated view;
- no private selector, KVC, helper PID, or process operation is used.

Calling BBR's own delegate implementation is an adapter contract test, not a
claim that the test caused WebKit itself to terminate. The test report must use
that wording.

### A3 — Portable multi-session reconciliation contract

Use `bbr-browser` with the deterministic fake engine.

Required evidence:

- an active or warm session receiving `PageCrashed` is destroyed exactly once,
  while its persistent tab identity and correct profile ownership remain;
- split layout never publishes a false live pane after a member crash;
- every still-live sibling is re-probed independently;
- sequential `PageCrashed` events for multiple views model a shared-process
  burst without assuming event count or order from WebKit;
- duplicate, unknown, destroyed, and old-incarnation events fail closed;
- invariants hold after each event, not only at the end of the burst.

### A4 — Productive real-app crash/recovery transport

Use the authenticated testkit's existing typed `InjectWebEvent(PageCrashed)`
entry, which feeds the same app event pump/controller path as the native sink.
It does not claim to emulate the external WebKit callback invocation.

Required evidence:

- active-tab crash presents the production crash panel/status semantics,
  destroys exactly one disposable session, and preserves tab identity;
- Restore/Reload recreates exactly one fresh session in the correct profile
  store and clears only that tab's crash state;
- split/member and multi-view burst cases preserve layout/session invariants;
- stale event delivery after recreation cannot corrupt the fresh session;
- affected crash, error-surface, Peek, tabs, profiles, and responsiveness
  regressions remain green.

A2 proves the native adapter output. A4 proves the complete BBR-owned path from
that typed output through the productive transport, controller, session
registry, and UI. Together they leave no untested BBR-owned gap.

### A5 — Explicit platform assumption

The remaining WebKit-to-delegate hop is recorded as a public platform contract,
with the SDK headers as primary-source evidence. It is not reported as an
automated BBR pass.

Optional nondeterministic observation may be recorded only when an environment
provides a safe externally authenticated target. It is never a merge gate and
may not use same-name, path, UID, launch-time, coalition, candidate-count, or
per-view-PID inference. The current host provides no such target.

## Candidate remediation authorized after commit

After this decision and its independent review are committed into current
authority, exactly one bounded writer may continue the preserved Renderer
worktree from temporary tree
`4dc4deb042e0bc818c70ac1aae7ac81bd938bdd7`.

The writer's contract is:

1. remove the obsolete helper observation/termination feature completely,
   including process-selection code, typed request/response surface, dispatch,
   E2E process case, feature/dependency wiring, and documentation claims;
2. add A2's delegate and termination-error adapter tests/mapping;
3. add any missing A3 multi-view burst/idempotency tests;
4. add any missing A4 real-app crash/recovery transport cases using the existing
   typed event seam;
5. preserve A1's real busy/heavy/chrome/recovery/dialog behavior and re-run the
   focused functional suites;
6. preserve all performance code, budgets, baselines, sample guards, and
   `samples: 0`; execute no performance recipe under the active override;
7. update implemented-fact documentation to name the split evidence and the
   Apple-owned hop honestly;
8. return a fresh exact candidate fingerprint for independent whole-candidate
   review; do not commit or integrate.

Removing the process seam is required, not optional compatibility. The project
does not retain a dead test API or a dormant helper-kill path after its
acceptance purpose is rejected.

## Rejected alternatives

| Alternative | Decision |
|---|---|
| Keep real helper kill mandatory | Rejected: no supported safe target proof. |
| Accept `eligible=[]` as pass | Rejected: proves no lifecycle behavior. |
| Name/path/UID/time/lowest-PID selection | Rejected: foreign-process and PID-reuse risk. |
| Coalition or responsible-process query | Rejected: private or privileged; coalition also observed non-unique. |
| Private WebKit termination SPI/KVC | Rejected: unsupported product/test contract. |
| EndpointSecurity helper | Not part of this decision; requires separate explicit privileged product/security architecture. |
| Manual same-name Activity Monitor kill | Rejected on the current target: no exact BBR ownership proof. |
| Memory-pressure fixture as gate | Rejected: nondeterministic, host-destructive, and does not provide target ownership. |
| Treat injected `PageCrashed` alone as end-to-end | Rejected: A2 native adapter contract remains mandatory. |
| Treat delegate-unit test alone as product acceptance | Rejected: A1, A3, and A4 remain mandatory. |

## Independent review

A fresh read-only reviewer recomputed the staged candidate and preserved
Renderer state, then returned direct `PATH_ACCEPTANCE_REVIEW_PASS`. Verified
bindings:

- staged pre-authority-transition diff SHA-256
  `39ddac2f87abb693c231bb56a30941096267cbaf9c73f59a4b16ef02b8a1b4c9`;
- Renderer temporary tree
  `4dc4deb042e0bc818c70ac1aae7ac81bd938bdd7`;
- Renderer porcelain-v1 `-z` SHA-256
  `fa031f0d454a988b760d3a7982ad9fc3e4791284928a04ea49f8604d1d00123c`;
- HEAD-to-tree binary diff SHA-256
  `572be7933d7e6e6ac624e9ddbcafe2a9f8f2e72fe6c7d9eec888a703c721ffae`;
- 96-path fingerprint
  `bb91851dee4bb704cbcbcf281e631e6e7dd2d73be1ff639324a868bda32ab95d`.

The review accepted public-API correctness, the direct-delegate test boundary,
productive transport equivalence, multi-view ordering/idempotency requirements,
complete process-seam removal, performance/canonical-E2E deferrals, and the
absence of premature writer authority. The supervisor reached its turn budget
after the complete report was written; the full report remains preserved in
`.pi-subagents/artifacts/bc5bc651_reviewer_0_output.md`. No file was mutated by
the reviewer.

## Acceptance verdict

The Renderer lane may become reviewable without a real helper kill **only**
after A1–A4 are implemented and green, A5 is stated as unpassed external
platform dependence, obsolete process-kill code is removed, the exact candidate
receives independent whole-candidate review, and the Orchestrator integrates
that reviewed tree.

This decision does not claim performance, startup, packaging, accessibility,
manual site-matrix, or first-MVP completion. Those debts remain separately open
under current authority.
