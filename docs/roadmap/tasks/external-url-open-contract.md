# Task packet — portable external-URL open contract

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native-integration track](../tracks/native-integration.md).

## Outcome

`bbr-web` defines the dependency-free decision authority for OS-dispatched
external URL opens (BBR as the default handler for `http`/`https`): typed
requests, explicit URL classification with exact scheme rules, burst
deduplication for repeated OS deliveries, and a deterministic routing
decision (conventional macOS behavior: reuse the frontmost eligible window,
otherwise open a new window in the caller-declared default profile/space).
AppKit registration (`CFBundleURLTypes`, `LSSetDefaultHandlerForURLScheme`),
`Info.plist`, the `NSApplication` open-URLs delegate, and the adapter that
feeds OS events into this contract remain explicit later native work.

## Scope

- Opaque per-open request identity, raw URL passthrough, and an optional
  activation hint when the platform supplies one.
- Pure URL classification (`classify`): supported web URL, dangerous scheme
  (`javascript:`, `data:`, `file:`, `about:`, `vbscript:`), unsupported
  scheme, or malformed, with RFC 3986 scheme grammar, the crate's host:port
  look-alike judgment, port/authority/IPv6-bracket validation, and
  whitespace rules all documented and unit-proven.
- Privacy-reduced diagnostics matching the sibling web-error reduction rule
  (`scheme://host[:port]`, lower-cased, userinfo stripped; authority-less →
  scheme only; malformed → fixed placeholder) and a privacy-safe burst-dedup
  key (reduced origin plus an FNV-1a hash of the untransformed remainder —
  no full URLs, credentials, or query strings retained).
- Deterministic burst deduplication driven by caller-supplied monotonic
  ticks (no wall clock), first-anchored windows, and a bounded remembered
  set with oldest-first eviction.
