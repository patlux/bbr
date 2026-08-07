# Task packet — portable chrome-first hydration policy

Parents: [Phase 7](../phases/07-persistence.md) · [Portable persistence core](persistence-core.md) ·
[Performance contract](../../product/performance-contract.md) ·
[Performance E2E foundation](performance-e2e-foundation.md) ·
[Parallel-delivery contract](../../parallel-delivery.md)

Lane: **L3** of the 2026-08-07 browser-MVP parallel plan
([delivery ledger](../../../docs/delivery/2026-08-07-browser-mvp-parallel-plan.md)).

## Outcome

A portable chrome-first restore/hydration policy in the controller layer
(`bbr-browser`): given a persisted browser state, it restores browser
identities first and plans sessions **only** for the visible/active set — never
mass-starting WKWebViews. The policy is deterministic, contract-tested against
the fake engine, and ships with a truthful warm-startup performance scenario
specification/preparation that claims no app evidence it does not have.

This prepares, at the portable layer, two Phase-7 acceptance items:
"Restore browser chrome first and hydrate only visible sessions" and
"Startup does not instantiate every persisted WKWebView". The checkboxes stay
owned by the serial app-composition slice, which alone can run real-app
evidence.

## Scope

- `crates/bbr-browser/src/hydration/`: the portable policy module
  (`mod.rs` public contract types + `policy.rs` entry points/`RestoreTarget`
  seam + `validate.rs` planning internals) —
  [`RestoreIntent`] input DTOs (persistence-snapshot-shaped), typed
  [`HydrationError`] validation, the deterministic two-phase
  [`RestorePlan`] (identities first; sessions = active space's visible set;
  everything else deferred), the [`RestoreTarget`] seam, and
  [`HydrationPolicy::plan`]/[`HydrationPolicy::apply`].
- `crates/bbr-browser/src/hydration_tests/`: contract tests (`support.rs`
  fixtures/builders, `planning.rs`, `validation.rs`, `apply.rs`) proving the
  phase separation, the visible-set session rule, no eager session explosion
  (500 identities → exactly one session), determinism, cross-profile store
  selection, split hydration, identity preservation (pinned/order/hues/
  recently-closed), and every typed validation rejection.
- Additive `mod`/re-export lines only in `crates/bbr-browser/src/lib.rs`.
- `docs/roadmap/tasks/persistence-hydration.md`: this packet, including the
  warm-startup performance scenario specification/preparation.

## Non-goals

- No domain, controller-facade, app-composition, testkit-wire, performance-
  registry, manifest, lock, `ROADMAP.md`, or Phase-7 edits (serial surfaces).
- No id-preserving domain import API: the domain `Browser` allocates ids
  internally and exposes no id-injecting restore path, so the plan cannot yet
  be executed against the real domain. This is the documented serial
  integration need below, not a defect of this lane.
- No real-app E2E, no measured performance evidence, no repository or app
  startup wiring. Warm-startup acceptance remains the serial composition
  slice's; a test-only seed that bypasses the future persistence path is not
  acceptance evidence (E2E_ARCHITECTURE §10).

## Parallel-lane contract

```text
Base commit: 574750bd129df628a0f74ff95e48579f4c12e0ef
Depends on: persistence-core repository/DTO contract (integrated 3a3c7f1);
  performance contract; performance E2E foundation
Provides/API: RestoreIntent/Intent* input DTOs, RestorePlan (identities/
  sessions/deferred phases), RestoreTarget seam, HydrationPolicy::plan/apply/
  default_active_space, typed HydrationError validation
Exclusive paths: crates/bbr-browser/src/hydration/;
  crates/bbr-browser/src/hydration_tests/; additive mod/re-export lines in
  crates/bbr-browser/src/lib.rs; this task packet
Shared integration points: domain id-preserving import API; public controller
  restore facade; app startup composition; testkit wire; performance
  registry; ROADMAP.md / Phase 7 roll-ups
Integration order: land the portable policy first (this lane), then the serial
  domain import API + controller facade + app composition consuming it, then
  the warm-startup scenario wiring in the performance gate
```

## Constraints

- No new dependencies; `bbr-browser` keeps depending only on `bbr-domain`,
  `bbr-web`, and std (no SQLite, no GPUI, no frameworks, `#![forbid(unsafe_code)]`).
- The policy is fully deterministic: output order derives only from intent
  order; resolution maps are never iterated for output.
