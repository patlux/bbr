# Task packet — command-bar app composition

Parents: [Phase 5](../phases/05-command-bar.md) · [portable command core](command-core.md) · [browser interaction contract](../../product/browser-interactions.md) · [Design §11.5](../../../DESIGN.md) · [performance contract](../../product/performance-contract.md).

## Outcome

Deliver a keyboard-complete real-app command bar that consumes the integrated `bbr-command` core: `Cmd-L` selects the active URL, typing replaces the selection, local results render deterministically, arrows change the selected result, Enter and Cmd-Enter activate the intended destination, Escape restores the originating focus domain, and cancellable nonblocking ranking exposes truthful loading/empty/results/error/destructive states. Real-app E2E and optimized semantic-presentation scenarios prove the production path.

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
- Render the Design §11.5 result palette and rows using existing tokens/components. Expose deterministic selected result and typed states: inactive, empty, loading, results, activation-error. Mark destructive action results with the existing error/destructive visual language; no confirmation dialog is invented. (There is no production no-match state in this slice: every non-empty Location/NewTab query carries the navigate/search fallback row, so the `NoMatch` enum/render branch was removed rather than claimed.)
- Keyboard behavior:
  - Up/Down select results without moving focus out of the field.
  - Enter activates the selected result or the typed navigate/search fallback.
  - Cmd-Enter opens a Location/NewTab destination in a new tab; Shift/Option modifiers are non-goals.
  - Escape closes the result surface and restores exact origin mode/text plus chrome/web focus domain.
