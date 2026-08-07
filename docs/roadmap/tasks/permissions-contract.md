# Task packet — portable permissions contract

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native-integration track](../tracks/native-integration.md) ·
[Testing track](../tracks/testing.md).

## Outcome

`bbr-web` defines one dependency-free permission authority contract for camera,
microphone, location, notifications, and clipboard access. Requests carry
stable identity, owning profile, origin, capability, and user-gesture context;
decisions follow a typed lifecycle and expose explicit per-profile persistence
hooks without importing storage, WebKit, AppKit, GPUI, or app composition.

## Scope

- Stable request/scope identities and closed capability taxonomy.
- Typed request lifecycle: pending, resolved, cancelled, superseded, and stale
  callback rejection.
- One-time versus profile-persisted allow/deny decisions with pure validation
  and deterministic transition outcomes.
- Portable persistence-hook trait/DTOs keyed by `ProfileId + origin +
  capability`; no concrete repository implementation.
- Origin normalization/validation contract sufficient to prevent empty or
  cross-origin policy keys without introducing a URL dependency.
- Optional serde derives and focused default/serde contract tests.
- Truthful task ledger; adapter, UI, testkit, and E2E remain later Phase-8 work.

## Non-goals

- No macOS/WebKit permission delegate wiring or system prompts.
- No GPUI permission surface, settings UI, app/testkit changes, or GUI E2E.
- No persistence schema or `bbr-persistence` dependency.
- No per-device sync, enterprise policy, temporary timers, or speculative
  permission kinds beyond the five named capabilities.

## Parallel-lane contract

```text
Base commit: supplied at launch after the serial module anchor
Depends on: integrated bbr-web identities and optional serde feature
Provides/API: permissions request/decision/lifecycle + persistence hooks
Exclusive paths: crates/bbr-web/src/permissions/**;
                 docs/roadmap/tasks/permissions-contract.md
Shared integration points: later WebKit adapter, persistence implementation,
                           app surfaces/testkit, Phase-8 status
Integration order: portable contract may land whenever the serial bbr-web
                   kernel is free; adapters/UI remain later
```

## Constraints

- `#![forbid(unsafe_code)]` inherited; std plus existing optional serde only.
- Profile identity is mandatory on every request and persistence key.
- Persisted decisions never leak across profiles or origins.
- One-time decisions are never passed to persistence hooks.
- Request ids are stable and stale/mismatched completions reject whole.
- Cancellation/supersession cannot be converted into an allow decision.
- No new dependencies, manifests, lock changes, native frameworks, or broad
  shared-file edits. Keep each file below roughly 500 lines.

## Evidence

### Unit and contract

- [ ] Every capability and decision has stable Debug/Eq/serde shape.
- [ ] Lifecycle matrix covers request, durable-policy hit, resolve, cancel,
      supersede, duplicate completion, stale id, and invalid origin.
- [ ] Persistence-hook tests prove profile/origin isolation and that only
      durable decisions are stored/removed.
- [ ] Default and serde-enabled crate suites plus Clippy `-D warnings` pass.

### Real-app E2E

- [x] Not applicable to this pure contract packet. WebKit delegate wiring and
      user-visible permission surfaces own real-app E2E in a later slice.

### Performance

- [x] Not applicable: pure constant-size state transitions without a
      user-visible presentation path. Later prompt open/decision surfaces add
      optimized production-path evidence.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-web
cargo test -p bbr-web --features serde
cargo clippy -p bbr-web --all-targets -- -D warnings
cargo clippy -p bbr-web --all-targets --features serde -- -D warnings
just roadmap
```

## Delivery sequence

1. Implement identities/taxonomy and request validation.
2. Implement the pure lifecycle and persistence-hook boundary.
3. Add exhaustive default/serde tests and documentation.
4. Independent read-only review; one focused remediation and recheck only for
   concrete findings.

## Iteration policy

Start with one camera request and one one-time decision, then add durable policy,
all capabilities, cancellation/supersession, stale callbacks, and serde. Diagnose
failures as identity, lifecycle, origin key, persistence boundary, or wire shape
before changing the contract.

## Blocked stop condition

If a coherent portable boundary cannot express per-profile/origin policy
without importing persistence or native APIs, stop with the branch preserved
and report the missing identity/policy decision. Do not add a storage dependency
or app-level fallback to make the lane compile.

## Delivery ledger

```text
Outcome: packet and exclusive module anchor prepared; implementation pending
Commits: pending
Functional evidence: pending
Real-app E2E: not applicable to contract-only lane
Performance evidence: not applicable to contract-only lane
Open blockers and non-goals: adapters/UI/persistence implementation deferred
Worktree state: launch pending
```