- Validation mirrors the persistence identity contract (global uniqueness
  across profiles/spaces/tabs/recently-closed, pinned prefix, member active
  tab, distinct-member split, bounded URL rule) and rejects with typed errors
  before any plan.
- The two phases are structurally separate: no session request can precede or
  exceed identity restoration, and no session request targets a tab outside
  the active space's visible set.
- Do not weaken tests, budgets, validators, or acceptance criteria.

## Evidence

### Unit and contract

- [x] `cargo test -p bbr-browser`: **97 passed** (75 pre-existing + 22 new
      hydration contract tests). `cargo clippy -p bbr-browser --all-targets
      -- -D warnings` green; `cargo fmt --all -- --check` green;
      `cargo check --workspace --all-targets` green; `cargo test --workspace`
      green.
- [x] Chrome-first phase separation: the combined `apply` op log proves every
      identity op precedes the session ops; the plan's `identities`/
      `sessions`/`deferred` phases are structurally separate.
- [x] Visible-set rule: sessions = the active space's selected tab (single
      pane) or both split panes in primary/secondary order; splits and active
      tabs of non-active spaces hydrate nothing; an empty active space
      hydrates nothing.
- [x] No eager session explosion: a dense 500-tab intent across two profiles
      (every space with an active tab and a pinned prefix, one non-active
      split space, recently-closed entries) plans and executes exactly
      **one** engine create + show (never 500), with 499 identities
      deferred; a split active space creates exactly two.
- [x] Apply short-circuit: `apply` rejects an invalid intent **before**
      executing a single target op (plan-before-execute, no partial
      mutation on a corrupt intent).
- [x] Apply stop-on-error contract (pinned for the serial facade): `apply`
      is non-transactional — a mid-apply target error propagates with every
      earlier identity op already applied and no rollback attempted, so the
      serial controller facade must define its rollback/idempotency
      contract.
- [x] Split/active-tab cross-constraint: when a snapshot carries a split
      plus an active tab outside its panes, the policy hydrates the panes
      (split precedence matches domain `Browser::is_visible`) and restores
      both split and outside active tab as identities; the domain dissolves
      a split only when a non-member tab is selected later.
- [x] Determinism: the same intent plans identically; identity order is
      intent order (never map-iteration order).
- [x] Identity preservation: profile hues, pinned flags, tab order/urls,
      active tabs, splits, and recently-closed entries survive into the plan;
      recently-closed entries restore as identities, never sessions.
- [x] Cross-profile store selection: a session request carries its tab's
      owning `ProfileId`.
- [x] Typed validation: unknown active space, duplicate identities (all
      kinds), non-pinned prefix, non-member active tab, invalid split,
      recently-closed collisions, invalid URLs, and propagating target errors
      all reject with the exact typed error.
- [x] `default_active_space` deterministically prefers the first space with
      an active tab (else the first space, else `None`).

### Real-app E2E

- [ ] Not applicable to this portable-core lane: it creates no app startup
      behavior and launches no WKWebView. The serial composition slice owns
      the real-app force-quit/chrome-first-restore E2E.

### Performance

- [ ] No measured app performance is claimed in this lane. The warm-startup
      performance scenario is **specified and prepared** below; its gate
      wiring and canonical measurements remain serial.

## Warm-startup performance scenario specification/preparation

This section is the truthful scenario contract the serial composition slice
implements. It claims nothing measured; every number below is a requirement
from the product contract, and the only evidence this lane contributes is the
portable session-bound proof.

### Scenario

`persisted_warm_startup` (closes the open "Warm startup" item in
[performance-e2e-foundation.md](performance-e2e-foundation.md)): launch the
real optimized BBR app with a **persisted profile directory** whose browser
model was written through the actual `bbr-persistence` repository
(`replace_snapshot`), with a warmed filesystem cache, and measure the startup
milestones of the performance contract against the dense persisted model.

### Dense persisted fixture (prepared shape)

- The structural snapshot the repository restores maps mechanically into
  this lane's `RestoreIntent` — the exact shape
  `hydration_tests::support::dense_500`
  builds: 500 tab identities across two profiles, three spaces, per-space
  active tabs, a pinned prefix in every space, one (non-active) split space,
  and one recently-closed entry per space.
- The composition selects the active space via
  `HydrationPolicy::default_active_space`. This is a policy default, not
  persisted truth: the `Snapshot` persists each space's `active_tab` plus
  window geometry/sidebar only — there is **no window-level active-space
  selection**, so the pre-quit showing space is unrecoverable. The warm-
  startup E2E must therefore encode "first space with an active tab", never
  "the space that was showing".
