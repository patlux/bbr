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
Outcome: Portable bbr-command core: typed modes/identities, strict deterministic
         URL-vs-search authority parsing, provider-neutral owned corpus,
         13-action registry, standalone tab/space/history/action/fallback
         ranking with explicit profile identity, typed activations, and a
         release-mode first-result benchmark (500 tabs + 10,000 history)
         enforcing strict p95/max <50 ms with raw distribution output.
Commits: 14 total branch commits from 9352ded through the settled parser
         remediation ledger: 1 packet-definition commit plus 13 subsequent
         product/test/refactor/documentation/remediation commits. Coherent
         product/remediation work groups: parsing/identities; corpus/registry;
         ranking; parser/ranking/benchmark evidence; collector split; score/key
         correction; five-finding remediation; dotted bare-colon authority
         remediation; documentation/ledger truth. Exact subjects remain
         authoritative in `git log 9352ded..HEAD`.
Functional evidence: cargo test -p bbr-command: 12 lib + 1 benchmark + 5 parse
         + 24 rank + 7 standalone-space + 1 doc = 50 green tests. Public parser
         regressions cover invalid IPv4, strict domain labels, ports, bracketed
         IPv6, unbracketed IPv6, punycode, malformed dotted bare-colon
         authorities, opaque undotted schemes, and explicit dotted schemes.
         Ranking covers standalone empty
         spaces, current-space exclusion, same/cross/unknown profile labels,
         duplicate names, SelectSpace activation, explicit index/id ordering,
         reversed provider vectors, all groups, dedup, fallback, Unicode, empty
         input, and limits. The built-in registry is locked by an exact 13-row
         id/title/category/shortcut table.
Real-app E2E: not applicable; later app-composition packet owns it.
Performance evidence: cargo test -p bbr-command --release --test
         benchmark_50ms -- --nocapture: 400 raw samples; p50 0.117000 ms, p95
         1.452917 ms, max 1.539583 ms, min 0.001500 ms. Both p95 and max remain
         enforced below 50 ms; corpus remains 500 tabs + 10,000 history and
         fixture setup remains outside timed samples.
Roadmap/documentation updates: Phase-5 detail now states standalone-space
         ranking/activation and exact remediation benchmark evidence; this
         ledger distinguishes total branch commits from coherent work groups.
         ARCHITECTURE.md remains owned by serial integration.
Open blockers and non-goals: no GPUI/real-app surface, focus routing, keyboard
         selection, E2E, SQLite/history provider, debounce/cancellation runtime,
         fuzzy dependency, or product-budget change. No root manifest,
         Cargo.lock, justfile, testkit, shared crate, ROADMAP.md, performance
         baseline, or URL-input surface modified.
Worktree state: parallel/command-core; 14 commits ahead of 9352ded after this
         final documentation commit; no remote configured and nothing pushed;
         sibling worktrees untouched. Final gate/status evidence is recorded in
         the remediation handoff.

Planning/read-only: original estimates retained as estimates; remediation
         runtime not reconstructed because exact workflow timing was not
         captured.
Review: initial independent result was REMEDIATE with five findings; focused
         recheck found one remaining major dotted bare-colon parser bypass.
Remediation: all initial findings and the focused-recheck parser bypass are
         addressed in the bounded owned paths.
Human intervention: none.
```
