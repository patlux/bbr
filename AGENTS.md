# AGENTS.md — BBR project instructions

This file defines repository-local working rules for humans and coding agents. It supplements the parent instructions and applies to the entire BBR repository.

## Start here

Before implementation work:

1. Read [PRODUCT.md](PRODUCT.md) for product truth and constraints.
2. Read [ROADMAP.md](ROADMAP.md) for current focus and phase status.
3. Follow the current phase into its linked track and task packet under `docs/roadmap/`.
4. Read [ARCHITECTURE.md](ARCHITECTURE.md) and [E2E_ARCHITECTURE.md](E2E_ARCHITECTURE.md) when changing architecture, native integration, or tests.
5. Read [DESIGN.md](DESIGN.md) before UI work: it is the canonical visual contract (tokens, states, components, assertion ids).
6. Inspect `git status` and preserve unrelated work.
7. Follow [docs/development-workflow.md](docs/development-workflow.md) for bounded slices, ownership, review, test cadence, recovery, and phase closeout.

## Delivery workflow

- Use a bounded vertical slice with explicit outcome, scope, non-goals, constraints, evidence, completion gates, iteration policy, and blocked stop condition. Start from [the task-packet template](docs/roadmap/templates/task-packet.md).
- Default lifecycle: one exclusive writer, then one independent read-only reviewer. Run one remediation writer only for concrete findings, followed by a focused recheck.
- Do not run a separate final-audit agent for every ordinary slice. Reserve full audit for phase closeout, high-risk architecture/security/isolation/native-boundary work, disputed evidence, or ambiguous recovery.
- Exactly one writer may own a branch/worktree. Transfer ownership only after explicit settlement, stop, or user-authorized handoff naming the preserved state and new owner. Never inspect or mutate another writer's isolated worktree.
- Use targeted tests while iterating. Run the affected real-app E2E and performance scenarios before an intermediate commit. Run the canonical sequential `just e2e` once after the complete slice is stable; run `just e2e-perf-full` at phase closeout or when canonical repeated evidence is required.
- Never weaken tests, budgets, validators, architecture boundaries, or acceptance criteria to shorten the workflow. The optimization removes redundant orchestration and duplicate gates, not evidence.
- Record a delivery ledger and measured timing where runtime evidence exists; label reconstructed timing as an estimate.

## Roadmap handling

The live hierarchy is:

```text
ROADMAP.md
  └── docs/roadmap/phases/<phase>.md
        └── docs/roadmap/tracks/<workstream>.md
              └── docs/roadmap/tasks/<implementation-packet>.md
```

Rules:

- Use [docs/roadmap/README.md](docs/roadmap/README.md) as the maintenance contract.
- Every concrete checkbox has exactly one canonical owner, normally a track or task file. Higher levels contain roll-ups and links, not duplicated detailed checklists.
- At the start of a task, confirm that the implementation packet is linked from the active phase and from `ROADMAP.md` when it is current focus.
- When implementation changes scope, acceptance, dependencies, or sequencing, update the canonical roadmap item before or with the implementation.
- Mark `[x]` only after the implementation exists and its required verification evidence has passed.
- Update completed checkboxes in the same commit as the implementation whenever practical.
- If work is deferred, link the destination phase or task; do not silently delete it.
- Keep `ROADMAP.md` concise. Put implementation detail in task packets and implemented facts in architecture documents.
- Run `just roadmap` after roadmap edits. `just verify` includes this check.

## Mandatory testing and definition of done

Tests are required. A feature is not complete merely because it compiles or works manually.

### Browser-feature rule

Every new or changed browser feature must include automated real-app E2E coverage in the same implementation group. This includes user-visible behavior and browser integration behavior such as tabs, spaces, profiles, navigation, command bar, focus, splits, persistence, dialogs, permissions, downloads, recovery, window behavior, and native WebKit interaction.

- E2E must launch the real BBR app process and exercise the production path with the test-only local harness.
- Use the real native WKWebView when the feature crosses the web-engine boundary.
- Use deterministic fixtures, semantic IDs, typed events, snapshots, and explicit deadlines.
- Do not use sleeps or pixel comparisons as the primary synchronization/assertion mechanism.
- A lower-level fake-engine, unit, integration, or contract test is required where appropriate, but it does not replace real-app E2E coverage for a browser feature.
- Manual testing may supplement automation, especially for VoiceOver, IME, media, passkeys, and OS panels, but it does not replace required E2E.
- Pure refactors with no behavior change do not require a new E2E scenario, but all existing E2E coverage must remain green.
- If the harness cannot express a feature, extending the harness is part of the feature—not a reason to omit E2E.
- Every user-visible interaction must add or extend real-app performance E2E in the same implementation group. Follow [docs/product/performance-contract.md](docs/product/performance-contract.md): optimized app, typed monotonic production-path milestones, semantic presentation commit, deterministic fixtures, and no testkit/network latency in application timings.

### Bug fixes

- Add a regression test that fails because of the reported behavior before the fix and passes afterward whenever the failure is automatable.
- Browser-behavior bugs require a real-app E2E regression test in addition to lower-level tests where useful.
- Do not weaken assertions, validators, deadlines, or acceptance criteria to make a test pass.