- The fixture pins the active space to space 10 (single selected tab 100,
  no split), so the expected session count is unambiguous for the serial
  implementer.
- The selected tab's URL is a deterministic loopback fixture
  (`bbr-fixtures`), so the page-ready milestone is network-independent.
- The prepared policy output for this shape is deterministic and proven:
  `sessions.len() == 1` (the active space is single-pane by fixture pinning;
  2 only when the active space itself carries the split, proven by the
  split-panes plan/apply tests), `deferred.len() == 499`, never 500.

### Milestones (performance contract startup contract)

1. process spawned;
2. domain/window state restored or initialized;
3. sidebar/chrome laid out;
4. selected tab's WKWebView created or restored, attached, visible, and given
   a non-zero final content rectangle;
5. first usable presentation committed (milestones 3 and 4 must refer to the
   same selected tab and window state);
6. deterministic fixture content ready.

### Required semantic assertions (reject fast-but-wrong)

Before a sample is accepted the snapshot must show: `engine_sessions ==`
the visible-set size — exactly **1** for this fixture's pinned single-pane
active space (2 only for a split active space) — **never** 500 or
any per-space count; the selected tab's `WebViewId` presented with a non-zero
`content_rect`; the sidebar state; the window geometry; and, at the
page-ready milestone, the fixture title on the expected view. Milestone 5's
"same selected tab and window state" refers to space 10's tab 100 (the
active-space determination described above), not to the unrecoverable
pre-quit showing space.

### Probe and gate requirements

- Typed benchmark operation id + monotonic in-process timestamps from process
  spawn; presentation-commit at the existing `NativeWebViewSlot` paint path;
  testkit transport excluded from application timings.
- Budgets from the product contract: first usable frame ≤ 250 ms target /
  p95 ≤ 500 ms / max ≤ 750 ms; warm page-ready p50 ≤ 500 ms / p95 ≤ 1,000 ms;
  ≥ 10 warm samples; discarded warm-ups; raw samples + p50/p95/max + machine
  metadata artifacts; absolute ceilings enforced everywhere, the >15% p95
  relative rule on a named canonical machine.
- Register the scenario in `just e2e-perf`/`just e2e-perf-full` and the full
  gate only in the serial composition slice (performance registry is a shared
  integration surface).

### Explicitly NOT faked

- No test-only seed that bypasses the repository restore path is acceptance
  evidence; the fixture is written and read through the real repository.
- No fake-engine timing, no synthetic session counts, no debug-build numbers.
- This lane reports the portable plan-level session bound only; any measured
  startup latency claim belongs to the composition slice on the canonical
  runner.

### Exact serial integration needs

1. **Domain id-preserving import API** (`bbr-domain`, serial): `Browser`
   allocates ids internally (`next_id`) and its fields are `pub(crate)`; no
   path exists to restore a model with persisted ids. The composition needs a
   domain restore entry that accepts ids (or an equivalent contract) — the
   `Restored*` types in this lane define the exact shape it consumes.