- Routing decision (`decide_routing`): reuse the frontmost eligible window
  (fresh tab in the window's own profile/space) or open a new window in the
  caller-declared default profile/space; profile isolation and space
  ownership are explicit invariants.
- `ExternalUrlCoordinator` composing classification, dedup, and routing with
  explicit reject reasons (duplicate request id, non-monotonic tick) and no
  panics.
- Optional serde derives on value/wire shapes; default and serde-enabled
  suites plus Clippy `-D warnings` pass.
- Truthful task ledger; the native registration/adapter slice is named
  explicitly as the follow-up.

## Non-goals

- No `CFBundleURLTypes`, `LSSetDefaultHandlerForURLScheme`, default-handler
  claims, `Info.plist` edits, or `NSApplication` open-URLs delegate wiring.
- No adapter that maps OS events into `ExternalUrlRequest`s, no GPUI
  orchestration, no UI, no testkit, no GUI E2E, and no performance scenario.
- No URL rewriting, normalization, scheme defaulting, or "cleaning": the
  navigated URL is the request's own string; ambiguous input is rejected,
  never guessed.
- No persistence of recent URLs or opens.
- No edit to the central `WebEngine` trait; native composition is a later
  serial slice.

## Parallel-lane contract

```text
Base commit: d4cf240 (supplied at launch)
Depends on: integrated bbr-web ProfileId/WebViewId and the optional serde feature
Provides/API: external_urls request/classification/dedup/routing contract +
              ExternalUrlCoordinator store; root re-exports of classify,
              decide_routing, the coordinator, the request, and the decision
Exclusive paths: crates/bbr-web/src/external_urls/**;
                 crates/bbr-web/src/lib.rs (one additive module line + one
                 additive re-export);
                 docs/roadmap/tasks/external-url-open-contract.md;
                 docs/roadmap/phases/08-daily-driver.md (link only)
Shared integration points: later AppKit registration/adapter slice, app
                           orchestration, Phase-8 status
Integration order: portable contract first; the native registration +
                   adapter slice (named below) lands after it
```

## Constraints

- `#![forbid(unsafe_code)]` inherited; std plus existing optional serde only.
- Profiles are isolated containers: a routing decision can never silently
  cross a profile boundary (reuse takes the window's own profile; a new
  window takes the caller-declared default).
- Spaces organize tabs; a tab always belongs to exactly one space. The
  contract never invents or moves a space.
- Dangerous and unsupported schemes are classified explicitly and never
  opened; malformed input is rejected rather than guessed.
- Diagnostics and retention never contain full URLs with credentials or
  query strings; the reduction rule matches the sibling web-error contract.
- No new dependencies, manifests, lock changes, native frameworks, or shared
  root edits beyond the two additive lib.rs lines. Keep each file below
  roughly 500 lines.

## Evidence

### Unit and contract

- [x] Classification tests cover the exact scheme rules: web URL shape
      (host, port, userinfo strip, IPv6 brackets, case-insensitive schemes),
      every dangerous scheme, unsupported schemes, each malformed reason,
      and the privacy reduction rule. *(18 tests in
      `crates/bbr-web/src/external_urls/tests_classify.rs`, verified this
      slice.)*
- [x] Routing tests prove frontmost-eligible reuse, ineligible-window
      skipping, new-window fallback with caller-declared defaults, profile
      isolation, space ownership, deterministic caller order, and purity.
      *(8 tests in `tests_routing.rs`.)*
- [x] Dedup tests prove the inclusive tick-window boundary, first anchoring
      (duplicates never refresh), byte-exact remainder hashing with
      origin-case collapse, port-variant separation, capacity eviction,
      config rejection, and determinism without a wall clock. *(9 tests in
      `tests_dedup.rs`.)*
- [x] Coordinator tests prove the full pipeline, guard order (request-id
      reuse then tick monotonicity), error state preservation, rejection
      outcomes, dedup-through-the-store, and panic-free pathological input.
      *(9 tests in `tests_coordinator.rs`.)*
- [x] Serde tests prove opaque ids serialize as bare integers and every wire
      type satisfies `Serialize + Deserialize`. *(3 feature-gated tests in
      `tests_serde.rs`.)*
- [x] Default and serde-enabled crate suites plus Clippy `-D warnings` pass
      (default 318 lib tests, serde 360 lib tests, 11 doc-adjacent, 1
      doctest).

### Real-app E2E

- [x] Not applicable to this pure contract packet. Real default-browser
      registration, `NSApplication` open-URLs delivery, and window/tab
      creation require the later native adapter slice's real-app E2E.

### Performance

- [x] Not applicable: pure contract only. The later native slice adds
      measured external-open-to-presentation evidence where applicable.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-web
cargo test -p bbr-web --features serde
cargo clippy -p bbr-web --all-targets -- -D warnings
cargo clippy -p bbr-web --all-targets --features serde -- -D warnings
timeout 300 python3 scripts/check_roadmap.py
just roadmap
```

## Delivery sequence

1. One exclusive writer implements the contract in the parallel worktree.
2. One independent read-only reviewer maps requirements to evidence.
3. One remediation writer for concrete findings only, then a focused recheck.

## Iteration policy

Start with the minimal pipeline (request → classify → route), then add burst
deduplication, coordinator discipline guards, and serde. Diagnose failures as
classification rules, dedup-key semantics, routing eligibility, or store
guards before changing the contract. Do not broaden scope into native work.

## Blocked stop condition

If the portable seam requires a native URL handler, `Info.plist` entry,
AppKit delegate, central `WebEngine` edit, or wall-clock timing to be
coherent, preserve the branch and stop. Document the exact ambiguity and
defer to the native adapter packet; do not smuggle platform data into the
contract.

## Delivery ledger

```text
Outcome: portable external-URL open contract (request / classification /
         burst dedup / routing decision / coordinator store) implemented in
         crates/bbr-web/src/external_urls/ with 47 unit/contract/serde
         tests; packet linked from the Phase-8 checkbox without ticking it
Commits: none — changes left uncommitted in the parallel worktree per lane rule
Functional evidence: cargo fmt --all -- --check; cargo test -p bbr-web
                     (318 lib + 11 doc-adjacent + 1 doctest); cargo test
                     -p bbr-web --features serde (360 lib + 11 + 1); both
                     Clippy -D warnings gates; timeout 300 python3
                     scripts/check_roadmap.py; just roadmap — all green
Real-app E2E: not applicable to contract-only lane
Performance evidence: not applicable to contract-only lane
Open blockers and non-goals: native default-browser registration
                             (CFBundleURLTypes /
                             LSSetDefaultHandlerForURLScheme), Info.plist,
                             NSApplication open-URLs delegate, and the
                             adapter slice are the explicitly named
                             follow-up
Worktree state: complete and uncommitted; independent read-only review is the
                next delivery-sequence step
```
