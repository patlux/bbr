# Task packet — split-core portable interaction state

Parents: [Phase 6](../phases/06-split-view.md) · [split shared contract](split-shared-contract.md) · [browser interaction contract](../../product/browser-interactions.md) · [Design §11.7](../../../DESIGN.md).

## Outcome

Provide the smallest portable split-core layer that can describe a user-facing two-pane split invocation without mutating shared GPUI/native composition contracts: a typed horizontal split direction, a bounded `f32` divider position, deterministic geometry projection for a content rectangle using the established `bbr_web::WebRect`, and explicit open/focus/close/divider intents validated and evaluated **statelessly** against caller-supplied authoritative values that preserve the existing domain/controller focus rules. The later serial app-composition slice consumes this portable API to render and drive the production split UI.

## Scope

- Add a new portable `bbr-split` workspace crate with exactly one existing portable dependency (`bbr-web`, for `WebRect`) and no GPUI, AppKit, WebKit, testkit, persistence, network, or async dependency.
- Model the Phase-6 v1 layout only:
  - exactly two horizontal panes (`SplitDirection::Horizontal` is the only variant);
  - ordered primary/secondary identity supplied by the caller;
  - a normalized `f32` divider position in `[0, 1]` (non-finite rejected, finite out-of-range clamped);
  - pure projection from a `bbr_web::WebRect` content rect into primary/divider/secondary rects that preserves the requested divider width;
  - typed open, focus, close, and divider-resize intents/results suitable for later controller/app composition.
- No stored pane, focus, or divider authority in this crate: `bbr-domain` split owns pane identities/order and the space active tab / controller pane layout owns focus. Expose stateless functions only:
  - `validate_open` validates an open request into an ephemeral `OpenPlan`;
  - `evaluate_focus` evaluates a focus request against caller-supplied authoritative primary/secondary/current focus;
  - `close_outcome` produces the close outcome from caller-supplied authoritative values;
  - `evaluate_resize` evaluates a divider resize against only the caller-supplied current divider value.
- Encode invariants as plain values and focused unit/contract tests:
  - no identical pane identities;
  - pre-split selected pane remains focused when opening;
  - selecting a member changes focus (explicit `Changed`); requesting the already-focused member is an explicit `Unchanged` no-op; pane order never changes;
  - an outside identity cannot become split focus (`Rejected`);
  - close keeps both pane identities available to the caller and identifies the focused survivor;
  - invalid geometry is rejected with typed errors (non-finite fields, non-positive width/height, negative or non-finite divider, overflowing coordinate endpoints, insufficient width for two minimum panes plus the divider, or ordered `f32` boundaries that cannot advance representably at the content offset/scale) and never silently rewritten;
  - for accepted geometry the divider width is preserved exactly, both panes stay at or above a documented minimum, ordered `f32` boundaries advance representably, adjacency holds, and the final endpoint equals the validated content endpoint exactly; unchanged divider input is a `changed: false` no-op.
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
- Use established repository value types when dependency direction remains portable (`bbr_web::WebRect` with `f32`); otherwise define the smallest crate-local plain values rather than introducing a reverse dependency.
- Geometry math is pure and testable. Reject invalid geometry with typed errors (non-finite fields, non-positive width/height, negative or non-finite divider, overflowing coordinate endpoints, insufficient width) rather than silently propagating NaN/Infinity or rewriting malformed finite values.
- Divider policy is product-neutral: the requested divider width is preserved, the primary pane width clamps continuously between a documented positive minimum and the available remainder, and no drag feel, animation, or keyboard step assumption is made.
- Do not evolve the already integrated public `bbr-web`, `bbr-browser`, domain, testkit, or app seams from this sibling worktree.
- Exactly one writer owns this worktree. No commit, push, merge, rebase, reset, clean, stash, or amend by a workflow agent.

## Evidence

### Unit and contract

