# Task packet — portable file upload/save-panel contract

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native-integration track](../tracks/native-integration.md) ·
[Testing track](../tracks/testing.md).

## Outcome

`bbr-web` defines a dependency-free request/completion lifecycle for web-engine
file uploads and save destinations. The portable boundary describes what the
engine requests and receives through opaque adapter-owned selection tokens;
AppKit open/save panels, filesystem paths, sandbox access, security-scoped
bookmarks, file I/O, and UI orchestration remain explicit later adapter work.

## Scope

- Stable request identity with owning `ProfileId`, `WebViewId`, request kind,
  user-gesture context, and optional origin.
- Upload request metadata: accepted MIME/extensions, single/multiple selection,
  file versus directory capability, and capture hint where representable.
- Save request metadata: suggested filename, content type, replacement policy,
  and destination requirement without choosing or exposing a native path.
- Opaque adapter-issued selection/destination tokens. Portable code may compare,
  serialize, and return them, but cannot inspect a path, bookmark, file handle,
  URL, native object, or security scope.
- Pure validation and deterministic lifecycle: pending, selection requested,
  selected, cancelled, failed, superseded, duplicate/stale/mismatched completion
  rejection.
- A module-local object-safe `FilePanelPort` (or equivalent command/result
  boundary) for engine-to-orchestrator request delivery and completion return.
  Do not change the shared root `WebEngine` trait in this parallel lane.
- Optional serde derives and focused default/serde contract tests.
- Truthful task ledger; native panel orchestration and WebKit delegate binding
  are explicitly later adapter packets.

## Non-goals

- No `NSOpenPanel`, `NSSavePanel`, AppKit/WebKit delegate implementation, GPUI
  panel, testkit, GUI E2E, or performance scenario.
- No filesystem reads/writes, file existence checks, path normalization,
  security-scoped bookmark creation/access, sandbox entitlements, or download
  destination implementation.
- No portable absolute/relative path strings, `file://` URLs, file descriptors,
  native pointers, or platform payloads.
- No persistence of recent folders or selections.
- No edit to `bbr-web/src/lib.rs` or the central `WebEngine` trait; serial
  adapter integration decides how the module-local port is composed later.

## Parallel-lane contract

```text
Base commit: supplied at launch after the serial module anchor
Depends on: integrated bbr-web ProfileId/WebViewId and optional serde feature
Provides/API: file-panel requests, opaque selections, completion lifecycle,
              object-safe engine/orchestrator port
Exclusive paths: crates/bbr-web/src/file_panels/**;
                 docs/roadmap/tasks/file-panels-contract.md
Shared integration points: later WebKit/AppKit adapter, app UI/testkit,
                           sandbox/entitlements, Phase-8 status
Integration order: portable contract may land after the serial bbr-web kernel
                   is free; native panel orchestration remains later
```

## Constraints

- `#![forbid(unsafe_code)]` inherited; std plus existing optional serde only.
- Every request/completion carries one stable request id and owning profile/view.
- Upload and save tokens are distinct types and cannot cross-complete requests.
- Selection cardinality and file/directory capability are validated before a
  completion can resolve the lifecycle.
- Cancel/supersede never becomes selected; stale or duplicate completions reject
  whole and cannot mutate a newer request.
- Opaque tokens must not expose paths or claim that portable code owns sandbox
  lifetime; adapter documentation states that responsibility explicitly.
- No new dependencies, manifests, lock changes, native frameworks, or shared-
  root edits. Keep each file below roughly 500 lines.

## Evidence

### Unit and contract

- [ ] Request tests cover upload/save metadata, invalid accept filters,
      single/multiple cardinality, directory legality, and suggested-name rules.
- [ ] Lifecycle matrix covers request, selected, cancelled, failed, superseded,
      duplicate/stale/mismatched completions, and cross-kind token rejection.
- [ ] Port tests prove object safety, exact request delivery, and completion
      identity without filesystem or native types.
- [ ] Serde tests prove stable opaque-token shapes without path/native payload.
- [ ] Default and serde-enabled crate suites plus Clippy `-D warnings` pass.

### Real-app E2E

- [x] Not applicable to this pure contract packet. Real `NSOpenPanel`/
      `NSSavePanel`, WebKit delegate composition, sandbox access, cancellation,
      and upload/save behavior require later real-app adapter E2E.

### Performance

- [x] Not applicable: pure contract only. Later visible panel launch/completion
      adds optimized production-path evidence where measurable.

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

1. Implement request ids/kinds/metadata, opaque tokens, and validation.
2. Implement the deterministic completion lifecycle and module-local port.
3. Add exhaustive default/serde/object-safety tests and adapter-boundary docs.
4. Independent read-only review; one focused remediation and recheck only for
   concrete findings.

## Iteration policy

Start with a single-file upload request, opaque selection token, and successful
completion. Then add cancellation/failure, multiple/directory rules, save
requests, supersession/stale callbacks, port object safety, and serde. Diagnose
failures as metadata validation, token-kind separation, lifecycle identity, or
adapter ownership before changing the contract.

## Blocked stop condition

If the portable seam requires a filesystem path, native object, bookmark,
entitlement decision, or central `WebEngine` edit to be coherent, preserve the
branch and stop. Document the exact ambiguity and defer the contract to the
serial native-adapter packet; do not invent a cross-platform path abstraction
or smuggle platform data through an opaque catch-all.

## Delivery ledger

```text
Outcome: portable boundary accepted; packet and exclusive module anchor prepared; implementation pending
Commits: pending
Functional evidence: pending
Real-app E2E: not applicable to contract-only lane
Performance evidence: not applicable to contract-only lane
Open blockers and non-goals: native panel orchestration, filesystem/sandbox access, and WebKit/AppKit composition explicitly deferred
Worktree state: launch pending
```
