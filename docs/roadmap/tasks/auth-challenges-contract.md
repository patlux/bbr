# Task packet — portable authentication challenges contract

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native-integration track](../tracks/native-integration.md) ·
[Testing track](../tracks/testing.md).

## Outcome

`bbr-web` defines one dependency-free authentication challenge/response
contract for HTTP origin/proxy authentication and supported client-certificate
selection. Challenges preserve stable request, profile, web-view, protection-
space, and user-gesture identity; responses follow a typed lifecycle without
exposing native objects, certificate/private-key bytes, or password material in
Debug output.

## Scope

- Stable challenge identity and closed challenge taxonomy:
  HTTP origin authentication, HTTP proxy authentication, and client-certificate
  selection.
- Typed protection-space context: normalized host, port, realm, authentication
  scheme, proxy/origin distinction, owning `ProfileId` and `WebViewId`.
- Closed HTTP schemes for supported adapter mappings; unsupported schemes are a
  typed challenge kind/rejection, never an opaque native payload.
- Opaque credential/certificate handles suitable for later profile-scoped
  credential storage and native identity lookup. No certificate/private-key
  bytes or platform object references cross the portable boundary.
- Response choices: use an opaque credential/identity, continue without one
  where legal, perform default handling where legal, reject/cancel.
- Pure lifecycle: pending, response requested, resolved, failed, cancelled,
  superseded, duplicate/stale/mismatched callback rejection.
- Optional serde derives and focused default/serde contract tests, including
  redacted Debug behavior for sensitive handle metadata.
- Truthful task ledger; WebKit delegates, credential UI/storage, native client-
  certificate enumeration, trust evaluation, and E2E remain later adapter work.

## Non-goals

- No TLS server-trust bypass or certificate-exception policy. TLS failures and
  legal recovery remain owned by the web-error contract.
- No password manager, Keychain/KeePass integration, persistence schema, or
  direct secret storage.
- No WebKit/AppKit delegate implementation, native identity enumeration,
  GPUI prompt, testkit, GUI E2E, or performance scenario.
- No raw `NSURLAuthenticationChallenge`, `SecIdentity`, certificate DER,
  private-key data, platform error codes, or arbitrary native dictionaries.
- No speculative WebAuthn/passkey contract.

## Parallel-lane contract

```text
Base commit: supplied at launch after the serial module anchor
Depends on: integrated bbr-web ProfileId/WebViewId and optional serde feature
Provides/API: authentication challenge taxonomy, response and lifecycle
Exclusive paths: crates/bbr-web/src/auth_challenges/**;
                 docs/roadmap/tasks/auth-challenges-contract.md
Shared integration points: later WebKit adapter, credential authority/UI,
                           profile persistence, Phase-8 status
Integration order: portable contract may land after the serial bbr-web kernel
                   is free; consuming adapters/UI remain later
```

## Constraints

- `#![forbid(unsafe_code)]` inherited; std plus existing optional serde only.
- Every challenge identifies its profile, web view, and stable challenge id.
- Protection-space keys cannot be empty, cross-profile, or ambiguous between
  proxy and origin authority.
- Response legality is derived from the validated challenge kind; client-
  certificate identities cannot answer HTTP password challenges and vice versa.
- Cancellation/supersession can never become credential use; stale completions
  reject whole.
- Sensitive handles and labels have intentional Debug behavior; no password,
  certificate, or private-key bytes exist in the portable types.
- No new dependencies, manifests, lock changes, native frameworks, or shared-
  root edits. Keep each file below roughly 500 lines.

## Evidence

### Unit and contract

- [x] Taxonomy tests cover origin/proxy HTTP and client-certificate challenges,
      every supported scheme, invalid protection spaces, and stable identity.
      *(this slice: ids, protection space, taxonomy, opaque handles — proven)*
- [x] Legal-response matrix rejects cross-kind credentials, illegal default/
      continue choices, stale ids, duplicate callbacks, and superseded requests.
      *(this slice: `AuthResponse` + `validate_response`, exhaustive kind ×
      response matrix, reject-only server trust)*
- [x] Lifecycle tests cover request, response, native success/failure, cancel,
      supersede, and late callback rejection. *(this slice: `AuthState`/
      `AuthEvent`/`apply_event`, exhaustive state × event matrix, cancel/
      supersede never become credential use)*
- [x] Serde and Debug tests prove stable wire shapes and absence of secret/
      certificate/private-key material. *(this slice: lifecycle wire shapes,
      round trips, no-secret Debug and wire assertions)*
- [x] Default and serde-enabled crate suites plus Clippy `-D warnings` pass.

### Real-app E2E

- [x] Not applicable to this pure contract packet. WebKit delegate mapping,
      credential/client-certificate UI, and representative authenticated sites
      own real-app E2E in later Phase-8 slices.

### Performance