- [x] Exact public value and wire-free intent shapes have equality/debug tests.
- [x] Open/focus/close/divider invariants match the browser interaction contract; the stateless functions evaluate against caller-supplied authoritative values and store no pane/focus/divider authority, and an outside caller-supplied `current` focus is rejected (`InvalidCurrent`) before any `Unchanged` result.
- [x] Geometry projection covers exact default, clamped-low, clamped-high, exact boundary, continuous near-boundary, realistic large representable, zero-divider ordering, and overflow/insufficient-width/negative/zero/non-finite/unrepresentable-boundary typed rejections.
- [x] Property-style table coverage proves minimum positive panes, preserved divider width, ordered adjacency with representably advancing boundaries, a final endpoint on the validated content endpoint (to `f32` rounding), total-width conservation (to `f32` rounding), finite values, and deterministic results.
- [x] The crate remains object/runtime independent, forbids unsafe code, and depends only on the existing portable `bbr-web` for `WebRect`.

### Real-app E2E

- [x] Not applicable to this non-user-visible portable landing zone. The follow-up app-composition packet must add real-app split interaction E2E in the same implementation group as production invocation.

### Performance

- [x] A release-mode microcheck/benchmark may demonstrate trivial pure geometry cost, but it is not a substitute for the required optimized real-app split interaction scenario in the follow-up composition packet. No microcheck was run in this lane; the production scenario stays with the composition packet.

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
Outcome:       Portable split-core landing zone remediated as a reviewable
               working-tree diff: `bbr-split` workspace crate with exactly
               one existing portable dependency (`bbr-web`, used for the
               established f32 host-layout `WebRect`; no duplicate Rect
               type). The crate stores no pane/focus/divider authority:
               `validate_open` validates a typed horizontal open request
               (SplitDirection::Horizontal is the only variant) into an
               ephemeral OpenPlan; `evaluate_focus` evaluates a focus
               request against caller-supplied authoritative
               primary/secondary/current focus, rejecting an outside
               caller-supplied `current` (`InvalidCurrent`) before any
               `Unchanged` result, with explicit Unchanged for the
               already-focused member, Changed only on a real change, and
               Rejected for outside request targets; `close_outcome`
               produces the focused survivor plus both identities from
               caller values; `evaluate_resize` evaluates a divider resize
               against only the caller-supplied current value with f32
               no-op equality. Geometry rejects invalid input with typed
               errors (non-finite fields, non-positive width/height,
               negative or non-finite divider, overflowing coordinate
               endpoints, insufficient width, or boundaries that cannot
               advance representably at the content offset/scale:
               `UnrepresentableProjection`), preserves the requested
               divider width, clamps the primary pane continuously between
               a documented minimum (1 pt, compatible with the 2 pt Design
               §11.7 divider) and the available remainder, keeps ordered
               adjacency with representably advancing boundaries, and lands
               the final endpoint on the validated content
               endpoint (to f32 rounding); a zero divider width may keep zero extent while
               the panes stay ordered. The invented equal-thirds compact
               policy and the f64 Rect are removed; obsolete
               SplitState/Snapshot types were deleted, not wrapped.
               #![forbid(unsafe_code)]; no GPUI/AppKit/WebKit/testkit/
               persistence/network/async dependency. No commit made by the
               writer per lane constraints.
Anchor:        29c34e68f4fad5f5338f7b69e50c44dd84145e74 (serial split shared
               contract); lane branch parallel/split-core created at clean
               HEAD e95d69b15474648fa3a9c949609ebd6a510e6d9a
               (docs(split): define the portable core lane).
Writer:        exclusive writer session completed; working tree left
               uncommitted for the independent read-only review.
Review:        independent reviewer completed; remediation findings are
               listed in the task label and all were addressed by the
               remediation pass below.
