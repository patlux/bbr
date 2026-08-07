# Task packet — portable web error and recovery contract

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native-integration track](../tracks/native-integration.md) ·
[Testing track](../tracks/testing.md).

## Outcome

`bbr-web` defines a dependency-free, typed error taxonomy for TLS failures,
navigation failures, offline state, and web-process crashes, together with the
closed recovery actions each class may offer. The contract preserves profile,
web-view, URL/origin, retry safety, and stale-event identity without importing
WebKit, AppKit, GPUI, controller, or UI concerns.

## Scope

- Stable incident identity and closed error categories/subreasons for TLS,
  navigation, offline, and process crash.
- Typed severity, retryability, privacy-safe display context, and recovery
  action taxonomy (retry, reload, go back, open settings, inspect certificate,
  restore/recreate process where valid, dismiss where valid).
- Pure validation that exposes only category-legal actions and rejects unsafe
  or contradictory combinations.
- Incident lifecycle: reported, superseded, recovery requested, recovered,
  failed, dismissed, and stale callback rejection.
- Optional serde derives and exhaustive default/serde contract tests.
- Documentation describing adapter mapping boundaries without embedding native
  error codes or user-facing copy.

## Non-goals

- No GPUI error panel/copy/design, testkit wire, real-app E2E, or performance
  scenario.
- No WebKit NSError mapping or certificate inspection implementation.
- No controller lifecycle mutation, automatic retry scheduler, telemetry, or
  crash reporting.
- No compatibility bucket that preserves arbitrary untyped native payloads.

## Parallel-lane contract

```text
Base commit: supplied at launch after the serial module anchor
Depends on: integrated bbr-web ProfileId/WebViewId and optional serde feature
Provides/API: typed web-error incidents, lifecycle, recovery actions
Exclusive paths: crates/bbr-web/src/web_errors/**;
                 docs/roadmap/tasks/error-surface-contract.md
Shared integration points: later native adapter/controller/UI/testkit mapping,
                           Phase-8 status
Integration order: portable contract may land whenever the serial bbr-web
                   kernel is free; consuming UI/adapters remain later
```

## Constraints

- `#![forbid(unsafe_code)]` inherited; std plus existing optional serde only.
- Every incident identifies the owning profile and web view; stale ids cannot
  mutate a newer incident.
- TLS bypass is not a generic recovery action. Any future exception needs an
  explicit product/security contract and is out of scope here.
- Offline and navigation failures remain distinct; process crash is not modeled
  as an ordinary navigation retry.
- Recovery actions are derived from validated taxonomy, not arbitrary UI input.
- No new dependencies, manifests, lock changes, native frameworks, or broad
  shared-file edits. Keep each file below roughly 500 lines.

## Evidence

### Unit and contract

- [x] Taxonomy tests cover every category/subreason and stable serde shape.
- [x] Legal-action matrix proves TLS/navigation/offline/process-crash recovery
      options and rejects illegal combinations, especially TLS bypass.
- [x] Lifecycle tests cover supersession, stale completion, retry/reload,
      successful recovery, failed recovery, and dismissal legality.
- [x] Default and serde-enabled crate suites plus Clippy `-D warnings` pass.

### Real-app E2E

- [x] Not applicable to this pure contract packet. Native mapping and visible
      error/recovery surfaces own real-app E2E in a later Phase-8 slice.

### Performance

- [x] Not applicable: pure state/validation contract. Later visible recovery
      actions require optimized semantic-presentation evidence.

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

1. Implement incident identities/taxonomy and action validation.
2. Implement the pure incident/recovery lifecycle.
3. Add exhaustive default/serde tests and adapter-boundary documentation.
4. Independent read-only review; one focused remediation and recheck only for
   concrete findings.

## Iteration policy

Start with one navigation failure and retry action, then add offline, process
crash, TLS constraints, lifecycle supersession, and serde. Diagnose failures as
taxonomy, action legality, identity/staleness, lifecycle, or wire shape before
changing the contract.

## Blocked stop condition

If native error classes cannot map into a closed portable taxonomy without
losing a security-critical distinction, preserve the branch and report the
ambiguous codes/categories plus the smallest security/product decision needed.
Do not add an opaque catch-all payload or generic TLS bypass.

## Delivery ledger

```text
Outcome: packet complete and proven — taxonomy/legal actions and lifecycle
        implemented; step-3 evidence closeout verified all gates on a fresh
        rebuild and found no contract gaps; ready for handoff to the
        independent read-only review
Step 1: incident identity, closed categories/subreasons, severity/retryability,
        privacy-safe context, definition validation, recovery-action matrix
Step 2: pure lifecycle (reported/superseded/recovery requested/recovered/
        recovery failed/dismissed), requestable-action validation (category
        legal set minus Dismiss; crash accepts only Restore, TLS never a
        bypass), duplicate/stale/mismatched report and callback rejection
Step 3 (evidence closeout): re-ran every gate on a fresh rebuild after
        `cargo clean -p bbr-web`; re-checked matrix/serde/doc coverage and
        the crate-level missing-debug deny; fixed one doc-hygiene wart
        (stray empty first doc line on `IncidentContext`); all unit/contract
        checkboxes marked only on passing evidence; no remediation needed
Commits: none (all changes left uncommitted)
Functional evidence: cargo test -p bbr-web (94 lib + 11 + 1 doc)
                     cargo test -p bbr-web --features serde (99 lib + 11 + 1 doc)
                     cargo clippy -p bbr-web --all-targets -- -D warnings
                     cargo clippy -p bbr-web --all-targets --features serde -- -D warnings
                     cargo fmt --all -- --check; just roadmap (53 files valid); git diff --check
Real-app E2E: not applicable to contract-only lane
Performance evidence: not applicable to contract-only lane
Open blockers and non-goals: native adapter mapping and visible error/recovery
        surfaces deferred to a later Phase-8 slice; packet complete and proven,
        ready for the independent read-only review
Worktree state: all steps (1+2+evidence) uncommitted in parallel/p8b-error-surface-contract

Step 4 (remediation, review finding): fixed the privacy edge in
        `origin_of` — authority-less scheme URLs (`javascript:…`, `mailto:…`,
        `data:…`) previously kept their whole lowercased payload as "origin"
        when it contained no `/ ? #` delimiter (probe: `javascript:alert('secret')`
        -> `Some("javascript:alert('secret')")`); now reduced to the lower-cased
        scheme only, `about:blank` (any case) whitelisted verbatim, scheme-less
        typed hosts (`example.com/path`) unchanged. Added `javascript:`/`mailto:`/
        `data:`/`tel:`/`about:config`/`ABOUT:BLANK` regression cases to
        `context_from_url_is_privacy_safe` (fail against the pre-fix behavior).
        Re-ran all invalidated gates fresh: fmt, diff-check, both crate suites
        (94/99 lib + 11 + 1 doc), Clippy `-D warnings` both feature sets,
        check_roadmap.py (53 files); all green. No scope change, no new deps,
        no manifest/lock/lib.rs edits. Blockers: none; handoff to reviewer recheck
        complete. Native adapter/UI mapping remains a later Phase-8 slice.
```
