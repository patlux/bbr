# Task packet — portable general settings: global search engine contract

Parents: [Phase 8](../phases/08-daily-driver.md) (general Settings page) ·
[Phase 5](../phases/05-command-bar.md) (command-bar search fallback) ·
[Testing track](../tracks/testing.md) ·
[Parallel-delivery contract](../../parallel-delivery.md) ·
user spec `/Users/patwoz/dev/Personal/macos/bbr-spaces-theming-settings-spec.md` §4.

## Outcome

`bbr-command` defines the portable search-engine half of the general Settings
page: the five curated engines (DuckDuckGo, Google, Bing, Ecosia, Kagi), a
validated custom URL template containing the `%s` search placeholder,
deterministic search-URL rendering/encoding that reproduces the settled
command-bar DuckDuckGo semantics exactly, a **global-v1** `SearchSettingsV1`
value with defaults, and an object-safe `SettingsStore` port with typed
load/store/reset outcomes — all dependency-free and provider-neutral, with the
existing parser, ranking, and registry untouched.

## Scope

- Curated `SearchEngine` enum (DuckDuckGo, Google, Bing, Ecosia, Kagi) with
  stable `id()`/`display_name()`/`template()` accessors and a canonical
  `CURATED` order for the later settings UI.
- `CustomUrlTemplate` validated at construction: non-empty after trim,
  absolute `http://`/`https://` scheme (case-insensitive; excludes `file:`,
  `javascript:`, `data:`, relative templates), exactly one `%s` placeholder
  (`%25s` is a literal and does not count), and every other `%` must begin a
  valid `%XX` percent-encoding — with a typed `TemplateError` for each rule.
- Deterministic `render_search_url`/`percent_encode`: the single `%s` is
  replaced with RFC 3986 unreserved-preserving, uppercase-hex percent-encoding
  of the UTF-8 terms (space → `%20`), byte-identical to the settled app
  `command_activate` DuckDuckGo fallback
  (`https://duckduckgo.com/?q=<percent-encoded terms>`).
- `SearchSettingsV1` with `SettingsVersion::V1` and `SettingsScope::Global`
  fixed by the type; `defaults()` is the DuckDuckGo curated engine, matching
  the already-shipping command-bar fallback endpoint.
- Object-safe `SettingsStore` port (`Send + Sync`) with `LoadOutcome`
  (`Stored`/`Empty`/`Unavailable`), `StoreOutcome` (`Stored`/`Rejected`),
  `ResetOutcome` (`Reset`/`NothingStored`/`Failed`), and a default
  `load_effective_settings()` that applies `SearchSettingsV1::defaults()` on
  `Empty`/`Unavailable`. No SQLite, filesystem, or native object appears in the
  API; an in-memory fake proves external implementability and object safety.
- Comprehensive dependency-free deterministic tests inside the module
  (validation matrix, encoding table, curated-endpoint table, store lifecycle,
  object safety, typed-outcome reachability).
- Task packet recording global v1 ownership and the later
  persistence/composition/UI boundaries.

## Non-goals

- No persistence repository, SQLite migration, schema-v1→v2 decision, or
  dependency on `bbr-persistence`; a later serial Phase-7 slice owns storage
  and the per-profile-vs-global evolution.
- No app wiring: the app's `command_activate` fallback is not changed here;
  later Phase-5 composition renders `Activation::Search { terms }` through
  `render_search_url` with the effective settings.
- No general Settings UI, section layout, DESIGN changes, snapshot/testkit
  exposure, E2E, or performance work — the grouped Search settings page is a
  later Phase-8 UI slice.
- No new dependencies, manifests, lockfiles, `Cargo.toml`/`Cargo.lock`
  changes, `justfile`, `ROADMAP.md`, or sibling-crate edits.
- No changes to `parse.rs`, `rank.rs`, `registry.rs`, `collect.rs`, `corpus.rs`,
  or `id.rs`: provider-neutral ranking and existing parse behavior stay
  untouched.
- No serde: wire/serde shapes are owned by the later persistence slice.

## Parallel-lane contract

