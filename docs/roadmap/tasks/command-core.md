# Task packet — portable command core

Parents: [Phase 5](../phases/05-command-bar.md) · [Performance contract](../../product/performance-contract.md) · [Parallel-delivery contract](../../parallel-delivery.md).

## Outcome

A portable `bbr-command` core parses command-bar input into typed intents and
ranks deterministic local results across open tabs, spaces, history records,
actions, and navigate/search fallback. It returns the first correct local
result below 50 ms for a corpus of 500 tabs and 10,000 history entries without
GPUI, SQLite, macOS frameworks, testkit transport, or synchronous I/O.

## Scope

- Typed input/query, intent, result, group, identity, score, and activation
  types in `crates/bbr-command/**`.
- Table-driven URL/search parsing for location and staged new-tab semantics;
  command/action prefixes only if the product contract defines them during the
  slice.
- Stable ranking groups: exact/open tabs, current-space and pinned tabs, other
  spaces with profile identity, history, actions, navigate/search fallback.
- Deterministic tie-breaking and result limits suitable for later cancellable
  orchestration; no render-path blocking or global state.
- A provider-neutral owned input corpus and pure ranking API. History storage
  and cancellation/debounce orchestration integrate later through serial
  contracts.
- Unit/property-style cases for ambiguous URLs/searches, duplicate titles,
  cross-profile identity, Unicode, empty input, and deterministic ordering.
- An optimized portable benchmark test for 500 tabs + 10,000 history entries,
  retaining raw distribution evidence and enforcing a 50 ms first-result
  ceiling without weakening the real-app command-bar performance requirement.
- Truthful Phase-5/roadmap and crate documentation updates.

## Non-goals

- No GPUI command-bar result surface, focus routing, keyboard selection, or
  real-app E2E; those require the later serial app-composition slice.
- No SQLite repository or direct dependency on `bbr-persistence`.
- No changes to `bbr-app`, `bbr-browser`, `bbr-domain`, `bbr-web`, testkit
  protocol, root workspace manifests, `Cargo.lock`, `justfile`, performance
  baseline, or `ROADMAP.md` current-focus roll-up.
- No network search suggestions, fuzzy-search dependency, debounce timer, or
  cancellation runtime until profiling and the integration contract justify
  them.

## Parallel-lane contract

```text
Base commit: 9352ded94075d3c82f90c682fb7ff248d0ee2fe6
Depends on: parallel-delivery foundation only
Provides/API: portable parser, registry, ranking, result and corpus contracts
Exclusive paths: crates/bbr-command/**; this task packet; Phase-5 detail only
Shared integration points: app composition, command snapshot/testkit wire,
  history repository/provider, root Cargo/Cargo.lock, global perf matrix
Integration order: land portable command core before command app composition;
  integrate shared surfaces serially from a fresh clean anchor
```

## Constraints

- `#![forbid(unsafe_code)]`; no GPUI, AppKit/WebKit, filesystem, SQLite, socket,
  or platform dependency.
- Ranking is deterministic and does not hide profile/space identity in
  cross-space results.
- Exact/open/current/pinned priority follows Phase 5; ties must never depend on
  hash-map iteration order.
- The benchmark measures core parsing/ranking, not fixture setup or test
  transport, and it cannot substitute for later optimized real-app evidence.
- Do not modify or inspect sibling worktrees; one writer owns this branch.

## Evidence

### Unit and contract

- [x] Table-driven parser and intent tests cover URL/search ambiguity and modes.
- [x] Ranking tests prove every priority group, cross-profile labels,
      deterministic ties, deduplication, and fallback behavior.
- [x] Public API remains portable and dependency direction is documented.

### Real-app E2E

- [x] Not applicable to this pure-core packet: it creates no browser behavior.
      The Phase-5 app-composition packet owns production GPUI/WKWebView E2E,
      keyboard focus, semantic snapshots, and activation behavior.

### Performance

- [x] Optimized local benchmark uses 500 tabs + 10,000 history entries and
      proves first correct result below 50 ms with raw repeated samples.
- [x] The packet explicitly preserves later in-process real-app presentation
      measurement; no product budget or baseline is changed here.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-command
