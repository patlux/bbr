# Task packet — command-bar app composition

Parents: [Phase 5](../phases/05-command-bar.md) · [portable command core](command-core.md) · [browser interaction contract](../../product/browser-interactions.md) · [Design §11.5](../../../DESIGN.md) · [performance contract](../../product/performance-contract.md).

## Outcome

Deliver a keyboard-complete real-app command bar that consumes the integrated `bbr-command` core: `Cmd-L` selects the active URL, typing replaces the selection, local results render deterministically, arrows change the selected result, Enter and Cmd-Enter activate the intended destination, Escape restores the originating focus domain, and cancellable nonblocking ranking exposes truthful loading/empty/no-match/error/destructive states. Real-app E2E and optimized semantic-presentation scenarios prove the production path.

## Scope

- Add the serial composition dependency `bbr-app -> bbr-command`.
- Add an app-owned command text-edit model: UTF-8-safe cursor/selection state, full selection on `Cmd-L`, replace-selection typing, backspace, and horizontal cursor movement.
- Map app modes without duplicating the portable core:
  - `Location -> bbr_command::Mode::Location`
  - `NewTab -> Mode::NewTab`
  - existing `NewSpace` / `MoveTab` retain their current staged behavior and do not invoke the result corpus
  - no dedicated Switcher/Action entry shortcut in this slice.
- Build the provider-neutral local corpus from `ControllerSnapshot` on demand, preserving tab/space/profile identity and deterministic ordering. History stays empty until Phase-7 composition supplies it.
- Rank through `bbr_command::rank` behind a cancellable, nonblocking debounce. A generation/token invalidates stale results; ranking never runs from the GPUI render method.
- Render the Design §11.5 result palette and rows using existing tokens/components. Expose deterministic selected result and typed states: inactive, empty, loading, results, no-match, activation-error. Mark destructive action results with the existing error/destructive visual language; no confirmation dialog is invented.
- Keyboard behavior:
  - Up/Down select results without moving focus out of the field.
  - Enter activates the selected result or the typed navigate/search fallback.
  - Cmd-Enter opens a Location/NewTab destination in a new tab; Shift/Option modifiers are non-goals.
  - Escape closes the result surface and restores exact origin mode/text plus chrome/web focus domain.
- Route activations through existing app/controller methods: select tab/space, run existing registry actions, navigate, search, or staged new-tab open. Profile isolation and warm-session rules remain controller-owned.
- Additive typed testkit snapshot/request support only as required for deterministic assertions and debounce flushing; protocol stays version 6 unless a non-additive wire change is proven necessary.
- Add real-app `e2e_shell_command.rs`, an `e2e-shell-command` recipe, and optimized `command_type` / `command_activate` scenarios with semantic presentation commits and fast-but-wrong guards.
- Update Phase 5, architecture/E2E facts, performance baseline/matrix, and this ledger only after evidence passes.

## Settled product values

- Search fallback URL: `https://duckduckgo.com/?q=<percent-encoded query>`; E2E uses deterministic intent/fixture assertions and never depends on the live provider.
- Cmd-Enter: open the selected/typed destination in a new tab and focus that page.
- Result limit: 16.
- Destructive actions: typed/destructive tint only in v1; no confirmation dialog.
- History corpus: empty in this slice; no fake history provider.

## Non-goals

- Dedicated command/action palette shortcut or inferred `>` mode.
- Persistence/history records, network suggestions, fuzzy-search dependency, IME composition, mouse result activation, virtualization, destructive confirmation, or settings UI.
- Changes to `bbr-command`, `bbr-domain`, `bbr-browser`, `bbr-web`, `bbr-web-macos`, `bbr-persistence`, or `bbr-split` public behavior.
- Root dependency upgrades, GPUI migration, `gpui-component`, or compatibility layers.

## Parallel-lane contract

This packet first lands the minimal serial dependency/ownership anchor on clean `main`, then the exclusive writer works in a sibling worktree from that exact commit.