```text
Base commit: 3139939bbd5c911b515e079e2333fa9b37cfa335 (docs(delivery): enforce
             ledger continuation protocol)
Depends on: integrated Phase-5 portable command core (bbr-command crate present
            with parser/ranking/registry); no other lane
Provides/API: SearchEngine + CURATED, CustomUrlTemplate + TemplateError,
              render_search_url + percent_encode, SearchSettingsV1 +
              SettingsVersion/SettingsScope, SettingsStore + typed
              Load/Store/Reset outcomes
Exclusive paths: crates/bbr-command/src/search_settings/**;
                 additive module/export hunk in
                 crates/bbr-command/src/lib.rs;
                 docs/roadmap/tasks/general-search-settings-contract.md
Shared integration points: later settings persistence migration/repository
                           (Phase 7), command-bar activation rendering
                           (Phase 5 composition), general Settings UI
                           (Phase 8), root roadmap promotion
Integration order: portable contract may land after the integrated command
                   core; storage, composition, and UI remain later serial
                   slices
```

## Constraints

- `#![forbid(unsafe_code)]` inherited; std only, no new dependency.
- Rendering is deterministic: identical terms + engine ⇒ byte-identical URL on
  every platform; the encoding matches the settled command-bar boundary
  (unreserved-preserving, uppercase hex, `%20` for space).
- Custom templates cannot smuggle non-http(s) destinations (`file:`,
  `javascript:`, `data:`, relative URLs are rejected whole).