2. **Controller restore facade** (`bbr-browser` public controller facade,
   serial): implement `RestoreTarget` over `BrowserController` (identity
   restores into the domain, `hydrate_session` through the existing session
   path), and mark deferred tabs session-less so a later activation recreates
   exactly one session. The facade must also define the apply contract this
   lane pins by test: `apply` is **non-transactional** — a mid-apply target
   error leaves every earlier op applied (e.g. a failed second split pane
   leaves the first pane's session live) and the trait promises no
   rollback/idempotency; the facade decides retry, rollback, or recovery-UI
   semantics before wiring startup.
3. **App startup composition** (serial): repository restore → map the
   recovered snapshot into `RestoreIntent` → `HydrationPolicy::apply` in the
   window-open path, before/around the first usable frame.
4. **Testkit wire** (serial): a typed restore request/event and snapshot
   evidence (`engine_sessions`, deferred counts) for E2E.
5. **Performance registry** (serial): `persisted_warm_startup` scenario
   registration, gate wiring, canonical runner.
6. **Window restore** (serial, app layer): geometry/sidebar state is optional
   presentation state outside the policy's intent by design; the app
   composes it per the recovery matrix. Two contract edges are flagged here
   for the composition, not defects of this lane:
   - **The pre-quit active space is not persisted.** `Snapshot` persists
     per-space `active_tab` plus window geometry/sidebar only; there is no
     window-level active-space selection. Restore must use
     `HydrationPolicy::default_active_space` (first space with an active
     tab, else the first space, else none) and warm-startup E2E must assert
     that choice — never "the space that was showing".
   - **Split/active-tab cross-constraint.** A snapshot can carry a split
     plus an active tab outside its panes. The policy hydrates the panes
     (split precedence matches domain `Browser::is_visible`/`sync_lifecycles`)
     and restores both split and outside active tab as identities; the
     domain dissolves the split only when a non-member tab is selected
     later. Milestone-5 "same selected tab and window state" for that edge
     is defined by the composition's reconcile order.
7. **Structured error detail** (serial, info): `HydrationError::Target(String)`
   deliberately flattens target failures; the facade may surface the string
   or map it to typed controller errors. Id-range and `former_position`
   bound validation stays upstream in `bbr-persistence::validation` by
   design; this layer assumes a post-restore intent.

## Completion gates

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p bbr-browser
cargo test --workspace
cargo clippy -p bbr-browser --all-targets -- -D warnings
just roadmap
```

No GUI gate is required: the lane is portable and changes no browser surface.

## Delivery sequence

1. The orchestrator records the L3 base commit (`574750b`) and this packet.
2. Exactly one writer owns the L3 worktree and modifies only the exclusive
   paths.
3. Implement the portable policy + contract tests, then this packet.
4. One independent read-only reviewer maps the policy, tests, and scenario
   spec to the outcome.
5. Remediate concrete findings, leave changes uncommitted on
   `parallel/mvp-l3-hydration`, and hand the settled diff to the serial
   integration owner.

## Iteration policy

Start with the plan phases, then the visible-set session rule, then dense
explosion proof, then validation, then the executor + recording target. When a
shared contract is missing (domain import API, controller facade, testkit
wire), stop and name the serial need instead of editing the shared surface.

## Blocked stop condition

If the portable policy cannot be expressed without editing a shared surface
(domain DTOs, public controller facade, manifests/lock, persistence DTOs,
testkit wire, performance registry), stop with the preserved diff and the
exact serial contract needed. This lane did not need to stop: the policy is
fully expressible in `bbr-browser` with additive lib.rs lines, and the domain
import gap is documented as the next serial integration need.

## Delivery ledger

```text
Outcome: Portable chrome-first restore/hydration policy in the controller
  layer: RestoreIntent input, typed validation, deterministic two-phase
  RestorePlan (identities first, sessions = active space's visible set,
  deferred rest), RestoreTarget seam + apply executor, 22 contract tests
  proving no eager session explosion, the apply short-circuit and
  stop-on-error contracts, and split/active-tab precedence, plus a truthful
  warm-startup scenario specification/preparation.
Commits: none — changes left uncommitted on parallel/mvp-l3-hydration
  (base 574750b), per lane instructions.
Functional evidence: cargo test -p bbr-browser 97 passed (22 new);
  cargo clippy -p bbr-browser --all-targets -- -D warnings green;
  cargo fmt --all -- --check green; cargo check --workspace --all-targets
  green. cargo test --workspace: 46 suites green, 0 failures (the
  bbr-command benchmark_50ms timing test passed in this worktree run; it is
  a std-only crate with zero dependencies — unaffected by this diff).
Real-app E2E: not applicable; the serial composition slice owns
  force-quit/chrome-first-restore E2E.
Performance evidence: none claimed. persisted_warm_startup scenario
  specified/prepared in this packet; budgets quoted from the product
  contract; measured evidence is serial composition + canonical-runner work.
Roadmap/documentation updates: this packet only; ROADMAP.md and Phase 7
  remain serial integration-owned (their checkboxes are not touched).
Open blockers and non-goals: domain id-preserving import API, controller
  RestoreTarget facade, app startup composition, testkit wire, performance
  registry wiring, and real-app warm-startup measurement are serial; the
  policy makes no app-level claim.
Worktree state: parallel/mvp-l3-hydration at 574750b with only the three
  exclusive files modified, uncommitted; no other worktree touched.

Planning/read-only: estimate — ~40 min (docs + API reading).
Implementation: estimate — ~60 min.
Test execution: measured — bbr-browser 97 tests 0.83 s; workspace check +
  bbr-browser clippy green; workspace test 46 suites green, 0 failures
  (bbr-command benchmark_50ms passed this worktree run; std-only, unrelated
  to this diff).
Review: pending (independent read-only reviewer).
Remediation: pending (only for concrete findings).
Waiting/retries: none.
Total elapsed: estimate — ~2 h.
Human intervention: none.
```