```text
Serial base:      3746074f1f3d5a08e59a6963586019b4d2c00308
                  (split core integration recorded; icon system integrated)
Branch/worktree:  parallel/command-composition
                  /Users/patwoz/dev/Personal/macos/bbr-command-composition
Depends on:       integrated command core b7c475a, persistence core 3a3c7f1,
                  split shared contract 29c34e6, icon system 2e10037,
                  portable split core 9eeb6b2
Provides/API:     production command composition only; no new portable public
                  contract
Exclusive paths:  Cargo.toml workspace dependency entry
                  Cargo.lock deterministic bbr-app dependency edge
                  crates/bbr-app/Cargo.toml
                  crates/bbr-app/src/app.rs
                  crates/bbr-app/src/app/interaction.rs
                  crates/bbr-app/src/app/sidebar_view.rs
                  crates/bbr-app/src/app/render.rs
                  crates/bbr-app/src/command_bar.rs
                  new focused crates/bbr-app/src/app/command*.rs modules
                  crates/bbr-app/src/app/testkit/** command snapshot mapping
                  crates/bbr-app/src/shell/snapshot.rs
                  crates/bbr-app/src/testkit/dispatch*.rs
                  crates/bbr-app/src/performance*.rs only for command
                  presentation probes
                  crates/bbr-app/tests/e2e_shell_command.rs
                  crates/bbr-app/tests/perf_common/** command scenarios only
                  crates/bbr-app/tests/e2e_performance*.rs command registration
                  crates/bbr-app/tests/performance-baseline.json command seeds
                  crates/bbr-testkit/src/protocol/{snapshot,request,benchmark,mod}.rs
                  justfile command recipe/order
                  docs/roadmap/tasks/command-composition.md ledger
Shared/serial:     Cargo.lock if deterministic; ARCHITECTURE.md,
                  E2E_ARCHITECTURE.md, ROADMAP.md, Phase-5 status are updated
                  by the integration owner after lane review/gates
Forbidden:        every portable crate source; split/persistence schemas;
                  unrelated app/sidebar/dense/icon refactors
Integration order: command composition rebases/merges only after split/icon/
                  split-core anchors; run affected GUI/perf then canonical
                  just e2e serially on main
```

## Constraints

- Exactly one writer owns the sibling worktree; workflows do not commit or integrate.
- Native WKWebViews remain real child views; result chrome may overlay glass only as Design §11.5 permits.
- Focus origin is one explicit transient captured at activation, not inferred after mutation.
- No synchronous database scan or ranking in `Render::render`.
- Stale debounce generations never publish or satisfy a benchmark.
- Activation always routes through controller/app seams; no direct engine/session mutation.
- Profile identity and cross-profile recreation rules remain unchanged.
- Every user-visible interaction has real-app E2E and optimized performance evidence in this implementation group.
- Files remain below ~500 lines; split command concerns into focused modules.

## Evidence

### Unit and contract

- [ ] Text model covers select-all, UTF-8 cursor movement, replace selection, backspace, and empty boundaries.
- [ ] Corpus mapping preserves ids/order/pinned/profile/space labels and uses an empty real history collection.
- [ ] Result state machine proves empty/loading/results/no-match/error and stale-generation cancellation.
- [ ] Activation mapping proves every supported `Activation` and rejects unavailable history/action destinations truthfully.
- [ ] Origin focus restoration is deterministic for web and chrome invocation.

### Real-app E2E

- [ ] `Cmd-L` selects the current URL; typing replaces it; Escape restores the web origin.
- [ ] Deterministic local result keys/groups, Up/Down selection, Enter tab/space/action/navigation activation.
- [ ] Cmd-Enter creates exactly one new tab/session and focuses it.
- [ ] Burst typing publishes only the final generation after deterministic debounce flush.
- [ ] Empty/no-match/error/destructive states expose stable ids and typed snapshots.
- [ ] Cross-space selection preserves warm sessions and profile isolation.

### Performance

- [ ] `command_type`: keystroke/debounce/rank to correct first result presented; 50 ms p95 / 100 ms max absolute warm budget.
- [ ] `command_activate`: Enter to correct destination/focus/mode presented; 50 ms p95 / 100 ms max for warm activation, with session-recreation class used where appropriate.
- [ ] Fast-but-wrong guards reject incorrect first key, selection, mode, focus, destination, or stale generation.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-command
just e2e-perf
just e2e
just roadmap
git diff --check
```

At Phase-5 closeout or when canonical repeated evidence is required, run `just e2e-perf-full` on the qualified canonical runner. This host's report-only performance result cannot establish the canonical baseline.

## Delivery sequence

1. Commit this serial packet/ownership anchor on clean `main`.
2. Create `parallel/command-composition` sibling worktree from the exact anchor.
3. One exclusive writer implements the smallest end-to-end slice, with targeted tests while iterating.
4. One independent read-only reviewer; one remediation writer only for concrete findings; focused recheck.
5. Owner runs lower-level, affected GUI, optimized performance, and canonical sequential gates; commits the lane.
6. Serially integrate on clean `main`, update shared architecture/roadmap facts, and rerun affected verification.

## Iteration policy

Start with the text model and Cmd-L selection in the real app, then corpus/ranking and palette, then activation, then debounce/state/performance hardening. Keep the app working end-to-end after each layer. For every failure, identify text-state, ranking, focus, controller activation, paint synchronization, or benchmark-guard cause before editing; never weaken deadlines, budgets, or assertions.

## Blocked stop condition

Preserve the branch/worktree with evidence if GPUI 0.2.2 cannot express a keyboard-accessible result overlay without a fork, if origin focus cannot be restored without duplicating native focus ownership, if a semantic performance commit cannot distinguish correct from fast-but-wrong output, or if an unresolved product decision outside the settled values above is required. Report the exact API/product gap and smallest needed input; do not add a fallback surface or guess.

## Delivery ledger

```text
Outcome:       pending
Serial anchor: pending
Writer:        pending
Review:        pending
Functional evidence: pending
Real-app E2E:  pending
Performance:   pending
Commits:       pending
Open blockers: none at packet creation
```
