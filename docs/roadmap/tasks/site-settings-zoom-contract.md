# Task packet — portable per-site settings and zoom contract

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native-integration track](../tracks/native-integration.md) ·
[Testing track](../tracks/testing.md) · [Phase 7 persistence](../phases/07-persistence.md).

## Outcome

`bbr-web` defines a dependency-free per-site settings authority and zoom
lifecycle. Site keys are profile-isolated and origin-scoped; effective settings
combine defaults with stored overrides; zoom uses validated bounded steps and
explicit persistence commands without importing SQLite, WebKit, AppKit, GPUI,
or app composition.

## Scope

- Stable normalized site/origin key owned by `ProfileId`; no cross-profile
  settings lookup or write.
- Typed zoom factor/percent with one documented default, minimum, maximum, and
  deterministic step/reset behavior.
- Per-site settings value and sparse override model. MVP fields remain bounded
  to zoom plus Phase-8-ready portable policy values that are already unambiguous
  without native APIs; do not speculate broad browser preferences.
- Pure effective-value resolution from defaults + optional site override.
- Deterministic zoom lifecycle: load default/stored value, zoom in/out, set,
  reset, persist, clear override, supersede, stale persistence callback reject.
- Portable store port/DTOs for get/put/remove keyed by profile + site. No
  repository implementation or schema migration in this packet.
- Optional serde derives and focused default/serde contract tests.
- Truthful task ledger; persistence schema/repository, WebKit page-zoom mapping,
  UI, testkit, and E2E remain later slices.

## Non-goals

- No `bbr-persistence` dependency, SQLite migration, direct filesystem/storage,
  app settings UI, command registration, testkit, GUI E2E, or performance work.
- No WebKit page/text zoom calls, CSS injection, viewport emulation, or native
  adapter implementation.
- No URL parser dependency, registrable-domain/Public Suffix List behavior,
  wildcard domains, subdomain inheritance, private-mode policy, sync, or
  enterprise policy.
- No permission decisions; the permissions contract remains authoritative.
- No speculative settings beyond fields with a complete portable semantic
  contract. Zoom-only is acceptable if additional fields are ambiguous.

## Parallel-lane contract

```text
Base commit: supplied at launch after the serial module anchor
Depends on: integrated bbr-web ProfileId and optional serde feature
Provides/API: profile/site key, settings overrides/effective resolution,
              zoom lifecycle and store port
Exclusive paths: crates/bbr-web/src/site_settings/**;
                 docs/roadmap/tasks/site-settings-zoom-contract.md
Shared integration points: later persistence migration/repository, WebKit
                           adapter, command/settings UI, Phase-8 status
Integration order: portable contract may land after the serial bbr-web kernel
                   is free; storage/native/UI remain later serial slices
```

## Constraints

- `#![forbid(unsafe_code)]` inherited; std plus existing optional serde only.
- Site keys are explicit normalized origins, not display URLs or arbitrary
  strings; invalid/empty origins reject whole.
- Profile identity is mandatory on every key and store command.
- Zoom values are finite, bounded, deterministic, and never represented by raw
  unchecked `f32` in public state.
- Default values do not create persistence records; reset removes an override.
- Stale/mismatched persistence acknowledgements cannot mutate newer state.
- No new dependencies, manifests, lock changes, native frameworks, or shared-
  root edits. Keep each file below roughly 500 lines.

## Evidence

### Unit and contract

- [ ] Site-key tests cover normalization/validation, profile isolation, port
      distinctions, and stable equality/order/serde shape.
- [ ] Zoom tests cover default, bounds, step sequence, set, reset, no floating-
      point drift, and invalid inputs.
- [ ] Store/effective-value tests prove default versus override behavior,
      profile/site isolation, put/remove commands, supersession, and stale ack.
- [ ] Additional settings, if any, have complete default/override/serde tests;
      ambiguous fields are omitted rather than guessed.
- [ ] Default and serde-enabled crate suites plus Clippy `-D warnings` pass.

### Real-app E2E

- [x] Not applicable to this pure contract packet. Persistence integration,
      WebKit zoom application, settings UI, restart restoration, and profile
      isolation own real-app E2E in later slices.

### Performance

- [x] Not applicable: pure validation/state transitions. Later user-visible zoom
      commands add optimized presentation evidence.

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

1. Implement profile/site identity and deterministic zoom value/steps.
2. Implement effective settings, lifecycle, and portable store boundary.
3. Add exhaustive default/serde/object-safety tests and integration docs.
4. Independent read-only review; one focused remediation and recheck only for
   concrete findings.

## Iteration policy

Start with one normalized HTTPS site key and default 100% zoom. Then add bounded
steps/reset, profile isolation, persisted override commands, stale callback
handling, optional unambiguous settings, and serde. Diagnose failures as origin
identity, zoom arithmetic, override resolution, store lifecycle, or wire shape
before changing the contract.

## Blocked stop condition

If a correct site identity requires a registrable-domain/PSL decision, or if
zoom semantics cannot be represented without native/WebKit behavior, preserve
the branch and report the exact product/platform choice needed. Do not add a URL
dependency, wildcard matching, raw float state, or persistence dependency as a
shortcut.

## Delivery ledger

```text
Outcome: packet and exclusive module anchor prepared; implementation pending
Commits: pending
Functional evidence: pending
Real-app E2E: not applicable to contract-only lane
Performance evidence: not applicable to contract-only lane
Open blockers and non-goals: persistence schema/repository, WebKit mapping, and UI deferred
Worktree state: launch pending
```