Remediation:   stateless authority model (SplitState deleted; validate_open/
               evaluate_focus/close_outcome/evaluate_resize replace it),
               bbr_web::WebRect f32 instead of the f64 Rect
               (+ bbr-web workspace dependency, deterministic Cargo.lock
               edge), typed geometry errors incl. non-positive dimensions,
               negative divider, overflowing endpoints, and insufficient
               width (equal-thirds compact policy removed, divider width
               preserved, continuous clamping), explicit FocusOutcome::
               Unchanged vs Changed, typed SplitDirection::Horizontal on
               open intent/plan, and tests for negative/zero dimensions,
               negative divider, overflow endpoints, insufficient compact
               width, exact boundary, continuous near-boundary behavior,
               realistic large representable acceptance, f32 no-op equality,
               focus unchanged, stateless/no-duplicate-state purity, and
               direction; crate docs and this packet updated to the
               remediated dependency direction and authority model.
Final        evaluate_focus now rejects an outside caller-supplied `current`
remediation: (FocusOutcome::InvalidCurrent) before any Unchanged result, so an
               outside value can never be reported as the focused pane
               (regression covers outside current/request plus preserved
               member behavior); geometry now rejects accepted projections
               whose ordered f32 boundaries cannot advance representably
               (typed GeometryError::UnrepresentableProjection), with
               primary/divider/secondary ends that must each advance,
               adjacency that holds, and a far edge that must land on the
               validated content endpoint to f32 rounding; a zero divider
               width may keep zero extent while the panes stay ordered;
               regressions cover a huge x/width or a tiny divider swallowed
               by the local ulp, and the misleading extreme 1e30 acceptance
               was replaced by a realistic large representable acceptance
               (1,000,000 pt offset/width, 2 pt divider). The
               pre-remediation ledger miscounted the suites as 16 geometry /
               20 plan; the true baseline was 17 geometry / 19 plan, and the
               final counts
               after these additions are below.
Functional evidence: 41 unit/contract tests + 1 lib doctest green
               (cargo test -p bbr-split: 21 geometry tests covering exact
               default/clamped-low/clamped-high projection, exact acceptance
               boundary at 2*MIN + divider, continuous near-boundary
               behavior, insufficient compact width rejection, zero/negative
               dimension rejection, negative-divider rejection, non-finite
               position/content/divider rejection, overflowing-endpoint
               rejection, realistic large representable acceptance,
               fractional-layout acceptance at every position, huge
               offset/width and tiny-divider unrepresentable-boundary
               rejection, zero-divider ordering, position bound clamping,
               f32 clamped no-op equality, property table over 4 widths x
               5 positions x 4 divider widths asserting minimum positive
               panes, preserved divider width, ordered adjacency with
               representably advancing boundaries, a final endpoint on the
               validated content endpoint (to f32 rounding), total-width
               conservation to f32 rounding, determinism, and exact Debug
               strings; 20 plan tests covering typed plan output with
               SplitDirection::Horizontal, pre-split focus retention on
               primary/secondary, identical-pane rejection, outside-focus
               rejection at open and focus, outside-current rejection before
               Unchanged (InvalidCurrent), member focus with explicit
               Changed/Unchanged and no reordering, stateless purity of
               focus and resize evaluation, close survivor + both
               identities, low/high clamp, unchanged and clamped-to-current
               no-ops, non-finite resize rejection, genericity over a custom
               identity type, and exact Debug strings). The counts were
               corrected from the previously recorded 16 geometry / 20 plan
               to the true pre-remediation baseline 17 geometry / 19 plan,
               then updated to the final 21 geometry / 20 plan (41 total + 1
               lib doctest) after the final remediation additions. Gates
               green: cargo fmt --all -- --check, cargo test -p bbr-split,
               cargo clippy -p bbr-split --all-targets -- -D warnings,
               cargo check --workspace --all-targets, just roadmap, git
               diff --check.
Real-app E2E:  not applicable until app composition.
Performance:   portable-only; no microcheck run; the production scenario
               stays with the follow-up composition packet.
Commits:       none (writer leaves a reviewable working-tree diff; no
               commit/push/reset/clean/stash/merge/rebase/amend performed).
Open blockers: none at handoff.
```