- Route activations through existing app/controller methods: select tab/space, run existing registry actions, navigate, search, or staged new-tab open. Profile isolation and warm-session rules remain controller-owned. `SelectTab { tab, space }` routes by persistent identity: the corpus carries every space's tabs with their domain ids (`all_tabs`), the owning space is activated first (restoring its warm sessions), and the exact owning-space index is then selected; Cmd-Enter opens that tab's exact URL in a fresh tab.
- Reject unavailable destinations truthfully: Back/Forward are refused when the session's nav flags are false, Reload/Stop success is governed by the controller's navigate result, and a successful Close Tab is a deliberate terminal action (palette dismissed; the space's own empty/new-tab or chrome-domain aftermath).
- Additive typed testkit snapshot/request support only as required for deterministic assertions and debounce flushing; protocol stays version 6 unless a non-additive wire change is proven necessary. (The unreachable `NoMatch` result state was removed from the un-released protocol wire before integration; the wire gains the additive global tab identity snapshot.)
- Add real-app `e2e_shell_command/` (focused scenario modules behind the one `e2e-shell-command` gate), and optimized `command_type` / `command_activate` scenarios with semantic presentation commits and fast-but-wrong guards.
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
Serial base:      74dac78ac6922de33d42431b59179dbbde1bf63f
                  (complete tab identity snapshot; the command composition
                  anchor this lane bases its uncommitted slice on)
Branch/worktree:  parallel/command-composition
                  /Users/patwoz/dev/Personal/macos/bbr-command-composition
Depends on:       integrated command core b7c475a, persistence core 3a3c7f1,
                  split shared contract 29c34e6, icon system 2e10037,
                  portable split core 9eeb6b2, tab identity 74dac78
Provides/API:     production command composition only; no new portable public
                  contract
Exclusive paths:  Cargo.toml workspace dependency entry
                  Cargo.lock deterministic bbr-app dependency edge
                  crates/bbr-app/Cargo.toml
                  crates/bbr-app/src/app.rs
                  crates/bbr-app/src/app/interaction.rs
                  crates/bbr-app/src/app/sidebar_view.rs
                  crates/bbr-app/src/app/render.rs
                  crates/bbr-app/src/app/spaces.rs
                  crates/bbr-app/src/app/tab.rs
                  crates/bbr-app/src/command_bar.rs
                  new focused crates/bbr-app/src/app/command*.rs modules
                  crates/bbr-app/src/app/testkit/** command snapshot mapping
                  crates/bbr-app/src/shell/snapshot.rs
                  crates/bbr-app/src/shell/semantics/{mod,tests}.rs
                  crates/bbr-app/src/testkit/dispatch*.rs
                  crates/bbr-app/src/performance{,.rs}/** command presentation
                  probes (incl. the probe child modules split for size)
                  crates/bbr-app/tests/e2e_shell_command/**
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
- Every new/touched command module stays below ~500 lines; command
  additions are extracted into focused command modules (app-side
  command_*.rs, performance/command.rs, perf_common/command.rs and
  perf_common/command_drive.rs) so the pre-existing oversized files they
  were added into no longer grow with command content.

## Evidence

### Unit and contract

- [x] Text model covers select-all, UTF-8 cursor movement, replace selection, backspace, and empty boundaries.
- [x] Corpus maps the complete global tab collection (`all_tabs`) with persistent tab/space/profile ids and owning-space indexes; ranking exposes `OtherSpace` tab rows with never-hidden identity.
- [x] Result state machine proves empty/loading/results/activation-error and stale-generation cancellation; no unreachable no-match state is claimed.
- [x] Activation mapping proves every supported `Activation`, rejects unavailable history/action/back/forward/reload/stop destinations truthfully, routes cross-space `SelectTab` through the owning space, and makes successful Close Tab deliberately terminal.
- [x] Origin focus restoration is deterministic for web and chrome invocation, including after a repeated `Cmd-L`.
- [x] Fast-but-wrong guard tests: the presentation probe rejects a stale generation, wrong first/selected result key, or wrong typed state, and commits exactly once for the matching publication.

### Real-app E2E

- [x] `Cmd-L` selects the current URL; typing replaces it; Escape restores the web origin.
- [x] Repeated `Cmd-L` preserves the original return transient for Escape (web and chrome origins).
- [x] Deterministic local result keys/groups, Up/Down selection, Enter tab/space/action/navigation activation.
- [x] Cmd-Enter creates exactly one new tab/session and focuses it.
- [x] Burst typing publishes only the final generation after deterministic debounce flush.
- [x] Empty/loading/results/activation-error states expose stable ids and typed snapshots (no production no-match state exists).
- [x] Unavailable actions are rejected truthfully (Close Tab in an empty space; Back/Forward on false engine nav flags; Reload/Stop governed by the controller's navigate result — Err rejected, Ok terminal); successful Close Tab is terminal with neighbor selection and session destruction.
- [x] Cross-space selection preserves warm sessions and profile isolation — for standalone spaces and for warm tabs in other spaces/profiles, with exact persistent identity and WebViewId reuse.
- [x] Same-profile and cross-profile warm-tab activation evidence is available in the snapshot (`all_tabs` global tab identity).

### Performance

- [x] `command_type`: production text entry + the real ranking debounce (30 ms) + rank to the correct first result, committed only from a render of the published result surface (render-inclusive presentation-commit: generation/first/selected key/state/mode/focus enforced by the probe predicate; the ranking publication itself never emits the sample); 50 ms p95 / 100 ms max absolute warm budget.
- [x] `command_activate`: Enter to correct destination/focus/mode presented, exact expectations armed before dispatch; 50 ms p95 / 100 ms max for warm activation, with session-recreation class used where appropriate.
- [x] Fast-but-wrong guards reject incorrect first key, selection, state, generation, mode, focus, destination, or stale generation — unit-tested at the probe level and asserted by the drivers.
- [x] The deterministic `FlushCommandRanking` request remains for functional E2E only; the measured command-type scenario never flushes.

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
Outcome:       command bar composed into the real app; ranking surface rendered
               in the sidebar rail, Cmd-L/Enter/Cmd-Enter/Escape complete,
               typed result states + deterministic testkit flush, real-app
               E2E suite (focused scenario modules behind the one
               e2e-shell-command gate), and optimized command_type/
               command_activate probes measuring the real production path
Serial anchor: 74dac78 (complete tab identity snapshot; lane branch is
               parallel/command-composition at 74dac78 + uncommitted slice)
Writer:        continuation writer (exclusive lane owner; no commits per lane)
Review:        independent read-only reviewer — REMEDIABLE findings remediated:
               (1) command_type now measures production entry + real 30 ms
               debounce + rank and commits only from a subsequent RENDER of
               the result surface (render-inclusive presentation-commit:
               the ranking publication arms the identity but never emits;
               Render::render observes generation/first/selected key/state
               after constructing the palette; pre-render publication and
               wrong/old render identity guard unit tests added; the
               deterministic flush stays out of the measured scenario);
               (2) activation benchmarks arm exact expectations before
               dispatch and the production activation path suppresses its
               generic commit while a command benchmark is pending (driver
               validates caller-specific destination/session state first);
               (3) corpus now maps the complete global tab collection
               (all_tabs) with persistent ids and SelectTab routes by
               identity — owning space activated first, exact owning-space
               index selected, Cmd-Enter opens the exact URL; (4) repeated
               Cmd-L preserves the origin return transient; (5) Back/
               Forward rejected on false nav flags, Reload/Stop governed by
               controller.navigate, Close Tab success is deliberately
               terminal; (6) unreachable NoMatch state removed (app enum/
               render/wire) rather than claimed; (7) packet allowlist
               updated for spaces/tab/semantics/performance probe paths;
               command benchmarks split into command_benchmarks.rs; e2e
               suite split into scenario modules; sidebar_view trimmed to
               499; (8) remediation-run fixes: the production debounce now
               fires through GPUI's platform-driven timer
               (`background_executor().timer` — `smol::Timer` can never
               wake a foreground-spawned task, so the real debounce never
               published), and the perf driver derives the expected
               first-result keys from `all_tabs` (the reopen cycle
               reallocates persistent ids, so hard-coded `tab:0`/`tab:1`
               keys rejected every sample); truthful Back/Forward/Reload/
               Stop E2E added as `actions.rs`; (9) final remediation-writer
               round: SelectTab now routes by the tab's CURRENT all_tabs
               record (current owning space + index; the ranked space is
               never used, with an after-activation re-read verifying the
               exact persistent id + owning space) — real-app regression
               `e2e_shell_command/moved.rs` ranks a tab, moves it into
               another space before Enter, and proves the exact moved tab
               activates while the unrelated stale-index tab stays
               untouched; the command_type commit is render-inclusive (a
               bounded forced draw after the publication renders the
               palette and the probe commits at the presentation
               milestone; an omitted palette cannot pass); the debounce was
               defensibly lowered to 30 ms (nonblocking cancellable
               semantics unchanged) so the render-inclusive p95 keeps
               headroom; command-specific perf content extracted into
               focused perf_common/command.rs + command_drive.rs modules
               (benchmarks.rs back to its pre-command 783 lines; drive.rs
               retains only the minimal call seam on top of its
               pre-existing non-command content; both new modules <500)
Functional evidence: 98 bbr unit (incl. pre-render and wrong/old render
               identity guard probe tests) + 18 bbr-testkit tests green;
               clippy bbr testkit --all-targets -D warnings green;
               workspace clippy -D warnings green; roadmap + diff-check green
Real-app E2E:  just e2e-shell-command 11/11 green (cmd-L/typing, repeated
               Cmd-L, burst typing, states, activation, Cmd-Enter,
               terminal Close Tab, truthful Back/Forward/Reload/Stop
               outcomes, cross-space spaces + warm tabs, moved-tab
               current-space regression); affected suites re-run green
               (e2e-shell-organize, e2e-shell-spaces)
Performance:   just e2e-perf report-only, 0 regressions, measured on this
               host (real 30 ms debounce fires via the platform-driven GPUI
               timer; the command_type sample is render-inclusive — it
               commits only after the result palette was actually rendered):
               command_type     30 samples, p50 36.9 / p95 40.6 / max 43.4 ms
                               (production entry + real 30 ms debounce + rank
                               + rendered result presentation; budget 50/100)
               command_activate 30 samples, p95 0.21 ms (budget 50/100)
               command_activate_new_tab 10 samples, p95 20.0 ms
                               (session-recreation, budget 100/150)
               seeds updated in performance-baseline.json + baseline.rs;
               canonical baseline still requires the qualified runner
               (report-only host)
Commits:       none — per instructions the lane leaves the slice uncommitted
Open blockers: canonical `just e2e-perf-full` and shared-doc updates
               (ARCHITECTURE.md/E2E_ARCHITECTURE.md/ROADMAP.md/Phase-5) are
               integration-owner work after lane review
```
