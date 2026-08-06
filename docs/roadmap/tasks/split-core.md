# Task packet — split-core portable interaction state

Parents: [Phase 6](../phases/06-split-view.md) · [split shared contract](split-shared-contract.md) · [browser interaction contract](../../product/browser-interactions.md) · [Design §11.7](../../../DESIGN.md).

## Outcome

Provide the smallest portable split-core layer that can describe a user-facing two-pane split invocation without mutating shared GPUI/native composition contracts: a typed horizontal split direction and bounded divider position, deterministic geometry projection for a content rectangle, and explicit open/close/resize intents that preserve the existing domain/controller focus rules. The later serial app-composition slice consumes this portable API to render and drive the production split UI.

## Scope

- Add a new portable `bbr-split` workspace crate with no GPUI, AppKit, WebKit, testkit, persistence, network, or async dependency.
- Model the Phase-6 v1 layout only:
  - exactly two horizontal panes;
  - ordered primary/secondary identity supplied by the caller;
  - a normalized divider position clamped to keep both panes strictly positive;
  - pure projection from a portable content rect into primary/divider/secondary rects;
  - typed open, close, focus, and divider-resize intents/results suitable for later controller/app composition.
- Encode invariants as plain values and focused unit/contract tests:
  - no identical pane identities;
  - pre-split selected pane remains focused when opening;
  - selecting either member changes focus without changing pane order;
  - an outside identity cannot become split focus;
  - close keeps both pane identities available to the caller and identifies the focused survivor;
  - divider clamp remains deterministic for compact content and never yields zero/negative panes;
  - unchanged divider input is a no-op.
- Add the crate to workspace metadata and document the settled serial anchor and lane evidence here.

## Non-goals

- No production GPUI surface, divider element, drag zone, pointer listener, keyboard binding, command, menu, tooltip, focus ring, or visual polish.
- No changes to `bbr-app`, `bbr-browser`, `bbr-domain`, `bbr-web`, `bbr-web-macos`, `bbr-testkit`, E2E protocol, performance baseline, persistence schema, or existing split shared contracts.
- No vertical/top/bottom pane orientation, general tiling tree, more than two panes, persistence, compatibility layer, or unresolved split-close shortcut decision.
- No real-app E2E or new performance scenario in this portable lane; those are mandatory in the later serial app-composition slice that creates user-visible behavior.

## Parallel-lane contract

```text
Base commit:      29c34e68f4fad5f5338f7b69e50c44dd84145e74
                  (feat(browser): add keyed split geometry contract)
Branch/worktree:  parallel/split-core
                  /Users/patwoz/dev/Personal/macos/bbr-split-core
Depends on:       frozen WebEngine::set_web_view_rect(WebViewId, ...),
                  BrowserController::pane_layout(), and existing two-pane
                  domain/controller behavior
Provides/API:     portable split layout/intents only; exact public names are
                  settled inside this packet's constraints and verified by
                  crate tests
Exclusive paths:  crates/bbr-split/**
                  Cargo.toml (workspace member/dependency only)
                  Cargo.lock only if Cargo deterministically changes it
                  docs/roadmap/tasks/split-core.md (ledger only)
Shared integration points: root workspace membership is the sole serial
                  manifest touch; no existing production crate may consume
                  bbr-split in this lane
Integration order: review and commit this portable crate, then integrate it
                  serially; only afterward may a separate app-composition
                  packet add the bbr-app dependency and user-visible E2E/perf
```

## Constraints

- `#![forbid(unsafe_code)]`; portable, deterministic, dependency-minimal.
- Use established repository value types when dependency direction remains portable; otherwise define the smallest crate-local plain values rather than introducing a reverse dependency.
- Geometry math is pure and testable. Reject non-finite inputs rather than silently propagating NaN/Infinity.
- Divider policy is product-neutral: a bounded normalized position, no drag feel, animation, or keyboard step assumption.
- Do not evolve the already integrated public `bbr-web`, `bbr-browser`, domain, testkit, or app seams from this sibling worktree.
- Exactly one writer owns this worktree. No commit, push, merge, rebase, reset, clean, stash, or amend by a workflow agent.

## Evidence

### Unit and contract

- [ ] Exact public value and wire-free intent shapes have equality/debug tests.
- [ ] Open/focus/close invariants match the browser interaction contract.
- [ ] Geometry projection covers default, compact, clamped-low, clamped-high, unchanged, and invalid inputs.
- [ ] Property-style table coverage proves positive panes, ordered adjacency, total-width conservation, and deterministic results.
- [ ] The crate remains object/runtime independent and forbids unsafe code.

### Real-app E2E

- [ ] Not applicable to this non-user-visible portable landing zone. The follow-up app-composition packet must add real-app split interaction E2E in the same implementation group as production invocation.

### Performance

- [ ] A release-mode microcheck/benchmark may demonstrate trivial pure geometry cost, but it is not a substitute for the required optimized real-app split interaction scenario in the follow-up composition packet.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-split
cargo clippy -p bbr-split --all-targets -- -D warnings
cargo check --workspace --all-targets
just roadmap
git diff --check
```

No GUI suite runs in this isolated portable lane. After serial integration, the owner runs `just verify`; user-visible app composition later requires its affected real-app E2E, optimized performance scenario, and canonical `just e2e`.

## Delivery sequence

1. Create the sibling branch/worktree from exact anchor `29c34e6`.
2. One exclusive writer implements only the portable crate and task ledger.
3. One independent read-only reviewer maps API/invariants/tests to this packet.
4. One remediation writer only for concrete findings, then focused recheck.
5. Owner runs gates, commits, and serially integrates on clean `main`.
6. Draft a separate serial app-composition packet from the integrated anchor; do not expand this lane into shared UI/testkit surfaces.

## Iteration policy

Begin with plain layout values and exhaustive tests, then add intents only when a concrete contract test requires them. After any failure, identify whether the issue is input validation, invariant modeling, geometry conservation, or dependency direction before editing. Never weaken a test or add a compatibility fallback.

## Blocked stop condition

Preserve the branch/worktree and report exact evidence if the browser interaction contract requires an unresolved product decision (notably split-close shortcut semantics), if the portable model would need to own native/session focus, or if a coherent API cannot be expressed without changing existing shared public contracts. The minimal next input is the specific product or architecture decision; do not guess it inside this lane.

## Delivery ledger

```text
Outcome:       pending
Anchor:        29c34e68f4fad5f5338f7b69e50c44dd84145e74
Writer:        pending
Review:        pending
Functional evidence: pending
Real-app E2E:  not applicable until app composition
Performance:   portable-only; production scenario deferred with the UI
Commits:       pending
Open blockers: none at packet creation
```