- [x] Not applicable: pure constant-size validation/transitions. Later visible
      authentication surfaces add optimized presentation evidence.

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

1. Implement ids, protection-space validation, taxonomy, and opaque handles.
2. Implement legal response derivation and the deterministic lifecycle.
3. Add exhaustive default/serde/security-shape tests and adapter-boundary docs.
4. Independent read-only review; one focused remediation and recheck only for
   concrete findings.

## Iteration policy

Start with one Basic origin challenge answered by one opaque credential handle.
Then add proxy distinction, remaining supported schemes, client certificates,
cancellation/supersession, stale callbacks, and serde. Diagnose failures as
protection-space identity, challenge taxonomy, response legality, lifecycle,
or sensitive-data shape before changing the contract.

## Blocked stop condition

If supported HTTP/client-certificate behavior cannot be represented without raw
native objects, secret bytes, or a security-critical ambiguity, preserve the
branch and report the exact missing distinction plus the smallest adapter or
product decision needed. Never add generic trust bypass or an opaque native
payload escape hatch.

## Delivery ledger

```text
Outcome (slice 2): challenge-derived legal responses and the deterministic
        lifecycle landed — closed AuthResponse set (credential/identity
        handles, continue, engine default, cancel) validated per challenge
        kind; server trust is reject-only; AuthState/AuthEvent/AuthFailure
        with apply_event enforcing one decision cycle per id, challenge-kind
        response re-validation, mismatched-id/early/duplicate/stale-callback
        rejection, and terminal cancellation/supersession that can never
        become credential use. Exhaustive kind×response and state×event
        matrices; serde wire shapes + round trips; redacted Debug.
Commits: none (lane rule: changes stay uncommitted in this worktree)
Functional evidence: cargo fmt --all -- --check; cargo test -p bbr-web
        (83+11+1); cargo test -p bbr-web --features serde (98+11+1);
        clippy -D warnings for both feature sets; roadmap links check
        (check_roadmap.py, `just` unavailable on this host). All green on
        base 92a0bfb.
Real-app E2E: not applicable to contract-only lane
Performance evidence: not applicable to contract-only lane
Open blockers and non-goals: native delegates, credential UI/storage,
        client-cert enumeration, trust evaluation, persistence, and E2E
        remain later adapter work.
Worktree state: uncommitted changes confined to crates/bbr-web/src/auth_challenges/**

Evidence/closeout pass (independent): re-exported MAX_HOST_LEN and
        MAX_REALM_LEN in mod.rs (rustdoc private-intra-doc-link errors in
        protection_space.rs fixed; constants are public contract bounds).
        Full gate set re-run green: fmt --check, git diff --check, tests
        83+11+1 / 98+11+1, clippy -D warnings both feature sets, `just
        roadmap` (links valid, 56 files). Pre-existing rustdoc error outside
        lane scope: downloads/machine.rs broken intra-doc links
        (DownloadAction::Cancel, DownloadFailure::Network) at base; not
        touched. `just` is available inside the nix develop shell (earlier
        ledger note about missing binary applies only to the bare host).

Remediation pass (review findings): manual `Deserialize` for
        `ProtectionSpace` now routes through `ProtectionSpace::new`, so the
        constructor's validation *and* normalization (default-port
        canonicalization, host case folding, realm trimming) are re-applied
        on the wire — an https space spelled with `port: 443` deserializes
        equal to the canonical `port: 0` space for the same authority, so
        wire and constructor keys can never diverge (regression
        `deserialize_canonicalizes_default_port_and_host` in tests_serde).
        Self-supersession is rejected: `Supersede { by: <own id> }` returns
        `EventRejection::SelfSupersede` from `Pending` and
        `ResponseRequested` instead of reaching the terminal state
        (regression `self_supersede_is_rejected`; classify arm added to the
        transition-matrix support). Findings 2 and 3 assessed as deliberate
        non-fixes: the unbounded `UnsupportedChallenge::Other { mechanism }`
        string is diagnostic-only on an internal seam (not an untrusted
        boundary) — bounding it would require a public rejection variant and
        would create a wire/constructor asymmetry; IPv6 zone-id lowercasing
        is documented normalization with negligible real-world exposure
        (Darwin interface names are lowercase in practice), and changing it
        would alter the documented wire normalization. Full gate set re-run
        green in the nix develop shell (rustc 1.97.1): cargo fmt --check,
        git diff --check, cargo test -p bbr-web (84+11+1), cargo test -p
        bbr-web --features serde (100+11+1), clippy -D warnings for both
        feature sets, `just roadmap` (links valid, 56 files). Ledger note:
        the review's serde baseline of 98 undercounted by one pre-existing
        serde-gated downloads test (downloads/tests_contract.rs:236); the
        true baseline is 99 and this pass added exactly one serde-gated
        test (99→100) and one default-suite test (83→84).
```