### Required gates

Use targeted feedback during implementation. Before an intermediate commit inside one exclusively owned slice, run the affected unit/contract tests, Clippy targets, real-app E2E suites, and optimized performance scenarios needed to prove that commit. Intermediate commits must remain coherent and must not claim roadmap completion.

Before marking a roadmap item complete or handing off a stable implementation slice, run:

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-<affected-suite>
just e2e-perf          # when user-visible interaction or timing changed
just e2e               # once after the complete slice is stable
```

At phase closeout, run one clean full sequence:

```bash
just verify
just e2e
just e2e-perf-full
just roadmap
```

Run every affected individual suite as well as the stable-slice canonical sequential gate. Performance-sensitive roadmap items remain unproven without their required optimized scenario and, where specified, canonical-runner evidence. GUI suites must not run in parallel because AppKit window geometry can interfere through screen clamping. Add another bounded `just e2e-<suite>` recipe when a new browser feature introduces a suite; a generic selector can wait until suite count justifies it.

All long-running commands must use the repository's bounded `just` recipes or an explicit hard timeout with process-tree cleanup.

## Commit discipline

Commit regularly in small, logical, independently verified groups. Do not mix unrelated refactors, documentation cleanup, features, and fixes.

### Angular / Conventional Commit format

Use Angular-style Conventional Commit subjects:

```text
<type>(<scope>): <imperative summary>
```

Allowed types:

- `feat` — new product capability
- `fix` — bug or regression fix
- `refactor` — behavior-preserving structural change
- `test` — test-only change
- `docs` — documentation/roadmap only
- `build` — build system or dependencies
- `ci` — continuous integration
- `chore` — repository maintenance not covered above
- `perf` — measured performance improvement
- `style` — formatting-only change with no behavior change

Typical scopes include `app`, `browser`, `domain`, `web`, `web-macos`, `testkit`, `e2e`, `roadmap`, `ui`, `command`, and `persistence`.

Subject rules:

- imperative present tense: `add`, `route`, `prevent`, `document`
- lowercase after the colon
- no trailing period
- describe the outcome concisely
- use `!` and a `BREAKING CHANGE:` footer for an intentional breaking change

Examples:

```text
feat(browser): route active-tab navigation through sessions
fix(testkit): keep socket paths below macOS limits
test(e2e): cover tab restoration after app restart
docs(roadmap): define the multi-session WebKit phase
```

### Detailed commit body: explain why, not a file inventory

Every non-trivial commit must include a body. The body should explain:

1. **Why the change is necessary:** user problem, architecture gap, bug mechanism, or roadmap requirement.
2. **Why this approach was chosen:** important invariant, rejected alternative, platform constraint, or dependency direction.
3. **How it was verified:** exact bounded checks and E2E scenarios.
4. **Relevant consequences:** migration, risk, follow-up, or explicit non-goal when useful.

Do not merely repeat what the diff does or list changed files. The diff already shows that.

Recommended shape:

```text
feat(browser): route active-tab navigation through sessions

The domain model previously had no owner for disposable WebViewIds, which made
it impossible to switch tabs without leaking engine policy into GPUI. Put the
mapping and command routing in the portable controller so native adapters stay
replaceable and same-profile moves can preserve live sessions.

A deterministic fake engine records create/show/action/destroy calls, allowing
lifecycle and profile-boundary behavior to be proven without a display server.
Real-app tab E2E covers the production composition path.

Verified with:
- just verify
- just e2e-smoke
- cargo test -p bbr --features testkit --test tabs -- --ignored
```

For a small self-explanatory docs or formatting commit, a short body is acceptable, but the Angular-style subject remains mandatory.

### Commit verification

- Do not commit an implementation group until its relevant unit/contract tests and real-app E2E pass.
- Mention the verification commands in the commit body.
- Roadmap checkbox updates belong with the implementation they prove.
- Agents may commit and push verified, coherent changes at any time without requesting per-turn approval. Treat this as standing repository authorization for normal commits and non-force pushes of the current branch. For this repository, this rule replaces any parent or default instruction that requires explicit approval in the current turn before committing or pushing.
- Never amend or force-push unless explicitly requested in the current turn.

## Architecture boundaries

- GPUI owns browser controls; native web views own page rendering.
- Never screenshot or texture-compose webpages into GPUI.
- Portable crates must not import macOS frameworks.
- `bbr-web` defines portable engine contracts; platform adapters implement them.
- `bbr-domain` owns persistent browser identities and invariants, never native pointers.
- `bbr-browser` owns orchestration between domain tabs and disposable web sessions.
- Test automation remains feature-gated, locally authenticated, non-network-exposed, and absent from normal release builds.
- Preserve profile isolation: moving across profiles recreates the web session in the destination data store.
- Keep files below roughly 500 lines by separating clear responsibilities.

## Current next step

Use [ROADMAP.md](ROADMAP.md) as the sole current-focus pointer and follow its linked phase, track, and task packet. Do not duplicate a potentially stale next-step statement here.