- Default Search Engine is **global in v1** (spec §4: "Default Search Engine =
  global in v1"): `SearchSettingsV1` cannot express a per-profile value;
  per-profile settings require an explicit later schema decision.
- The store port exposes no SQLite/filesystem/native objects and remains
  implementable outside the crate (object-safe, `Send + Sync`).
- Keep every file below roughly 500 lines.
- Do not modify or inspect sibling worktrees; one writer owns this branch.

## Evidence

### Unit and contract

- [x] Curated-engine table tests lock the five ids, display names, templates,
      and canonical order; custom templates cover accept/trim/reject matrices
      including `%25s`-not-a-placeholder, `%%s`, malformed `%zz`, multiple
      placeholders, and non-http schemes.
- [x] Rendering tests cover the settled DuckDuckGo parity, all five curated
      endpoints, the unreserved-set encoding table, uppercase hex, Unicode
      UTF-8, empty terms, literal `%XX` preservation, and terms that contain
      `%s` without template collision.
- [x] Settings tests prove global-v1 defaults (DuckDuckGo), `with_engine`
      preserving version/scope, `Default` parity, and structural equality.
- [x] Store tests prove empty→defaults, store/load round-trip, supersession,
      reset→empty, `NothingStored`, `Unavailable`/`Rejected`/`Failed` paths,
      object safety via `&dyn`/`Box<dyn>`/`Arc<dyn>`, and closed outcome
      reachability.
- [x] `cargo fmt --all -- --check`, `cargo test -p bbr-command`, and
      `cargo clippy -p bbr-command --all-targets -- -D warnings` pass; `just
      roadmap` links valid.

### Real-app E2E

- [x] Not applicable to this pure-core contract packet: it creates no browser
      or UI behavior. The Phase-8 settings-UI and Phase-5 composition slices
      own production GPUI/E2E evidence; the app's DuckDuckGo fallback remains
      untouched and its settled E2E stays green.

### Performance

- [x] Not applicable: pure validation/state transitions and a constant-time
      template render. Later user-visible settings/command interactions add
      optimized presentation evidence through their own slices.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-command
cargo clippy -p bbr-command --all-targets -- -D warnings
just roadmap
```

No GUI gate is required because the lane is portable and changes no browser
surface. The integration lane runs `just verify` after landing.

## Delivery sequence

1. One writer owns `/Users/patwoz/dev/Personal/macos/bbr-settings-search-contract`
   on `parallel/settings-search-contract` from the exact base commit.
2. Implement and verify only the portable contract.
3. One independent read-only reviewer maps the spec §4 requirements and this
   packet to tests and evidence.
4. Remediate only concrete findings, then hand off the settled branch to the
   serial integration owner; do not mutate shared app/persistence/UI surfaces
   in this worktree.

## Iteration policy

Start with the curated engines and the exact settled DuckDuckGo rendering
parity, then add custom-template validation, the global-v1 settings value, and
the object-safe store port with its lifecycle tests. Diagnose failures as
template validation, encoding, settings identity, or store lifecycle before
changing the contract. Do not broaden scope to solve unrelated findings.

## Blocked stop condition

If the default engine, placeholder semantics, encoding rules, or global-v1
ownership cannot be resolved from the spec §4 and the settled command-bar
boundary, stop with the preserved branch, the evidence gathered, and the exact
product decision needed. Do not invent a persistence schema, serde shape, UI
layout, or per-profile semantics to unblock the pure contract.

## Delivery ledger

```text
Outcome: Portable bbr-command search-settings contract: five curated engines
         (DuckDuckGo, Google, Bing, Ecosia, Kagi) with canonical order and
         stable ids, a validated CustomUrlTemplate (exactly one %s, http(s)
         only, valid %XX elsewhere) with typed TemplateError, deterministic
         render_search_url/percent_encode matching the settled command-bar
         DuckDuckGo boundary byte-for-byte, global-v1 SearchSettingsV1
         (DuckDuckGo defaults, scope/version fixed by the type), and an
         object-safe Send+Sync SettingsStore port with typed
         Load/Store/Reset outcomes and a default effective-settings helper.
         Parser, ranking, registry, corpus, and collect behavior untouched.
Commits: none yet — writer handoff pending (per parallel-delivery contract,
         the orchestrator integrates serially; agent commits are not part of
         this lane's handoff)
Functional evidence: cargo test -p bbr-command: 41 lib tests (29 new
         search_settings tests + 12 pre-existing) + 1 benchmark + 5 parse +
         24 rank + 7 spaces = 78 tests, plus 2 doctests (1 new module
         doctest) — all green. New coverage: curated table, custom-template
         accept/trim/reject matrix (%25s, %%s, %zz, multiple placeholders,
         non-http schemes), settled DuckDuckGo parity, all-five-endpoint
         rendering, encoding table incl. Unicode/uppercase-hex/unreserved set,
         literal %XX preservation, %s-in-terms non-collision, global-v1
         defaults/scope/version, store lifecycle (round-trip, supersede,
         reset, NothingStored, Unavailable, Rejected, Failed), object safety
         via dyn/Box/Arc, closed outcome reachability.
Real-app E2E: not applicable to this pure-core packet; app fallback and its
         settled E2E remain untouched.
Performance evidence: not applicable to contract-only lane; no GUI/perf
         command run.
Roadmap/documentation updates: this packet only (docs/roadmap/tasks/
         general-search-settings-contract.md); ROADMAP.md, phase files, and
         the delivery plan await serial promotion of the lane.
Open blockers and non-goals: persistence repository/schema-v1→v2 decision,
         Phase-5 Activation::Search rendering wiring, Phase-8 grouped
         Search-settings UI, serde/wire shapes, testkit/snapshot exposure —
         all later serial slices.
Worktree state: /Users/patwoz/dev/Personal/macos/bbr-settings-search-contract
         on parallel/settings-search-contract at exact base
         3139939bbd5c911b515e079e2333fa9b37cfa335; implementation uncommitted
         and preserved; allowlist holds (7 files: 5 module files + additive
         lib.rs hunk + this packet).

Planning/read-only: estimates not reconstructed; runtime evidence from the
         executed gate commands.
Implementation: single writer pass; two self-caught compile fixes (lock
         deref/match ownership, template clone) before green.
Test execution: cargo fmt --check clean; cargo test -p bbr-command green
         (78 tests + 2 doctests); cargo clippy -p bbr-command --all-targets
         -- -D warnings clean; just roadmap green (57 Markdown files).
Review: pending (independent read-only reviewer per parallel-delivery
         contract).
Remediation: none requested yet.
Waiting/retries: none.
Total elapsed: estimates; not reconstructed exactly.
Human intervention: none.
```

## Spec cross-reference

- Spec §4 "Allgemeine Settings-Seite": curated list DuckDuckGo, Google, Bing,
  Ecosia, Kagi + Custom-URL with `%s`-Template → `SearchEngine`/`CURATED`/
  `CustomUrlTemplate`.
- "Palette-/URL-Feld-Suche nutzt diese Engine" → later Phase-5 composition
  renders `Activation::Search` through `render_search_url` with the effective
  global settings.
- "Settings-Persistenz: profilweit vs. global sauber entscheiden und
  dokumentieren (Default Search Engine = global in v1)" → `SettingsScope::Global`
  is fixed by the `SearchSettingsV1` type; per-profile settings are an explicit
  later schema decision recorded in Non-goals.
- Night-queue items 17–20: this packet covers the portable contract half
  ("Settings-Store und Search-Engine-Typen/Custom-Template"); the grouped
  Search-settings UI (item 20) remains a later Phase-8 slice.