cargo clippy -p bbr-command --all-targets -- -D warnings
cargo test -p bbr-command --release
just roadmap
```

No GUI gate is required because the lane is portable and changes no browser
surface. The integration lane runs `just verify` after landing.

## Delivery sequence

1. One writer owns `/Users/patwoz/dev/Personal/macos/bbr-command-core` on
   `parallel/command-core`.
2. Implement and verify only the portable core.
3. One independent read-only reviewer maps Phase-5 ranking/parser requirements
   to tests and benchmark evidence.
4. Remediate only concrete findings, then commit coherent verified work.
5. Handoff the settled branch to the serial integration owner; do not mutate
   shared app/testkit/root surfaces in this worktree.

## Iteration policy

Start with typed inputs/results and exact URL-vs-search parsing, then implement
one ranking group at a time with table-driven tests. Add a dependency only if
existing std/project facilities cannot meet a measured need. Benchmark after
correctness is stable; optimize the identified mechanism rather than weakening
coverage or limits.

## Blocked stop condition

Stop with the preserved branch/worktree, parser/ranking evidence, attempted
approaches, and the smallest product decision needed if ranking precedence,
URL/search ambiguity, command syntax, or cross-profile result identity cannot
be resolved from Phase 5 and existing product contracts. Do not invent app UI
or persistence contracts to unblock the pure core.

## Delivery ledger

```text
Outcome: Portable bbr-command core: typed modes/identities, deterministic
         URL-vs-search parsing, provider-neutral owned corpus, ordered command
         registry, Phase-5 group ranking with cross-profile identity, typed
         activations, and a release-mode first-result benchmark (500 tabs +
         10,000 history entries) enforcing the 50 ms ceiling with raw
         distribution output.
Commits: 6 verified groups — parsing/identities; corpus/registry; ranking;
         private collector split (refactor); parser/ranking contract tests;
         first-result benchmark; roadmap/phase documentation (subjects
         recorded in the branch log).
Functional evidence: cargo test -p bbr-command (11 lib + 5 parse + 23 rank + 1
         doc = 40 tests) green; cargo clippy -p bbr-command --all-targets --
         -D warnings clean; cargo fmt --all -- --check green; every Phase-5
         group, cross-profile label, dedup, fallback, unicode, empty-input,
         and determinism case locked by table-driven tests.
Real-app E2E: not applicable; later app-composition packet owns it.
Performance evidence: cargo test -p bbr-command --release — benchmark
         first_correct_result_below_50ms_for_500_tabs_and_10k_history: 400 raw
         samples, p50 ≈ 0.12 ms, p95 ≈ 1.45 ms, max ≈ 1.6 ms (≈ 30× headroom
         under the 50 ms ceiling), raw per-sample distribution printed with
         --nocapture and retained in the run output.
Roadmap/documentation updates: Phase-5 checklist items 1/2/5/6/11 marked with
         truthful scope notes; this packet's evidence/ledger completed; crate
         docs in crates/bbr-command/src/lib.rs document the pipeline and
         boundaries. ARCHITECTURE.md crate-table wording stays owned by the
         serial integration lane per the parallel-delivery contract.
Open blockers and non-goals: no GPUI/real-app command-bar surface, focus
         routing, keyboard selection, E2E, SQLite/history repository, debounce/
         cancellation runtime, fuzzy-search dependency, or product budget
         change; real-app presentation measurement remains the app-composition
         packet. No root manifest, Cargo.lock, justfile, testkit, or shared
         surface modified.
Worktree state: parallel/command-core with 5 coherent verified commits; no
         remote configured, so nothing pushed; sibling worktrees untouched.

Planning/read-only: ~20 min (packet + product/performance/interaction
         contracts read).
Implementation: ~4 h (design + 8 source/test files + iteration).
Test execution: ~1 h (gates + benchmark stability runs).
Review: pending (one independent read-only reviewer maps Phase-5 requirements
         to tests and benchmark evidence).
Remediation: none yet.
Waiting/retries: none.
Total elapsed: ~6 h (estimates; no workflow timestamps captured).
Human intervention: none.
```
